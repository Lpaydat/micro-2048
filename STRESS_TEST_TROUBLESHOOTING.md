# 🔧 Stress Test Troubleshooting Guide

## Common Errors and Solutions

### 1. **"Player not found" Errors**

**Symptoms:**
- Players fail to register or authenticate
- GraphQL errors about missing players
- Authentication failures during gameplay

**Root Causes:**
- Player registration timing issues
- Player chain ID resolution problems
- Cache synchronization delays

**Solutions:**

#### Quick Fix:
```bash
# Reduce concurrent load initially
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 4

# Add delays in K6 script
# Edit stress_test_k6.js, increase sleep times:
sleep(Math.random() * 5 + 3); // After registration
sleep(Math.random() * 3 + 2); // Between operations
```

#### Advanced Fix:
```javascript
// In stress_test_k6.js, add retry logic for player operations
const retryPlayerOperation = (operation, maxRetries = 3) => {
    for (let i = 0; i < maxRetries; i++) {
        const result = operation();
        if (result && result.status === 200) {
            return result;
        }
        sleep(1 + i); // Exponential backoff
    }
    return null;
};
```

### 2. **"Invalid digit" Errors (Line 53 in game.rs)**

**Symptoms:**
- Crashes during board creation or move processing
- Parse errors in timestamp handling
- Random number generation failures

**Root Cause Analysis:**

The error likely occurs in:
```rust
// src/service_handlers/mutations.rs:36
timestamp: timestamp.parse::<u64>().unwrap(),
```

Or in the random number generation:
```rust
// src/game.rs:404 (spawn_tile function)
let mut idx = rnd_range(board_id, username, timestamp, 0, Self::count_empty(board));
```

**Critical Issues:**

#### Issue 1: Division by Zero in Random Range
```rust
// src/random.rs:14
(seed % (max - min)) + min  // CRASHES if max == min
```

**When this happens:**
- Empty board: `count_empty(board)` returns 0
- Call: `rnd_range(..., 0, 0)` → Division by zero!

#### Issue 2: Invalid Timestamps
- K6 sends timestamps as strings
- Some may be malformed or too large
- `timestamp.parse::<u64>().unwrap()` panics

#### Issue 3: Infinite Loop Potential
```rust
// src/game.rs:407-421 (spawn_tile)
loop {
    while (tmp & 0xF) != 0 {
        tmp >>= 4;
        t <<= 4;
    }
    // Could loop forever if idx >= count_empty
}
```

**Solutions:**

#### Immediate Fix (Add to codebase):
```rust
// Fix random range division by zero
pub fn rnd_range(board_id: &str, username: &str, timestamp: u64, min: u32, max: u32) -> u32 {
    if max <= min {
        return min; // Prevent division by zero
    }
    let seed = hash_seed(board_id, username, timestamp);
    (seed % (max - min)) + min
}

// Fix spawn_tile bounds checking
pub fn spawn_tile(board_id: &str, username: &str, timestamp: u64, board: u64) -> u64 {
    let empty_count = Self::count_empty(board);
    if empty_count == 0 {
        return 0; // No empty tiles, can't spawn
    }
    
    let mut tmp = board;
    let mut idx = rnd_range(board_id, username, timestamp, 0, empty_count);
    let mut t = Self::tile(board_id, username, timestamp);
    
    // Add safety counter to prevent infinite loops
    let mut safety_counter = 0;
    loop {
        if safety_counter > 16 { // Max 16 tiles on board
            break;
        }
        
        while (tmp & 0xF) != 0 {
            tmp >>= 4;
            t <<= 4;
        }

        if idx == 0 {
            break;
        } else {
            idx -= 1
        }

        tmp >>= 4;
        t <<= 4;
        safety_counter += 1;
    }

    t
}

// Fix timestamp parsing
timestamp: timestamp.parse::<u64>().map_err(|e| {
    format!("Invalid timestamp: {}", e)
})?,
```

### 3. **Self-Healing Triggers / Infinite Loops**

**Potential Infinite Loop Sources:**

#### Event Streaming Loops:
```rust
// If events trigger more events infinitely
Player Move → Shard Update → Leaderboard Trigger → Shard Emit → Leaderboard Process → Repeat
```

#### Tournament Cache Refresh Loops:
```rust
// If tournament updates trigger more updates
Tournament Update → Cache Invalidation → Re-fetch → Update Again → Repeat
```

**Detection:**
```bash
# Monitor for excessive CPU usage
top -p $(pgrep linera)

# Check for rapid log generation
tail -f /path/to/linera/logs | grep -E "(trigger|event|update)" | wc -l

# Monitor request patterns
netstat -an | grep :8088 | wc -l
```

**Solutions:**

#### Immediate Circuit Breaker:
```bash
# Add request rate limiting to stress test
# Edit stress_test_k6.js:

// Add global rate limiter
const rateLimiter = {
    lastRequest: 0,
    minInterval: 100, // Minimum 100ms between requests
    
    wait() {
        const now = Date.now();
        const elapsed = now - this.lastRequest;
        if (elapsed < this.minInterval) {
            sleep((this.minInterval - elapsed) / 1000);
        }
        this.lastRequest = Date.now();
    }
};

// Use before each request:
rateLimiter.wait();
const response = makeGraphQLRequest(...);
```

### 4. **Performance Optimization for High Load**

#### Reduce Stress Test Intensity:
```bash
# Conservative settings for initial testing
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 2 --shards 4

# Modified K6 stages for gentler load:
stages: [
    { target: 20, duration: '3m' },   // Start very low
    { target: 50, duration: '5m' },   // Gentle increase
    { target: 50, duration: '5m' },   // Sustain medium load
    { target: 20, duration: '2m' }    // Ramp down
]
```

#### Optimize Move Patterns:
```javascript
// Reduce move frequency
const TOTAL_MOVES = 20; // Instead of 50
const BATCH_SIZE = 5;   // Instead of 25

// Add longer pauses
sleep(Math.random() * 5 + 3); // 3-8 second pauses
```

### 5. **Emergency Stop Procedures**

#### Stop Stress Test:
```bash
# Kill K6 process
pkill -f k6

# Kill Python coordinator
pkill -f stress_test_coordinator

# Clean up processes
ps aux | grep -E "(k6|stress_test)" | awk '{print $2}' | xargs kill
```

#### Reset System State:
```bash
# Restart Linera service (if accessible)
sudo systemctl restart linera

# Clear temporary files
rm -f stress_test_config.json
rm -rf stress_test_results/run_$(date +%Y%m%d)*
```

### 6. **Debugging Commands**

#### Monitor System Health:
```bash
# Check memory usage
free -h

# Check CPU usage
iostat 1

# Check network connections
ss -tuln | grep 8088

# Monitor logs (if accessible)
journalctl -u linera -f
```

#### Validate Configuration:
```bash
# Test single player flow manually
python3 -c "
import requests
response = requests.post('http://localhost:8088/chains/CHAIN_ID/applications/APP_ID', 
    json={'query': '{ leaderboards { leaderboardId name } }'})
print(response.json())
"
```

#### Check Tournament State:
```bash
# Query tournament status
curl -X POST http://localhost:8088/chains/CHAIN_ID/applications/APP_ID \
  -H "Content-Type: application/json" \
  -d '{"query":"{ leaderboards { leaderboardId name totalBoards totalPlayers } }"}' | jq
```

### 7. **Safe Testing Approach**

#### Phase 1: Single Player Test
```bash
# Test with just 1 player first
# Edit stress_test_k6.js:
startVUs: 1,
stages: [{ target: 1, duration: '2m' }]
```

#### Phase 2: Small Group Test
```bash
# Test with 10 players
startVUs: 5,
stages: [
    { target: 10, duration: '2m' },
    { target: 10, duration: '3m' }
]
```

#### Phase 3: Gradual Scale Up
```bash
# Only proceed if previous phases succeed
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 2  # 20 players
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 2 --shards 4  # 50 players  
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 3 --shards 8  # 200 players
```

### 8. **Code Fixes Priority List**

**High Priority (Immediate):**
1. Fix division by zero in `rnd_range`
2. Add bounds checking in `spawn_tile`
3. Add timestamp parsing error handling
4. Add safety counters to prevent infinite loops

**Medium Priority:**
1. Add retry logic for player operations
2. Implement circuit breakers for event loops
3. Add request rate limiting

**Low Priority:**
1. Optimize move pattern generation
2. Add comprehensive logging
3. Implement graceful degradation

Use this troubleshooting guide to systematically identify and resolve issues during stress testing!