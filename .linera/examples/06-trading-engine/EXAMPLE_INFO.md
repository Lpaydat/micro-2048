# 06 - Trading Engine

## 📋 Description

A sophisticated order book matching engine demonstrating advanced trading systems, order management, and price-time priority matching. This example shows how to build centralized exchange functionality with bid/ask orders and atomic trade execution.

## 🎯 Key Features Demonstrated

- ✅ **Order Book Management** - Bid and ask order organization
- ✅ **Price-Time Priority** - Fair order matching algorithms
- ✅ **Order Types** - Market and limit orders with modifications
- ✅ **Atomic Trading** - Ensuring trade execution consistency
- ✅ **Order Lifecycle** - Create, modify, cancel, and execute orders
- ✅ **Multi-Token Trading** - Trading between different token pairs
- ✅ **Temporary Chains** - Atomic swaps using temporary chains

## 🏗️ Architecture Overview

```
Trading Engine
├── Contract (src/contract.rs)
│   ├── State: Order books, executed trades
│   ├── Operations: InsertOrder, ModifyOrder, CancelOrder
│   ├── Logic: Order matching and execution
│   └── Integration: Fungible token transfers
├── Service (src/service.rs)
│   ├── Queries: Order book state, trade history
│   ├── Mutations: Order management operations
│   └── Real-time order book updates
└── Dependencies
    ├── Token0: First trading token
    ├── Token1: Second trading token
    └── Cross-application calls
```

## 🎨 Code Patterns

### Order Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: OrderId,
    pub owner: AccountOwner,
    pub nature: OrderNature, // Bid or Ask
    pub amount: Amount,
    pub price: Price,
    pub timestamp: Timestamp,
    pub filled_amount: Amount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderNature {
    Bid,  // Buy token1, pay with token0
    Ask,  // Sell token1, receive token0
}
```

### Order Book State
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct MatchingEngineState {
    pub bids: MapView<Price, VecDeque<Order>>,     // Highest price first
    pub asks: MapView<Price, VecDeque<Order>>,     // Lowest price first
    pub orders: MapView<OrderId, Order>,
    pub executed_trades: LogView<Trade>,
}
```

### Order Insertion and Matching
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::ExecuteOrder { order } => {
            match order {
                OrderRequest::Insert { owner, amount, nature, price } => {
                    // Transfer tokens to escrow
                    self.escrow_tokens(&owner, &nature, amount, price).await?;
                    
                    let mut new_order = Order {
                        id: self.generate_order_id(),
                        owner,
                        nature: nature.clone(),
                        amount,
                        price,
                        timestamp: self.runtime.system_time(),
                        filled_amount: Amount::ZERO,
                    };
                    
                    // Match against existing orders
                    self.match_order(&mut new_order).await?;
                    
                    // Insert remaining amount if not fully filled
                    if new_order.amount > new_order.filled_amount {
                        self.insert_order_in_book(new_order).await?;
                    }
                    
                    Ok(())
                }
            }
        }
    }
}
```

### Order Matching Logic
```rust
async fn match_order(&mut self, order: &mut Order) -> Result<(), MatchingError> {
    let matching_orders = match order.nature {
        OrderNature::Bid => self.get_matching_asks(order.price),
        OrderNature::Ask => self.get_matching_bids(order.price),
    };
    
    for mut matching_order in matching_orders {
        if order.amount <= order.filled_amount {
            break; // Order fully filled
        }
        
        let trade_amount = std::cmp::min(
            order.amount - order.filled_amount,
            matching_order.amount - matching_order.filled_amount
        );
        
        // Execute trade
        self.execute_trade(order, &mut matching_order, trade_amount).await?;
        
        // Update filled amounts
        order.filled_amount += trade_amount;
        matching_order.filled_amount += trade_amount;
        
        // Remove fully filled orders
        if matching_order.filled_amount >= matching_order.amount {
            self.remove_order_from_book(&matching_order).await?;
        }
    }
    
    Ok(())
}
```

### Trade Execution
```rust
async fn execute_trade(
    &mut self,
    taker_order: &Order,
    maker_order: &mut Order,
    amount: Amount,
) -> Result<(), MatchingError> {
    let trade_price = maker_order.price; // Price-time priority
    
    // Calculate token amounts
    let (token0_amount, token1_amount) = match taker_order.nature {
        OrderNature::Bid => (amount * trade_price, amount),
        OrderNature::Ask => (amount, amount * trade_price),
    };
    
    // Transfer tokens between parties
    self.transfer_trade_tokens(
        &taker_order.owner,
        &maker_order.owner,
        token0_amount,
        token1_amount,
    ).await?;
    
    // Record trade
    let trade = Trade {
        id: self.generate_trade_id(),
        taker: taker_order.owner,
        maker: maker_order.owner,
        amount,
        price: trade_price,
        timestamp: self.runtime.system_time(),
    };
    
    self.state.executed_trades.push(trade);
    
    Ok(())
}
```

## 🚀 Use Cases

### **Perfect For:**
- 📈 **Cryptocurrency Exchanges** - Digital asset trading platforms
- 🏦 **Financial Markets** - Stock and commodity exchanges
- 🎮 **Gaming Marketplaces** - In-game asset trading
- 💼 **Corporate Exchanges** - Internal asset trading systems
- 🌐 **Cross-Chain Trading** - Multi-blockchain asset exchange
- 📊 **Prediction Markets** - Betting and prediction platforms

### **Real-World Applications:**
- Centralized cryptocurrency exchanges
- Gaming item marketplaces
- Corporate internal trading systems
- Prediction and betting markets
- NFT trading platforms
- Commodity trading systems

## 📚 When to Reference This Example

### **Building Trading Systems**
- Copy order book management patterns
- Adapt order matching algorithms
- Use price-time priority logic
- Reference atomic trade execution

### **Creating Marketplaces**
- Learn order lifecycle management
- Understand bid/ask mechanics
- See fair pricing mechanisms
- Study trade settlement patterns

### **Financial Applications**
- Implement exchange functionality
- Build market making systems
- Create arbitrage opportunities
- Develop trading bots

## 🔗 Related Examples

### **Builds Upon:**
- **02-cross-chain-tokens** - Requires fungible tokens
- **01-basic-application** - Basic Linera patterns

### **Combines With:**
- **05-defi-amm** - Hybrid AMM + order book
- **08-ai-integration** - AI trading algorithms
- **04-external-api-integration** - External price feeds

### **Extends To:**
- Multi-asset trading engines
- Derivatives and futures trading
- Margin trading systems
- High-frequency trading

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - Trading engine ABI
- `src/contract.rs` - Order matching logic
- `src/service.rs` - Trading GraphQL API
- `src/state.rs` - Order book state

### **Dependencies:**
- Two fungible token applications
- Cross-application call capabilities
- Precise mathematical calculations

### **Testing:**
- Order matching scenarios
- Price-time priority verification
- Atomic trade execution testing
- Edge case handling

## 💡 Customization Ideas

### **Order Types:**
- Add stop-loss orders
- Implement iceberg orders
- Create time-in-force options
- Add order expiration

### **Advanced Features:**
- Multi-asset trading pairs
- Margin trading capabilities
- Derivatives and futures
- Market maker incentives

### **Integration Features:**
- External price feeds
- Trading bot APIs
- Risk management systems
- Compliance and reporting

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Order Book Mechanics** - How centralized exchanges work
2. **Price-Time Priority** - Fair order matching algorithms
3. **Atomic Operations** - Ensuring trade consistency
4. **Order Lifecycle** - Complete order management
5. **Market Microstructure** - How trading engines operate
6. **Cross-Application Calls** - Token transfer coordination
7. **Financial System Design** - Building robust trading systems

## 📊 Order Book Structure

### **Bid Orders (Buy Orders)**
```
Price | Amount | Total
$102  |   50   |  50   ← Best Bid (Highest Price)
$101  |  100   | 150
$100  |  200   | 350
```

### **Ask Orders (Sell Orders)**
```
Price | Amount | Total
$103  |   75   |  75   ← Best Ask (Lowest Price)
$104  |  125   | 200
$105  |  150   | 350
```

### **Spread**
- **Bid-Ask Spread**: $103 - $102 = $1
- **Market Price**: Between $102 and $103

## 🔄 Trade Execution Flow

```
New Buy Order @ $103
    ↓ Matches with Best Ask @ $103
    ↓ Execute trade at $103
    ↓ Transfer tokens between parties
    ↓ Update order book
    ↓ Record trade history
    ↓ Emit trade events
```

## 📈 Complexity: ⭐⭐⭐ Advanced

**Time to Understand:** 8-12 hours
**Prerequisites:** 02-cross-chain-tokens, trading concepts, financial markets
**Next Example:** 05-defi-amm for AMM comparison