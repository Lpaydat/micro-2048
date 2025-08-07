# 07 - NFT System

## 📋 Description

A comprehensive Non-Fungible Token (NFT) system demonstrating unique digital asset creation, management, and cross-chain transfers. This example shows how to build NFT platforms with metadata handling, ownership tracking, and blob storage integration.

## 🎯 Key Features Demonstrated

- ✅ **Unique Token Management** - Each NFT has a unique identifier
- ✅ **NFT Minting** - Creating new unique digital assets
- ✅ **Ownership Tracking** - Managing NFT ownership across chains
- ✅ **Metadata Handling** - Storing NFT properties and attributes
- ✅ **Blob Storage** - Handling large files and media
- ✅ **Cross-Chain NFTs** - Transferring NFTs between chains
- ✅ **Transfer Operations** - Safe NFT ownership transfers

## 🏗️ Architecture Overview

```
NFT System
├── Contract (src/contract.rs)
│   ├── State: NFT registry, ownership mapping
│   ├── Operations: Mint, Transfer, Claim
│   ├── Logic: Ownership validation and transfers
│   └── Integration: Blob storage for metadata
├── Service (src/service.rs)
│   ├── Queries: NFT details, ownership, collections
│   ├── Mutations: Minting and transfer operations
│   └── Metadata resolution
└── Storage
    ├── Blob Storage: Images, videos, metadata
    ├── On-Chain Registry: Ownership and basic info
    └── Cross-Chain State: Multi-chain NFT tracking
```

## 🎨 Code Patterns

### NFT Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nft {
    pub token_id: TokenId,
    pub owner: AccountOwner,
    pub name: String,
    pub minter: AccountOwner,
    pub blob_hash: Option<BlobHash>, // For images/metadata
    pub payload: Vec<u8>,           // Additional data
}

pub type TokenId = u64;
```

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct NonFungibleTokenState {
    pub nfts: MapView<TokenId, Nft>,
    pub owned_nfts: MapView<AccountOwner, SetView<TokenId>>,
    pub next_token_id: RegisterView<TokenId>,
}
```

### Minting Operation
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Mint { minter, name, blob_hash, payload } => {
            let token_id = *self.state.next_token_id.get();
            
            let nft = Nft {
                token_id,
                owner: minter,
                name,
                minter,
                blob_hash,
                payload,
            };
            
            // Store NFT in registry
            self.state.nfts.insert(&token_id, nft)?;
            
            // Update ownership mapping
            let mut owned_nfts = self.state.owned_nfts
                .get(&minter)
                .unwrap_or_default();
            owned_nfts.insert(&token_id)?;
            self.state.owned_nfts.insert(&minter, owned_nfts)?;
            
            // Increment token ID counter
            self.state.next_token_id.set(token_id + 1);
            
            Ok(token_id)
        }
    }
}
```

### Transfer Operation
```rust
Operation::Transfer { source_owner, token_id, target_account } => {
    // Verify ownership
    let mut nft = self.state.nfts.get(&token_id)
        .ok_or(NftError::TokenNotFound)?;
    
    if nft.owner != source_owner {
        return Err(NftError::NotOwner);
    }
    
    // Remove from source owner
    let mut source_owned = self.state.owned_nfts
        .get(&source_owner)
        .unwrap_or_default();
    source_owned.remove(&token_id)?;
    self.state.owned_nfts.insert(&source_owner, source_owned)?;
    
    if target_account.chain_id == self.runtime.chain_id() {
        // Local transfer
        nft.owner = target_account.owner;
        self.state.nfts.insert(&token_id, nft)?;
        
        // Add to target owner
        let mut target_owned = self.state.owned_nfts
            .get(&target_account.owner)
            .unwrap_or_default();
        target_owned.insert(&token_id)?;
        self.state.owned_nfts.insert(&target_account.owner, target_owned)?;
    } else {
        // Cross-chain transfer
        self.state.nfts.remove(&token_id)?;
        
        let message = Message::Transfer {
            nft,
            target_owner: target_account.owner,
        };
        self.runtime.send_message(target_account.chain_id, message);
    }
    
    Ok(())
}
```

### Cross-Chain Message Handling
```rust
async fn execute_message(&mut self, message: Message) {
    match message {
        Message::Transfer { mut nft, target_owner } => {
            // Update ownership for received NFT
            nft.owner = target_owner;
            let token_id = nft.token_id;
            
            // Store NFT on new chain
            self.state.nfts.insert(&token_id, nft)?;
            
            // Update ownership mapping
            let mut owned_nfts = self.state.owned_nfts
                .get(&target_owner)
                .unwrap_or_default();
            owned_nfts.insert(&token_id)?;
            self.state.owned_nfts.insert(&target_owner, owned_nfts)?;
        }
        
        Message::Claim { source_account, token_id, target_account } => {
            // Handle claim requests from other chains
            self.transfer_nft(source_account.owner, token_id, target_account).await?;
        }
    }
}
```

### Metadata and Blob Integration
```rust
#[Object]
impl QueryRoot {
    async fn nft(&self, token_id: TokenId) -> Option<NftWithMetadata> {
        let nft = self.state.nfts.get(&token_id)?;
        
        // Resolve blob data if present
        let metadata = if let Some(blob_hash) = nft.blob_hash {
            self.resolve_blob_metadata(blob_hash).await.ok()
        } else {
            None
        };
        
        Some(NftWithMetadata {
            nft,
            metadata,
            image_url: self.generate_image_url(nft.blob_hash),
        })
    }
    
    async fn owned_nfts(&self, owner: AccountOwner) -> Vec<TokenId> {
        self.state.owned_nfts
            .get(&owner)
            .map(|set| set.indices().collect())
            .unwrap_or_default()
    }
}
```

## 🚀 Use Cases

### **Perfect For:**
- 🎨 **Digital Art Platforms** - NFT art marketplaces and galleries
- 🎮 **Gaming Assets** - In-game items, characters, and collectibles
- 🏆 **Collectibles** - Trading cards, memorabilia, and rare items
- 📜 **Certificates** - Diplomas, licenses, and credentials
- 🏠 **Real Estate** - Property deeds and ownership records
- 🎵 **Music & Media** - Albums, videos, and digital content
- 🎫 **Event Tickets** - Concert tickets and event passes

### **Real-World Applications:**
- OpenSea-style NFT marketplaces
- Gaming item trading platforms
- Digital art galleries
- Certificate and credential systems
- Real estate tokenization
- Music and media platforms

## 📚 When to Reference This Example

### **Building NFT Platforms**
- Copy NFT minting and management patterns
- Adapt ownership tracking systems
- Use metadata handling approaches
- Reference cross-chain NFT transfers

### **Creating Digital Asset Systems**
- Learn unique asset identification
- Understand ownership verification
- See blob storage integration
- Study transfer validation

### **Gaming Applications**
- Implement in-game asset systems
- Create collectible mechanisms
- Build trading functionality
- Manage player inventories

## 🔗 Related Examples

### **Builds Upon:**
- **01-basic-application** - Basic Linera patterns
- **02-cross-chain-tokens** - Cross-chain messaging concepts

### **Combines With:**
- **09-multiplayer-gaming** - Gaming NFTs and assets
- **03-social-messaging** - Social NFT features
- **05-defi-amm** - NFT trading and liquidity
- **08-ai-integration** - AI-generated NFT content

### **Extends To:**
- NFT marketplaces with trading
- Fractionalized NFT ownership
- NFT-based governance systems
- Dynamic and evolving NFTs

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - NFT ABI and types
- `src/contract.rs` - NFT core logic
- `src/service.rs` - NFT GraphQL API
- `src/state.rs` - NFT state management
- `web-frontend/` - NFT management UI

### **Dependencies:**
- Blob storage for metadata
- Cross-chain messaging capabilities
- Image and media handling

### **Testing:**
- NFT minting scenarios
- Ownership transfer testing
- Cross-chain NFT movement
- Metadata resolution verification

## 💡 Customization Ideas

### **NFT Features:**
- Add NFT collections and series
- Implement royalty mechanisms
- Create NFT burning functionality
- Add NFT staking and rewards

### **Metadata Features:**
- Rich metadata schemas
- Dynamic NFT properties
- Upgradeable NFT content
- Interactive NFT experiences

### **Advanced Features:**
- Fractionalized NFT ownership
- NFT-based governance voting
- Rental and lending systems
- Cross-chain NFT bridges

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Unique Asset Management** - How NFTs differ from fungible tokens
2. **Ownership Tracking** - Managing unique asset ownership
3. **Metadata Systems** - Storing and retrieving NFT properties
4. **Blob Storage** - Handling large files and media
5. **Cross-Chain Assets** - Moving unique assets between chains
6. **Transfer Validation** - Ensuring secure ownership transfers
7. **Digital Asset Standards** - NFT best practices and patterns

## 🖼️ NFT Structure Example

```json
{
  "token_id": 1,
  "owner": "0x123...",
  "name": "Cosmic Dragon #001",
  "minter": "0x456...",
  "blob_hash": "0xabc...",
  "metadata": {
    "description": "A rare cosmic dragon",
    "attributes": [
      {"trait_type": "Rarity", "value": "Legendary"},
      {"trait_type": "Element", "value": "Fire"},
      {"trait_type": "Power", "value": 95}
    ],
    "image": "https://example.com/dragon001.png"
  }
}
```

## 🔄 NFT Transfer Flow

```
Owner A (Chain 1) → Transfer to Owner B (Chain 2)
    ↓ Verify ownership on Chain 1
    ↓ Remove NFT from Chain 1 state
    ↓ Send transfer message to Chain 2
Chain 2 receives message
    ↓ Add NFT to Chain 2 state
    ↓ Update ownership to Owner B
    ↓ NFT now exists only on Chain 2
```

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 4-6 hours
**Prerequisites:** 01-basic-application, 02-cross-chain-tokens, NFT concepts
**Next Example:** 09-multiplayer-gaming for gaming NFTs