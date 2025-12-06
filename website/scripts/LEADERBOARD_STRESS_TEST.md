# 🎯 Leaderboard Stress Test

A focused stress testing script specifically designed to test the leaderboard triggerer system, score updates, and concurrent player activity.

## 🎮 Test Flow

```
1. Register N players (configurable)
   ↓
2. Wait X seconds for registration propagation
   ↓
3. Each player creates a board in the tournament
   ↓
4. Each player plays batches of moves every Y seconds
   ↓
5. Repeat from step 3 for multiple cycles
```

## 🚀 Quick Start

### Prerequisites

1. **Create a tournament first** (via UI or API)
2. **Get the tournament ID** (you'll need this)
3. **Have k6 installed**: `brew install k6` or see [k6.io](https://k6.io/docs/getting-started/installation/)

### Basic Usage

```bash
# Local testing with 10 players
ENVIRONMENT=local \
TOURNAMENT_ID="your_tournament_id_here" \
NUM_PLAYERS=10 \
k6 run website/scripts/leaderboard-stress-test.ts
```

```bash
# Production testing with 50 players
ENVIRONMENT=production \
TOURNAMENT_ID="your_tournament_id_here" \
NUM_PLAYERS=50 \
k6 run website/scripts/leaderboard-stress-test.ts
```

## ⚙️ Configuration Parameters

| Variable | Default | Description |
|----------|---------|-------------|
| `ENVIRONMENT` | `production` | `local` or `production` |
| `TOURNAMENT_ID` | *(required)* | Tournament/leaderboard ID to join |
| `NUM_PLAYERS` | `20` | Number of mock players to create |
| `GAMES_PER_CYCLE` | `3` | Games each player creates per cycle |
| `MOVES_PER_BATCH` | `10` | Number of moves in each batch |
| `BATCH_INTERVAL` | `5` | Seconds between batches |
| `BATCHES_PER_GAME` | `5` | Number of batches per game |
| `REGISTRATION_WAIT` | `10` | Seconds to wait after registration |

## 📊 Example Scenarios

### Scenario 1: Light Load (Join & Play Along)

```bash
# 10 players, slow gameplay
TOURNAMENT_ID="abc123..." \
NUM_PLAYERS=10 \
GAMES_PER_CYCLE=2 \
BATCH_INTERVAL=10 \
k6 run website/scripts/leaderboard-stress-test.ts
```

**What this does:**
- 10 concurrent players
- Each creates 2 boards
- 5 batches of 10 moves per board
- 10 seconds between batches
- Good for playing alongside in UI

### Scenario 2: Medium Load (Test Triggerer System)

```bash
# 30 players, moderate speed
TOURNAMENT_ID="abc123..." \
NUM_PLAYERS=30 \
GAMES_PER_CYCLE=3 \
BATCH_INTERVAL=5 \
BATCHES_PER_GAME=5 \
k6 run website/scripts/leaderboard-stress-test.ts
```

**What this does:**
- 30 concurrent players
- Each creates 3 boards
- 5 batches of 10 moves per board (50 moves total)
- 5 seconds between batches
- Tests triggerer pool rotation

### Scenario 3: High Load (Stress Test)

```bash
# 100 players, fast gameplay
TOURNAMENT_ID="abc123..." \
NUM_PLAYERS=100 \
GAMES_PER_CYCLE=5 \
BATCH_INTERVAL=3 \
BATCHES_PER_GAME=10 \
MOVES_PER_BATCH=20 \
k6 run website/scripts/leaderboard-stress-test.ts
```

**What this does:**
- 100 concurrent players
- Each creates 5 boards
- 10 batches of 20 moves per board (200 moves total)
- 3 seconds between batches
- Maximum stress on triggerer system

### Scenario 4: Endurance Test

```bash
# Long-running test with many cycles
TOURNAMENT_ID="abc123..." \
NUM_PLAYERS=50 \
GAMES_PER_CYCLE=10 \
BATCH_INTERVAL=5 \
BATCHES_PER_GAME=8 \
k6 run website/scripts/leaderboard-stress-test.ts
```

**What this does:**
- 50 concurrent players
- Each creates 10 boards (long test)
- 8 batches of 10 moves per board (80 moves total)
- Tests system stability over time

## 📈 Monitoring & Metrics

### What to Watch While Testing

1. **Leaderboard UI** - Open the tournament leaderboard in your browser
   - Watch scores update in real-time
   - Note update frequency
   - Check if all players appear

2. **Player Activity** - Monitor the script output
   ```
   🎯 [Player 1/50] Starting leaderboard stress test
   📝 [StressPlayer_1_1234567890] Registering player...
   ✅ [StressPlayer_1_1234567890] Registered successfully
   🔗 [StressPlayer_1_1234567890] Chain ID: 7b9613d4da9ea6ad...
   🎮 [StressPlayer_1_1234567890] Starting game cycle: 3 games, 5 batches each
   ```

3. **K6 Metrics** - Watch the k6 output
   ```
   http_req_duration..........: avg=1.2s  p(95)=3.5s
   http_req_failed............: 5.23%
   iterations.................: 50/50
   ```

4. **System Logs** - If you have access
   - Check triggerer activity
   - Monitor tier escalations
   - Watch for errors

### Expected Behavior

✅ **Good Signs:**
- All players register successfully
- Boards create without errors
- Moves process consistently
- Leaderboard updates within 5-15 seconds
- No tier 6 emergency mode activations

⚠️ **Warning Signs:**
- Registration failures > 10%
- Move batch failures > 15%
- Leaderboard not updating for > 30 seconds
- Tier 6 emergency mode activating

❌ **Critical Issues:**
- Massive registration failures (> 50%)
- Leaderboard frozen for > 1 minute
- System errors or crashes
- Triggerer system completely broken

## 🔍 Troubleshooting

### Issue: "TOURNAMENT_ID is required"

**Solution:** Make sure you set the `TOURNAMENT_ID` environment variable:
```bash
TOURNAMENT_ID="your_actual_tournament_id" k6 run website/scripts/leaderboard-stress-test.ts
```

### Issue: High registration failure rate

**Possible causes:**
- Too many concurrent registrations (reduce `NUM_PLAYERS`)
- Network issues
- Server capacity

**Solution:** Start with fewer players (e.g., 10) and gradually increase

### Issue: Move batches failing

**Possible causes:**
- Invalid board state
- Timeout issues
- Server overload

**Solution:** 
- Increase `BATCH_INTERVAL` to reduce load
- Check server logs for errors
- Reduce `NUM_PLAYERS`

### Issue: Leaderboard not updating

**This is what we're testing!** Notes to check:
- Are triggerers being selected?
- Is tier 6 activating?
- Are ShardScoreUpdate events being emitted?
- Check player chain and shard chain logs

## 💡 Tips for Effective Testing

1. **Start Small** - Begin with 10 players to ensure basic functionality
2. **Gradual Increase** - Double player count each test (10 → 20 → 40 → 80)
3. **Monitor First** - Watch the first few cycles before walking away
4. **Play Along** - Join the tournament yourself to see real-time updates
5. **Document Findings** - Note when issues appear (e.g., "leaderboard stopped updating at 75 players")

## 🎯 Test Objectives

Use this script to answer:

- ✅ **Does the triggerer system work?**
  - Are players being selected as triggerers?
  - Does the pool rotate properly?
  - Do backups take over when needed?

- ✅ **What's the update latency?**
  - How long from move → leaderboard shows new score?
  - Does latency increase with player count?

- ✅ **What's the breaking point?**
  - How many concurrent players before issues?
  - What fails first (registration, moves, leaderboard)?

- ✅ **Does tier escalation work?**
  - Do higher tiers activate under load?
  - Does tier 6 emergency mode save the day?

## 📝 Example Test Session

```bash
# Test 1: Baseline (10 players)
TOURNAMENT_ID="abc123" NUM_PLAYERS=10 k6 run website/scripts/leaderboard-stress-test.ts
# Result: ✅ All good, updates within 5 seconds

# Test 2: Medium load (30 players)
TOURNAMENT_ID="abc123" NUM_PLAYERS=30 k6 run website/scripts/leaderboard-stress-test.ts
# Result: ✅ Still good, updates within 10 seconds

# Test 3: Heavy load (50 players)
TOURNAMENT_ID="abc123" NUM_PLAYERS=50 k6 run website/scripts/leaderboard-stress-test.ts
# Result: ⚠️ Tier 2 activated, updates within 15 seconds

# Test 4: Stress test (100 players)
TOURNAMENT_ID="abc123" NUM_PLAYERS=100 k6 run website/scripts/leaderboard-stress-test.ts
# Result: ❌ Leaderboard frozen, tier 6 activated but not helping
```

## 🤝 Playing Along

**Best scenario for human participation:**

```bash
# Terminal 1: Start bots
TOURNAMENT_ID="your_tournament_id" \
NUM_PLAYERS=20 \
BATCH_INTERVAL=8 \
k6 run website/scripts/leaderboard-stress-test.ts

# Terminal 2: Join the tournament in your browser
# - Register as yourself
# - Create boards
# - Make moves
# - Watch leaderboard update with your score among the bots
```

This gives you a realistic feel for how the system performs under load!
