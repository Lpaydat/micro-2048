# 🤖 2048 Simulation Test Suite

A comprehensive simulation test suite for the 2048 game that creates realistic bot players to test the system under various load conditions. Perfect for playing alongside bots or stress testing the infrastructure.

## 📁 Test Scripts

- **`simulation.ts`** - General gameplay simulation with bot personalities
- **`leaderboard-stress-test.ts`** - Focused leaderboard/triggerer system stress test ⭐ NEW
- **`run-leaderboard-stress-test.sh`** - Easy wrapper for leaderboard tests

## 🎯 Features

- **Realistic Bot Behavior**: Multiple personality types (aggressive, strategic, casual)
- **Environment Support**: Both local development and production environments
- **Tournament Integration**: Auto-discovers active tournaments or uses specific ones
- **Configurable Load**: From light testing to heavy stress testing
- **Human-Friendly**: Bots use identifiable names and realistic timing
- **Leaderboard Testing**: Dedicated stress test for triggerer system ⭐ NEW

## 🚀 Quick Start

### Play Alongside Bots (Recommended)

```bash
# Start 20 realistic bots for 30 minutes
ENVIRONMENT=production BOT_PERSONALITY=mixed GAMES_PER_BOT=3 k6 run --vus 20 --duration 30m website/scripts/simulation.ts
```

### Local Development Testing

```bash
# Light load for local testing
ENVIRONMENT=local BOT_PERSONALITY=mixed GAMES_PER_BOT=2 k6 run --vus 10 --duration 5m website/scripts/simulation.ts
```

### High Activity Testing

```bash
# Fast-playing aggressive bots
ENVIRONMENT=production BOT_PERSONALITY=aggressive GAMES_PER_BOT=5 k6 run --vus 30 --duration 15m website/scripts/simulation.ts
```

### 🎯 Leaderboard Stress Testing (NEW)

Test the triggerer system and leaderboard updates:

```bash
# Easy way - using helper script
./website/scripts/run-leaderboard-stress-test.sh <TOURNAMENT_ID> light

# Available scenarios: light, medium, heavy, stress
./website/scripts/run-leaderboard-stress-test.sh abc123def456 medium

# Manual way - full control
TOURNAMENT_ID="abc123..." NUM_PLAYERS=30 k6 run website/scripts/leaderboard-stress-test.ts
```

**See [LEADERBOARD_STRESS_TEST.md](./LEADERBOARD_STRESS_TEST.md) for detailed documentation.**

## 📋 Bot Personalities

### 🚀 Aggressive

- **Speed**: Very fast (0.5-2s per move)
- **Behavior**: Quick decisions, random moves
- **Best for**: High activity testing
- **Names**: Speed_123, Flash_456, Turbo_789

### 🧠 Strategic

- **Speed**: Slow and thoughtful (3-8s per move)
- **Behavior**: Prefers Down/Right moves, smaller batches
- **Best for**: Realistic gameplay simulation
- **Names**: Think_123, Smart_456, Wise_789

### 😌 Casual

- **Speed**: Variable (1-10s per move)
- **Behavior**: Mixed timing, relaxed play
- **Best for**: Natural human-like simulation
- **Names**: Chill_123, Relax_456, Easy_789

### 🎭 Mixed

- **Speed**: Wide range (0.5-10s per move)
- **Behavior**: Combination of all styles
- **Best for**: General testing
- **Names**: Bot_123, Player_456, AI_789

## ⚙️ Configuration

### Environment Variables

| Variable          | Options                                      | Default              | Description                 |
| ----------------- | -------------------------------------------- | -------------------- | --------------------------- |
| `ENVIRONMENT`     | `local`, `production`                        | `production`         | Target environment          |
| `TOURNAMENT_ID`   | Tournament ID                                | `""` (auto-discover) | Specific tournament to join |
| `BOT_PERSONALITY` | `aggressive`, `strategic`, `casual`, `mixed` | `mixed`              | Bot behavior style          |
| `GAMES_PER_BOT`   | Number                                       | `3`                  | Games each bot creates      |
| `MOVES_PER_GAME`  | Number                                       | `50`                 | Moves per game              |

### Load Testing Scenarios

#### Light Load (Development)

```bash
ENVIRONMENT=local k6 run --vus 5 --duration 2m website/scripts/simulation.ts
```

#### Medium Load (Testing)

```bash
ENVIRONMENT=production k6 run --vus 25 --duration 10m website/scripts/simulation.ts
```

#### Heavy Load (Stress Testing)

```bash
ENVIRONMENT=production k6 run --vus 100 --duration 1h website/scripts/simulation.ts
```

#### Peak Load (Maximum Stress)

```bash
ENVIRONMENT=production k6 run --vus 200 --duration 30m website/scripts/simulation.ts
```

## 🎮 Use Cases

### 1. Playing Alongside Bots

Perfect for when you want to experience the game with realistic opponents:

```bash
# Start bots, then join the same tournament
ENVIRONMENT=production BOT_PERSONALITY=mixed k6 run --vus 15 --duration 1h website/scripts/simulation.ts
```

### 2. Tournament Testing

Test specific tournaments:

```bash
TOURNAMENT_ID="your_tournament_id" ENVIRONMENT=production k6 run --vus 50 --duration 30m website/scripts/simulation.ts
```

### 3. Performance Testing

Stress test the infrastructure:

```bash
ENVIRONMENT=production BOT_PERSONALITY=aggressive GAMES_PER_BOT=5 k6 run --vus 150 --duration 45m website/scripts/simulation.ts
```

### 4. Development Testing

Local development with lightweight load:

```bash
ENVIRONMENT=local GAMES_PER_BOT=2 MOVES_PER_GAME=20 k6 run --vus 8 --duration 5m website/scripts/simulation.ts
```

## 📊 Monitoring

### Bot Activity Logs

The simulation provides detailed logs:

```
🤖 Bot Speed_1234 (aggressive) starting simulation...
✅ Speed_1234 registered with chain ID: a1b2c3d4e5f6...
🎯 Speed_1234 selected tournament: Weekly Championship
🎮 Speed_1234 creating 3 games...
🎲 Speed_1234 playing board 9f8e7d6c...
✅ Speed_1234 completed simulation
```

### Performance Metrics

Monitor these metrics during testing:

- **Response Times**: 95th percentile should be < 3 seconds
- **Error Rates**: Should stay below 15%
- **Active Players**: Number of concurrent bot players
- **Move Frequency**: Moves per second across all bots

## 🔧 Advanced Usage

### Custom Bot Behavior

Modify the personality configurations in `simulation-utils.ts`:

```typescript
export const BOT_PERSONALITIES = {
	custom: {
		moveDelay: { min: 1.0, max: 5.0 },
		batchSize: 15,
		thinkingTime: 1.5,
		namePrefixes: ['Custom', 'Test', 'Demo']
	}
};
```

### Tournament Discovery

The script automatically discovers active tournaments. You can also specify:

```bash
# Use specific tournament
TOURNAMENT_ID="ae41b40b288a1e7ed064e2ff749a9ce3e780a5742dca074e6015e77e9dd373f8" k6 run website/scripts/simulation.ts
```

### Batch Processing

Bots create and play multiple games concurrently:

```bash
# More games per bot for higher load
GAMES_PER_BOT=10 MOVES_PER_GAME=100 k6 run --vus 50 --duration 2h website/scripts/simulation.ts
```

## 🛠️ File Structure

```
website/scripts/
├── simulation.ts          # Main simulation script
├── simulation-config.ts   # Configuration presets
├── simulation-utils.ts    # Utility functions
├── README.md             # This documentation
├── test.ts               # Original test script
└── register.ts           # Registration test
```

## 🚨 Important Notes

### Local Testing Requirements

- Ensure your local development server is running on `localhost:8080`
- Have active tournaments available for bot participation
- Monitor local resource usage during testing

### Production Testing Guidelines

- Start with smaller loads and gradually increase
- Monitor system resources and response times
- Be considerate of other players on the platform
- Use appropriate bot counts for the time of day

### Performance Considerations

- Higher VU counts require more system resources
- Aggressive bots generate more network traffic
- Monitor memory usage during long-running tests
- Consider using batch operations for efficiency

## 🐛 Troubleshooting

### Common Issues

**Bots can't register:**

- Check if the tournament is active and accepting players
- Verify API endpoints are accessible
- Check network connectivity

**High error rates:**

- Reduce the number of concurrent bots
- Increase timeouts in the script
- Check server capacity and response times

**Slow performance:**

- Monitor server resource usage
- Check database performance
- Consider reducing bot activity levels

### Debug Mode

Add verbose logging by modifying the log levels in the script.

## 📈 Results Analysis

After running simulations, analyze:

- **Throughput**: Moves per second the system can handle
- **Latency**: Response times under different loads
- **Scalability**: How performance degrades with load
- **Stability**: Error rates over extended periods

## 🤝 Contributing

To add new features:

1. Modify personality configurations in `simulation-utils.ts`
2. Add new presets in `simulation-config.ts`
3. Update the main simulation logic in `simulation.ts`
4. Update this documentation

## 📞 Support

For issues or questions:

1. Check the troubleshooting section above
2. Review the logs for error details
3. Verify environment configuration
4. Test with smaller loads first
