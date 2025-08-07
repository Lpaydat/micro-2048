# 05 - DeFi AMM (Automated Market Maker)

## 📋 Description

A comprehensive Automated Market Maker (AMM) implementation demonstrating DeFi protocols, liquidity management, and atomic swaps. This example shows how to build decentralized exchange functionality with liquidity pools and trading mechanisms.

## 🎯 Key Features Demonstrated

- ✅ **Liquidity Pool Management** - Add/remove liquidity operations
- ✅ **Token Swapping** - Automated market making with price discovery
- ✅ **Mathematical Formulas** - Constant product formula (x * y = k)
- ✅ **Atomic Operations** - Ensuring transaction atomicity
- ✅ **Multi-Token Interactions** - Coordinating between different tokens
- ✅ **Temporary Chains** - Atomic swaps using temporary chains
- ✅ **Application Composition** - Building on fungible token example

## 🏗️ Architecture Overview

```
AMM Protocol
├── Contract (src/contract.rs)
│   ├── State: Token reserves, liquidity positions
│   ├── Operations: Swap, AddLiquidity, RemoveLiquidity
│   ├── Logic: Constant product formula
│   └── Integration: Fungible token calls
├── Service (src/service.rs)
│   ├── Queries: Pool state, prices, liquidity
│   ├── Mutations: Trading operations
│   └── Price calculations
└── Dependencies
    ├── Token0: First fungible token
    ├── Token1: Second fungible token
    └── Cross-application calls
```

## 🎨 Code Patterns

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct AmmState {
    pub token0_reserve: RegisterView<Amount>,
    pub token1_reserve: RegisterView<Amount>,
    pub liquidity_owners: MapView<AccountOwner, Amount>,
    pub total_liquidity: RegisterView<Amount>,
}
```

### Swap Operation
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Swap { owner, input_token_idx, input_amount } => {
            let (reserve_in, reserve_out) = if input_token_idx == 0 {
                (*self.state.token0_reserve.get(), *self.state.token1_reserve.get())
            } else {
                (*self.state.token1_reserve.get(), *self.state.token0_reserve.get())
            };
            
            // Constant product formula: x * y = k
            let output_amount = self.calculate_output_amount(
                input_amount, reserve_in, reserve_out
            )?;
            
            // Transfer tokens via cross-application calls
            self.transfer_tokens(owner, input_token_idx, input_amount, output_amount).await?;
            
            Ok(output_amount)
        }
    }
}
```

### Liquidity Management
```rust
Operation::AddLiquidity { owner, max_token0_amount, max_token1_amount } => {
    let (token0_amount, token1_amount) = self.calculate_liquidity_amounts(
        max_token0_amount, max_token1_amount
    )?;
    
    let liquidity_minted = self.calculate_liquidity_minted(
        token0_amount, token1_amount
    )?;
    
    // Transfer tokens from user to AMM
    self.transfer_tokens_to_amm(owner, token0_amount, token1_amount).await?;
    
    // Update reserves and mint liquidity tokens
    self.update_reserves(token0_amount, token1_amount).await;
    self.mint_liquidity(owner, liquidity_minted).await;
    
    Ok((token0_amount, token1_amount, liquidity_minted))
}
```

### Price Calculation
```rust
fn calculate_output_amount(
    &self,
    input_amount: Amount,
    reserve_in: Amount,
    reserve_out: Amount,
) -> Result<Amount, AmmError> {
    // Constant product formula with 0.3% fee
    let input_amount_with_fee = input_amount * 997; // 0.3% fee
    let numerator = input_amount_with_fee * reserve_out;
    let denominator = (reserve_in * 1000) + input_amount_with_fee;
    
    Ok(numerator / denominator)
}
```

## 🚀 Use Cases

### **Perfect For:**
- 🏦 **Decentralized Exchanges** - Token trading platforms
- 💱 **Currency Exchange** - Multi-currency conversion
- 🎮 **Gaming Economies** - In-game asset trading
- 💰 **DeFi Protocols** - Yield farming and liquidity mining
- 🏢 **Corporate Trading** - Internal asset exchanges
- 🌐 **Cross-Chain Bridges** - Asset swapping across chains

### **Real-World Applications:**
- Uniswap-style DEXs
- Gaming asset marketplaces
- Corporate internal exchanges
- Stablecoin trading pairs
- Yield farming protocols
- Liquidity mining programs

## 📚 When to Reference This Example

### **Building DeFi Applications**
- Copy AMM mathematical formulas
- Adapt liquidity pool management
- Use atomic swap patterns
- Reference price discovery mechanisms

### **Creating Trading Systems**
- Learn automated market making
- Understand liquidity provision
- See fee calculation methods
- Study slippage protection

### **Multi-Token Applications**
- Coordinate between multiple token contracts
- Handle cross-application calls
- Manage complex state interactions
- Implement atomic operations

## 🔗 Related Examples

### **Builds Upon:**
- **02-cross-chain-tokens** - Requires fungible tokens
- **01-basic-application** - Basic Linera patterns

### **Combines With:**
- **06-trading-engine** - Order book + AMM hybrid
- **04-external-api-integration** - External price feeds
- **08-ai-integration** - AI-powered trading strategies

### **Extends To:**
- Multi-asset AMMs
- Concentrated liquidity (Uniswap V3 style)
- Yield farming protocols
- Governance token systems

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - AMM ABI and types
- `src/contract.rs` - AMM core logic
- `src/service.rs` - AMM GraphQL API
- `src/state.rs` - Pool state management

### **Dependencies:**
- Two fungible token applications
- Cross-application call capabilities
- Mathematical precision handling

### **Testing:**
- Multi-token test scenarios
- Price calculation verification
- Liquidity management testing
- Atomic operation validation

## 💡 Customization Ideas

### **AMM Features:**
- Add multiple trading pairs
- Implement concentrated liquidity
- Add governance token rewards
- Create flash loan functionality

### **Advanced Features:**
- Multi-hop swaps (A→B→C)
- Impermanent loss protection
- Dynamic fee structures
- MEV protection mechanisms

### **Integration Features:**
- Yield farming rewards
- Governance voting
- Cross-chain liquidity
- Oracle price feeds

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **AMM Mathematics** - Constant product formula and price discovery
2. **Liquidity Management** - Adding/removing liquidity from pools
3. **Atomic Operations** - Ensuring transaction consistency
4. **Cross-Application Calls** - Interacting with other applications
5. **DeFi Protocols** - Core decentralized finance concepts
6. **Fee Mechanisms** - Trading fees and liquidity incentives
7. **Slippage Protection** - Managing price impact

## 💰 Economic Model

### **Liquidity Providers**
- Earn trading fees (0.3% of each swap)
- Receive liquidity tokens representing pool share
- Subject to impermanent loss risk

### **Traders**
- Pay 0.3% fee on each swap
- Get guaranteed execution (no order book needed)
- Price impact based on trade size vs pool size

### **Pool Dynamics**
- Larger pools = lower slippage
- More trading = more fees for LPs
- Arbitrage keeps prices in line with external markets

## 📈 Complexity: ⭐⭐⭐ Advanced

**Time to Understand:** 6-10 hours
**Prerequisites:** 02-cross-chain-tokens, DeFi concepts, mathematical formulas
**Next Example:** 06-trading-engine for order book patterns