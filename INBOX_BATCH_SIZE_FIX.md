# Leaderboard Refresh Fix: Increase Inbox Batch Size

## Problem

With `--listener-skip-process-inbox` enabled:
- Leaderboard refresh only updates **10 players per click** (not all pending scores)
- Multiple `SubmitScore` messages queue up in the inbox
- Each block processes only **10 messages by default**
- Need to click refresh 10+ times to process 100 messages

## Root Cause

Linera has a default limit on how many messages are processed per block:

```bash
--max-pending-message-bundles <MAX_PENDING_MESSAGE_BUNDLES>
    The maximum number of incoming message bundles to include in a block proposal
    [default: 10]
```

When you call `updateLeaderboard` or `processInbox`:
1. Creates **1 block**
2. Processes **10 messages** (default limit)
3. Remaining messages still queued in inbox

## Solution: Increase Batch Size

Add the `--max-pending-message-bundles` flag when starting the leaderboard service.

**IMPORTANT:** This is a **global Linera flag**, not a `service` subcommand flag. It must come **BEFORE** the `service` keyword.

### Correct Usage

```bash
# ✅ Correct - Global flag before 'service'
linera --max-pending-message-bundles 100 \
  service \
  --port 8080 \
  --listener-skip-process-inbox

# ❌ Wrong - Flag after 'service' won't work
linera service \
  --port 8080 \
  --listener-skip-process-inbox \
  --max-pending-message-bundles 100  # ERROR: unrecognized flag
```

### Recommended Settings

For different scales:

```bash
# Small tournaments (< 50 players)
--max-pending-message-bundles 50

# Medium tournaments (50-200 players)
--max-pending-message-bundles 200

# Large tournaments (200+ players)
--max-pending-message-bundles 500

# Stress testing (thousands of messages)
--max-pending-message-bundles 1000
```

## Example Deployment Commands

### Main Chain
```bash
linera service \
  --port 8080
```

### Leaderboard Chain (with batch processing)
```bash
linera --max-pending-message-bundles 200 \
  service \
  --port 8081 \
  --listener-skip-process-inbox
```

### Player Chain
```bash
linera service \
  --port 8082
```

## How It Works Now

**Before (default 10 messages/block):**
```
Inbox: [msg1, msg2, ..., msg100]
           ↓ Click refresh
Block 1 processes: msg1-msg10  (90 left)
Click refresh again...
Block 2 processes: msg11-msg20 (80 left)
...need 10 clicks total!
```

**After (with --max-pending-message-bundles 200):**
```
Inbox: [msg1, msg2, ..., msg100]
           ↓ Click refresh ONCE
Block 1 processes: msg1-msg100 ✅ (all done!)
```

## Testing

1. **Start leaderboard service with increased limit:**
   ```bash
   linera --max-pending-message-bundles 100 \
     service --port 8081 --listener-skip-process-inbox
   ```

2. **Simulate many score submissions:**
   - Run stress test with 50+ players
   - Check inbox has many queued messages

3. **Click refresh button ONCE**

4. **Expected result:**
   - All 50+ scores update in one batch ✅
   - No need to click multiple times

5. **Verify in logs:**
   ```
   Before: "Processed 10 messages"
   After:  "Processed 50 messages"
   ```

## Discord Reference

From Linera Discord (October 17, 2025):

> **nut1shot:** I test 100 messages it process 10 messages per block  
> **Zhao KK:** With `--max-pending-message-bundles` parameter you can set your own messages limitation of one block.

The flag was moved from `service` subcommand to global Linera options in newer versions.

## Files to Update

### Deployment Scripts

Update any scripts that start the leaderboard service:

```bash
# scripts/start_leaderboard.sh
linera --max-pending-message-bundles 200 \
  service \
  --port 8081 \
  --listener-skip-process-inbox
```

### Documentation

Update `DEPLOYMENT.md` or README with the correct command syntax.

## Trade-offs

### Higher Values
- ✅ Faster batch processing (fewer blocks needed)
- ✅ Better user experience (one click refreshes all)
- ⚠️ Larger block size (more data per block)
- ⚠️ Longer block execution time

### Lower Values  
- ✅ Smaller, faster blocks
- ❌ Need multiple refreshes to process all messages
- ❌ Slower to process large inbox queues

### Recommended Default: 200
- Handles most tournaments (< 200 players)
- Reasonable block size
- Good balance between UX and performance

## Alternative: Multiple processInbox Calls

If you don't want to increase the limit, you could call `processInbox` multiple times in a loop:

```typescript
// Frontend - batch process in chunks
for (let i = 0; i < 10; i++) {
  await processInbox(leaderboardClient);
  await sleep(1000); // Wait between blocks
}
```

But this is **slower** and **less efficient** than just increasing `--max-pending-message-bundles`.

## Related Files

- `ARCHITECTURE_REFACTOR.md` - Message-based architecture overview
- `src/contract_domain/handlers/messages/leaderboard.rs` - Message handler
- `website/src/lib/graphql/mutations/requestLeaderboardRefresh.ts` - Refresh mutation
