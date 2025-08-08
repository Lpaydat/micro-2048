# Implementation Plan

## Implementation Strategy

**APPROACH**: Build the Linera 2048 Smart Contract using the proven patterns from the `./example` project as a blueprint. This ensures we follow working Linera SDK patterns and maintain compatibility with the current ecosystem.

## Safe Development Protocol

**CRITICAL**: Each task must pass all validation steps before proceeding. This ensures blockchain smart contract safety and prevents deployment of broken code.

### Required Validation Steps for Each Task:

1. **Unit Testing**: `cargo test --lib` (test domain logic)
2. **Integration Testing**: `cargo test tests::integration` (test with mock blockchain)
3. **WASM Build**: `cargo build --release --target wasm32-unknown-unknown` (ensure WASM compatibility)
4. **Deployment Test**: Deploy to local testnet and verify functionality

### Reference Implementation:
- Use `./example` as the working reference for Linera project structure and patterns
- The example project is up-to-date and working with current Linera SDK v0.14.1
- Follow the same module organization, dependency patterns, and architectural decisions as example
- Adapt the DDD layering from example to fit 2048 game domain requirements

### Project Structure (Based on Example):
```
src/
├── lib.rs              # ABI definitions and re-exports
├── contract.rs         # Contract entry point with operation handlers
├── service.rs          # GraphQL service entry point
├── core/               # Domain layer (pure business logic)
│   ├── mod.rs
│   ├── types/          # Domain models and value objects
│   ├── domain/         # Domain services
│   └── validation/     # Domain validators
├── infrastructure/     # Infrastructure layer (blockchain concerns)
│   ├── mod.rs
│   ├── state/          # Modular state management
│   ├── operations.rs   # Operation definitions
│   ├── messages.rs     # Cross-chain messages
│   ├── errors.rs       # Error types
│   ├── time_utils.rs   # Blockchain time utilities
│   └── handlers/       # Operation and message handlers
└── api/                # API layer (GraphQL interface)
    ├── mod.rs
    ├── queries/        # GraphQL queries
    └── mutations/      # GraphQL mutations
```

### Deployment Environment Setup:
```bash
export LINERA_WALLET="/tmp/.tmpeM9UO4/wallet_0.json"
export LINERA_STORAGE="rocksdb:/tmp/.tmpeM9UO4/client_0.db"
linera publish-and-create target/wasm32-unknown-unknown/release/game2048_contract.wasm target/wasm32-unknown-unknown/release/game2048_service.wasm
```

### Failure Protocol:
- If any test fails: Fix the issue before proceeding
- If WASM build fails: Check for incompatible dependencies (especially time-related)
- If deployment fails: Verify contract instantiation and operation handling
- If functionality test fails: Debug using GraphQL queries and operation responses

---

- [ ] 1. Create minimal working Linera contract (absolute basics)
  - Create new Cargo.toml based on example project with Linera SDK v0.14.1 dependencies only
  - Create minimal lib.rs with basic ABI (no complex types yet)
  - Create minimal contract.rs with empty state and one simple operation (like Ping)
  - Create minimal service.rs with empty GraphQL (just introspection)
  - **Testing**: Run `cargo test --lib` (should pass with no tests)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment with environment variables (must deploy successfully)
  - **Validation**: Call the Ping operation and verify it returns a simple string
  - _Requirements: Basic Linera contract functionality_

- [ ] 2. Add basic state and one simple operation
  - Add minimal Game2048State with one MapView (like participants: MapView<String, String>)
  - Add RegisterPlayer operation that just stores username by ID
  - Update contract.rs to handle the RegisterPlayer operation
  - **Testing**: Run `cargo test --lib` (add one basic test)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and call RegisterPlayer operation
  - **Validation**: Verify player registration works and state persists
  - _Requirements: Basic state management and operations_

- [ ] 3. Add basic GraphQL query support
  - Update service.rs to add one simple query (getPlayer by ID)
  - Add basic GraphQL types for Player
  - Test GraphQL introspection and basic query
  - **Testing**: Run `cargo test --lib` (test GraphQL schema builds)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and GraphQL query functionality
  - **Validation**: Query a registered player via GraphQL
  - _Requirements: Basic GraphQL service functionality_

- [ ] 4. Add blockchain time utilities and improve operations
  - Create infrastructure/time_utils.rs with TimeManager using runtime.system_time()
  - Update RegisterPlayer to use blockchain time for created_at timestamp
  - Add GetCurrentTime operation for testing time functionality
  - **Testing**: Run `cargo test tests::time_utils` (test time functions)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and verify blockchain time works
  - **Validation**: Verify operations use blockchain time, not client time
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [ ] 5. Add basic 2048 game types and simple game creation
  - Create core/types/game.rs with basic GameSession and Board types
  - Add CreateGame operation that creates a game with initial board state
  - Add GetGame query to retrieve game state
  - Use simple board representation first (not optimized bit-packing yet)
  - **Testing**: Run `cargo test tests::game_types` (test game creation)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and game creation
  - **Validation**: Create a game and verify initial state is correct
  - _Requirements: 2.1, 2.2 (basic game state)_

- [ ] 6. Implement basic move processing (simple version)
  - Add MakeMove operation with basic move validation
  - Implement simple tile movement logic (not optimized yet)
  - Add basic scoring calculation
  - Update game state after moves
  - **Testing**: Run `cargo test tests::game_moves` (test move processing)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and move processing
  - **Validation**: Create game, make moves, verify board updates correctly
  - _Requirements: 2.2, 2.3, 2.4_

- [ ] 7. Add basic error handling and validation
  - Create infrastructure/errors.rs with basic error types
  - Add validation for player registration (username length, etc.)
  - Add validation for game moves (valid direction, game exists, etc.)
  - Return proper error messages instead of panicking
  - **Testing**: Run `cargo test tests::validation` (test error cases)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and error handling
  - **Validation**: Try invalid operations and verify proper error responses
  - _Requirements: 1.2, 1.3, 2.6, 6.4, 6.5_

- [ ] 8. Add basic tournament/competition support
  - Create core/types/tournament.rs with basic Tournament type
  - Add CreateTournament operation for simple tournaments
  - Add JoinTournament operation for player participation
  - Add basic tournament state management
  - **Testing**: Run `cargo test tests::tournaments` (test tournament creation)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and tournament functionality
  - **Validation**: Create tournament, join players, verify state
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 9. Add basic cross-chain messaging (minimal)
  - Create infrastructure/messages.rs with one simple message type
  - Add basic message handling in execute_message
  - Test message sending and receiving between chains
  - Keep it simple - just tournament notifications for now
  - **Testing**: Run `cargo test tests::messaging` (test message handling)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test with TestValidator multi-chain setup
  - **Validation**: Send messages between chains and verify processing
  - _Requirements: 8.1, 8.2, 8.3_

- [ ] 10. Add leaderboard and ranking functionality
  - Add basic leaderboard calculation for tournaments
  - Add GetLeaderboard query for tournament rankings
  - Implement simple ranking by score (highest first)
  - Add tournament completion and winner determination
  - **Testing**: Run `cargo test tests::leaderboards` (test ranking logic)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and leaderboard functionality
  - **Validation**: Complete tournament games and verify leaderboard accuracy
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 11. Add administrative operations
  - Add basic admin role management
  - Add BanPlayer and UnbanPlayer operations
  - Add admin permission checking
  - Add basic audit logging for admin actions
  - **Testing**: Run `cargo test tests::admin` (test admin operations)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and admin functionality
  - **Validation**: Test admin operations and permission enforcement
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 12. Optimize game logic and add deterministic mechanics
  - Implement bit-packed board representation for efficiency
  - Add precomputed move tables for O(1) operations
  - Implement deterministic tile spawning using blockchain randomness
  - Optimize scoring calculations
  - **Testing**: Run `cargo test tests::optimized_game` (test optimized logic)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment and verify optimized game works
  - **Validation**: Verify games are deterministic and performant
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 11.1_

- [ ] 13. Add comprehensive testing and validation
  - Add integration tests with TestValidator
  - Test complete user flows end-to-end
  - Add performance benchmarks for critical operations
  - Test error conditions and edge cases
  - **Testing**: Run `cargo test` (comprehensive test suite)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (must succeed)
  - **Deploy**: Test deployment with full test coverage
  - **Validation**: Verify all functionality works correctly
  - _Requirements: All requirements - comprehensive testing_

- [ ] 14. Final optimization and deployment preparation
  - Optimize for WASM deployment and performance
  - Ensure all code is WASM-compatible
  - Add final performance optimizations
  - Test deployment with production settings
  - **Testing**: Run `cargo test --release` (test optimized builds)
  - **Build**: Run `cargo build --release --target wasm32-unknown-unknown` (clean build)
  - **Deploy**: Test final deployment and verify all operations
  - **Validation**: Complete end-to-end testing of deployed contract
  - _Requirements: 11.5, 11.7, 10.3, 10.4, 10.5_