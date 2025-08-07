# 08 - AI Integration

## 📋 Description

Demonstrates integration of Large Language Models (LLMs) with Linera applications, showing how to run AI inference on-chain, manage AI models, and create AI-powered blockchain applications with resource management and caching strategies.

## 🎯 Key Features Demonstrated

- ✅ **LLM Integration** - Running language models in Linera services
- ✅ **AI Inference** - On-chain AI computation and responses
- ✅ **Model Management** - Loading, caching, and optimizing AI models
- ✅ **Resource Management** - Handling memory-intensive AI operations
- ✅ **Long-Lived Services** - Persistent services for performance
- ✅ **External Model Loading** - Fetching models from external sources
- ✅ **GGUF Format** - Efficient model format for inference

## 🏗️ Architecture Overview

```
AI-Powered Application
├── Contract (src/contract.rs)
│   ├── State: Conversation history, user sessions
│   ├── Operations: StartChat, SendMessage
│   └── Logic: Chat management and user tracking
├── Service (src/service.rs)
│   ├── AI Model: TinyLlama 40M parameter model
│   ├── Inference Engine: Candle-based inference
│   ├── Model Caching: Persistent model loading
│   └── GraphQL API: Chat interface
└── External Resources
    ├── Model Files: model.bin, tokenizer.json
    ├── HTTP Server: Local model serving
    └── GGUF Conversion: Model format optimization
```

## 🎨 Code Patterns

### Model Loading and Caching
```rust
pub struct LlmService {
    model: Option<Model>,
    tokenizer: Option<Tokenizer>,
    runtime: Arc<ServiceRuntime<Self>>,
}

impl Service for LlmService {
    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        LlmService {
            model: None,
            tokenizer: None,
            runtime: Arc::new(runtime),
        }
    }
}
```

### AI Inference
```rust
#[Object]
impl QueryRoot {
    async fn prompt(&self, prompt: String) -> Result<String, Error> {
        // Load model if not already loaded
        if self.model.is_none() {
            self.load_model().await?;
        }
        
        // Tokenize input
        let tokens = self.tokenizer.encode(&prompt)?;
        
        // Run inference
        let output_tokens = self.model.generate(tokens, GenerationConfig {
            max_length: 100,
            temperature: 0.7,
            top_p: 0.9,
        })?;
        
        // Decode response
        let response = self.tokenizer.decode(output_tokens)?;
        Ok(response)
    }
}
```

### Model Loading from External Source
```rust
async fn load_model(&mut self) -> Result<(), Error> {
    // Fetch model from external HTTP server
    let model_url = "http://localhost:8000/model.bin";
    let tokenizer_url = "http://localhost:8000/tokenizer.json";
    
    let model_bytes = fetch_url(model_url).await?.bytes().await?;
    let tokenizer_bytes = fetch_url(tokenizer_url).await?.bytes().await?;
    
    // Convert to GGUF format for efficient inference
    let model = Model::from_gguf_bytes(&model_bytes)?;
    let tokenizer = Tokenizer::from_bytes(&tokenizer_bytes)?;
    
    self.model = Some(model);
    self.tokenizer = Some(tokenizer);
    
    Ok(())
}
```

### Conversation Management
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct ChatState {
    pub conversations: MapView<UserId, Conversation>,
    pub active_sessions: SetView<SessionId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<Message>,
    pub created_at: Timestamp,
    pub last_activity: Timestamp,
}

async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::SendMessage { user_id, message } => {
            let mut conversation = self.state.conversations
                .get(&user_id)
                .unwrap_or_default();
            
            conversation.messages.push(Message {
                role: Role::User,
                content: message.clone(),
                timestamp: self.runtime.system_time(),
            });
            
            // Generate AI response (via service query)
            let ai_response = self.runtime.query_service(&format!(
                "query {{ prompt(prompt: \"{}\") }}", message
            )).await?;
            
            conversation.messages.push(Message {
                role: Role::Assistant,
                content: ai_response,
                timestamp: self.runtime.system_time(),
            });
            
            self.state.conversations.insert(&user_id, conversation)?;
            Ok(ai_response)
        }
    }
}
```

## 🚀 Use Cases

### **Perfect For:**
- 🤖 **AI Chatbots** - Conversational AI applications
- 📝 **Content Generation** - AI-powered content creation
- 🎮 **Gaming AI** - NPCs and game AI systems
- 📊 **Data Analysis** - AI-powered analytics and insights
- 🔍 **Smart Search** - Intelligent search and recommendation
- 💼 **Business Automation** - AI-driven business processes
- 🎨 **Creative Applications** - AI art, music, and content generation

### **Real-World Applications:**
- Decentralized AI assistants
- Gaming NPCs with AI personalities
- Content moderation systems
- Automated customer support
- AI-powered trading bots
- Creative content generation platforms

## 📚 When to Reference This Example

### **Building AI-Powered Applications**
- Copy model loading and caching patterns
- Adapt inference logic for your AI models
- Use conversation management for chat applications
- Reference resource management strategies

### **Integrating Machine Learning**
- Learn on-chain AI inference patterns
- Understand model optimization techniques
- See how to handle large model files
- Study performance optimization strategies

### **Creating Intelligent Systems**
- Build AI-driven decision making
- Implement natural language processing
- Create recommendation systems
- Develop automated analysis tools

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns
- **04-external-api-integration** - External model loading

### **Combines With:**
- **03-social-messaging** - AI content moderation
- **09-multiplayer-gaming** - AI-powered NPCs
- **02-cross-chain-tokens** - AI trading bots
- **07-nft-system** - AI-generated NFT content

### **Extends To:**
- Multi-modal AI (text, image, audio)
- Federated learning systems
- AI governance and voting
- Decentralized AI marketplaces

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - AI application ABI
- `src/contract.rs` - Chat and session management
- `src/service.rs` - AI inference service
- `web-frontend/` - Chat interface

### **Dependencies:**
- Candle ML framework
- Tokenizers library
- GGUF model format
- HTTP model serving

### **Performance Considerations:**
- Model loading is expensive (cache models)
- Inference adds latency to operations
- Memory usage scales with model size
- Use `--long-lived-services` flag

## 💡 Customization Ideas

### **AI Features:**
- Add different model types (GPT, BERT, etc.)
- Implement fine-tuning capabilities
- Add multi-modal AI (text + images)
- Create AI model marketplaces

### **Application Features:**
- Add conversation persistence
- Implement user preferences
- Create AI personality customization
- Add conversation analytics

### **Advanced Features:**
- Federated learning across chains
- AI model governance and voting
- Decentralized AI training
- AI-powered smart contracts

## ⚠️ Important Considerations

### **Performance Impact**
- Model loading can take significant time
- Inference adds latency to operations
- Memory usage scales with model complexity
- Use long-lived services for better performance

### **Resource Management**
- Large models require substantial memory
- Consider model quantization for efficiency
- Implement model caching strategies
- Monitor resource usage

### **Model Management**
- Ensure model compatibility with inference engine
- Handle model versioning and updates
- Consider model licensing and distribution
- Implement fallback mechanisms

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **AI Integration Patterns** - How to integrate AI with blockchain
2. **Model Management** - Loading, caching, and optimizing AI models
3. **Resource Optimization** - Managing memory and compute resources
4. **Inference Strategies** - On-chain vs off-chain AI computation
5. **Service Architecture** - Long-lived services for AI applications
6. **External Dependencies** - Managing AI model dependencies
7. **Performance Trade-offs** - Balancing AI capabilities with performance

## 🔄 AI Inference Flow

```
User Input
    ↓ Contract receives message
    ↓ Query service for AI response
Service
    ↓ Load model (if not cached)
    ↓ Tokenize input
    ↓ Run inference
    ↓ Generate response
    ↓ Return to contract
Contract
    ↓ Store conversation
    ↓ Return AI response to user
```

## 📈 Complexity: ⭐⭐⭐ Advanced

**Time to Understand:** 8-12 hours
**Prerequisites:** 01-basic-application, 04-external-api-integration, AI/ML concepts
**Next Example:** Combine with other examples for AI-powered applications