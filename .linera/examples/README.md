# Linera Examples Collection

This directory contains curated examples demonstrating key Linera blockchain development patterns. Each example is self-contained and focuses on specific features or use cases.

## 🚀 Quick Start

### For Beginners
Start with these examples in order:
1. **01-basic-application** - Learn Linera fundamentals
2. **02-cross-chain-tokens** - Understand cross-chain messaging
3. **04-external-api-integration** - Connect to external APIs

### For Specific Use Cases
Jump directly to the example that matches your project needs:
- **DeFi/Trading**: Examples 05, 06
- **Social/Gaming**: Examples 03, 09
- **Digital Assets**: Example 07
- **AI Integration**: Example 08
- **Fundraising**: Example 10

## 📚 Examples Overview

| # | Name | Complexity | Key Features | Best For |
|---|------|------------|--------------|----------|
| 01 | Basic Application | ⭐ | State, Operations, GraphQL | Learning Linera |
| 02 | Cross-Chain Tokens | ⭐⭐ | Messaging, Accounts, Multi-chain | Token systems |
| 03 | Social Messaging | ⭐⭐ | Channels, Broadcasting, Subscriptions | Social apps |
| 04 | External API Integration | ⭐⭐ | HTTP requests, Oracles, Off-chain data | Real-world integration |
| 05 | DeFi AMM | ⭐⭐⭐ | Liquidity pools, Swaps, Financial math | DeFi applications |
| 06 | Trading Engine | ⭐⭐⭐ | Order books, Matching, Trading | Financial exchanges |
| 07 | NFT System | ⭐⭐ | Unique tokens, Metadata, Digital assets | NFT platforms |
| 08 | AI Integration | ⭐⭐⭐ | LLM, AI inference, Resource management | AI-powered apps |
| 09 | Multiplayer Gaming | ⭐⭐ | Turn-based games, Shared chains | Gaming applications |
| 10 | Crowdfunding | ⭐⭐ | Campaigns, Goals, Time-based logic | Fundraising platforms |

**Complexity Legend:**
- ⭐ Beginner - Good starting point
- ⭐⭐ Intermediate - Requires basic Linera knowledge  
- ⭐⭐⭐ Advanced - Complex patterns and logic

## 🎯 Feature-Based Index

### 🏗️ **Core Development Patterns**
- **Basic State Management**: 01-basic-application
- **GraphQL Integration**: 01-basic-application, 03-social-messaging
- **Operation Handling**: All examples
- **Testing Patterns**: All examples

### 🌐 **Cross-Chain Features**
- **Message Passing**: 02-cross-chain-tokens, 03-social-messaging
- **Multi-Chain State**: 02-cross-chain-tokens, 05-defi-amm
- **Channel Broadcasting**: 03-social-messaging
- **Cross-Chain Assets**: 02-cross-chain-tokens, 07-nft-system

### 🔗 **External Integration**
- **HTTP Requests**: 04-external-api-integration
- **Oracle Patterns**: 04-external-api-integration
- **Off-Chain Data**: 04-external-api-integration, 08-ai-integration

### 💰 **Financial Applications**
- **Token Systems**: 02-cross-chain-tokens
- **DeFi Protocols**: 05-defi-amm
- **Trading Systems**: 06-trading-engine
- **Fundraising**: 10-crowdfunding

### 🎨 **Digital Assets & Media**
- **NFTs**: 07-nft-system
- **Metadata Handling**: 07-nft-system
- **Blob Storage**: 07-nft-system

### 🤖 **AI & Machine Learning**
- **LLM Integration**: 08-ai-integration
- **AI Inference**: 08-ai-integration
- **Resource Management**: 08-ai-integration

### 🎮 **Gaming & Interactive**
- **Turn-Based Games**: 09-multiplayer-gaming
- **Multi-Player Coordination**: 09-multiplayer-gaming
- **Shared State**: 09-multiplayer-gaming

### 👥 **Social & Community**
- **User Interactions**: 03-social-messaging
- **Broadcasting**: 03-social-messaging
- **Subscriptions**: 03-social-messaging
- **Community Features**: 10-crowdfunding

### ⏰ **Time-Based Features**
- **Campaigns**: 10-crowdfunding
- **Deadlines**: 10-crowdfunding
- **Scheduling**: 09-multiplayer-gaming

## 🛠️ **Development Workflow**

### 1. Choose Your Example
Use the feature index above to find examples that match your needs.

### 2. Study the Code
Each example includes:
- `src/` - Smart contract implementation
- `web-frontend/` - Frontend integration (where applicable)
- `README.md` - Detailed usage instructions
- `Cargo.toml` - Dependencies and configuration

### 3. Run the Example
Follow the README instructions in each example directory.

### 4. Adapt for Your Project
Copy patterns and modify them for your specific use case.

## 📖 **Learning Path**

### Beginner Path (2-3 weeks)
1. **Week 1**: 01-basic-application + 02-cross-chain-tokens
2. **Week 2**: 04-external-api-integration + 03-social-messaging  
3. **Week 3**: Choose one specialized example (05-10) based on your project

### Advanced Path (1-2 weeks)
If you're experienced with blockchain development:
1. **Day 1-2**: 01-basic-application (quick overview)
2. **Day 3-5**: 02-cross-chain-tokens (core patterns)
3. **Day 6-10**: Focus on examples relevant to your project
4. **Day 11-14**: Combine patterns from multiple examples

## 🔍 **When to Reference Each Example**

### **Building a Token System?**
- **Primary**: `02-cross-chain-tokens` - Core token patterns
- **Extensions**: `05-defi-amm` (liquidity), `06-trading-engine` (trading)
- **Integration**: `04-external-api-integration` (price feeds)

### **Building a Social Platform?**
- **Primary**: `03-social-messaging` - Broadcasting and subscriptions
- **Extensions**: `04-external-api-integration` (external data), `07-nft-system` (profile NFTs)
- **Foundation**: `01-basic-application` (if new to Linera)

### **Building a Gaming Platform?**
- **Primary**: `09-multiplayer-gaming` - Game mechanics and shared chains
- **Extensions**: `07-nft-system` (gaming assets), `08-ai-integration` (AI NPCs)
- **Economy**: `02-cross-chain-tokens` (gaming tokens)

### **Building a DeFi Platform?**
- **Foundation**: `02-cross-chain-tokens` - Token infrastructure
- **Core**: `05-defi-amm` (automated market making), `06-trading-engine` (order books)
- **Extensions**: `04-external-api-integration` (price oracles)

### **Building an AI Application?**
- **Foundation**: `01-basic-application` - Basic patterns
- **Core**: `08-ai-integration` - LLM integration and inference
- **Integration**: `04-external-api-integration` - External AI APIs

### **Building a Fundraising Platform?**
- **Foundation**: `02-cross-chain-tokens` - Token handling
- **Core**: `10-crowdfunding` - Campaign management
- **Extensions**: `03-social-messaging` (social features), `07-nft-system` (rewards)

## 🚨 **Common Patterns Across Examples**

### State Management
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct ApplicationState {
    // Common pattern across all examples
}
```

### Cross-Chain Messages
```rust
impl Contract for MyContract {
    async fn execute_message(&mut self, message: MyMessage) {
        // Pattern from examples 02, 03, 05, 06, 07, 10
    }
}
```

### GraphQL Service
```rust
impl Service for MyService {
    async fn handle_query(&self, request: Request) -> Response {
        // Pattern from all examples with frontends
    }
}
```

## 🤝 **Contributing**

To add a new example:
1. Follow the naming convention: `##-descriptive-name`
2. Include comprehensive README with usage instructions
3. Add entry to this index with proper categorization
4. Ensure code follows Linera best practices

## 📞 **Support**

- **Documentation**: Each example has detailed README
- **Code Comments**: All examples are well-commented
- **Patterns**: Look for similar patterns across examples
- **Community**: Share your adaptations and improvements

---

**Next Steps**: Choose an example that matches your project needs and dive in! Start with the basics if you're new to Linera, or jump to specialized examples if you have specific requirements.