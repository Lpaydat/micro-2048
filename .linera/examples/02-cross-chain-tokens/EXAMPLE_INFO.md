# 02 - Cross-Chain Tokens

## 📋 Description

A comprehensive fungible token system demonstrating cross-chain messaging, account management, and multi-chain operations. This example shows how to create tokens that can be transferred between different chains while maintaining consistent state.

## 🎯 Key Features Demonstrated

- ✅ **Cross-Chain Messaging** - Transfer and Claim operations across chains
- ✅ **Account Management** - Multi-chain account balances
- ✅ **Application Instantiation** - Creating token instances with parameters
- ✅ **Complex State Management** - BTreeMap for efficient account lookups
- ✅ **Multi-Chain Operations** - Coordinated operations across multiple chains
- ✅ **Token Economics** - Minting, transferring, and balance management

## 🏗️ Architecture Overview

```
Fungible Token System
├── Contract (src/contract.rs)
│   ├── State: BTreeMap<Owner, Amount>
│   ├── Operations: Transfer, Claim
│   ├── Messages: Credit, Withdraw
│   └── Logic: Cross-chain token movement
├── Service (src/service.rs)
│   ├── Queries: Account balances
│   ├── Mutations: Transfer operations
│   └── Multi-chain state aggregation
└── Frontend (web-frontend/)
    ├── Account management UI
    ├── Transfer interface
    └── Multi-chain balance display
```

## 🎨 Code Patterns

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct FungibleTokenState {
    pub accounts: MapView<AccountOwner, Amount>,
    pub total_supply: RegisterView<Amount>,
}
```

### Cross-Chain Messages
```rust
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Credit { target: Account, amount: Amount },
    Withdraw { owner: AccountOwner, amount: Amount, target_account: Account },
}
```

### Transfer Operation
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Transfer { owner, amount, target_account } => {
            self.debit(owner, amount).await?;
            if target_account.chain_id == self.runtime.chain_id() {
                self.credit(target_account.owner, amount).await;
            } else {
                let message = Message::Credit { target: target_account, amount };
                self.runtime.send_message(target_account.chain_id, message);
            }
            Ok(())
        }
    }
}
```

### Message Handling
```rust
async fn execute_message(&mut self, message: Message) {
    match message {
        Message::Credit { target, amount } => {
            self.credit(target.owner, amount).await;
        }
        Message::Withdraw { owner, amount, target_account } => {
            self.debit(owner, amount).await?;
            let message = Message::Credit { target: target_account, amount };
            self.runtime.send_message(target_account.chain_id, message);
        }
    }
}
```

## 🚀 Use Cases

### **Perfect For:**
- 💰 **Token Systems** - Creating custom cryptocurrencies
- 🏦 **DeFi Applications** - Foundation for financial protocols
- 🎮 **Gaming Tokens** - In-game currencies and rewards
- 🏢 **Corporate Tokens** - Internal company currencies
- 🌐 **Cross-Chain Applications** - Multi-chain asset management
- 📊 **Payment Systems** - Digital payment solutions

### **Real-World Applications:**
- Stablecoins and cryptocurrencies
- Loyalty points and rewards
- Gaming economies
- Corporate internal currencies
- Cross-border payments
- DeFi protocol tokens

## 📚 When to Reference This Example

### **Building Token-Based Systems**
- Copy token creation and management patterns
- Adapt cross-chain messaging for your use case
- Use account management structures
- Reference balance tracking methods

### **Implementing Cross-Chain Features**
- Learn message passing between chains
- Understand state synchronization
- See error handling for failed transfers
- Study multi-chain coordination patterns

### **DeFi Development**
- Foundation for AMM protocols
- Base for lending/borrowing systems
- Template for staking mechanisms
- Starting point for yield farming

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns

### **Extends To:**
- **05-defi-amm** - Uses fungible tokens for liquidity
- **06-trading-engine** - Trades between fungible tokens
- **10-crowdfunding** - Uses tokens for campaign funding

### **Combines With:**
- **04-external-api-integration** - Price feeds for tokens
- **08-ai-integration** - AI-powered trading bots

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - Token ABI and types
- `src/contract.rs` - Core token logic
- `src/service.rs` - GraphQL token API
- `src/state.rs` - Account state management
- `web-frontend/` - Token management UI

### **Testing:**
- Multi-chain test scenarios
- Cross-chain message testing
- Balance consistency verification
- Error handling validation

### **Deployment:**
- Requires multiple chains for full testing
- Module publication before instance creation
- Parameter configuration for token properties

## 💡 Customization Ideas

### **Token Features:**
- Add token metadata (name, symbol, decimals)
- Implement token burning mechanisms
- Add transfer fees or taxes
- Create token vesting schedules

### **Advanced Features:**
- Multi-signature token approvals
- Token staking and rewards
- Governance token voting
- Token wrapping/unwrapping

### **Integration Patterns:**
- Oracle price feeds
- Automated market makers
- Lending protocols
- Cross-chain bridges

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Cross-Chain Messaging** - How chains communicate via messages
2. **State Consistency** - Maintaining consistent state across chains
3. **Account Management** - Multi-chain account balance tracking
4. **Token Economics** - Supply, transfers, and balance management
5. **Error Handling** - Dealing with failed cross-chain operations
6. **Application Parameters** - Configuring applications at creation
7. **Complex State Structures** - Using BTreeMap for efficient lookups

## 🔄 Message Flow Example

```
Chain A (User has 100 tokens)
    ↓ Transfer 50 tokens to Chain B
    ↓ Debit 50 from Chain A account
    ↓ Send Credit message to Chain B
Chain B (Receives message)
    ↓ Credit 50 to user account on Chain B
    ↓ User now has 50 on Chain A, 50 on Chain B
```

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 4-8 hours
**Prerequisites:** 01-basic-application, understanding of blockchain concepts
**Next Example:** 03-social-messaging or 05-defi-amm