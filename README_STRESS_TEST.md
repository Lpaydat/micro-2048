# U2048 Stress Test Suite

A comprehensive stress testing suite for the U2048 leaderboard system that simulates **30 concurrent players** across multiple tournaments with realistic gaming patterns.

## 🎯 **Overview**

This stress test suite validates the performance and scalability of the U2048 leaderboard system under high concurrent load. It consists of three main components:

1. **Tournament Coordinator** (Python) - Sets up shared tournaments and configuration
2. **Stress Test Runner** (K6) - Simulates 30 concurrent players with realistic gameplay
3. **Orchestration Script** (Bash) - Coordinates the entire testing process

## 🚀 **Quick Start**

### Prerequisites

- **Python 3.x** with `requests` module
- **K6** load testing tool ([installation guide](https://k6.io/docs/get-started/installation/))
- **bc** calculator (usually pre-installed on Linux/macOS)
- **jq** JSON processor (optional, for enhanced result analysis)

```bash
# Install Python dependencies
pip3 install requests

# Install K6 (example for Ubuntu/Debian)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Install jq for result analysis (optional)
sudo apt-get install jq bc
```

### Running the Stress Test

```bash
# Basic usage
./run_stress_test.sh <CHAIN_ID> <APP_ID>

# With custom configuration
./run_stress_test.sh <CHAIN_ID> <APP_ID> --tournaments 5 --shards 16

# Example
./run_stress_test.sh 363c9c77... 2519e58e... --tournaments 3 --shards 8
```

## 📊 **Test Architecture**

### Phase 1: Tournament Setup
- Creates multiple tournaments with configurable shard counts
- Validates tournament accessibility and configuration
- Generates shared configuration for K6 stress test

### Phase 2: Concurrent Player Simulation
- **200 virtual users** ramping up over 5 minutes
- **Sustained load** for 10 minutes at peak concurrency
- **Graceful ramp down** over 5 minutes

### Phase 3: Realistic Gameplay Simulation
Each player performs:
- **Registration** with unique credentials
- **2-5 game boards** per tournament
- **20-50 moves** per board with realistic patterns
- **Strategic move sequences** based on actual gameplay

### Phase 4: Performance Monitoring
- Real-time metrics collection
- Error rate and response time tracking
- Success criteria validation
- Comprehensive result analysis

## 🎮 **Realistic Gaming Patterns**

The stress test includes sophisticated move pattern generators:

### Strategic Patterns (40%)
- **Corner Strategy**: `Down → Right → Down → Left`
- **Build-up Pattern**: `Right → Down → Right → Down`
- **Mixed Strategy**: `Down → Down → Right → Up`

### Weighted Random (30%)
- More frequent `Down` and `Right` moves (common in 2048)
- Simulates casual gameplay

### Burst Patterns (15%)
- **8 rapid moves** followed by thinking pauses
- Simulates intense gaming moments

### Panic Patterns (15%)
- **Random rapid moves** when players are struggling
- Simulates desperate attempts to recover

## 📈 **Performance Metrics**

### Success Criteria
- **Error Rate**: < 5%
- **95th Percentile Response Time**: < 2 seconds
- **Request Rate**: Target ~50-100 req/s sustained
- **Player Registration Success**: > 95%
- **Board Creation Success**: > 95%

### Monitored Metrics
- Total HTTP requests and response times
- Player registrations and success rate
- Board creations across all tournaments
- Move operations and throughput
- Error rates by operation type
- Tournament-specific performance

## 🛠 **Configuration Options**

### Tournament Coordinator Options
```bash
python3 stress_test_coordinator.py CHAIN_ID APP_ID [options]

Options:
  -t, --tournaments NUM    Number of tournaments (1-10, default: 3)
  -s, --shards NUM         Shards per tournament (1-32, default: 8)
  -u, --url URL            Service URL (default: http://localhost:8088)
  -o, --output FILE        Config output file (default: stress_test_config.json)
```

### Stress Test Runner Options
```bash
./run_stress_test.sh CHAIN_ID APP_ID [options]

Options:
  -t, --tournaments NUM    Number of tournaments to create
  -s, --shards NUM         Shards per tournament  
  -u, --url URL            Base service URL
  -h, --help               Show help message
```

## 📁 **Output and Results**

### Results Directory Structure
```
stress_test_results/
└── run_YYYYMMDD_HHMMSS/
    ├── stress_test_config.json     # Tournament configuration
    ├── k6_output.log              # K6 execution log
    └── stress_test_results.json   # Detailed performance metrics
```

### Sample Results Output
```
================================================================================
                           U2048 STRESS TEST RESULTS
================================================================================

🎯 Target Configuration:
   • Players: 200
   • Tournaments: 3
   • Boards per Player: 3
   • Total Duration: 25 minutes

📊 Performance Metrics:
   • Total Requests: 45,678
   • Request Rate: 87.23 req/s
   • Error Rate: 2.34%
   • Avg Response Time: 456.78ms
   • 95th Percentile: 1,234.56ms

🎮 Game Metrics:
   • Player Registrations: 198
   • Board Creations: 594
   • Move Operations: 15,840

🏆 Success Criteria:
   • Error Rate < 5%: ✅ PASS
   • 95th Percentile < 2s: ✅ PASS
```

## 🔧 **Troubleshooting**

### Common Issues

**1. Tournament Creation Failures**
```bash
# Check service connectivity
curl -X POST http://localhost:8088/chains/CHAIN_ID/applications/APP_ID \
  -H "Content-Type: application/json" \
  -d '{"query":"{ leaderboards { leaderboardId } }"}'
```

**2. K6 Installation Issues**
```bash
# Verify K6 installation
k6 version

# Alternative installation methods available at:
# https://k6.io/docs/get-started/installation/
```

**3. High Error Rates**
- Reduce concurrent users in the K6 script
- Increase timeouts in the configuration
- Check service resource limits

**4. Performance Issues**
- Monitor system resources during test
- Adjust ramp-up duration for gentler load increase
- Consider running test on dedicated hardware

### Debug Mode
Run individual components for debugging:

```bash
# Test tournament coordinator only
python3 stress_test_coordinator.py CHAIN_ID APP_ID --tournaments 1

# Test K6 with existing config
k6 run stress_test_k6.js

# Validate configuration
jq . stress_test_config.json
```

## 🎛 **Advanced Configuration**

### Custom K6 Test Scenarios
Edit `stress_test_k6.js` to modify:
- Ramp-up stages and timing
- Player behavior patterns
- Move frequency and batch sizes
- Error handling and retry logic

### Custom Tournament Settings
Edit `stress_test_coordinator.py` to modify:
- Tournament time constraints
- Shard distribution strategies
- Coordinator player settings

## 🏆 **Best Practices**

1. **Start Small**: Begin with fewer tournaments and players for initial validation
2. **Monitor Resources**: Watch CPU, memory, and network during tests
3. **Baseline Testing**: Run single-player tests first to establish baselines
4. **Gradual Scaling**: Increase load gradually to identify breaking points
5. **Result Analysis**: Always analyze results to identify optimization opportunities

## 📚 **Test Scenarios**

### Scenario 1: Standard Load Test (Default)
- 3 tournaments, 8 shards each
- 200 concurrent players
- Standard ramp-up profile

### Scenario 2: High Concurrency Test
```bash
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 5 --shards 16
```

### Scenario 3: Single Tournament Stress
```bash
./run_stress_test.sh CHAIN_ID APP_ID --tournaments 1 --shards 32
```

### Scenario 4: Network Latency Test
```bash
./run_stress_test.sh CHAIN_ID APP_ID --url http://remote-server:8088
```

## 🤝 **Contributing**

To improve the stress test suite:

1. **Add new move patterns** in `stress_test_k6.js`
2. **Enhance monitoring** with additional metrics
3. **Optimize load distribution** across tournaments
4. **Add new test scenarios** for specific use cases

## 📄 **License**

This stress test suite is part of the U2048 project and follows the same licensing terms.