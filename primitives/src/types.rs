//! Common type definitions

use codec::{Encode, Decode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// dWallet Account ID
pub type AccountId = sp_core::sr25519::Public;

/// Block number type
pub type BlockNumber = u32;

/// Balance type (18 decimals)
pub type Balance = u128;

/// Asset ID
pub type AssetId = u64;

/// Timestamp (Unix epoch)
pub type Moment = u64;

/// Signature type
pub type Signature = sp_core::sr25519::Signature;

/// Layer ID (0-10 for 10-layer security)
pub type LayerId = u8;

/// Proposal ID for governance
pub type ProposalId = u64;

/// Transaction nonce
pub type Nonce = u64;

/// Chain ID for cross-chain operations
pub type ChainId = u64;

/// Fee tier levels
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum FeeTier {
    None,      // No discount
    Bronze,    // 10% discount
    Silver,    // 25% discount
    Gold,      // 50% discount
    Platinum,  // 80% discount
}

impl FeeTier {
    /// Get discount multiplier in basis points
    pub fn discount_bps(&self) -> u32 {
        match self {
            FeeTier::None => 10000,    // 0% discount (pay 100%)
            FeeTier::Bronze => 9000,   // 10% discount (pay 90%)
            FeeTier::Silver => 7500,   // 25% discount (pay 75%)
            FeeTier::Gold => 5000,     // 50% discount (pay 50%)
            FeeTier::Platinum => 2000, // 80% discount (pay 20%)
        }
    }
}

/// dWallet-specific error types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DWalletError {
    InsufficientBalance,
    InsufficientAllowance,
    Overflow,
    Underflow,
    Unauthorized,
    CircuitBreakerActive,
    RateLimitExceeded,
    InvalidSignature,
}

/// Security threat level (0-10)
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ThreatLevel(pub u8);

impl ThreatLevel {
    pub fn is_critical(&self) -> bool {
        self.0 >= 8
    }
    
    pub fn is_high(&self) -> bool {
        self.0 >= 6
    }
    
    pub fn is_elevated(&self) -> bool {
        self.0 >= 4
    }
}

/// Membership tier for NFT memberships
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MembershipTier {
    Bronze,   // ~$125
    Silver,   // ~$375
    Gold,     // ~$1,250
    Platinum, // ~$3,750
}

/// Vote choice for governance
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}
