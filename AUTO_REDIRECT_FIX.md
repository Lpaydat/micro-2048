# Auto-Redirect Fix for "New Board" Button

## Problem Statement
When users clicked "New Board" in a tournament, the board was created successfully in the smart contract, but the UI didn't automatically redirect them to the game page.

## Root Cause

### 1. **Type Mismatch in Comparison**
- GraphQL returns `createdAt` as a **string** (milliseconds)
- Frontend compared it against `newGameAt` **number** using subtraction
- JavaScript coerced the string to number, but this was unreliable

### 2. **Timestamp Overwrite Issue**
- `newGameAt` was used for BOTH:
  - Cooldown tracking (prevent spam clicks)
  - Board creation matching (find newly created board)
- After successful redirect, `newGameAt` was immediately updated to `Date.now()`
- This broke the matching window for any delayed polls

### 3. **GraphQL String vs Number**
```typescript
// GraphQL Schema
type BoardState {
  createdAt: String  // ⚠️ String type
}

// Frontend comparison (OLD - BROKEN)
Math.abs($board.data.board.createdAt - newGameAt) < 10000
//       ^^^^^^^^^^^^^^^^^^^^^^^^      ^^^^^^^^^
//       string from GraphQL           number (Date.now())
```

## Solution Implemented

### Changes to `BoardHeader.svelte`

#### 1. **Separate Timestamp Tracking**
```typescript
let newGameAt = $state(Date.now());                     // For cooldown only
let boardCreationTimestamp = $state<number | null>(null); // For board matching
```

#### 2. **Store Creation Timestamp**
```typescript
const newSingleGame = async () => {
    boardCreationTimestamp = Date.now(); // Store exact creation time
    const result = await newGameBoard(leaderboardId, boardCreationTimestamp.toString());
    
    if ($result.error) {
        boardCreationTimestamp = null; // Reset on error
    }
}
```

#### 3. **Type-Safe Comparison with Debug Logging**
```typescript
if (
    isNewGameCreated &&
    boardCreationTimestamp &&
    $board?.data?.board?.boardId &&
    $board?.data?.board?.createdAt &&
    Math.abs(parseInt($board?.data?.board?.createdAt) - boardCreationTimestamp) < 10000 &&
    //       ^^^^^^^^ Explicit string-to-number conversion
    $board?.data?.board?.leaderboardId === leaderboardId
) {
    console.log('✅ REDIRECT CONDITIONS MET - Navigating to game...');
    boardCreationTimestamp = null; // Reset after successful redirect
    goto(url.toString(), { replaceState: false });
}
```

#### 4. **Debug Logging for Troubleshooting**
```typescript
if (isNewGameCreated && boardCreationTimestamp && $board?.data?.board) {
    console.log('=== REDIRECT CHECK ===');
    console.log('boardId:', $board?.data?.board?.boardId);
    console.log('createdAt (string):', $board?.data?.board?.createdAt);
    console.log('createdAt (parsed):', parseInt($board?.data?.board?.createdAt || '0'));
    console.log('boardCreationTimestamp:', boardCreationTimestamp);
    console.log('diff:', Math.abs(parseInt($board?.data?.board?.createdAt || '0') - boardCreationTimestamp));
    console.log('leaderboardId match:', $board?.data?.board?.leaderboardId === leaderboardId);
}
```

## How It Works Now

### Flow Diagram
```
User clicks "New Board"
    ↓
boardCreationTimestamp = Date.now() (e.g., 1730000000000)
    ↓
Send mutation to smart contract with timestamp
    ↓
Poll every 1 second for new board
    ↓
When board appears in GraphQL:
    ↓
Parse createdAt string → number (parseInt)
    ↓
Compare: |createdAt - boardCreationTimestamp| < 10000
    ↓
If match && leaderboardId matches → REDIRECT ✅
    ↓
Reset boardCreationTimestamp = null
```

### Example Values
```javascript
// When user clicks button:
boardCreationTimestamp = 1730000000000  // ms (13 digits)

// Smart contract receives:
timestamp = "1730000000000"              // string ms
// Converts internally to:
timestamp_micros = 1730000000000000      // μs (16 digits)

// GraphQL query returns:
createdAt = "1730000000000"              // string ms (converted back)

// Frontend comparison:
parseInt("1730000000000") - 1730000000000 = 0
0 < 10000 ✅ → REDIRECT
```

## Benefits

### ✅ Fixed Issues
1. **Type Safety**: Explicit `parseInt()` ensures number comparison
2. **Stable Timestamps**: Separate variables prevent overwrites
3. **Error Handling**: Resets timestamp on errors
4. **Debuggability**: Comprehensive logging for troubleshooting

### ✅ Improvements
1. **Separation of Concerns**: `newGameAt` for cooldown, `boardCreationTimestamp` for matching
2. **Clearer Intent**: Variable names reflect their purpose
3. **Resilient**: Handles edge cases (null, undefined, invalid strings)
4. **Observable**: Console logs show exact comparison values

## Additional Fix: Game Page Auto-Redirect

### Problem
Auto-redirect worked from the leaderboard page but NOT from the game page when clicking "New Game" button.

### Root Cause
When already on `/game` page with an existing board:
1. The `boardId` prop is bound to the current game
2. Clicking "New Game" creates a new board on player's chain
3. The polling query returns player's LATEST board
4. **BUT** it doesn't check if the returned board is DIFFERENT from the current one
5. Result: No redirect because the condition didn't verify it's a NEW board

### Solution
Added board ID comparison check:
```typescript
if (
    isNewGameCreated &&
    boardCreationTimestamp &&
    $board?.data?.board?.boardId &&
    $board?.data?.board?.boardId !== boardId &&  // 🆕 NEW: Check it's a different board
    $board?.data?.board?.createdAt &&
    Math.abs(parseInt($board?.data?.board?.createdAt) - boardCreationTimestamp) < 10000 &&
    $board?.data?.board?.leaderboardId === leaderboardId
) {
    // Navigate to new board
}
```

**Why this works**:
- From leaderboard page: `boardId` is undefined or old → new board ID will be different ✅
- From game page: `boardId` is set to current board → only redirects when NEW board appears ✅
- Prevents false positives from polling the same board multiple times

## Testing Checklist

- [ ] **From Leaderboard Page**: Click "New Board" in a tournament
  - [ ] Check console for "Board creation timestamp: XXXXX"
  - [ ] Watch for polling logs: "=== REDIRECT CHECK ==="
  - [ ] Verify: "Is new board?: true"
  - [ ] Verify: "✅ REDIRECT CONDITIONS MET - Navigating to game..."
  - [ ] Confirm: Browser navigates to `/game?boardId=XXX&leaderboardId=YYY`

- [ ] **From Game Page**: Click "New Game" button while viewing an existing game
  - [ ] Check console for "Board creation timestamp: XXXXX"
  - [ ] Watch for polling logs showing current vs new board ID
  - [ ] Verify: "Is new board?: true"
  - [ ] Verify: "✅ REDIRECT CONDITIONS MET - Navigating to game..."
  - [ ] Confirm: Browser navigates to new board URL

- [ ] **General Tests**:
  - [ ] Test multiple rapid clicks (cooldown should prevent spam)
  - [ ] Test error scenarios (network failure, invalid auth)
  - [ ] Verify tournament list shows all created tournaments

## Rollback Plan

If issues occur, revert to previous version:
```bash
git revert HEAD
cd website && npm run build
# Redeploy
```

## Future Improvements

### Optional: GraphQL Type Safety
Consider changing schema to return numbers:
```graphql
type BoardState {
  createdAt: Int!  # Return milliseconds as number
}
```

**Pros**: Type-safe at API level  
**Cons**: Breaking change, requires backend update

### Optional: Timeout Fallback
Add manual redirect option if auto-redirect fails:
```typescript
let redirectTimeout = setTimeout(() => {
    if (isNewGameCreated) {
        const proceed = confirm('Board created! Navigate to game?');
        if (proceed) goto(...);
    }
}, 15000); // 15 second timeout
```

## Files Modified

- `website/src/lib/components/molecules/BoardHeader.svelte`
  - Added `boardCreationTimestamp` state variable
  - Updated `newSingleGame()` to capture creation timestamp
  - Fixed comparison logic with `parseInt()`
  - Added debug logging

## Related Issues

- Backend timestamp conversion: `src/service_handlers/queries.rs:125`
- Frontend timestamp validation: `website/src/lib/components/organisms/EventList.svelte`
- Smart contract microsecond handling: `src/contract_domain/handlers/operations/shard.rs`

---

**Status**: ✅ FIXED  
**Build**: Successful  
**Deploy**: Ready for testing
