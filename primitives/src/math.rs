//! Mathematical utilities for dWallet protocol
//!
//! Provides safe arithmetic operations and financial calculations.

use crate::{Balance, BlockNumber};
use sp_runtime::traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv, Zero};

/// Calculate fee tier based on DWT balance
/// Returns discount multiplier (in basis points)
pub fn calculate_fee_tier(balance: Balance) -> u32 {
    // Fee tiers based on holdings
    // Platinum: >= 100,000 DWT (80% discount = 2000 basis points)
    // Gold: >= 10,000 DWT (50% discount = 5000 basis points)
    // Silver: >= 1,000 DWT (25% discount = 7500 basis points)
    // Bronze: >= 100 DWT (10% discount = 9000 basis points)
    // None: < 100 DWT (no discount = 10000 basis points)
    
    let one_dwt = 10u128.pow(18);
    
    if balance >= 100_000 * one_dwt {
        2000  // 80% discount
    } else if balance >= 10_000 * one_dwt {
        5000  // 50% discount
    } else if balance >= 1_000 * one_dwt {
        7500  // 25% discount
    } else if balance >= 100 * one_dwt {
        9000  // 10% discount
    } else {
        10000 // No discount
    }
}

/// Calculate actual fee after applying tier discount
/// base_fee is in basis points (e.g., 30 = 0.3%)
pub fn calculate_discounted_fee(base_fee: u32, tier_discount: u32) -> u32 {
    // tier_discount is the portion to pay (e.g., 2000 means pay 20%, get 80% off)
    // final_fee = base_fee * tier_discount / 10000
    (base_fee as u64 * tier_discount as u64 / 10000) as u32
}

/// Safe addition with overflow check
pub fn safe_add(a: Balance, b: Balance) -> Option<Balance> {
    a.checked_add(&b)
}

/// Safe subtraction with underflow check
pub fn safe_sub(a: Balance, b: Balance) -> Option<Balance> {
    a.checked_sub(&b)
}

/// Safe multiplication with overflow check
pub fn safe_mul(a: Balance, b: Balance) -> Option<Balance> {
    a.checked_mul(&b)
}

/// Safe division with zero check
pub fn safe_div(a: Balance, b: Balance) -> Option<Balance> {
    if b.is_zero() {
        None
    } else {
        a.checked_div(&b)
    }
}

/// Calculate percentage of amount
/// Returns (amount * percent) / 100
pub fn calculate_percentage(amount: Balance, percent: u32) -> Option<Balance> {
    safe_mul(amount, percent as Balance)
        .and_then(|result| safe_div(result, 100))
}

/// Calculate basis points of amount
/// Returns (amount * bps) / 10000
pub fn calculate_basis_points(amount: Balance, bps: u32) -> Option<Balance> {
    safe_mul(amount, bps as Balance)
        .and_then(|result| safe_div(result, 10000))
}

/// Calculate simple interest
/// interest = principal * rate * time / (rate_base * time_base)
/// rate is in basis points (e.g., 500 = 5%)
/// time is in blocks
/// rate_base is 10000 (for basis points)
/// time_base is blocks per year (~5,256,000 for 6s block time)
pub fn calculate_simple_interest(
    principal: Balance,
    rate_bps: u32,
    time_blocks: BlockNumber,
    blocks_per_year: BlockNumber,
) -> Option<Balance> {
    safe_mul(principal, rate_bps as Balance)
        .and_then(|result| safe_mul(result, time_blocks as Balance))
        .and_then(|result| safe_div(result, 10000 * blocks_per_year as Balance))
}

/// Calculate compound interest factor
/// Returns (1 + rate)^time - 1
/// Simplified version for blockchain use
pub fn calculate_compound_factor_approx(rate_bps: u32, periods: u32) -> Option<Balance> {
    if periods == 0 {
        return Some(0);
    }
    
    let rate = rate_bps as Balance;
    let base = 10000 as Balance;
    
    // Approximation: (1 + r)^n ≈ 1 + n*r + n*(n-1)*r^2/2
    // For small rates and reasonable periods
    let linear_term = safe_mul(periods as Balance, rate)?;
    
    if periods <= 1 {
        return Some(linear_term);
    }
    
    let quadratic_term = safe_mul(periods as Balance, (periods - 1) as Balance)?;
    let quadratic_term = safe_mul(quadratic_term, rate)?;
    let quadratic_term = safe_mul(quadratic_term, rate)?;
    let quadratic_term = safe_div(quadratic_term, 2)?;
    
    safe_add(linear_term, quadratic_term)
        .map(|result| safe_div(result, base).unwrap_or(0))
}

/// Calculate square root using integer Newton's method
pub fn sqrt(n: Balance) -> Balance {
    if n.is_zero() {
        return 0;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    
    x
}

/// Calculate minimum of two balances
pub fn min_balance(a: Balance, b: Balance) -> Balance {
    if a < b { a } else { b }
}

/// Calculate maximum of two balances
pub fn max_balance(a: Balance, b: Balance) -> Balance {
    if a > b { a } else { b }
}

/// Clamp value between min and max
pub fn clamp_balance(value: Balance, min: Balance, max: Balance) -> Balance {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_tier_platinum() {
        let balance = 100_000 * 10u128.pow(18);
        assert_eq!(calculate_fee_tier(balance), 2000);
    }

    #[test]
    fn test_fee_tier_gold() {
        let balance = 10_000 * 10u128.pow(18);
        assert_eq!(calculate_fee_tier(balance), 5000);
    }

    #[test]
    fn test_fee_tier_silver() {
        let balance = 1_000 * 10u128.pow(18);
        assert_eq!(calculate_fee_tier(balance), 7500);
    }

    #[test]
    fn test_fee_tier_bronze() {
        let balance = 100 * 10u128.pow(18);
        assert_eq!(calculate_fee_tier(balance), 9000);
    }

    #[test]
    fn test_fee_tier_none() {
        let balance = 50 * 10u128.pow(18);
        assert_eq!(calculate_fee_tier(balance), 10000);
    }

    #[test]
    fn test_discounted_fee() {
        // 0.3% base fee with 80% discount (pay 20%)
        assert_eq!(calculate_discounted_fee(30, 2000), 6);
        // 0.3% base fee with 50% discount
        assert_eq!(calculate_discounted_fee(30, 5000), 15);
    }

    #[test]
    fn test_safe_operations() {
        assert_eq!(safe_add(100, 200), Some(300));
        assert_eq!(safe_sub(300, 200), Some(100));
        assert_eq!(safe_mul(10, 20), Some(200));
        assert_eq!(safe_div(200, 10), Some(20));
        assert_eq!(safe_div(200, 0), None);
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(0), 0);
        assert_eq!(sqrt(1), 1);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(100), 10);
        assert_eq!(sqrt(10000), 100);
    }

    #[test]
    fn test_percentage() {
        assert_eq!(calculate_percentage(1000, 10), Some(100));
        assert_eq!(calculate_percentage(1000, 50), Some(500));
        assert_eq!(calculate_percentage(1000, 100), Some(1000));
    }

    #[test]
    fn test_basis_points() {
        assert_eq!(calculate_basis_points(10000, 100), Some(100)); // 1%
        assert_eq!(calculate_basis_points(10000, 1000), Some(1000)); // 10%
        assert_eq!(calculate_basis_points(10000, 5000), Some(5000)); // 50%
    }
}
