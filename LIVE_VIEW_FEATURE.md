# Live-View Feature: Game Replay System

## Overview
Complete implementation of a game replay system that allows players to watch how any finished game was played, move by move.

## Backend Changes

### 1. State Schema (`src/state.rs`)
```rust
// New struct to store individual move records
pub struct MoveRecord {
    pub direction: RegisterView<u8>,      // 0=Up, 1=Down, 2=Left, 3=Right
    pub timestamp: RegisterView<u64>,     // Move timestamp in microseconds
    pub board_after: RegisterView<u64>,   // Board state after move
    pub score_after: RegisterView<u64>,   // Score after move
}

// Updated BoardState to include move history
pub struct BoardState {
    // ... existing fields ...
    pub move_history: CollectionView<u32, MoveRecord>,
    pub move_count: RegisterView<u32>,
}
```

### 2. Game Logic (`src/contract_domain/game_logic.rs`)
- Added `ProcessedMove` struct to track each move's result
- Modified `GameMoveProcessor::process_moves()` to return `Vec<ProcessedMove>`
- Each move now captures: direction, timestamp, resulting board state, and score

### 3. Contract Handler (`src/contract_domain/handlers/operations/game.rs`)
- Updated `handle_make_moves()` to store each move in `board.move_history`
- Stores moves sequentially indexed by `move_count`
- Persists complete move-by-move game history

### 4. GraphQL Service (`src/service_handlers/`)
**New Types** (`types.rs`):
```rust
pub struct MoveHistoryRecord {
    pub direction: String,              // "Up", "Down", "Left", "Right"
    pub timestamp: String,              // milliseconds
    pub board_after: [[u16; 4]; 4],    // Board state as matrix
    pub score_after: u64,
}

pub struct BoardMoveHistory {
    pub board_id: String,
    pub player: String,
    pub total_moves: u32,
    pub moves: Vec<MoveHistoryRecord>,
}
```

**New Query** (`queries.rs`):
```graphql
boardMoveHistory(boardId: String!, limit: Int): BoardMoveHistory
```
- Fetches complete move history for any board
- Supports optional limit (default 1000 moves)
- Converts timestamps from microseconds to milliseconds
- Converts board states to 4x4 matrix format

## Frontend Changes

### 1. GraphQL Query (`website/src/lib/graphql/queries/getBoardMoveHistory.ts`)
```typescript
export const GET_BOARD_MOVE_HISTORY = gql`
  query GetBoardMoveHistory($boardId: String!, $limit: Int) {
    boardMoveHistory(boardId: $boardId, limit: $limit) {
      boardId
      player
      totalMoves
      moves {
        direction
        timestamp
        boardAfter
        scoreAfter
      }
    }
  }
`;
```

### 2. BoardReplay Component (`website/src/lib/components/organisms/BoardReplay.svelte`)
**Features**:
- Full playback control system
- Reactive state management with Svelte 5 runes
- Displays current board state, score, and move direction
- Auto-pause at end of replay

**Controls**:
- ▶ Play / ⏸ Pause
- ⏮ Restart
- ⏪ Previous Move / Next Move ⏩
- Speed selector: 0.5x, 1x, 2x, 4x
- Progress slider for direct navigation
- Move counter (e.g., "Move 45 / 120")

**State Management**:
```typescript
let currentMoveIndex = $state(0);
let isPlaying = $state(false);
let playbackSpeed = $state(1);
```

### 3. Replay Route (`website/src/routes/replay/+page.svelte`)
- URL: `/replay?boardId={id}&chainId={id}`
- Displays BoardReplay component
- Shows player name and current score
- "Back to Leaderboard" navigation

### 4. Game Overlay Integration (`website/src/lib/components/organisms/Board.svelte`)
- Added "🎬 Watch Replay" button to finished game overlay
- Only shows when game is ended
- Links directly to replay page with correct parameters

### 5. Game Component Updates (`website/src/lib/components/organisms/Game.svelte`)
- Passes `boardId`, `chainId`, and `showReplayButton={true}` to Board
- Enables replay functionality for all finished games

## How to Use

### As a Player
1. **Finish a game** - Play until board is full or time expires
2. **See "Watch Replay" button** - Appears on game over overlay
3. **Click button** - Navigate to `/replay` page
4. **Watch and control**:
   - Click Play to watch automatically
   - Adjust speed (0.5x to 4x)
   - Step through moves manually
   - Jump to any move with slider
   - See exact board state, score, and direction for each move

### Direct URL Access
```
/replay?boardId=<board_id>&chainId=<chain_id>
```

## Storage Efficiency

**Per Move**: ~28 bytes
- direction: 1 byte (u8)
- timestamp: 8 bytes (u64)
- board_after: 8 bytes (u64)
- score_after: 8 bytes (u64)
- index overhead: ~3 bytes

**Example**: 100-move game = ~2.8 KB

## Performance Considerations

1. **Default Limit**: GraphQL query limits to 1000 moves by default
2. **Lazy Loading**: Only fetches moves when replay page is accessed
3. **Sequential Storage**: Uses CollectionView for efficient indexed access
4. **Client-Side Playback**: All replay logic runs in browser (no server load)

## Technical Implementation Details

### Backend Storage Flow
```
1. Player makes moves → MakeMoves operation
2. GameMoveProcessor processes each move
3. For each move:
   - Execute move
   - Capture board state + score
   - Create ProcessedMove record
4. Store in board.move_history[index]
5. Increment board.move_count
```

### Frontend Playback Flow
```
1. Load boardMoveHistory from GraphQL
2. Initialize at move 0
3. On play:
   - Start interval at (500ms / playbackSpeed)
   - Increment currentMoveIndex
   - Update displayed board/score
4. On pause:
   - Clear interval
5. On slider drag:
   - Jump to specific move
   - Pause playback
```

## Future Enhancements (Ideas)

1. **Live Streaming**: Watch games in progress (WebSocket subscription)
2. **Highlight Reel**: Show only moves that achieved new high tiles
3. **Comparison Mode**: Watch two games side-by-side
4. **Export**: Download replay as video or GIF
5. **Social Sharing**: Share replay links on social media
6. **Analytics**: Show move efficiency, average time per move, etc.

## Testing Checklist

- [x] Backend compiles successfully
- [x] Frontend compiles with no TypeScript errors
- [ ] Create test game with moves
- [ ] Deploy new contract WASM
- [ ] Verify move storage in contract state
- [ ] Test GraphQL query returns correct data
- [ ] Test replay page loads
- [ ] Test all playback controls work
- [ ] Test different playback speeds
- [ ] Test slider navigation
- [ ] Test on mobile devices
- [ ] Test with long games (500+ moves)

## Files Modified

### Backend
- `src/state.rs` - Added MoveRecord, move_history to BoardState
- `src/contract_domain/game_logic.rs` - Added ProcessedMove, move history tracking
- `src/contract_domain/handlers/operations/game.rs` - Store moves in state
- `src/service_handlers/types.rs` - Added MoveHistoryRecord, BoardMoveHistory
- `src/service_handlers/queries.rs` - Added boardMoveHistory query

### Frontend
- `website/src/lib/graphql/queries/getBoardMoveHistory.ts` - New GraphQL query
- `website/src/lib/components/organisms/BoardReplay.svelte` - Replay component
- `website/src/routes/replay/+page.svelte` - Replay page route
- `website/src/lib/components/organisms/Board.svelte` - Added replay button
- `website/src/lib/components/organisms/Game.svelte` - Pass replay props
- `website/src/lib/components/molecules/BoardHeader.svelte` - TypeScript fixes

## Commits
1. `541ac5a` - Backend: Move history storage and replay query
2. `96bda56` - Frontend: Replay UI with playback controls

## Branch
`feature/live-view`

Created from: `main`
Ready to merge: ✅ (after testing)
