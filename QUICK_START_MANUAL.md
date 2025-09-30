# 🚀 U2048 Stress Test - Quick Start Manual

## ⚡ **TL;DR - Run Stress Test in 2 Minutes**

```bash
# 1. Install dependencies (if needed)
pip3 install requests

# 2. Run the stress test
./run_stress_test.sh YOUR_CHAIN_ID YOUR_APP_ID

# 3. Check results in stress_test_results/run_TIMESTAMP/
```

## 📋 **Prerequisites Checklist**

- [ ] **Python 3** installed (`python3 --version`)
- [ ] **K6** installed (`k6 version`) - [Get K6](https://k6.io/docs/get-started/installation/)
- [ ] **requests** module (`pip3 install requests`)
- [ ] **U2048 service** running on localhost:8088
- [ ] **Chain ID and App ID** ready

## 🎯 **Basic Usage Examples**

### **Scenario 1: Quick Test (Recommended for first run)**
```bash
# Small scale test - 15 players, 1 tournament
./run_stress_test.sh 363c9c77abc123... 2519e58e789def... --tournaments 1 --shards 2
```

### **Scenario 2: Medium Scale Test**
```bash
# Medium scale - 30 players, 2 tournaments  
./run_stress_test.sh 363c9c77abc123... 2519e58e789def... --tournaments 2 --shards 4
```

### **Scenario 3: Full Stress Test**
```bash
# Full scale - 30 players, 3 tournaments
./run_stress_test.sh 363c9c77abc123... 2519e58e789def... --tournaments 3 --shards 8
```

### **Scenario 4: Custom Configuration**
```bash
# Maximum intensity - 30 players, 5 tournaments
./run_stress_test.sh 363c9c77abc123... 2519e58e789def... --tournaments 5 --shards 16 --url http://remote:8088
```

## ⚙️ **Command Options**

| Option | Description | Default | Range |
|--------|-------------|---------|-------|
| `--tournaments` | Number of tournaments | 3 | 1-10 |
| `--shards` | Shards per tournament | 8 | 1-32 |
| `--url` | Service URL | http://localhost:8088 | Any URL |

## 📊 **What the Test Does**

1. **Setup Phase** (2-3 minutes)
   - Creates tournaments with specified configuration
   - Validates accessibility and exports config

2. **Stress Phase** (20-25 minutes)
   - Ramps up to 200 concurrent players over 5 minutes
   - Sustains peak load for 10 minutes
   - Ramps down gracefully over 5 minutes

3. **Each Player Simulates:**
   - Registration with unique credentials
   - 2-5 game boards per tournament
   - 20-50 moves per board with realistic patterns
   - Strategic, burst, and panic move sequences

## 🎮 **Move Patterns Simulated**

- **Strategic** (40%): Corner strategies, systematic building
- **Weighted Random** (30%): More Down/Right moves (realistic)
- **Burst** (15%): Rapid move sequences with pauses
- **Panic** (15%): Random desperate moves when struggling

## 📈 **Success Criteria**

✅ **Error Rate** < 5%  
✅ **95th Percentile Response Time** < 2 seconds  
✅ **Player Registration Success** > 95%  
✅ **Board Creation Success** > 95%  

## 🔍 **Reading Results**

### **Console Output**
```
🎯 Target Configuration:
   • Players: 200
   • Tournaments: 3
   • Total Duration: 25 minutes

📊 Performance Metrics:
   • Total Requests: 45,678
   • Request Rate: 87.23 req/s
   • Error Rate: 2.34%
   • 95th Percentile: 1,234.56ms

🏆 Success Criteria:
   • Error rate < 5%: ✅ PASS
   • 95th percentile < 2s: ✅ PASS
```

### **Result Files**
```
stress_test_results/run_20241201_143022/
├── stress_test_config.json     # Tournament setup
├── k6_output.log              # Detailed execution log  
└── stress_test_results.json   # Performance metrics
```

## ⚠️ **Common Issues & Quick Fixes**

### **Issue: "Player not found" errors**
```bash
# Solution: Reduce load and add delays
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 2
```

### **Issue: "Invalid digit" errors**
```bash
# Solution: Check timestamp parsing, restart service if needed
curl -X POST http://localhost:8088/chains/CHAIN_ID/applications/APP_ID \
  -H "Content-Type: application/json" \
  -d '{"query":"{ leaderboards { leaderboardId } }"}'
```

### **Issue: High error rates**
```bash
# Solution: Use conservative settings
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 4
```

### **Issue: Service overload**
```bash
# Emergency stop
pkill -f k6
pkill -f stress_test_coordinator
```

## 🛠 **Manual Component Testing**

### **Test Tournament Coordinator Only**
```bash
python3 stress_test_coordinator.py CHAIN_ID APP_ID --tournaments 1 --output test_config.json
```

### **Test K6 with Existing Config**
```bash
k6 run stress_test_k6.js
```

### **Validate Service Health**
```bash
curl http://localhost:8088/health || echo "Service check your service"
```

## 🔧 **Debugging Mode**

### **Enable Verbose Logging**
```bash
# Edit stress_test_k6.js, uncomment debug lines:
console.log(`[VU ${__VU}] Player ${playerId} action: ...`);
```

### **Monitor System Resources**
```bash
# During test execution:
top -p $(pgrep linera)
netstat -an | grep :8088 | wc -l
```

### **Test Single Player Flow**
```bash
# Manual GraphQL test:
curl -X POST http://localhost:8088/chains/CHAIN_ID/applications/APP_ID \
  -H "Content-Type: application/json" \
  -d '{"query":"mutation { registerPlayer(username:\"test\", passwordHash:\"test\") }"}'
```

## 📋 **Pre-Flight Checklist**

Before running stress test:

- [ ] Service is running and responsive
- [ ] Chain ID and App ID are correct and accessible
- [ ] System has adequate resources (4GB+ RAM recommended)
- [ ] No other heavy processes running
- [ ] Network connection is stable
- [ ] Sufficient disk space for result logs

## 🎯 **Recommended Testing Strategy**

### **Day 1: Baseline Testing**
```bash
# Start small to establish baseline
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 2
```

### **Day 2: Scale Testing**
```bash
# Increase load gradually
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 2 --shards 4
```

### **Day 3: Stress Testing**
```bash
# Full load test
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 3 --shards 8
```

### **Day 4: Peak Testing**
```bash
# Maximum configuration
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 5 --shards 16
```

## 🏆 **Interpreting Success**

### **Excellent Performance**
- Error rate < 2%
- 95th percentile < 1 second
- Request rate > 100 req/s

### **Good Performance**
- Error rate < 5%
- 95th percentile < 2 seconds
- Request rate > 50 req/s

### **Needs Optimization**
- Error rate > 5%
- 95th percentile > 2 seconds
- Frequent timeouts or crashes

---

## 🚨 **Emergency Contacts**

If you encounter issues beyond this guide:
1. Check the detailed `README_STRESS_TEST.md`
2. Review `STRESS_TEST_TROUBLESHOOTING.md`
3. Monitor system logs for specific error messages
4. Consider reducing load parameters for initial testing

**Happy Stress Testing! 🎮⚡**