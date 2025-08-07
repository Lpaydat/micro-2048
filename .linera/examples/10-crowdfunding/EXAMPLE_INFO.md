# 10 - Crowdfunding

## 📋 Description

A comprehensive crowdfunding platform demonstrating campaign management, pledge collection, goal-based logic, and time-based operations. This example shows how to build fundraising systems with contributor management and automatic refund mechanisms.

## 🎯 Key Features Demonstrated

- ✅ **Campaign Management** - Creating and managing fundraising campaigns
- ✅ **Pledge Collection** - Collecting contributions from multiple users
- ✅ **Goal-Based Logic** - Success/failure based on funding targets
- ✅ **Time-Based Operations** - Campaign deadlines and expiration
- ✅ **Automatic Refunds** - Returning funds for failed campaigns
- ✅ **Multi-Contributor Systems** - Managing multiple campaign backers
- ✅ **Application Composition** - Building on fungible token system

## 🏗️ Architecture Overview

```
Crowdfunding Platform
├── Campaign Chain (src/contract.rs)
│   ├── State: Campaign info, pledges, contributors
│   ├── Operations: Pledge, Cancel, Collect
│   ├── Logic: Goal tracking and deadline management
│   └── Integration: Fungible token transfers
├── Service (src/service.rs)
│   ├── Queries: Campaign status, pledge amounts
│   ├── Mutations: Pledge and campaign operations
│   └── Real-time campaign updates
└── Dependencies
    ├── Fungible Token: Campaign currency
    ├── Cross-application calls
    └── Time-based automation
```

## 🎨 Code Patterns

### Campaign Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Campaign {
    pub owner: AccountOwner,
    pub target: Amount,
    pub deadline: Timestamp,
    pub total_pledged: Amount,
    pub status: CampaignStatus,
    pub created_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CampaignStatus {
    Active,
    Successful,
    Failed,
    Cancelled,
}
```

### State Definition
```rust
#[derive(RootView)]
#[view(context = "ViewStorageContext")]
pub struct CrowdFundingState {
    pub campaign: RegisterView<Campaign>,
    pub pledges: MapView<AccountOwner, Amount>,
    pub total_pledged: RegisterView<Amount>,
}
```

### Pledge Operation
```rust
async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
    match operation {
        Operation::Pledge { owner, amount } => {
            let campaign = self.state.campaign.get();
            
            // Validate campaign is active
            if campaign.status != CampaignStatus::Active {
                return Err(CrowdFundingError::CampaignNotActive);
            }
            
            // Check deadline
            if self.runtime.system_time() > campaign.deadline {
                return Err(CrowdFundingError::CampaignExpired);
            }
            
            // Transfer tokens from pledger to campaign
            self.transfer_tokens_to_campaign(owner, amount).await?;
            
            // Update pledge amount
            let current_pledge = self.state.pledges.get(&owner).unwrap_or_default();
            let new_pledge = current_pledge + amount;
            self.state.pledges.insert(&owner, new_pledge)?;
            
            // Update total pledged
            let new_total = *self.state.total_pledged.get() + amount;
            self.state.total_pledged.set(new_total);
            
            // Update campaign total
            let mut updated_campaign = campaign.clone();
            updated_campaign.total_pledged = new_total;
            
            // Check if goal reached
            if new_total >= campaign.target {
                updated_campaign.status = CampaignStatus::Successful;
            }
            
            self.state.campaign.set(updated_campaign);
            
            Ok(new_pledge)
        }
    }
}
```

### Campaign Collection
```rust
Operation::Collect => {
    let campaign = self.state.campaign.get();
    
    // Only campaign owner can collect
    if self.runtime.authenticated_signer() != Some(campaign.owner) {
        return Err(CrowdFundingError::NotCampaignOwner);
    }
    
    // Campaign must be successful
    if campaign.status != CampaignStatus::Successful {
        return Err(CrowdFundingError::CampaignNotSuccessful);
    }
    
    let total_amount = *self.state.total_pledged.get();
    
    // Transfer all pledged tokens to campaign owner
    self.transfer_tokens_to_owner(campaign.owner, total_amount).await?;
    
    // Mark campaign as collected
    let mut updated_campaign = campaign.clone();
    updated_campaign.status = CampaignStatus::Collected;
    self.state.campaign.set(updated_campaign);
    
    Ok(total_amount)
}
```

### Automatic Refund System
```rust
Operation::Cancel => {
    let campaign = self.state.campaign.get();
    
    // Check if campaign can be cancelled
    let can_cancel = match campaign.status {
        CampaignStatus::Active => {
            // Can cancel if deadline passed and goal not reached
            self.runtime.system_time() > campaign.deadline 
                && campaign.total_pledged < campaign.target
        }
        _ => false,
    };
    
    if !can_cancel {
        return Err(CrowdFundingError::CannotCancel);
    }
    
    // Refund all pledgers
    for (pledger, amount) in self.state.pledges.indices_and_values() {
        if amount > Amount::ZERO {
            self.refund_pledger(pledger, amount).await?;
        }
    }
    
    // Update campaign status
    let mut updated_campaign = campaign.clone();
    updated_campaign.status = CampaignStatus::Failed;
    self.state.campaign.set(updated_campaign);
    
    Ok(())
}
```

### Cross-Application Token Transfers
```rust
async fn transfer_tokens_to_campaign(
    &mut self,
    from: AccountOwner,
    amount: Amount,
) -> Result<(), CrowdFundingError> {
    let token_app_id = self.runtime.application_parameters().token_id;
    
    // Call fungible token application to transfer tokens
    let transfer_operation = FungibleOperation::Transfer {
        owner: from,
        amount,
        target_account: Account {
            chain_id: self.runtime.chain_id(),
            owner: self.runtime.application_id().into(), // Campaign as recipient
        },
    };
    
    self.runtime.call_application(
        token_app_id,
        &transfer_operation,
    ).await?;
    
    Ok(())
}

async fn refund_pledger(
    &mut self,
    pledger: AccountOwner,
    amount: Amount,
) -> Result<(), CrowdFundingError> {
    let token_app_id = self.runtime.application_parameters().token_id;
    
    let refund_operation = FungibleOperation::Transfer {
        owner: self.runtime.application_id().into(),
        amount,
        target_account: Account {
            chain_id: self.runtime.chain_id(),
            owner: pledger,
        },
    };
    
    self.runtime.call_application(
        token_app_id,
        &refund_operation,
    ).await?;
    
    Ok(())
}
```

### Time-Based Campaign Logic
```rust
fn check_campaign_status(&mut self) -> Result<(), CrowdFundingError> {
    let mut campaign = self.state.campaign.get().clone();
    let current_time = self.runtime.system_time();
    
    if campaign.status == CampaignStatus::Active {
        if current_time > campaign.deadline {
            if campaign.total_pledged >= campaign.target {
                campaign.status = CampaignStatus::Successful;
            } else {
                campaign.status = CampaignStatus::Failed;
                // Trigger automatic refunds
                self.initiate_refunds().await?;
            }
            self.state.campaign.set(campaign);
        }
    }
    
    Ok(())
}
```

## 🚀 Use Cases

### **Perfect For:**
- 💰 **Fundraising Platforms** - Kickstarter-style project funding
- 🏥 **Charity Campaigns** - Non-profit and charitable fundraising
- 🚀 **Startup Funding** - Early-stage company fundraising
- 🎨 **Creative Projects** - Art, music, and creative endeavors
- 🏗️ **Community Projects** - Local community initiatives
- 🎮 **Game Development** - Indie game funding campaigns
- 📚 **Educational Initiatives** - School and educational funding

### **Real-World Applications:**
- Decentralized Kickstarter alternatives
- Charity and non-profit platforms
- Community-driven project funding
- Creative arts funding
- Open source project funding
- Disaster relief campaigns

## 📚 When to Reference This Example

### **Building Fundraising Systems**
- Copy campaign management patterns
- Adapt pledge collection mechanisms
- Use goal-based success logic
- Reference automatic refund systems

### **Creating Time-Based Applications**
- Learn deadline management
- Understand time-based state transitions
- See automatic expiration handling
- Study scheduled operations

### **Multi-Contributor Systems**
- Implement contributor tracking
- Manage collective funding
- Handle success/failure scenarios
- Create transparent funding processes

## 🔗 Related Examples

### **Builds Upon:**
- **02-cross-chain-tokens** - Requires fungible tokens
- **01-basic-application** - Basic Linera patterns

### **Combines With:**
- **03-social-messaging** - Social campaign features
- **07-nft-system** - NFT rewards for backers
- **04-external-api-integration** - External payment processing
- **08-ai-integration** - AI campaign optimization

### **Extends To:**
- DAO governance and voting
- Investment and equity platforms
- Insurance and mutual aid systems
- Subscription and membership platforms

## 🛠️ Development Notes

### **Key Files:**
- `src/lib.rs` - Crowdfunding ABI
- `src/contract.rs` - Campaign logic
- `src/service.rs` - Crowdfunding GraphQL API
- `src/state.rs` - Campaign state management

### **Dependencies:**
- Fungible token application
- Cross-application call capabilities
- Time-based operation handling

### **Testing:**
- Campaign lifecycle scenarios
- Pledge and refund testing
- Deadline and expiration handling
- Success/failure condition verification

## 💡 Customization Ideas

### **Campaign Features:**
- Add campaign categories and tags
- Implement milestone-based funding
- Create stretch goals and bonuses
- Add campaign updates and communication

### **Contributor Features:**
- Implement backer rewards and tiers
- Add contributor profiles and history
- Create social sharing and promotion
- Add anonymous contribution options

### **Advanced Features:**
- Multi-currency campaign support
- Escrow and milestone releases
- Campaign governance and voting
- Integration with external payment systems

## 🎯 Learning Objectives

After studying this example, you should understand:

1. **Campaign Management** - Creating and managing fundraising campaigns
2. **Time-Based Logic** - Handling deadlines and expiration
3. **Goal-Based Systems** - Success/failure based on targets
4. **Multi-Contributor Coordination** - Managing multiple participants
5. **Automatic Refunds** - Handling failed campaign scenarios
6. **Cross-Application Integration** - Building on other applications
7. **Financial System Design** - Creating transparent funding mechanisms

## 💰 Campaign Lifecycle

```
Campaign Creation
    ↓ Set target amount and deadline
    ↓ Campaign becomes active
Funding Phase
    ↓ Contributors make pledges
    ↓ Tokens transferred to campaign
    ↓ Track progress toward goal
Deadline Reached
    ↓ Check if goal was met
    ↓ If successful: Owner can collect funds
    ↓ If failed: Automatic refunds to contributors
```

## 📊 Campaign States

### **Active Campaign**
- Accepting pledges
- Before deadline
- Goal not yet reached

### **Successful Campaign**
- Goal reached (before or at deadline)
- Owner can collect funds
- No refunds needed

### **Failed Campaign**
- Deadline passed
- Goal not reached
- Automatic refunds triggered

### **Cancelled Campaign**
- Manually cancelled by owner
- All pledges refunded
- Campaign permanently closed

## 📈 Complexity: ⭐⭐ Intermediate

**Time to Understand:** 4-6 hours
**Prerequisites:** 02-cross-chain-tokens, time-based systems, financial concepts
**Next Example:** Combine with other examples for enhanced functionality