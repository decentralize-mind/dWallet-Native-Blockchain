//! Protocol constants

/// Maximum supply of DWT tokens (1,123,000,000 with 18 decimals)
pub const MAX_DWT_SUPPLY: u128 = 1_123_000_000 * 10u128.pow(18);

/// Block time target (6 seconds)
pub const TARGET_BLOCK_TIME: u64 = 6;

/// Epoch length in blocks (4 hours)
pub const EPOCH_LENGTH: u32 = 2400;

/// Number of validators
pub const VALIDATOR_COUNT: u32 = 21;

/// Bridge relayer threshold (7-of-15)
pub const BRIDGE_REQUIRED_SIGNERS: u8 = 7;
pub const BRIDGE_TOTAL_VALIDATORS: u8 = 15;

/// Governance timelock (48 hours in blocks)
pub const GOVERNANCE_TIMELOCK_BLOCKS: u32 = 28800;

/// Quorum requirement (4% of total supply)
pub const GOVERNANCE_QUORUM_PERCENT: u32 = 4;

/// Fee tiers (basis points)
pub const FEE_TIER_PLATINUM: u32 = 2000;  // 80% discount
pub const FEE_TIER_GOLD: u32 = 5000;      // 50% discount
pub const FEE_TIER_SILVER: u32 = 7500;    // 25% discount
pub const FEE_TIER_BRONZE: u32 = 9000;    // 10% discount

/// Revenue distribution
pub const REVENUE_VALIDATORS_PERCENT: u32 = 50;
pub const REVENUE_BURN_PERCENT: u32 = 20;
pub const REVENUE_TREASURY_PERCENT: u32 = 15;
pub const REVENUE_INSURANCE_PERCENT: u32 = 15;
