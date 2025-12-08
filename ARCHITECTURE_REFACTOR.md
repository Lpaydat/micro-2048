# Message-Based Leaderboard Architecture

## Overview

Refactor from event/subscription-based score propagation to direct message-based updates. This simplifies the architecture by eliminating shards as intermediaries and leveraging Linera's `--listener-skip-process-inbox` feature for batch processing.

## Current Architecture (Event-Based)

```
Player Chain ──emit──► PlayerScoreUpdate Event
                              │
                              ▼ (subscription)
Shard Chain ──────► Cache scores locally
                    │
                    ▼ (on TriggerShardAggregation)
                    emit ShardScoreUpdate Event
                              │
                              ▼ (subscription)
Leaderboard Chain ─► Merge scores from all shards
```

**Problems:**
1. Complex 4-stream event system
2. Shard chains add latency and complexity
3. `process_streams` only runs during block production
4. Race conditions between events and messages
5. Hard to debug event propagation issues

## New Architecture (Message-Based)

```
Player Chain ──send──► Message::SubmitScore
                              │
                              ▼ (message inbox - queued)
Leaderboard Chain ─► Messages accumulate in inbox
                    │
                    ▼ (on UpdateLeaderboard operation)
                    Process all pending scores at once
```

### Registration Flow (with --listener-skip-process-inbox)

```
Frontend: registerPlayer()
    ↓
Main Chain: Create/claim player chain, send messages
    ↓
Messages queued in player chain inbox (not processed yet)
    ↓
Frontend: Call claimChain() on player chain
    ↓
Player Chain: Triggers block production → processes inbox
    ↓
RegisterPlayer message processed → player data initialized
    ↓
Frontend: Query player chain for balance/data
```

**Benefits:**
1. **Simpler** - No events, no subscriptions, no intermediate shards
2. **Batching** - Messages queue up, processed all at once
3. **Predictable** - Leaderboard only updates on explicit trigger
4. **Debuggable** - Messages are visible in chain state
5. **Lower overhead** - Fewer chains, simpler state

---

## Detailed Design

### 1. New Message Variant

```rust
// In src/lib.rs
pub enum Message {
    // ... existing messages ...
    
    /// Player submits their score directly to leaderboard
    /// Sent on: new personal best score achieved
    SubmitScore {
        player: String,
        player_chain_id: String,
        board_id: String,
        score: u64,
        highest_tile: u64,
        game_status: GameStatus,
        timestamp: u64,
    },
    
    /// Player registers with leaderboard (on first board creation)
    RegisterPlayer {
        player: String,
        player_chain_id: String,
    },
}
```

### 2. Leaderboard State (Simplified)

```rust
// In src/state.rs
pub struct Leaderboard {
    // Identity
    pub leaderboard_id: RegisterView<String>,
    pub chain_id: RegisterView<String>,
    pub name: RegisterView<String>,
    pub description: RegisterView<String>,
    pub host: RegisterView<String>,
    
    // Time bounds
    pub start_time: RegisterView<u64>,
    pub end_time: RegisterView<u64>,
    
    // Score data (directly updated from messages)
    pub scores: MapView<String, u64>,              // player -> best_score
    pub board_ids: MapView<String, String>,        // player -> best_board_id
    pub highest_tiles: MapView<String, u64>,       // player -> highest_tile
    pub is_ended: MapView<String, bool>,           // player -> is_game_ended
    pub last_update: MapView<String, u64>,         // player -> last_update_timestamp
    
    // Player tracking
    pub registered_players: SetView<String>,       // Set of player_chain_ids
    pub total_players: RegisterView<u32>,
    pub total_boards: RegisterView<u32>,
    
    // REMOVED: shard_ids, triggerer system, activity scores, etc.
}
```

### 3. Message Handler (Leaderboard)

```rust
// In src/contract_domain/message_dispatcher.rs
Message::SubmitScore {
    player,
    player_chain_id,
    board_id,
    score,
    highest_tile,
    game_status,
    timestamp,
} => {
    let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
    
    // Get current best
    let current_best = leaderboard.scores.get(&player).await.unwrap().unwrap_or(0);
    
    // Only update if better score
    if score > current_best {
        leaderboard.scores.insert(&player, score).unwrap();
        leaderboard.board_ids.insert(&player, board_id).unwrap();
        leaderboard.highest_tiles.insert(&player, highest_tile).unwrap();
        leaderboard.last_update.insert(&player, timestamp).unwrap();
    }
    
    // Track game ended status
    if matches!(game_status, GameStatus::Ended(_)) {
        leaderboard.is_ended.insert(&player, true).unwrap();
    }
}

Message::RegisterPlayer { player, player_chain_id } => {
    let leaderboard = contract.state.leaderboards.load_entry_mut("").await.unwrap();
    
    // Track unique players
    if !leaderboard.registered_players.contains(&player_chain_id).await.unwrap() {
        leaderboard.registered_players.insert(&player_chain_id).unwrap();
        let count = *leaderboard.total_players.get();
        leaderboard.total_players.set(count + 1);
    }
}
```

### 4. Player Chain Changes (MakeMoves)

```rust
// In src/contract_domain/handlers/operations/game.rs
// After processing moves and updating local state:

if final_score > current_best {
    // Update local best score
    player_record.best_score.insert(&leaderboard_id, final_score).unwrap();
    
    // Send score directly to leaderboard
    if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
        contract.runtime
            .prepare_message(Message::SubmitScore {
                player: player.clone(),
                player_chain_id: contract.runtime.chain_id().to_string(),
                board_id: board_id.clone(),
                score: final_score,
                highest_tile: final_highest_tile,
                game_status: game_status.clone(),
                timestamp: latest_timestamp,
            })
            .send_to(leaderboard_chain_id);
    }
}
```

### 5. Board Creation Changes (NewBoard)

```rust
// In src/contract_domain/handlers/operations/game.rs
// On first board in tournament:

if is_first_board_in_tournament {
    if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
        contract.runtime
            .prepare_message(Message::RegisterPlayer {
                player: player.clone(),
                player_chain_id: contract.runtime.chain_id().to_string(),
            })
            .send_to(leaderboard_chain_id);
    }
}

// Increment board count
let new_count = current_board_count + 1;
player_state.boards_per_tournament.insert(&leaderboard_id, new_count).unwrap();

// Send initial score (0)
if let Ok(leaderboard_chain_id) = ChainId::from_str(&leaderboard_id) {
    contract.runtime
        .prepare_message(Message::SubmitScore {
            player: player.clone(),
            player_chain_id: contract.runtime.chain_id().to_string(),
            board_id: board_id.clone(),
            score: 0,
            highest_tile: 2,
            game_status: GameStatus::Created,
            timestamp,
        })
        .send_to(leaderboard_chain_id);
}
```

### 6. Leaderboard Service Configuration

```bash
# Run leaderboard chain with skip-process-inbox
linera service \
    --port 8080 \
    --listener-skip-process-inbox \
    ...
```

This means:
- Messages arrive → stored in inbox (NO automatic block production)
- Scores accumulate without triggering updates
- Manual refresh → `processInbox` mutation → processes ALL pending messages

### 7. Frontend Changes

```typescript
// Manual refresh - just process inbox
async function refreshLeaderboard() {
    await leaderboardClient.mutate({
        mutation: gql`
            mutation {
                processInbox
            }
        `
    });
    
    // Then query updated scores
    const result = await leaderboardClient.query({
        query: GET_LEADERBOARD_SCORES
    });
}
```

---

## Migration Plan

### Phase 1: Add New Message Types (Non-Breaking)
- [ ] Add `Message::SubmitScore` variant
- [ ] Add `Message::RegisterPlayer` variant  
- [ ] Add message handlers in `message_dispatcher.rs`
- [ ] Test message handling locally

### Phase 2: Update Player Chain Operations
- [ ] Modify `NewBoard` to send `RegisterPlayer` message
- [ ] Modify `NewBoard` to send initial `SubmitScore` message
- [ ] Modify `MakeMoves` to send `SubmitScore` on new best score
- [ ] Remove `PlayerScoreUpdate` event emissions

### Phase 3: Simplify Leaderboard State
- [ ] Remove shard-related state fields
- [ ] Remove triggerer system state
- [ ] Remove activity tracking state
- [ ] Keep simple score tracking

### Phase 4: Remove Shard System
- [ ] Remove `RegisterPlayerWithShard` message handling
- [ ] Remove `TriggerShardAggregation` message handling
- [ ] Remove `ShardScoreUpdate` event
- [ ] Remove shard subscription logic
- [ ] Remove shard chain state

### Phase 5: Remove Event System
- [ ] Remove `PlayerScoreUpdate` event
- [ ] Remove `ShardScoreUpdate` event
- [ ] Remove `LeaderboardUpdate` event (keep `ActiveTournaments`)
- [ ] Remove `process_streams` score processing logic
- [ ] Remove subscription setup for score events

### Phase 6: Update Frontend
- [ ] Update manual refresh to use `processInbox`
- [ ] Remove shard-related queries
- [ ] Simplify leaderboard queries

### Phase 7: Cleanup
- [ ] Remove unused imports
- [ ] Remove dead code
- [ ] Update documentation
- [ ] Remove shard chain deployment from scripts

---

## Files to Modify

### Core Changes
| File | Changes |
|------|---------|
| `src/lib.rs` | Add `SubmitScore`, `RegisterPlayer` messages; Remove `ShardScoreUpdate`, `PlayerScoreUpdate` events |
| `src/state.rs` | Simplify `Leaderboard` struct; Remove `LeaderboardShard` |
| `src/contract.rs` | Remove shard/event helper methods |
| `src/contract_domain/message_dispatcher.rs` | Add new message handlers; Remove shard message handlers |
| `src/contract_domain/handlers/operations/game.rs` | Send messages instead of events |
| `src/contract_domain/events/processors.rs` | Remove score-related stream processing |
| `src/contract_domain/events/emitters.rs` | Remove score event emitters |
| `src/contract_domain/events/subscriptions.rs` | Remove score event subscriptions |

### Files to Remove/Gut
| File | Action |
|------|--------|
| `src/contract_domain/handlers/operations/shard.rs` | Remove or gut |
| `src/contract_domain/handlers/messages/shard.rs` | Remove |

### Frontend Changes
| File | Changes |
|------|---------|
| `website/src/lib/graphql/mutations/requestLeaderboardRefresh.ts` | Use `processInbox` |
| `website/src/lib/components/templates/Leaderboard.svelte` | Simplify refresh logic |

---

## What We Keep

1. **`ActiveTournaments` event** - Still useful for tournament discovery
2. **Tournament creation flow** - `LeaderboardAction::Create` still works
3. **Player chain state** - Boards, player records, game logic unchanged
4. **Main chain** - Tournament registry unchanged

## What We Remove

1. **Shard chains** - No longer needed
2. **Triggerer system** - No auto-triggers needed
3. **Activity scoring** - Not needed without triggerer selection
4. **Event subscriptions for scores** - Direct messages replace this
5. **`process_streams` for scores** - No score events to process

---

## Risk Assessment

### Low Risk
- Message handling is well-understood Linera pattern
- `--listener-skip-process-inbox` is official Linera feature
- Simplification reduces bug surface area

### Medium Risk
- Large refactor touching many files
- Need to ensure backward compatibility during migration
- Frontend changes needed

### Mitigation
- Keep old code paths during migration (feature flag)
- Test extensively on devnet before mainnet
- Gradual rollout with monitoring

---

## Questions to Resolve

1. **Board counting**: How to track total boards without shards?
   - Option A: Each `SubmitScore` includes `boards_in_tournament` count
   - Option B: Separate `IncrementBoardCount` message on board creation
   - **Recommendation**: Option A - include in `SubmitScore`

2. **Rate limiting**: Without triggerer system, how to prevent spam?
   - Option A: Linera's natural message costs
   - Option B: Cooldown check in message handler
   - **Recommendation**: Linera costs + optional cooldown per player

3. **Active boards tracking**: Do we still need this?
   - If yes: Add to `SubmitScore` message
   - If no: Remove from leaderboard state
   - **Recommendation**: Keep simple, add later if needed
