# New Board UX Improvements

## Changes Implemented

### 1. Removed "Wait a sec!" Message
**Before**: Showed annoying cooldown message when clicking too fast
**After**: Smoothly disabled button with visual feedback

### 2. Free Board Creation
**Before**: 10-second cooldown between board creations
**After**: Can create boards freely, but button is disabled while creating

### 3. Board Moves Disabled While Creating
**Before**: Could make moves while creating new board (confusing state)
**After**: Board is locked during creation (prevents confusion)

### 4. 5-Second Button Cooldown
**Before**: Only disabled while actively polling
**After**: Button stays disabled for 5 seconds OR until redirect (whichever comes first)

## Technical Implementation

### BoardHeader.svelte

**State Management**:
```typescript
let boardCreationStartTime = $state<number | null>(null);
let isNewGameCreated = $state(false);

const isCreatingBoard = $derived(
    isNewGameCreated || 
    (boardCreationStartTime && Date.now() - boardCreationStartTime < 5000)
);

// Export state to parent
$effect(() => {
    isCreating = isCreatingBoard;
});
```

**Button State**:
```svelte
<button
    onclick={newSingleGame}
    disabled={isCreatingBoard}
    class="... {isCreatingBoard ? 'bg-[#9f8a76] cursor-not-allowed opacity-70' : 'bg-[#8f7a66]'}"
>
    {#if isCreatingBoard}
        <span class="animate-pulse">Creating...</span>
    {:else}
        <span>New Game</span>
    {/if}
</button>
```

### Game.svelte

**Propagate Creating State**:
```typescript
let isCreatingNewBoard = $state(false);

<Board
    canMakeMove={canMakeMove &&
        !boardEnded &&
        $game.data?.board?.player === $userStore.username &&
        !isFrozen &&
        !isCreatingNewBoard}  // 🆕 Disable moves during creation
    ...
>
    <BoardHeader
        bind:isCreating={isCreatingNewBoard}  // 🆕 Bind creation state
        ...
    />
</Board>
```

## User Experience Flow

### Creating New Board:

1. **User clicks "New Game"**
   - Button shows "Creating..." with pulse animation
   - Button becomes disabled and grayed out
   - Board moves are locked

2. **Board creation in progress**
   - Mutation sent to smart contract
   - 3-second wait for state propagation
   - Polling starts for new board

3. **New board found**
   - Auto-redirect to new board
   - Button re-enables
   - Board moves unlock

4. **Timeout scenario (5 seconds)**
   - If redirect hasn't happened in 5 seconds
   - Button automatically re-enables
   - User can try again or manually navigate

## Visual States

### Button States:
- **Normal**: Green background `#8f7a66`, clickable
- **Creating**: Lighter background `#9f8a76`, disabled, "Creating..." text with pulse
- **Hover** (when enabled): Slightly lighter `#9f8a76`

### Board States:
- **Normal**: Moves enabled
- **Creating**: Moves disabled (same as frozen state)
- **Ended**: Moves disabled

## Benefits

✅ No more annoying messages  
✅ Clear visual feedback during creation  
✅ Prevents confusion with disabled board  
✅ Auto-recovery if creation takes too long  
✅ Smooth UX without artificial delays  
✅ Can create multiple boards quickly (after redirect)

## Edge Cases Handled

1. **Creation fails**: Button re-enables, user can retry
2. **Slow network**: 5-second timeout ensures button doesn't stay disabled forever
3. **Multiple clicks**: Disabled state prevents duplicate creations
4. **Redirect delay**: Board stays locked until redirect completes

## Files Modified

- `website/src/lib/components/molecules/BoardHeader.svelte` - Button state and export
- `website/src/lib/components/organisms/Game.svelte` - Receive creating state, disable moves
