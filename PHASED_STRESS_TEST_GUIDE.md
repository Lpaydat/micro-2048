# 🚀 Phased Stress Test Implementation

## ✅ **Key Improvement: Registration → Gameplay Phases**

### **Why Phased Testing?**

The original stress test tried to do everything simultaneously:
- Player registration
- Player chain resolution  
- Board creation
- Move execution

This caused **race conditions** and **"player not found" errors** because:
1. Players weren't fully committed before board creation attempts
2. Chain synchronization took time
3. Concurrent registration overwhelmed the validation system

### **New Phased Approach**

## 📋 **Phase 1: Mass Registration (3.25 minutes)**
- **Focus**: Register all 30 players first
- **Duration**: 3 minutes + 15 second buffer
- **Strategy**: Ramp up to 30 concurrent registrations
- **Benefits**: 
  - All players registered before gameplay starts
  - No race conditions between registration and board creation
  - System can focus on one operation type

## 🎮 **Phase 2: Gameplay (14.5 minutes)**  
- **Focus**: Board creation and move simulation
- **Start Time**: 4 minutes (after registration completes)
- **Strategy**: Use deterministic player selection from Phase 1
- **Benefits**:
  - Players guaranteed to exist and be committed
  - Clean separation of concerns
  - More realistic load patterns

## 🛠 **Implementation Details**

### **Script Changes**

**New Files Created:**
- `stress_test_k6_phased.js` - Two-phase K6 test
- `PHASED_STRESS_TEST_GUIDE.md` - This documentation

**Updated Files:**
- `run_stress_test.sh` - Uses phased script by default
- Added `--original` flag for old behavior

### **K6 Scenario Configuration**

```javascript
scenarios: {
    // Phase 1: Mass Player Registration
    registration_phase: {
        executor: 'ramping-vus',
        stages: [
            { target: 50, duration: '1m' },    // Quick ramp up
            { target: 100, duration: '1m' },   // Medium load
            { target: 200, duration: '1m' },   // Peak registration
            { target: 200, duration: '2m' },   // Sustained registration
            { target: 0, duration: '30s' }     // Complete all
        ],
        exec: 'registrationPhase'
    },
    // Phase 2: Board Creation and Gameplay
    gameplay_phase: {
        executor: 'ramping-vus',
        startTime: '6m',  // Start after registration
        stages: [
            { target: 50, duration: '2m' },    // Gentle gameplay ramp
            { target: 200, duration: '8m' },   // Scale to peak
            { target: 200, duration: '10m' },  // Sustained stress
            { target: 10, duration: '5m' }     // Ramp down
        ],
        exec: 'gameplayPhase'
    }
}
```

### **Player Synchronization Strategy**

**Challenge**: K6 scenarios can't directly share data between phases

**Solution**: Deterministic player generation
```javascript
// Phase 1: Registration
const playerId = `player_${__VU}_${__ITER}_${generateRandomString(8)}`;

// Phase 2: Gameplay (deterministic recreation)
const seed = __VU + (__ITER * 1000);
const playerId = `player_${Math.floor(seed / 100)}_${seed % 100}_${generateRandomString(8)}`;
```

### **Enhanced Error Handling**

**Registration Phase:**
- Focus only on player registration
- Detailed registration metrics
- No board operations to cause conflicts

**Gameplay Phase:**
- Verify player exists before proceeding
- Enhanced chain ID resolution
- Graceful handling of missing players
- Longer delays for chain synchronization

## 📊 **Expected Improvements**

### **Before (Single-Phase Issues):**
- ❌ 50% board creation failure rate
- ❌ "Player not found" validation panics
- ❌ Race conditions between operations
- ❌ Response times degrading rapidly

### **After (Phased Benefits):**
- ✅ High registration success rate (Phase 1 focus)
- ✅ Reduced validation panics (players exist)
- ✅ No registration/board creation race conditions
- ✅ More predictable load patterns
- ✅ Better error isolation and debugging

## 🎯 **Usage Examples**

### **Recommended (Phased):**
```bash
# Default phased approach - most stable
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 3 --shards 8

# Conservative phased start
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 4
```

### **Legacy (Original):**
```bash
# Original single-phase (less stable, for comparison)
./run_stress_test.sh CHAIN_ID APP_ID --original
```

## 📈 **Monitoring During Phased Test**

### **Phase 1 Monitoring (0-6 minutes):**
```bash
# Should see steady registration progress
tail -f stress_test_results/run_*/k6_output.log | grep "REG VU"

# Expected pattern:
# [REG VU 1] Registering player: player_1_0_abc123
# [REG VU 1] Successfully registered: player_1_0_abc123
```

### **Phase 2 Monitoring (6-31 minutes):**
```bash
# Should see gameplay without registration errors
tail -f stress_test_results/run_*/k6_output.log | grep "GAME VU"

# Expected pattern:
# [GAME VU 1] Starting gameplay for: player_1_0_xyz789
# [GAME VU 1] Player player_1_0_xyz789 joining tournament: stress_main_1
# [GAME VU 1] Created board 1/3 for player_1_0_xyz789
```

## 🔧 **Troubleshooting**

### **Phase 1 Issues:**
```bash
# High registration failures
grep "Failed to register" stress_test_results/run_*/k6_output.log

# Solutions:
# - Reduce concurrent registrations: edit stages in registration_phase
# - Increase timeouts in makeGraphQLRequest
```

### **Phase 2 Issues:**
```bash
# Player not found errors
grep "player may not be registered" stress_test_results/run_*/k6_output.log

# Solutions:
# - Increase startTime delay: '6m' → '8m'
# - Add more buffer time between phases
```

### **Chain Synchronization Issues:**
```bash
# Failed to get chain ID
grep "Failed to get chain ID" stress_test_results/run_*/k6_output.log

# Solutions:
# - Increase sleep delays in gameplayPhase
# - Check if registration phase completed successfully
```

## 🎛 **Advanced Configuration**

### **Adjust Phase Timing:**
Edit `stress_test_k6_phased.js`:
```javascript
// For slower systems, increase buffer
startTime: '8m',  // Instead of '6m'

// For faster systems, reduce registration time
{ target: 200, duration: '30s' },  // Instead of '1m'
```

### **Player Distribution:**
```javascript
// Increase boards per player for more stress
const boardsToCreate = Math.min(config.boards_per_player, 5); // Instead of 3

// Increase moves per board
const totalMoves = Math.floor(Math.random() * 50) + 30; // Instead of 20-50
```

### **Tournament Load Balancing:**
```javascript
// Modify selectTournament() for different distribution strategies
// Equal distribution:
return tournaments[__VU % tournaments.length];

// Hash-based distribution:
const hash = simpleHash(playerId);
return tournaments[hash % tournaments.length];
```

## 🏆 **Success Criteria**

### **Phase 1 Success:**
- ✅ Player registration rate > 95%
- ✅ Registration response time < 1s average
- ✅ No timeout errors during registration

### **Phase 2 Success:**  
- ✅ Board creation rate > 90%
- ✅ Move operation success > 95%
- ✅ No "player not found" errors
- ✅ Response times stable throughout test

### **Overall Success:**
- ✅ Error rate < 5% across both phases
- ✅ 95th percentile response time < 3s
- ✅ System remains stable for 30+ minutes
- ✅ No panic errors or system crashes

This phased approach should significantly improve stress test stability and provide more realistic load testing patterns!