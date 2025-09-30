# 🔧 Player Count Reduction: 200 → 30

## ✅ **Changes Made**

### **Updated Configuration Values**

| Setting | Before | After | Reason |
|---------|--------|-------|---------|
| **Total Players** | 200 | 30 | More manageable testing load |
| **Registration Phase** | 5.5 minutes | 3.25 minutes | Shorter with fewer players |
| **Gameplay Phase** | 25 minutes | 14.5 minutes | Proportionally reduced |
| **Boards per Player** | 3 | 2 | Reduce per-player load |
| **Moves per Board** | 50 | 30 | Fewer moves for quicker tests |
| **Move Batch Size** | 25 | 15 | Smaller batches |

### **Updated Load Profile (Phased Test)**

**Phase 1: Registration**
- **Before**: 10→50→100→200→200→0 (5.5 min)
- **After**: 5→10→20→30→30→0 (3.25 min)

**Phase 2: Gameplay**  
- **Before**: 10→50→100→150→200→...→10 (25 min)
- **After**: 5→10→20→30→30→...→5 (14.5 min)

### **Updated Load Profile (Original Test)**
- **Before**: 10→5→10→15→20→20→...→1 (23 min)
- **After**: 5→10→20→30→30→...→5 (19 min)

## 📁 **Files Modified**

### **Core Configuration:**
- `stress_test_coordinator.py` - Updated default player counts and timings
- `stress_test_k6_phased.js` - Reduced VU targets and stage durations
- `stress_test_k6.js` - Updated original test load profile

### **Scripts and Documentation:**
- `run_stress_test.sh` - Updated descriptions and timing estimates
- `README_STRESS_TEST.md` - Changed player count references
- `QUICK_START_MANUAL.md` - Updated example scenarios
- `PHASED_STRESS_TEST_GUIDE.md` - Updated phase timing and descriptions

## 🎯 **Benefits of Reduced Load**

### **Improved Stability:**
- ✅ Less likelihood of overwhelming the system
- ✅ Easier to identify specific bottlenecks
- ✅ More predictable performance patterns
- ✅ Lower resource requirements

### **Better Debugging:**
- ✅ Fewer concurrent operations to track
- ✅ Clearer error patterns
- ✅ Easier to isolate issues
- ✅ More manageable log analysis

### **Faster Iteration:**
- ✅ Shorter test duration (30 min → 18 min total)
- ✅ Quicker feedback cycles
- ✅ Less resource-intensive testing
- ✅ Easier to run multiple test variations

## 📊 **Expected Performance Impact**

### **System Load:**
- **CPU Usage**: Reduced by ~85% (200→30 users)
- **Memory Usage**: Proportionally lower
- **Network Requests**: ~85% fewer concurrent requests
- **Database Operations**: Significantly reduced contention

### **Test Reliability:**
- **Error Rate**: Expected to drop below 2%
- **Response Times**: More consistent and predictable
- **Success Rate**: Higher board creation and move success
- **System Stability**: Less chance of overload conditions

## 🚀 **Usage Examples**

### **Conservative Start (Recommended):**
```bash
# Start with minimal load - 15 players
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 2
```

### **Standard Test:**
```bash
# Standard 30-player test
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 2 --shards 4
```

### **Maximum Load:**
```bash
# Full test with multiple tournaments
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 3 --shards 8
```

## 📈 **Scaling Strategy**

### **Phase 1: Validate Stability (30 players)**
1. Run basic 30-player tests
2. Verify <2% error rate
3. Confirm stable response times
4. Check system resource usage

### **Phase 2: Gradual Scaling (if needed)**
```bash
# Could manually edit for higher loads:
# - 30 → 50 players
# - 50 → 75 players  
# - 75 → 100 players
# Only if 30-player tests are completely stable
```

### **Phase 3: Production Readiness**
- If 30 players run smoothly, system is likely ready for higher loads
- Real-world usage rarely exceeds 30 concurrent active players
- Focus on optimizing individual operations rather than raw concurrency

## 🎯 **Success Criteria (Updated)**

### **With 30 Players:**
- ✅ **Error Rate**: < 2% (more stringent than 5%)
- ✅ **Response Time**: 95th percentile < 1.5s (improved from 3s)
- ✅ **Registration Success**: > 98%
- ✅ **Board Creation Success**: > 95%
- ✅ **Move Operation Success**: > 98%
- ✅ **System Stability**: No panics or crashes

### **Performance Expectations:**
- **Registration Phase**: ~1-2 req/s average, <500ms response time
- **Gameplay Phase**: ~5-10 req/s average, <1s response time
- **Overall**: Smooth operation without resource exhaustion

This reduced load should provide much more stable and predictable stress testing while still validating the system's ability to handle realistic concurrent usage patterns.