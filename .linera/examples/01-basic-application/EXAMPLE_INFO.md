# 01 - Basic Application

## 📋 Description

A simple counter application that demonstrates the fundamental concepts of Linera blockchain development. This example implements a basic state management system with increment operations and GraphQL integration.

## 🎯 Key Features Demonstrated

- ✅ **Basic State Management** - Using `RegisterView<u64>` for simple state
- ✅ **Operation Handling** - Processing increment operations
- ✅ **GraphQL Service** - Auto-generated GraphQL API
- ✅ **Frontend Integration** - React-based web interface
- ✅ **Unit Testing** - Comprehensive test patterns
- ✅ **Contract/Service Split** - Proper separation of concerns

## 🏗️ Architecture Overview

```
Counter Application
├── Contract (src/contract.rs)
│   ├── State: RegisterView<u64>
│   ├── Operations: Increment(u64)
│   └── Logic: Add value to counter
├── Service (src/service.rs)
│   ├── GraphQL Query: Get current value
│   ├── GraphQL Mutation: Increment counter
│   └── Real-time subscriptions
└── Frontend (web-frontend/)
    ├── React components
    ├── Apollo GraphQL client
    └── Real-time updates
```

## 🎨 Code Patterns

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct CounterState {
    pub value: RegisterView<u64>,
}
```

### Operation Handling
```rust
async fn execute_operation(&mut self, operation: u64) -> u64 {
    let new_value = self.state.value.get() + operation;
    self.state.value.set(new_value);
    new_value
}
```

### GraphQL Integration
```rust
#[Object]
impl QueryRoot {
    async fn value(&self) -> &u64 {
        &self.value
    }
}

#[Object]
impl MutationRoot {
    async fn increment(&self, value: u64) -> [u8; 0] {
        self.runtime.schedule_operation(&value);
        []
    }
}
```

## 🚀 Use Cases

### **Perfect For:**
- 🎓 **Learning Linera** - First application for new developers
- 🏗️ **Project Foundation** - Starting point for more complex apps
- 🧪 **Testing Patterns** - Understanding testing methodologies
- 📊 **Simple Counters** - Vote counting, view counters, statistics
- 🔧 **Development Setup** - Verifying development environment

### **Not Suitable For:**
- ❌ Complex business logic
- ❌ Multi-user interactions
- ❌ Cross-chain operations
- ❌ External integrations

## 📚 When to Reference This Example

### **Starting a New Project**
- Use as template for basic project structure
- Copy state management patterns
- Adapt GraphQL service setup
- Reference testing approaches

### **Learning Linera Concepts**
- Understand contract vs service separation
- Learn state management with views
- See GraphQL auto-generation in action
- Practice deployment and testing

### **Building Simple Applications**
- Voting systems (adapt counter for votes)
- Statistics tracking (page views, user actions)
- Simple games (score tracking)
- Basic dashboards (metric collection)

## 🔗 Related Examples

### **Next Steps:**
- **02-cross-chain-tokens** - Add cross-chain messaging
- **04-external-api-integration** - Connect to external APIs
- **03-social-messaging** - Add user interactions

### **Combines Well With:**
- Any example can build upon these basic patterns
- Use as foundation, then add specialized features

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - ABI definitions
- `src/contract.rs` - Core business logic
- `src/service.rs` - GraphQL service
- `src/state.rs` - State structure
- `web-frontend/` - React application

### **Testing:**
- Unit tests in `src/contract.rs`
- Integration tests possible
- Frontend testing with React Testing Library

### **Deployment:**
- Single chain deployment
- No cross-chain dependencies
- Minimal resource requirements

## 💡 Customization Ideas

### **Easy Modifications:**
- Change from u64 to different data types
- Add multiple counters with names
- Implement decrement operations
- Add counter limits or validation

### **Advanced Extensions:**
- Add user-specific counters
- Implement counter categories
- Add time-based counter resets
- Create counter leaderboards

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Linera Application Structure** - Contract, service, and state organization
2. **State Management** - Using views for persistent storage
3. **Operation Processing** - Handling user operations in contracts
4. **GraphQL Integration** - Auto-generated APIs and custom resolvers
5. **Frontend Integration** - Connecting web interfaces to blockchain
6. **Testing Patterns** - Unit testing blockchain applications
7. **Deployment Process** - Publishing and creating applications

## 📈 Complexity: ⭐ Beginner

**Time to Understand:** 2-4 hours
**Prerequisites:** Basic Rust knowledge
**Next Example:** 02-cross-chain-tokens