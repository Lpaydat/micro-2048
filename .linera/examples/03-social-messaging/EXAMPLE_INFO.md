# 03 - Social Messaging

## 📋 Description

A decentralized social media platform demonstrating channel-based cross-chain messaging, subscription patterns, and broadcasting. Each microchain represents one user who can subscribe to others and broadcast posts to their subscribers.

## 🎯 Key Features Demonstrated

- ✅ **Channel-Based Messaging** - Broadcasting to multiple subscribers
- ✅ **Subscription Management** - Subscribe/unsubscribe patterns
- ✅ **Time-Indexed Data** - Posts organized by timestamp
- ✅ **Multi-User Interactions** - Social networking patterns
- ✅ **Real-Time Updates** - Live post distribution
- ✅ **Content Management** - Posts with text, images, comments, likes

## 🏗️ Architecture Overview

```
Social Media Platform
├── Contract (src/contract.rs)
│   ├── State: Posts, Subscriptions, Received Posts
│   ├── Operations: Post, Subscribe, Unsubscribe, Like, Comment
│   ├── Messages: Subscribe, Unsubscribe, Posts
│   └── Logic: Social interactions and broadcasting
├── Service (src/service.rs)
│   ├── Queries: Posts, subscriptions, feed
│   ├── Mutations: Social operations
│   └── Real-time post streaming
└── Frontend (web-frontend/)
    ├── Social media UI
    ├── Post creation and display
    └── Subscription management
```

## 🎨 Code Patterns

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct SocialState {
    pub posts: LogView<Post>,
    pub received_posts: MapView<PostKey, Post>,
    pub subscriptions: SetView<ChainId>,
}
```

### Post Structure
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Post {
    pub key: PostKey,
    pub text: String,
    pub image_url: Option<String>,
    pub comments: Vec<Comment>,
    pub likes: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostKey {
    pub timestamp: u64,
    pub author: ChainId,
    pub index: u64,
}
```

### Broadcasting Posts
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Post { text, image_url } => {
            let post = Post {
                key: PostKey {
                    timestamp: self.runtime.system_time(),
                    author: self.runtime.chain_id(),
                    index: self.state.posts.count(),
                },
                text,
                image_url,
                comments: Vec::new(),
                likes: 0,
            };
            
            self.state.posts.push(post.clone());
            
            // Broadcast to channel (all subscribers receive)
            let message = Message::Posts(vec![post]);
            self.runtime.send_message_to_channel(SOCIAL_CHANNEL, message);
            
            Ok(())
        }
    }
}
```

### Subscription Management
```rust
Operation::Subscribe { chain_id } => {
    if self.state.subscriptions.insert(&chain_id)? {
        let message = Message::Subscribe;
        self.runtime.send_message(chain_id, message);
    }
    Ok(())
}
```

## 🚀 Use Cases

### **Perfect For:**
- 📱 **Social Media Platforms** - Decentralized Twitter/Facebook alternatives
- 📰 **Content Publishing** - Blogs, news, media distribution
- 💬 **Community Forums** - Discussion platforms and communities
- 📢 **Broadcasting Systems** - Announcements and notifications
- 🎮 **Gaming Social Features** - Player interactions and updates
- 🏢 **Corporate Communications** - Internal company social networks

### **Real-World Applications:**
- Decentralized social networks
- Content creator platforms
- Community discussion forums
- News and media distribution
- Gaming social features
- Corporate internal communications

## 📚 When to Reference This Example

### **Building Social Features**
- Copy subscription/follow patterns
- Adapt post creation and display
- Use channel broadcasting for notifications
- Reference real-time update mechanisms

### **Implementing Broadcasting Systems**
- Learn channel-based message distribution
- Understand subscription management
- See efficient content delivery patterns
- Study time-based content organization

### **Creating Community Platforms**
- Use social interaction patterns
- Adapt commenting and liking systems
- Reference user-generated content handling
- Study community moderation approaches

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns
- **02-cross-chain-tokens** - Cross-chain messaging concepts

### **Combines With:**
- **07-nft-system** - NFT-based profile pictures or content
- **08-ai-integration** - AI content moderation or recommendations
- **10-crowdfunding** - Social fundraising campaigns

### **Extends To:**
- Gaming platforms with social features
- Content monetization systems
- Decentralized governance platforms

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - Social media ABI
- `src/contract.rs` - Social interaction logic
- `src/service.rs` - Social GraphQL API
- `src/state.rs` - Social state management
- `web-frontend/` - Social media UI

### **Testing:**
- Multi-user interaction scenarios
- Subscription/unsubscription flows
- Post broadcasting verification
- Real-time update testing

### **Deployment:**
- Multiple chains for realistic testing
- Channel setup for broadcasting
- User onboarding considerations

## 💡 Customization Ideas

### **Content Features:**
- Add post categories or tags
- Implement post editing and deletion
- Add media upload and storage
- Create post scheduling

### **Social Features:**
- Add direct messaging
- Implement user profiles
- Create group/community features
- Add content moderation tools

### **Advanced Features:**
- Algorithmic feed curation
- Content monetization
- Reputation systems
- Privacy controls

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Channel Broadcasting** - Efficient one-to-many messaging
2. **Subscription Patterns** - Managing follower relationships
3. **Time-Based Indexing** - Organizing content chronologically
4. **Social Interactions** - Likes, comments, and engagement
5. **Real-Time Systems** - Live content distribution
6. **Content Management** - User-generated content handling
7. **Multi-User Coordination** - Social platform architecture

## 🔄 Message Flow Example

```
User A creates post
    ↓ Post stored on User A's chain
    ↓ Broadcast message sent to channel
    ↓ All subscribers (B, C, D) receive post
User B, C, D chains
    ↓ Receive and store post in received_posts
    ↓ Post appears in their feeds
    ↓ Can like, comment, or share
```

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 4-6 hours
**Prerequisites:** 01-basic-application, 02-cross-chain-tokens
**Next Example:** 04-external-api-integration or specialized examples