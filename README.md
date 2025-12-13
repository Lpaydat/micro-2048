# U2048 - Decentralized 2048 Game on Linera

A fully decentralized implementation of the classic 2048 puzzle game, powered by the [Linera blockchain](https://linera.io). This project combines the addictive gameplay of 2048 with blockchain technology to enable trustless leaderboards, tournaments, and competitive gaming.

## What is U2048?

U2048 is a blockchain-based version of the popular 2048 sliding tile puzzle game. Players combine numbered tiles to create higher values, aiming to reach the 2048 tile and beyond. What makes this implementation unique is:

- **Decentralized Architecture**: All game state, moves, and scores are stored on the Linera blockchain
- **Trustless Leaderboards**: Tournament rankings and player scores are verifiable on-chain
- **Multi-chain Design**: Each player gets their own blockchain for optimal performance
- **Real-time Updates**: WebSocket-based GraphQL subscriptions for live game state
- **High Performance**: Handles hundreds of concurrent players with batch move processing

The backend is written in Rust as a Linera application, while the frontend is built with SvelteKit and TypeScript.

## How to Run

### Prerequisites

- [Linera CLI](https://linera.dev) installed
- Rust with `wasm32-unknown-unknown` target
- Node.js and npm

### Step 1: Start Linera Local Network

```bash
linera net up
```

This command starts a local Linera development network with multiple validators.

### Step 2: Build and Deploy the Application

Build the WebAssembly binaries and publish to the blockchain:

```bash
cargo build --release --target wasm32-unknown-unknown && \
linera publish-and-create target/wasm32-unknown-unknown/release/game2048_{contract,service}.wasm \
  --json-argument 300
```

The argument `300` represents the tournament duration in seconds (5 minutes). Adjust as needed.

### Step 3: Configure Environment Variables

After deployment, the terminal will print the `chainId` and `applicationId`. Copy these values:

1. Create a `.env` file in the `website/` directory:

```bash
cd website
cp .env.example .env
```

2. Update the `.env` file with your `chainId` and `applicationId`

3. **(Optional)** Update `website/scripts/leaderboard-stress-test.ts` with the same values at lines 15-16:

```typescript
chainId: 'YOUR_CHAIN_ID_HERE',
applicationId: 'YOUR_APPLICATION_ID_HERE'
```

### Step 4: Start the Linera Service

Start the blockchain node service with increased message handling capacity:

```bash
linera --max-pending-message-bundles 1000 service --port 8088 --listener-skip-process-inbox
```

This command:
- Sets max pending messages to 1000 (important for high-load scenarios)
- Exposes the GraphQL API on port 8088
- Skips automatic inbox processing for better performance

### Step 5: Start the Frontend

In a new terminal, navigate to the website directory and start the development server:

```bash
cd website
npm run dev
```

### Step 6: Play the Game

Open your browser and navigate to:

```
http://localhost:5173
```

Create an account, start a game, and enjoy!

## Game Modes

U2048 offers two distinct gameplay experiences:

### Normal Mode
The classic 2048 experience where you play at your own pace. Combine tiles strategically to achieve the highest score possible. Perfect for practicing strategies and casual play.

### Rhythm Mode
An intense, time-pressured variant where you must make moves within a strict time limit. Each move has a countdown timer, adding an extra layer of challenge and excitement. Test your reflexes and decision-making under pressure!

## Performance Testing (Optional)

Want to experience the game during peak load conditions? Run the stress test script to simulate hundreds of concurrent players competing in real-time!

### Prerequisites

- Install [k6](https://k6.io/docs/get-started/installation/)

### Running the Stress Test

1. First, create a tournament in the game UI and copy its Tournament ID
2. Run the stress test with your tournament ID:

```bash
k6 run \
  -e ENVIRONMENT=local \
  -e NUM_PLAYERS=100 \
  -e TEST_DURATION=5m \
  -e TOURNAMENT_ID=<YOUR_TOURNAMENT_ID_HERE> \
  website/scripts/leaderboard-stress-test.ts
```

**Parameters:**
- `NUM_PLAYERS`: Number of simulated concurrent players (default: 100)
- `TEST_DURATION`: How long to run the test (e.g., `5m`, `10m`, `1h`)
- `TOURNAMENT_ID`: Replace the example tournament ID with your actual tournament ID
- `ENVIRONMENT`: Set to `local` for local testing, `production` for live testing

**What happens during the test:**
- The script simulates 100 players simultaneously creating accounts, joining the tournament, and playing games
- You can log in with your own account and play alongside the simulated players to experience gameplay under peak load
- Watch the leaderboard update in real-time as hundreds of moves are processed per second
- This demonstrates the scalability and performance of the Linera blockchain infrastructure

## Project Structure

```
u2048/
├── src/                    # Rust source code (Linera application)
│   ├── contract.rs        # Smart contract logic
│   ├── service.rs         # GraphQL service layer
│   ├── contract_domain/   # Game logic and event handling
│   └── ...
├── website/               # SvelteKit frontend
│   ├── src/              # Svelte components and routes
│   ├── scripts/          # Deployment and testing scripts
│   └── ...
├── scripts/              # Deployment and monitoring scripts
└── Cargo.toml           # Rust dependencies
```

## Architecture Highlights

- **Multi-chain Model**: Each player operates on their own Linera microchain
- **Event-driven Design**: Game moves emit events that update leaderboards
- **Batch Move Processing**: Multiple moves can be submitted and validated in a single transaction
- **GraphQL API**: Both queries and subscriptions for real-time updates
- **Tournament System**: Time-bounded competitions with on-chain leaderboards

## License

See the project license for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
