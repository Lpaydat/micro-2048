# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GameHub is a Linera blockchain smart contract for gaming leaderboards and tournaments. It implements a comprehensive gaming platform with player management, game approval workflows, scoring systems, moderation capabilities, and cross-chain event processing.

## Build Commands

### Core Development
```bash
# Regular development build
cargo check

# Full build with WASM target (required for Linera deployment)
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test --lib                    # All library tests
cargo test tests::core              # Core domain tests only
cargo test tests::infrastructure    # Infrastructure tests only
cargo test test_name                # Single test

# WASM build validation (critical for deployment)
cargo build --release --target wasm32-unknown-unknown
```

### Linera Deployment
```bash
# Set up Linera environment variables (required before deployment)
export LINERA_WALLET="/tmp/.tmpfVpXRt/wallet_0.json"
export LINERA_STORAGE="rocksdb:/tmp/.tmpfVpXRt/client_0.db"

# Deploy to Linera blockchain (MUST TEST EVERY TIME after WASM build)
linera publish-and-create target/wasm32-unknown-unknown/release/gamehub_contract.wasm target/wasm32-unknown-unknown/release/gamehub_service.wasm

# Note: Contract instantiate method expects () argument, not JSON parameters
# Initialization is handled internally with hardcoded system admin ID
```

### **CRITICAL: Test Deployment Requirement**
**EVERY TIME** you run `cargo build --release --target wasm32-unknown-unknown`, you **MUST** immediately test deploy with:
```bash
export LINERA_WALLET="/tmp/.tmpfVpXRt/wallet_0.json"
export LINERA_STORAGE="rocksdb:/tmp/.tmpfVpXRt/client_0.db"
linera publish-and-create target/wasm32-unknown-unknown/release/gamehub_contract.wasm target/wasm32-unknown-unknown/release/gamehub_service.wasm
```

This ensures that:
- WASM compilation produces deployable bytecode
- Contract initialization works correctly
- No runtime errors in the deployment process
- Blockchain integration functions properly

### Test Management
The project has 19+ comprehensive tests organized by functional area. Key test files:
- `src/tests/infrastructure/contract_tests.rs` - Contract operation tests
- `src/tests/infrastructure/state_player_tests.rs` - Player management integration tests
- `src/tests/core/` - Domain logic unit tests

## Architecture

### Three-Layer Clean Architecture

**Core Layer** (`src/core/`) - Pure business logic, no infrastructure dependencies:
- `types/` - Domain models organized by business area (Player, Game, Event, Scoring, etc.)
- `domain/` - Business services (PlayerService, GameService, PermissionService, etc.)
- `validation/` - Domain-specific validation logic

**Infrastructure Layer** (`src/infrastructure/`) - Blockchain and technical concerns:
- `state.rs` - Linera blockchain state management with MapView/SetView collections
- `operations.rs` - Contract operations (RegisterPlayer, ApproveGame, etc.)
- `messages.rs` - Cross-chain messaging for batch updates
- `errors.rs` - Comprehensive error types (43 variants)

**API Layer** (`src/api/`) - External interfaces:
- `graphql_types.rs` - GraphQL schema definitions

### Key State Management

The `GameHubState` struct in `src/infrastructure/state.rs` manages all blockchain state using Linera SDK views:
- `MapView<String, Player>` for player data
- `MapView<String, Game>` for approved games  
- `MapView<String, PendingGame>` for games awaiting approval
- `SetView<String>` for admin/moderator permissions
- `MapView<String, AuditLogEntry>` for audit logging

## Smart Contract Structure

### Contract Entry Points
- `src/contract.rs` - Main contract implementation with `execute_operation` and `execute_message`
- `src/service.rs` - GraphQL service for querying blockchain state

### Operation Handling
The contract handles 14 operation types including:
- Player management: `RegisterPlayer`, `UpdatePlayerProfile`
- Game lifecycle: `ApproveGame`, `RejectGame`, `SuspendGame`
- Moderation: `BanPlayer`, `SuspendPlayer`, `UnbanPlayer`
- Administration: `AddAdmin`, `RemoveAdmin`, `AssignModerator`

### Cross-Chain Messaging
- `RegisterGame` - Games submit for approval via cross-chain messages
- `BatchEventUpdate` - Bulk player score updates from game contracts

## Domain-Driven Design Patterns

### Business Services
Each domain service in `src/core/domain/` focuses on single business capabilities:
- `permission_service.rs` - Admin/moderator role management with audit logging
- `player_service.rs` - Player registration, profile management, activity tracking
- `game_service.rs` - Game approval workflows and lifecycle management
- `scoring_service.rs` - Points calculation with streak bonuses
- `moderation_service.rs` - Player moderation (banning, suspension)

### Validation Strategy
Domain-specific validators in `src/core/validation/`:
- `PlayerValidator` - Discord ID format, username validation
- `GameValidator` - Contract address validation, game data validation
- `ScoringValidator` - Scoring configuration validation
- `GeneralValidator` - Common validation utilities

## Implementation Status

### Completed Phases
- **Phase 1.1**: Player Management Methods (85% complete) - 39 comprehensive tests
- **Phase 1.2**: Permission System Methods (100% complete) - All admin/moderator functionality

### Current Development Focus
The project follows a phased implementation plan detailed in `gamehub-smart-contract-implementation-plan.md`:
- Phase 1.3: Audit Logging Methods (15% complete)
- Phase 2: Game Lifecycle Management (70% complete) 
- Phase 3: Player Moderation System
- Phase 4: Scoring System Completion
- Phase 5: Batch Processing & Leaderboards

## Key Development Patterns

### Error Handling
Uses comprehensive `GameHubError` enum with 43 variants covering all validation and business logic scenarios. All async methods return `Result<T, GameHubError>`.

### State Mutations
All state changes go through the `GameHubState` implementation which handles:
- Validation using domain validators
- Permission checks for admin/moderator operations  
- Audit logging for administrative actions
- Proper error propagation

### Testing Strategy
- Unit tests for validation logic and business rules
- Integration tests for state management with actual MapView/SetView storage
- Contract operation tests for end-to-end workflows
- Mock data helpers for consistent test setup

### Async Patterns
Heavy use of async/await throughout due to Linera SDK's async storage operations. Use `.blocking_wait()` in tests for synchronous execution.

## Development Workflow

### Adding New Features
1. Define types in appropriate `core/types/*.rs` module
2. Implement business logic in `core/domain/*.rs` service
3. Add validation in `core/validation/*.rs` validator
4. Update infrastructure layer if needed
5. Add comprehensive tests following existing patterns
6. Update contract operations if external interface needed

### Working with Blockchain State  
- Use `self.players.get(id).await.ok().flatten()` pattern for optional lookups
- Use `self.admins.contains(id).await.unwrap_or(false)` for permission checks
- **Collection Iteration**: Use `collection.indices().await` to iterate over MapView/SetView collections
- Always handle MapView/SetView errors gracefully
- Use audit logging for all administrative actions

### Collection Iteration Patterns

**IMPORTANT**: Unlike initial assumptions, Linera SDK DOES support iteration through the `indices()` method:

```rust
// Iterating over SetView (admins, moderators)
let admin_ids = self.admins.indices().await.unwrap_or_default();
for admin_id in admin_ids {
    // Process each admin
}

// Iterating over MapView (audit_log, players, games)
let log_ids = self.audit_log.indices().await.unwrap_or_default();
for log_id in log_ids {
    if let Ok(Some(entry)) = self.audit_log.get(&log_id).await {
        // Process each audit log entry
    }
}
```

**Error Handling Pattern**:
```rust
match self.collection.indices().await {
    Ok(indices) => {
        // Process indices
        indices
    }
    Err(_) => {
        // Handle storage errors gracefully
        Vec::new()
    }
}
```

**Performance Considerations**:
- `indices()` returns all keys/IDs in the collection
- For large collections, consider pagination or filtering
- Sort results when chronological order matters (e.g., audit logs by timestamp)
- Cache results in service layer when appropriate

## MCP Tools Available

This project has access to specialized MCP (Model Context Protocol) tools that enhance development efficiency:

### Serena MCP - Semantic Code Analysis

**When to Use:**
- Exploring unfamiliar parts of the codebase without reading entire files
- Finding specific functions, structs, or implementations by name
- Understanding code relationships and dependencies
- Searching for patterns across multiple files
- Getting high-level overviews of modules before diving into details

**Key Benefits:**
- **Token Efficient**: Read only the code you need instead of entire files
- **Semantic Search**: Find symbols by name path (e.g., `GameHubState/register_player`)
- **Relationship Discovery**: Find all references to a function or type
- **Smart Navigation**: Get overviews first, then targeted reads
- **Symbol-Based Editing**: Replace entire methods/structs precisely

**When NOT to Use:**
- When you already know the exact file and location
- For simple file reads where you need the full context
- When working with non-code files (README, config files)
- For small, single-file changes

**Essential Commands:**
```
mcp__serena__get_symbols_overview - Get high-level view of file/directory
mcp__serena__find_symbol - Find specific functions/structs by name
mcp__serena__search_for_pattern - Search for code patterns
mcp__serena__find_referencing_symbols - Find all uses of a symbol
mcp__serena__replace_symbol_body - Replace entire functions/methods
```

**Best Practices for This Codebase:**
- Start with `get_symbols_overview` on `src/infrastructure/state.rs` to understand state methods
- Use `find_symbol` with paths like `GameHubState/register_player` for specific methods
- Search for validation patterns across `src/core/validation/` modules
- Use symbol replacement when updating business logic in domain services

### Context7 MCP - Library Documentation

**When to Use:**
- Looking up Linera SDK documentation and examples
- Understanding async-graphql patterns and schema definitions
- Checking Rust crate documentation for dependencies
- Learning about serde serialization patterns
- Finding examples of blockchain development patterns

**Key Benefits:**
- **Up-to-Date Docs**: Always current library documentation
- **Code Examples**: Real implementation examples from documentation
- **API Reference**: Complete API coverage for dependencies
- **Best Practices**: Official recommended patterns

**When NOT to Use:**
- For project-specific business logic questions
- When you need to understand the existing codebase structure
- For debugging project-specific errors
- For understanding the GameHub domain logic

**Essential Commands:**
```
mcp__context7__resolve-library-id - Find the correct library identifier
mcp__context7__get-library-docs - Get documentation and examples
```

**Relevant Libraries for This Project:**
- `linera-sdk` - Core blockchain development patterns
- `async-graphql` - GraphQL schema and resolver patterns  
- `serde` - Serialization patterns for blockchain state
- `tokio` - Async patterns (for testing)

### Combined Workflow Example

For adding a new game management feature:

1. **Serena**: `get_symbols_overview` on `src/infrastructure/state.rs` to see existing game methods
2. **Serena**: `find_symbol` for `GameHubState/approve_game` to understand the pattern
3. **Context7**: Look up Linera SDK documentation for MapView operations
4. **Serena**: `search_for_pattern` to find similar validation patterns
5. **Serena**: Use `replace_symbol_body` or `insert_after_symbol` to add the new method

## Critical Discovery: Linera Collection Iteration

**IMPORTANT DISCOVERY**: During development, it was initially assumed that Linera SDK's MapView/SetView collections did not support iteration. This led to implementing placeholder methods that returned empty vectors.

**However**, research using Context7 MCP and examination of Linera examples revealed that **iteration IS supported** via the `indices()` method:

```rust
// Example from linera-protocol/examples/matching-engine
for order_id in self.state.orders.indices().await.unwrap() {
    // Process each order
}
```

**Additional Collection Iteration Patterns**:

Research indicates that **other Linera collection types also support iteration**:

- **`QueueView`**: Likely supports `iter()` or similar methods for queue traversal
- **`LogView`**: Probably supports iteration over log entries, possibly with range queries
- **`CollectionView`**: Should support iteration when values are other views
- **`for_each_index_while()`**: Advanced iteration method found in examples for conditional iteration

```rust
// Advanced iteration pattern from matching-engine example
self.state.asks.for_each_index_while(|price_ask| {
    let matches = price_ask.to_price() <= *price;
    if matches {
        matching_price_asks.push(price_ask);
    }
    Ok(matches)
}).await.expect("Failed to iterate over ask prices");
```

**Key Lessons**:
1. **Always verify assumptions** using Context7 for official documentation
2. **Check actual examples** in Linera repositories, not just basic tutorials  
3. **Search for patterns** like `indices().await`, `iter()`, `for_each_index_while()` in real codebases
4. **Don't assume limitations** without thorough investigation
5. **Explore advanced iteration methods** for performance-critical operations

**Impact on This Project**:
- Role listing methods (`get_all_admins`, `get_all_moderators`) ✅ **FULLY IMPLEMENTED** with real SetView iteration
- Audit log retrieval methods ✅ **FULLY IMPLEMENTED** with real MapView iteration and chronological sorting
- Future collection operations should leverage appropriate iteration patterns based on collection type
- Performance optimization opportunities exist with conditional iteration methods

This discovery transformed placeholder implementations into fully functional features, significantly improving the user experience and API completeness. **All placeholder methods have been replaced with real implementations using `indices().await` pattern.**

## Critical Discovery: Linera Time Handling and WASM Compatibility

**MAJOR DISCOVERY**: During integration testing, we discovered a critical WASM compatibility issue with time handling in Linera blockchain applications.

### **The Problem: SystemTime::now() Causes WASM Runtime Panics**

Initially, the `time_utils.rs` module used standard Rust time handling:

```rust
use std::time::{SystemTime, UNIX_EPOCH}; // ❌ WASM INCOMPATIBLE

pub fn now() -> Timestamp {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Timestamp::from(duration.as_micros() as u64)
}
```

**This caused severe WASM runtime panics in integration tests:**
```
panicked at 'Failed to call contract: ChainError("Abort unreachable executed")'
```

### **Root Cause Analysis**

1. **WASM Environment Limitations**: The `std::time` module is not fully supported in WASM environments
2. **Blockchain Determinism**: Blockchain applications require deterministic time sources for consensus
3. **Linera Design**: Linera provides blockchain-consistent time through the runtime context, not system calls

### **The Solution: Use runtime.system_time()**

**Proper Linera time handling pattern:**

```rust
// ✅ CORRECT: In contract operations
let timestamp = self.runtime.system_time();

// ✅ CORRECT: Pass timestamp as parameter to state methods
self.state.register_player(discord_id, username, avatar_url, timestamp).await
```

### **Complete Implementation Changes**

#### **1. Contract Layer (`src/contract.rs`)**
```rust
// Before: ❌ WASM incompatible
Operation::RegisterPlayer { discord_id, username, avatar_url } => {
    // time_utils::now() called internally
    self.state.register_player(&discord_id, &username, avatar_url).await
}

// After: ✅ WASM compatible  
Operation::RegisterPlayer { discord_id, username, avatar_url } => {
    let timestamp = self.runtime.system_time(); // Blockchain time
    match self.state.register_or_update_player(&discord_id, &username, avatar_url, timestamp).await {
        Ok(_) => format!("Player {} registered successfully", discord_id),
        Err(error) => format!("Registration failed: {}", error),
    }
}
```

#### **2. State Layer (`src/infrastructure/state.rs`)**
```rust
// Before: ❌ Internal time generation
pub async fn register_player(&mut self, discord_id: &str, username: &str, avatar_url: Option<String>) -> Result<Player, GameHubError> {
    let timestamp = time_utils::now(); // WASM panic source
    // ... rest of method
}

// After: ✅ External timestamp parameter
pub async fn register_or_update_player(&mut self, discord_id: &str, username: &str, avatar_url: Option<String>, timestamp: Timestamp) -> Result<Player, GameHubError> {
    // Use provided blockchain timestamp
    let new_player = Player {
        created_at: timestamp,
        last_active: timestamp,
        // ... rest of fields
    };
}
```

#### **3. Time Utilities (`src/infrastructure/time_utils.rs`)**
```rust
// Before: ❌ WASM incompatible
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> Timestamp {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Timestamp::from(duration.as_micros() as u64)
}

// After: ✅ WASM compatible
use linera_sdk::linera_base_types::Timestamp;

/// Create a timestamp from microseconds (blockchain-compatible utility)
pub fn from_micros(micros: u64) -> Timestamp {
    Timestamp::from(micros)
}

/// Time calculation utilities
pub mod calculations {
    // Pure timestamp manipulation functions only
    // No system time calls
}
```

### **Testing Impact**

The fix had dramatic impact on integration test results:

- **Before**: 0/23 integration tests passing (all WASM runtime panics)
- **After**: 18/23 integration tests passing (5 remaining failures are minor GraphQL syntax issues)
- **Unit tests**: Maintained 192/192 passing throughout

### **Key Lessons for Linera Development**

#### **❌ Never Use in WASM Context:**
- `std::time::SystemTime::now()`
- `std::time::Instant::now()` 
- `chrono::Utc::now()` (unless WASM-compatible features enabled)
- Any system-level time APIs

#### **✅ Always Use in Linera Context:**
- `runtime.system_time()` for contract operations
- `Timestamp::from(micros)` for utilities
- Pass timestamps as parameters to state methods
- Use blockchain-consistent time throughout

#### **🔧 Architecture Pattern:**
1. **Contract Layer**: Generate timestamps using `runtime.system_time()`
2. **State Layer**: Accept timestamps as parameters, never generate internally
3. **Service Layer**: Query existing timestamps, never generate new ones
4. **Test Layer**: Use mock timestamps or TestValidator time manipulation

#### **🧪 Testing Time-Based Logic:**
```rust
// ✅ CORRECT: Mock time in tests
#[cfg(test)]
let mock_timestamp = Timestamp::from(1000000);

// ✅ CORRECT: TestValidator time manipulation  
validator.clock().add(TimeDelta::from_secs(3600));
let current_time = validator.clock().current_time();
```

### **Debugging Techniques**

When encountering WASM runtime panics:

1. **Check for `std::time` imports** - remove all system time dependencies
2. **Trace panic origins** - WASM errors often mask the real source
3. **Test incrementally** - isolate time-related operations
4. **Use Context7 MCP** - research official Linera time handling patterns
5. **Check official examples** - verify patterns in working Linera applications

### **Performance Considerations**

- **Blockchain time is deterministic** and consensus-safe across nodes
- **No performance penalty** - `runtime.system_time()` is optimized for blockchain use
- **Timestamp propagation** adds method parameters but ensures correctness
- **Testing benefits** - mockable time enables comprehensive time-based testing

This discovery was crucial for achieving WASM compatibility and demonstrates the importance of following blockchain-specific patterns rather than standard Rust practices when developing for Linera.

## Linera Testing Patterns from Official Examples

**Source**: Analyzed from `.linera/examples/` directory containing 10 comprehensive Linera application examples

### **📊 Testing Pattern Categories**

#### **1. Integration Tests (Chain-Level Testing)**
**Examples**: `01-basic-application/tests/single_chain.rs`, `02-cross-chain-tokens/tests/cross_chain.rs`, `09-multiplayer-gaming/tests/hex_game.rs`

**Key Patterns:**
```rust
#[tokio::test(flavor = "multi_thread")]
async fn integration_test() {
    // Create isolated blockchain environment
    let (validator, module_id) = TestValidator::with_current_module::<MyAbi, (), InitialState>().await;
    let mut chain = validator.new_chain().await;
    
    // Deploy application with initial state
    let application_id = chain.create_application(module_id, params, initial_state, vec![]).await;
    
    // Execute operations via blocks
    chain.add_block(|block| {
        block.with_operation(application_id, operation);
    }).await;
    
    // Verify state via GraphQL queries
    let QueryOutcome { response, .. } = chain.graphql_query(application_id, "query { value }").await;
    assert_eq!(response["value"].as_u64().unwrap(), expected_value);
}
```

**Advanced Patterns:**
- **Cross-chain testing**: `receiver_chain.handle_received_messages().await`
- **Message bouncing**: `MessageAction::Reject` for testing failure scenarios
- **Multi-chain operations**: Multiple `new_chain()` calls for cross-chain workflows
- **Certificate handling**: `certificate.outgoing_message_count()` for message verification
- **Clock manipulation**: `validator.clock().add(duration)` for time-based testing

#### **2. Unit Tests (Contract & Service Level Testing)**
**Examples**: `04-external-api-integration/src/unit_tests/contract.rs`, `04-external-api-integration/src/unit_tests/service.rs`

**Contract Unit Testing Pattern:**
```rust
#[test]
fn contract_unit_test() {
    let mut contract = create_contract();
    
    // Execute operations directly
    contract.execute_operation(Operation::SomeOperation(data)).blocking_wait();
    
    // Test with expected panics
    #[should_panic(expected = "specific error message")]
    contract.execute_operation(Operation::FailingOperation).blocking_wait();
}

fn create_contract() -> Contract {
    let runtime = ContractRuntime::new();
    Contract { runtime }
}
```

**Service Unit Testing Pattern:**
```rust  
#[test]
fn service_unit_test() {
    let mut service = create_service();
    let runtime = Arc::get_mut(&mut service.runtime).unwrap();
    
    // Mock external dependencies
    runtime.add_expected_http_request(
        http::Request::get(url),
        http::Response::ok(response_data),
    );
    
    // Execute GraphQL queries
    let request = async_graphql::Request::new("query { data }");
    let response = service.handle_query(request).blocking_wait();
    
    // Verify scheduled operations
    let operations = service.runtime.scheduled_operations::<Operation>();
    assert_eq!(operations, expected_operations);
}
```

#### **3. State Management Testing**
**Examples**: State testing patterns from `02-cross-chain-tokens/src/state.rs`

**State Method Testing:**
```rust
impl State {
    // Async state operations with proper error handling
    pub(crate) async fn balance(&self, account: &AccountOwner) -> Option<Amount> {
        self.accounts.get(account).await.expect("Failure in retrieval")
    }
    
    // State mutation with validation
    pub(crate) async fn debit(&mut self, account: AccountOwner, amount: Amount) {
        let mut balance = self.balance_or_default(&account).await;
        balance.try_sub_assign(amount).unwrap_or_else(|_| {
            panic!("Insufficient balance for transfer")
        });
        
        if balance == Amount::ZERO {
            self.accounts.remove(&account).expect("Failed to remove empty account");
        } else {
            self.accounts.insert(&account, balance).expect("Failed insertion");
        }
    }
}
```

#### **4. Advanced Testing Features**

**HTTP Request Mocking:**
```rust
// Mock external API calls
runtime.add_expected_http_request(
    http::Request::get("http://api.example.com/data"),
    http::Response::ok(b"response_data".to_vec()),
);

// Test HTTP failures
runtime.add_expected_http_request(
    http::Request::get(url),
    http::Response::unauthorized(), // 401 status
);
```

**GraphQL Query Mocking:**
```rust
// Mock service-as-oracle queries
runtime.add_expected_service_query(
    application_id,
    async_graphql::Request::new("query { performHttpRequest }"),
    async_graphql::Response::new(expected_graphql_response),
);
```

**Time-Based Testing:**
```rust
// Manipulate blockchain time for timeout testing
let time = validator.clock().current_time();
validator.clock().add(TimeDelta::from_secs(60));

chain.add_block(|block| {
    block.with_operation(app_id, operation).with_timestamp(time);
}).await;
```

### **🎯 Testing Strategy Recommendations for GameHub**

#### **Immediate Opportunities (High Impact)**

1. **Integration Test Enhancement:**
   - Create `tests/integration/` directory following Linera patterns
   - Test complete player registration → game participation → leaderboard workflows
   - Add cross-chain batch update testing for multiple game scenarios
   - Implement TestValidator-based deployment and operation testing

2. **Unit Test Expansion:**
   - Add contract unit tests in `src/tests/unit/contract_tests.rs`
   - Add service unit tests in `src/tests/unit/service_tests.rs`  
   - Test isolated business logic with mocked ContractRuntime/ServiceRuntime
   - Use `blocking_wait()` pattern for synchronous test execution

3. **State Testing Enhancement:**
   - Test MapView/SetView operations directly with async patterns
   - Add comprehensive error scenario testing with expected panics
   - Test collection operations (`indices().await`) under various conditions

#### **Implementation Patterns to Follow**

1. **Use TestValidator for blockchain simulation** instead of complex mocking
2. **Follow `#[tokio::test(flavor = "multi_thread")]` for integration tests**
3. **Use `blocking_wait()` for synchronous test execution** in unit tests
4. **Implement helper functions** like `create_contract()` and `create_service()`
5. **Test both success and failure scenarios** with proper error assertions
6. **Use GraphQL queries** for state verification in integration tests

#### **Testing Coverage Gaps to Address**

- **Runtime context testing** for different blockchain conditions
- **Cross-chain message handling** for game registration and batch updates
- **Time-based operations** for streak calculations and grace periods
- **Permission validation** under various admin/moderator scenarios
- **Large dataset simulation** for performance and scalability testing

This comprehensive testing approach, based on proven Linera patterns, would significantly enhance GameHub's reliability and maintainability while following established blockchain testing best practices.

## GraphQL API Documentation

GameHub provides a comprehensive GraphQL API for querying blockchain state and executing operations. The API follows Linera patterns and provides both queries and mutations for all major functionality.

### **GraphQL Service Architecture**

The GraphQL service is implemented in `src/service.rs` using `async-graphql` and follows Linera patterns:
- **Query Root**: Provides read-only access to blockchain state
- **Mutation Root**: Provides convenience mutations with enhanced developer experience  
- **Schema Builder**: Integrates with Linera ServiceRuntime for blockchain operations
- **Type Mapping**: Converts domain types to GraphQL-compatible objects

### **Available Queries**

#### **Player Management**
```graphql
# Get player details by Discord ID
player(discordId: String): PlayerObject

# Get player statistics with computed metrics
playerStats(discordId: String): PlayerStatsObject

# Check if player exists
playerExists(discordId: String): Boolean

# Get top players leaderboard
leaderboard(limit: Int): [LeaderboardEntryObject]

# Get unregistered players with pending data
pendingPlayers: [PendingPlayerDataObject]
```

#### **Game Management**
```graphql
# Get all approved games
approvedGames: [GameObject]

# Get games awaiting approval (admin only)
pendingGames: [PendingGameObject]

# Check if specific game is approved
gameApproved(gameId: String): Boolean
```

#### **Administrative Queries**
```graphql
# Get audit log with activity tracking (admin)
auditLog(limit: Int): [AuditLogEntryObject]

# Get current scoring configuration
scoringConfig: ScoringConfigObject
```

### **Enhanced Mutations**

The API provides convenience mutations for common operations:

#### **Player Operations**
```graphql
# Register new player
mutation {
  registerPlayer(
    discordId: String!
    username: String!
    avatarUrl: String
  ): String
}

# Update player profile
mutation {
  updatePlayerProfile(
    discordId: String!
    username: String
    avatarUrl: String
  ): String
}
```

#### **Administrative Operations**
```graphql
# Ban player (admin only)
mutation {
  banPlayer(
    adminDiscordId: String!
    playerDiscordId: String!
    reason: String!
  ): String
}

# Submit game for approval
mutation {
  submitGame(
    gameName: String!
    description: String!
    contractAddress: String!
    developerName: String!
    developerContact: String!
  ): String
}
```

### **GraphQL Type System**

#### **Player Types**
- `PlayerObject`: Complete player information with status and statistics
- `PlayerStatsObject`: Extended statistics with participation metrics
- `PlayerStatusType`: Enum (ACTIVE, BANNED, SUSPENDED)
- `LeaderboardEntryObject`: Ranked player with score and completion data

#### **Game Types**
- `GameObject`: Approved game with developer info and status
- `PendingGameObject`: Game awaiting approval review
- `GameStatusType`: Enum (ACTIVE, SUSPENDED, DEPRECATED)

#### **Administrative Types**
- `AuditLogEntryObject`: Administrative action tracking
- `ScoringConfigObject`: Scoring rules and booster configuration

### **API Usage Examples**

#### **Query Player Leaderboard**
```graphql
query GetLeaderboard {
  leaderboard(limit: 10) {
    playerDiscordId
    playerUsername
    score
    rank
    pointsEarned
  }
}
```

#### **Get Player Statistics**
```graphql
query GetPlayerStats($discordId: String!) {
  playerStats(discordId: $discordId) {
    discordId
    username
    totalPoints
    participationStreak
    currentRank
    eventsParticipated
    averageScore
  }
}
```

#### **Check Scoring Configuration**
```graphql
query GetScoringConfig {
  scoringConfig {
    basePointsPerEvent
    streakGracePeriodHours
    bronzeBoosterThreshold
    silverBoosterThreshold
    goldBoosterThreshold
    bronzeMultiplier
    silverMultiplier
    goldMultiplier
  }
}
```

#### **Register New Player**
```graphql
mutation RegisterPlayer($input: RegisterPlayerInput!) {
  registerPlayer(
    discordId: $input.discordId
    username: $input.username
    avatarUrl: $input.avatarUrl
  )
}
```

### **GraphQL Testing**

The service includes comprehensive tests in `src/service.rs`:
- **Introspection Tests**: Schema validation and type discovery
- **Query Tests**: All major queries with realistic data scenarios
- **Mutation Tests**: Enhanced mutations with parameter validation
- **Error Handling**: GraphQL error responses and edge cases

### **Integration with Frontend**

The GraphQL API is designed for easy frontend integration:
- **Standardized Naming**: CamelCase field names following GraphQL conventions
- **Type Safety**: Strong typing with comprehensive object types
- **Pagination**: Limit parameters for large data sets
- **Error Handling**: Structured error responses with detailed messages
- **Real-time Queries**: Efficient state querying with blockchain integration

### **Performance Considerations**

- **Efficient State Access**: Direct GameHubState method calls with minimal overhead
- **Collection Iteration**: Uses Linera SDK `indices().await` pattern for large datasets
- **Lazy Loading**: Optional fields and computed statistics only when requested
- **Caching Strategy**: Leverage Linera SDK's built-in state caching
- **Batch Operations**: Support for bulk queries and mutations

## Important Notes

- **WASM Compatibility**: All dependencies must be WASM-compatible for Linera deployment
- **Permission Model**: Admin-only operations require `validate_admin_permission()` 
- **Audit Trail**: All admin actions automatically logged with `add_audit_log_entry()`
- **Cross-Chain Ready**: Architecture supports cross-chain game integration
- **Performance**: Designed for batch processing of player updates across multiple games
- **MCP Efficiency**: Use Serena for codebase exploration, Context7 for library documentation
- **Token Management**: Prefer symbolic operations over full file reads when possible
- **GraphQL API**: Comprehensive query and mutation support following Linera patterns