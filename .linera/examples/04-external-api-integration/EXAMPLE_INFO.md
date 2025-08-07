# 04 - External API Integration

## 📋 Description

Demonstrates how to perform HTTP requests and integrate external APIs with Linera applications. Shows three different approaches: service-side requests, contract-side requests, and oracle patterns for reliable external data integration.

## 🎯 Key Features Demonstrated

- ✅ **HTTP Requests from Service** - Client-side external API calls
- ✅ **HTTP Requests from Contract** - Validator-consensus external calls
- ✅ **Oracle Pattern** - Service as oracle for deterministic responses
- ✅ **External Data Integration** - Bringing off-chain data on-chain
- ✅ **Deterministic Operations** - Ensuring consensus with external data
- ✅ **Error Handling** - Managing external API failures

## 🏗️ Architecture Overview

```
External API Integration
├── Contract (src/contract.rs)
│   ├── Direct HTTP requests (consensus required)
│   ├── Oracle queries to service
│   └── Deterministic response handling
├── Service (src/service.rs)
│   ├── HTTP requests for queries
│   ├── HTTP requests for mutations
│   ├── Oracle responses to contract
│   └── Non-deterministic operations
└── External APIs
    ├── REST APIs
    ├── GraphQL endpoints
    └── Data feeds
```

## 🎨 Code Patterns

### Service HTTP Request (Query)
```rust
// Non-consensus, client-side only
async fn handle_query(&self, request: Request) -> Response {
    let schema = Schema::build(
        QueryRoot { base_url: self.base_url.clone() },
        MutationRoot { runtime: self.runtime.clone() },
        EmptySubscription,
    ).finish();
    schema.execute(request).await
}

#[Object]
impl QueryRoot {
    async fn perform_http_request(&self) -> String {
        let response = fetch_url(&self.base_url).await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))
    }
}
```

### Service HTTP Request (Mutation)
```rust
// Triggers contract operation with HTTP response
#[Object]
impl MutationRoot {
    async fn perform_http_request(&self) -> Vec<u8> {
        let response = fetch_url(&self.base_url).await?;
        let data = response.text().await?;
        
        // Schedule operation with HTTP response data
        self.runtime.schedule_operation(&data);
        vec![]
    }
}
```

### Contract HTTP Request
```rust
// Consensus required - all validators must get same response
async fn execute_operation(&mut self, operation: String) -> String {
    match operation.as_str() {
        "http_request" => {
            let response = fetch_url(&self.base_url).await?;
            let data = response.text().await?;
            
            // All validators must receive identical response
            // or block will be rejected
            data
        }
    }
}
```

### Oracle Pattern
```rust
// Contract queries service for deterministic response
async fn execute_operation(&mut self, operation: String) -> String {
    match operation.as_str() {
        "oracle_request" => {
            let query = "query { httpData }";
            let response = self.runtime.query_service(query).await?;
            
            // Service performs HTTP request and returns
            // only deterministic parts of response
            response.data
        }
    }
}
```

## 🚀 Use Cases

### **Perfect For:**
- 📊 **Price Feeds** - Cryptocurrency and asset price data
- 🌤️ **Weather Data** - Weather information for applications
- 📰 **News Feeds** - External news and information
- 🔍 **Data Verification** - Verifying external claims or data
- 🏦 **Financial Data** - Stock prices, exchange rates
- 🎮 **Game Data** - External game statistics or leaderboards
- 🌐 **Web3 Integration** - Connecting to other blockchain APIs

### **Real-World Applications:**
- DeFi price oracles
- Weather-based insurance
- Sports betting platforms
- Supply chain verification
- Identity verification systems
- Cross-chain data bridges

## 📚 When to Reference This Example

### **Integrating External Data**
- Copy HTTP request patterns for your APIs
- Adapt error handling for network failures
- Use oracle patterns for consensus-critical data
- Reference deterministic response handling

### **Building Oracle Systems**
- Learn service-as-oracle architecture
- Understand consensus requirements for external data
- See how to handle non-deterministic responses
- Study data validation and verification

### **Hybrid Applications**
- Combine on-chain logic with off-chain data
- Create applications that bridge Web2 and Web3
- Build systems that react to external events
- Implement data-driven smart contracts

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns

### **Combines With:**
- **02-cross-chain-tokens** - Price feeds for tokens
- **05-defi-amm** - External price data for AMM
- **08-ai-integration** - External AI API integration
- **10-crowdfunding** - External payment verification

### **Extends To:**
- Complex oracle networks
- Multi-source data aggregation
- Real-time data streaming

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - HTTP integration ABI
- `src/contract.rs` - Contract-side HTTP logic
- `src/service.rs` - Service-side HTTP logic
- Test HTTP server for development

### **Testing:**
- Mock HTTP server for testing
- Network failure simulation
- Consensus testing with multiple validators
- Response validation testing

### **Deployment:**
- External API dependencies
- Network connectivity requirements
- Rate limiting considerations

## 💡 Customization Ideas

### **Data Sources:**
- Add multiple API endpoints
- Implement data source failover
- Create data aggregation from multiple sources
- Add data caching mechanisms

### **Oracle Features:**
- Multi-signature oracle validation
- Reputation-based oracle selection
- Time-weighted average pricing
- Data freshness validation

### **Integration Patterns:**
- Webhook receivers for push data
- Scheduled data updates
- Event-driven data fetching
- Real-time data streaming

## ⚠️ Important Considerations

### **Consensus Requirements**
- Contract HTTP requests must return identical responses across all validators
- Use oracle pattern if responses might vary (timestamps, random data)
- Consider network latency and timeout differences

### **Performance Impact**
- HTTP requests add latency to operations
- External API failures can block operations
- Rate limiting from external APIs
- Network connectivity requirements

### **Security Considerations**
- Validate external data before using
- Handle malicious or incorrect responses
- Implement circuit breakers for failing APIs
- Consider data source reputation

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **HTTP Integration Patterns** - Different ways to call external APIs
2. **Consensus Considerations** - When external calls affect consensus
3. **Oracle Architecture** - Using services as data oracles
4. **Error Handling** - Managing external API failures
5. **Deterministic Operations** - Ensuring consistent validator responses
6. **Performance Trade-offs** - Balancing external data with performance
7. **Security Best Practices** - Safely integrating external data

## 🔄 Request Flow Comparison

### Service Request (Non-Consensus)
```
User → Service → External API → Response → User
(Only client makes request)
```

### Contract Request (Consensus Required)
```
User → Contract → All Validators → External API → Consensus → Response
(All validators must agree on response)
```

### Oracle Pattern
```
User → Contract → Service (Oracle) → External API → Deterministic Response → Contract
(Service handles non-deterministic parts)
```

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 3-5 hours
**Prerequisites:** 01-basic-application, understanding of HTTP/APIs
**Next Example:** Any specialized example that needs external data