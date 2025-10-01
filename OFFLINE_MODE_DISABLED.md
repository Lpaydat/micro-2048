# Offline Mode Disabled for Website

## Changes Made

Offline mode has been disabled for the website version of the game. All moves are now always submitted to the smart contract immediately.

## Files Modified

### `website/src/lib/components/organisms/Game.svelte`

**Changes**:

1. **Made `offlineMode` a constant**:
   ```typescript
   // Before: let offlineMode = false;
   // After:
   const offlineMode = false;
   ```

2. **Removed localStorage initialization**:
   ```typescript
   // Removed:
   // offlineMode = localStorage.getItem('offlineModePreference') !== 'false';
   ```

3. **Hidden offline mode toggle button**:
   ```svelte
   {#if false}
     <button onclick={toggleOfflineMode}>
       <!-- Offline/Online toggle button -->
     </button>
   {/if}
   ```

4. **Removed toggle function**:
   ```typescript
   // Removed: const toggleOfflineMode = () => { ... }
   ```

5. **Simplified sync status display**:
   ```svelte
   <!-- Before: {offlineMode ? 'Offline' : syncStatus} -->
   <!-- After: -->
   {syncStatus}
   ```

6. **Simplified submitMoves logic**:
   ```typescript
   // Before: if ((moves.length > 0 || force) && !offlineMode)
   // After: if ((moves.length > 0 || force))
   ```

## Behavior Changes

### Before (with offline mode):
- User could toggle offline mode
- In offline mode, moves were stored locally and not submitted
- User had to manually go online to submit moves
- "Offline" status shown in UI

### After (offline mode disabled):
- Moves are ALWAYS submitted to smart contract
- No offline/online toggle button
- Sync status shows only: `idle`, `syncing`, `synced`, or `failed`
- Simpler, more predictable UX

## Why This Change?

1. **Blockchain-first approach**: The game is designed to work on-chain
2. **Avoid confusion**: Offline mode could lead to lost progress if user forgets to sync
3. **Simpler UX**: One less thing for users to manage
4. **Better for tournaments**: All moves are recorded in real-time on-chain

## Technical Notes

- The `offlineMode` constant is kept in code (set to `false`) to avoid refactoring all conditional checks
- JavaScript compiler will optimize away `if (false)` blocks
- All move submission logic remains intact, just without the offline bypass

## If You Want to Re-enable Offline Mode

1. Change `const offlineMode = false` to `let offlineMode = false`
2. Remove `{#if false}` wrapper around toggle button
3. Uncomment localStorage initialization
4. Restore toggle function

## Testing

✅ Moves are submitted immediately  
✅ No offline toggle button visible  
✅ Sync status displays correctly  
✅ Game plays normally without offline mode  
