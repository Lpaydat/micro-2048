# Auto-Redirect Debug Guide

## Current Status
Auto-redirect works from leaderboard page but NOT from game page.

## Debug Steps

### 1. Open Browser Console
Press F12 to open developer tools

### 2. Navigate to Game Page
1. Go to a tournament (e.g., http://localhost:5173/events)
2. Click on a tournament or create a new one
3. Enter an existing game (if you have one)

### 3. Click "New Game" Button
Watch the console for these logs:

#### Expected Log Sequence:

```
Creating new board for leaderboard: <leaderboard-id>
Player chainId: <chain-id>
Board creation timestamp: 1730000000000
Board created successfully
```

Then every second you should see:
```
⏰ Polling for new board... {
  isNewGameCreated: true,
  hasTimestamp: true,
  hasBoard: true/false,
  fetching: true/false,
  error: null/error
}
```

#### What to Check:

**Case 1: hasBoard: false**
```
⏰ Polling for new board... {
  isNewGameCreated: true,
  hasTimestamp: true,
  hasBoard: false,  ❌ PROBLEM HERE
  fetching: false,
  error: null
}
```
**Issue**: GraphQL query returns no board data
**Possible Causes**:
- Player's chain doesn't have the board yet
- GraphQL query failing silently
- Wrong client being queried

**Fix**: Check that `$board.data` has content

---

**Case 2: hasBoard: true but no redirect**
```
⏰ Polling for new board... { hasBoard: true, ... }
=== REDIRECT CHECK ===
Current boardId prop: old-board-123
Queried boardId: old-board-123  ❌ SAME BOARD
```
**Issue**: Query returns OLD board, not new one
**Possible Causes**:
- `latest_board_id` not updated on player's chain
- GraphQL caching issue
- Race condition - new board not indexed yet

**Fix**: Need to wait longer or query differently

---

**Case 3: Board data but timestamp mismatch**
```
=== REDIRECT CHECK ===
Queried boardId: new-board-456
createdAt (string): 1730000005000
boardCreationTimestamp: 1730000000000
diff: 5000  ✅ Should match
Is new board?: true  ✅
```
**Issue**: All conditions look good but still no redirect
**Possible Causes**:
- LeaderboardId doesn't match
- Some other condition failing

**Fix**: Check leaderboardId match in logs

---

### 4. What Should Happen (Success Case)

```
⏰ Polling for new board... {
  isNewGameCreated: true,
  hasTimestamp: true,
  hasBoard: true,
  fetching: false,
  error: null
}
=== REDIRECT CHECK ===
Current boardId prop: old-board-123
Queried boardId: new-board-456  ✅ DIFFERENT
createdAt (string): 1730000000500
createdAt (parsed): 1730000000500
boardCreationTimestamp: 1730000000000
diff: 500  ✅ < 10000
leaderboardId match: true  ✅
Is new board?: true  ✅
✅ REDIRECT CONDITIONS MET - Navigating to game...
```

Then browser should navigate to `/game?boardId=new-board-456&leaderboardId=XXX`

---

## Common Issues & Solutions

### Issue 1: Query Returns NULL
```javascript
Queried board data: { board: null }
```

**Root Cause**: The GraphQL `board` query (without boardId parameter) returns null when player has no boards

**Solution**: Check if player has registered and has at least one board

---

### Issue 2: Query Returns Old Board
```javascript
Queried boardId: same-as-current
Is new board?: false  ❌
```

**Root Cause**: `latest_board_id` on player's chain hasn't updated yet

**Solution Options**:
1. Increase polling interval or add retry logic
2. Query player's chain state directly instead of GraphQL
3. Generate expected board ID client-side and poll for it specifically
4. Use subscriptions instead of polling

---

### Issue 3: Timestamp Doesn't Match
```javascript
diff: 50000  ❌ Too large (> 10000)
```

**Root Cause**: Clock skew or wrong timestamp comparison

**Solution**: Increase the 10-second window or use a different matching strategy

---

## Recommended Fix (If Current Approach Fails)

### Option A: Generate Board ID Client-Side
The board ID is deterministic:
```rust
let board_id = format!(
    "{}.{}",
    chain_id,
    hash_seed(&nonce.to_string(), &player, timestamp)
);
```

We could:
1. Get current nonce from player's chain
2. Calculate expected board ID
3. Poll for that specific board ID using `board(boardId: String!)` query

### Option B: Wait for Operation Completion
Instead of polling, listen for the mutation operation to complete, then query once.

### Option C: Backend Returns Board ID
Modify the mutation to return the board ID instead of empty array:
```rust
async fn new_board(...) -> String {
    // ... create board ...
    board_id  // Return this
}
```

---

## Next Steps

1. Test with enhanced logging
2. Share console output
3. Identify which case matches your issue
4. Apply appropriate fix
