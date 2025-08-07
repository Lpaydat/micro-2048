# 09 - Multiplayer Gaming

## 📋 Description

A turn-based multiplayer game (Hex) demonstrating gaming applications on Linera, shared temporary chains, multi-player coordination, and game state management. This example shows how to build competitive gaming systems with fair play mechanics.

## 🎯 Key Features Demonstrated

- ✅ **Turn-Based Game Logic** - Structured game flow and rules
- ✅ **Shared Temporary Chains** - Multi-player coordination on shared chains
- ✅ **Game State Management** - Board state, player turns, win conditions
- ✅ **Multi-Player Coordination** - Managing multiple players in one game
- ✅ **Time-Based Operations** - Game timers and move deadlines
- ✅ **Fair Play Mechanics** - Preventing cheating and ensuring fairness
- ✅ **Game Session Management** - Creating and managing game instances

## 🏗️ Architecture Overview

```
Multiplayer Gaming System
├── Main Chain (src/contract.rs)
│   ├── State: Active games, player registry
│   ├── Operations: StartGame, JoinGame
│   └── Logic: Game creation and management
├── Game Chain (Temporary)
│   ├── State: Board state, current turn
│   ├── Operations: MakeMove, Resign
│   ├── Logic: Game rules and win detection
│   └── Players: Shared ownership
└── Frontend (web-frontend/)
    ├── Game board UI
    ├── Player management
    └── Real-time game updates
```

## 🎨 Code Patterns

### Game State Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub players: [PublicKey; 2],
    pub game_status: GameStatus,
    pub move_history: Vec<Move>,
    pub start_time: Timestamp,
    pub last_move_time: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    pub size: u8,
    pub cells: HashMap<Position, Player>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}
```

### Game Creation
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Start { players, board_size, fee_budget } => {
            // Create temporary chain for the game
            let game_chain_id = self.runtime.create_temporary_chain(
                players.clone(),
                fee_budget,
            ).await?;
            
            // Initialize game state on temporary chain
            let game_state = GameState {
                board: Board::new(board_size),
                current_player: Player::One,
                players,
                game_status: GameStatus::InProgress,
                move_history: Vec::new(),
                start_time: self.runtime.system_time(),
                last_move_time: self.runtime.system_time(),
            };
            
            // Send initialization message to game chain
            let message = Message::InitializeGame { game_state };
            self.runtime.send_message(game_chain_id, message);
            
            // Track active game
            self.state.active_games.insert(&players[0], GameInfo {
                chain_id: game_chain_id,
                players,
                created_at: self.runtime.system_time(),
            })?;
            
            Ok(game_chain_id)
        }
    }
}
```

### Move Execution
```rust
// On game chain
async fn execute_operation(&mut self, operation: GameOperation) -> Self::Response {
    match operation {
        GameOperation::MakeMove { x, y } => {
            let player = self.get_current_player()?;
            let position = Position { x, y };
            
            // Validate move
            self.validate_move(&position, player)?;
            
            // Update board state
            self.state.board.cells.insert(position, player);
            self.state.move_history.push(Move {
                player,
                position,
                timestamp: self.runtime.system_time(),
            });
            
            // Check for win condition
            if self.check_win_condition(player, position)? {
                self.state.game_status = GameStatus::Won(player);
                self.handle_game_end(player).await?;
            } else {
                // Switch to next player
                self.state.current_player = self.next_player();
                self.state.last_move_time = self.runtime.system_time();
            }
            
            Ok(())
        }
    }
}
```

### Win Condition Detection (Hex Game)
```rust
fn check_win_condition(&self, player: Player, last_move: Position) -> Result<bool, GameError> {
    match player {
        Player::One => {
            // Player One wins by connecting left and right sides
            self.has_path_left_to_right(player)
        }
        Player::Two => {
            // Player Two wins by connecting top and bottom
            self.has_path_top_to_bottom(player)
        }
    }
}

fn has_path_left_to_right(&self, player: Player) -> bool {
    let left_positions: Vec<Position> = (0..self.state.board.size)
        .map(|y| Position { x: 0, y })
        .filter(|pos| self.state.board.cells.get(pos) == Some(&player))
        .collect();
    
    for start_pos in left_positions {
        if self.can_reach_right_side(player, start_pos, &mut HashSet::new()) {
            return true;
        }
    }
    
    false
}

fn can_reach_right_side(
    &self,
    player: Player,
    current: Position,
    visited: &mut HashSet<Position>,
) -> bool {
    if current.x == self.state.board.size - 1 {
        return true; // Reached right side
    }
    
    visited.insert(current);
    
    for neighbor in self.get_neighbors(current) {
        if !visited.contains(&neighbor) 
            && self.state.board.cells.get(&neighbor) == Some(&player) {
            if self.can_reach_right_side(player, neighbor, visited) {
                return true;
            }
        }
    }
    
    false
}
```

### Time Management
```rust
fn check_time_limits(&self) -> Result<(), GameError> {
    let current_time = self.runtime.system_time();
    let time_since_last_move = current_time - self.state.last_move_time;
    
    if time_since_last_move > self.move_timeout {
        // Current player loses by timeout
        let winner = self.next_player();
        self.state.game_status = GameStatus::Won(winner);
        return Err(GameError::Timeout);
    }
    
    Ok(())
}
```

## 🚀 Use Cases

### **Perfect For:**
- 🎮 **Turn-Based Games** - Chess, checkers, board games
- 🏆 **Competitive Gaming** - Tournaments and ranked matches
- 🎯 **Strategy Games** - Complex multi-player strategy
- 🎲 **Casino Games** - Poker, blackjack, dice games
- 🧩 **Puzzle Games** - Collaborative or competitive puzzles
- 🎪 **Party Games** - Social gaming experiences
- 📱 **Mobile Games** - Cross-platform gaming

### **Real-World Applications:**
- Blockchain-based chess platforms
- Competitive gaming tournaments
- Gambling and casino applications
- Educational game platforms
- Social gaming networks
- Esports and competitive gaming

## 📚 When to Reference This Example

### **Building Gaming Applications**
- Copy turn-based game logic patterns
- Adapt multi-player coordination systems
- Use shared chain management
- Reference fair play mechanisms

### **Creating Competitive Systems**
- Learn tournament and match management
- Understand player coordination
- See time-based game mechanics
- Study win condition detection

### **Multi-Player Applications**
- Implement shared state management
- Create collaborative experiences
- Build competitive interactions
- Manage player sessions

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns
- **02-cross-chain-tokens** - Multi-chain concepts

### **Combines With:**
- **07-nft-system** - Gaming NFTs and collectibles
- **02-cross-chain-tokens** - Gaming tokens and rewards
- **08-ai-integration** - AI opponents and NPCs
- **03-social-messaging** - Gaming social features

### **Extends To:**
- Real-time multiplayer games
- Massive multiplayer online games
- Gaming economies and marketplaces
- Esports and tournament systems

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - Gaming ABI and types
- `src/contract.rs` - Game logic and rules
- `src/service.rs` - Gaming GraphQL API
- `src/state.rs` - Game state management

### **Game Mechanics:**
- Hex game rules implementation
- Turn-based move validation
- Win condition algorithms
- Time limit enforcement

### **Testing:**
- Multi-player game scenarios
- Win condition verification
- Time limit testing
- Edge case handling

## 💡 Customization Ideas

### **Game Features:**
- Add different game types (Chess, Go, etc.)
- Implement spectator modes
- Create replay systems
- Add game statistics

### **Competitive Features:**
- Tournament brackets
- Ranking and rating systems
- Seasonal competitions
- Prize pools and rewards

### **Social Features:**
- Friend systems and challenges
- Chat and communication
- Game sharing and streaming
- Community features

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Game State Management** - Managing complex game states
2. **Multi-Player Coordination** - Handling multiple players
3. **Temporary Chains** - Creating shared execution environments
4. **Turn-Based Logic** - Implementing fair turn systems
5. **Win Condition Detection** - Algorithmic game ending
6. **Time Management** - Handling timeouts and deadlines
7. **Fair Play Mechanics** - Preventing cheating and ensuring fairness

## 🎲 Game Flow Example (Hex)

```
Game Start
    ↓ Create temporary chain with 2 players
    ↓ Initialize empty hex board
    ↓ Player 1 makes first move
Player 1 Turn
    ↓ Place stone at position (x, y)
    ↓ Check for win condition (left-right connection)
    ↓ Switch to Player 2
Player 2 Turn
    ↓ Place stone at position (x, y)
    ↓ Check for win condition (top-bottom connection)
    ↓ Continue until someone wins
Game End
    ↓ Declare winner
    ↓ Close temporary chain
    ↓ Distribute rewards/update rankings
```

## 🏆 Hex Game Rules

### **Objective:**
- **Player 1 (Red)**: Connect left and right sides of the board
- **Player 2 (Blue)**: Connect top and bottom sides of the board

### **Rules:**
- Players alternate placing stones on empty hexagonal cells
- Once placed, stones cannot be moved
- First player to create a continuous path wins
- No draws are possible in Hex

### **Strategy:**
- Block opponent's connections
- Build your own path efficiently
- Control center positions
- Use edge positions strategically

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 5-7 hours
**Prerequisites:** 01-basic-application, game theory concepts, algorithms
**Next Example:** 07-nft-system for gaming assets