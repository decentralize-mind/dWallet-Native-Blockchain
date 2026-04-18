use crate::{mock::*, Error, Balances, TotalSupply, ONE_TOKEN, MAX_SUPPLY};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_mint_tokens() {
    new_test_ext().execute_with(|| {
        // Mint tokens to account 1
        assert_ok!(DWTToken::mint(
            RuntimeOrigin::root(),
            1,
            1000 * ONE_TOKEN
        ));

        // Verify balance
        let balance = DWTToken::balance(1);
        assert_eq!(balance.free, 1000 * ONE_TOKEN);
        assert_eq!(balance.total(), 1000 * ONE_TOKEN);

        // Verify total supply
        assert_eq!(DWTToken::total_supply(), 1000 * ONE_TOKEN);

        // Check event
        System::assert_last_event(
            RuntimeEvent::DWTToken(crate::Event::Minted {
                to: 1,
                amount: 1000 * ONE_TOKEN,
            })
        );
    });
}

#[test]
fn test_cannot_mint_zero_amount() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DWTToken::mint(RuntimeOrigin::root(), 1, 0),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn test_cannot_exceed_max_supply() {
    new_test_ext().execute_with(|| {
        // Try to mint more than max supply
        assert_noop!(
            DWTToken::mint(RuntimeOrigin::root(), 1, MAX_SUPPLY + 1),
            Error::<Test>::MaxSupplyExceeded
        );
    });
}

#[test]
fn test_transfer_tokens() {
    new_test_ext_with_mint().execute_with(|| {
        // Transfer from account 1 to account 3
        assert_ok!(DWTToken::transfer(
            RuntimeOrigin::signed(1),
            3,
            100 * ONE_TOKEN
        ));

        // Verify balances
        let balance1 = DWTToken::balance(1);
        assert_eq!(balance1.free, 900 * ONE_TOKEN);

        let balance3 = DWTToken::balance(3);
        assert_eq!(balance3.free, 100 * ONE_TOKEN);

        // Check event
        System::assert_last_event(
            RuntimeEvent::DWTToken(crate::Event::Transferred {
                from: 1,
                to: 3,
                amount: 100 * ONE_TOKEN,
            })
        );
    });
}

#[test]
fn test_cannot_transfer_insufficient_balance() {
    new_test_ext_with_mint().execute_with(|| {
        // Try to transfer more than balance
        assert_noop!(
            DWTToken::transfer(RuntimeOrigin::signed(1), 3, 2000 * ONE_TOKEN),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn test_cannot_transfer_zero() {
    new_test_ext_with_mint().execute_with(|| {
        assert_noop!(
            DWTToken::transfer(RuntimeOrigin::signed(1), 3, 0),
            Error::<Test>::ZeroAmount
        );
    });
}

#[test]
fn test_burn_tokens() {
    new_test_ext_with_mint().execute_with(|| {
        let initial_supply = DWTToken::total_supply();

        // Burn tokens
        assert_ok!(DWTToken::burn(
            RuntimeOrigin::signed(1),
            100 * ONE_TOKEN
        ));

        // Verify balance reduced
        let balance = DWTToken::balance(1);
        assert_eq!(balance.free, 900 * ONE_TOKEN);

        // Verify total supply reduced
        assert_eq!(DWTToken::total_supply(), initial_supply - 100 * ONE_TOKEN);

        // Check event
        System::assert_last_event(
            RuntimeEvent::DWTToken(crate::Event::Burned {
                from: 1,
                amount: 100 * ONE_TOKEN,
            })
        );
    });
}

#[test]
fn test_cannot_burn_insufficient_balance() {
    new_test_ext_with_mint().execute_with(|| {
        assert_noop!(
            DWTToken::burn(RuntimeOrigin::signed(1), 2000 * ONE_TOKEN),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn test_approve_and_transfer_from() {
    new_test_ext_with_mint().execute_with(|| {
        // Approve account 2 to spend 500 tokens
        assert_ok!(DWTToken::approve(
            RuntimeOrigin::signed(1),
            2,
            500 * ONE_TOKEN
        ));

        // Verify allowance
        let allowance = DWTToken::allowance(1, 2).unwrap();
        assert_eq!(allowance.amount, 500 * ONE_TOKEN);

        // Transfer from account 1 to account 3 using allowance
        assert_ok!(DWTToken::transfer_from(
            RuntimeOrigin::signed(2),
            1,
            3,
            200 * ONE_TOKEN
        ));

        // Verify balances
        assert_eq!(DWTToken::balance(1).free, 800 * ONE_TOKEN);
        assert_eq!(DWTToken::balance(3).free, 200 * ONE_TOKEN);

        // Verify allowance reduced
        let allowance = DWTToken::allowance(1, 2).unwrap();
        assert_eq!(allowance.amount, 300 * ONE_TOKEN);
    });
}

#[test]
fn test_cannot_transfer_from_insufficient_allowance() {
    new_test_ext_with_mint().execute_with(|| {
        // Approve only 100 tokens
        assert_ok!(DWTToken::approve(
            RuntimeOrigin::signed(1),
            2,
            100 * ONE_TOKEN
        ));

        // Try to transfer 200 tokens
        assert_noop!(
            DWTToken::transfer_from(RuntimeOrigin::signed(2), 1, 3, 200 * ONE_TOKEN),
            Error::<Test>::InsufficientAllowance
        );
    });
}

#[test]
fn test_transfer_all() {
    new_test_ext_with_mint().execute_with(|| {
        let initial_balance = DWTToken::balance(1).free;

        // Transfer all
        assert_ok!(DWTToken::transfer_all(
            RuntimeOrigin::signed(1),
            3
        ));

        // Verify sender has 0 balance
        assert_eq!(DWTToken::balance(1).free, 0);

        // Verify recipient received all
        assert_eq!(DWTToken::balance(3).free, initial_balance);
    });
}

#[test]
fn test_fee_tier_calculations() {
    new_test_ext().execute_with(|| {
        // Mint different amounts to test tiers
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 1, 50 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 2, 500 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 3, 5000 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 4, 50000 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 5, 150000 * ONE_TOKEN));

        // Check fee tiers
        assert_eq!(DWTToken::get_fee_tier(&1), 10000); // None (< 100)
        assert_eq!(DWTToken::get_fee_tier(&2), 9000);  // Bronze (>= 100)
        assert_eq!(DWTToken::get_fee_tier(&3), 7500);  // Silver (>= 1,000)
        assert_eq!(DWTToken::get_fee_tier(&4), 5000);  // Gold (>= 10,000)
        assert_eq!(DWTToken::get_fee_tier(&5), 2000);  // Platinum (>= 100,000)
    });
}

#[test]
fn test_voting_power_updates() {
    new_test_ext_with_mint().execute_with(|| {
        // Initial voting power
        let vp1 = DWTToken::voting_power(1);
        assert_eq!(vp1.current, 1000 * ONE_TOKEN);

        // Transfer tokens
        assert_ok!(DWTToken::transfer(
            RuntimeOrigin::signed(1),
            3,
            200 * ONE_TOKEN
        ));

        // Verify voting power updated
        let vp1_after = DWTToken::voting_power(1);
        assert_eq!(vp1_after.current, 800 * ONE_TOKEN);

        let vp3 = DWTToken::voting_power(3);
        assert_eq!(vp3.current, 200 * ONE_TOKEN);
    });
}

#[test]
fn test_create_snapshot() {
    new_test_ext_with_mint().execute_with(|| {
        // Create snapshot for proposal 1
        assert_ok!(DWTToken::create_snapshot(RuntimeOrigin::signed(1), 1));

        // Verify snapshot was created
        let snapshot_block = DWTToken::get_voting_snapshot(&1, 1);
        assert!(snapshot_block.is_some());
    });
}

#[test]
fn test_helper_functions() {
    new_test_ext_with_mint().execute_with(|| {
        // Test total_balance
        assert_eq!(DWTToken::total_balance(&1), 1000 * ONE_TOKEN);

        // Test available_balance
        assert_eq!(DWTToken::available_balance(&1), 1000 * ONE_TOKEN);
    });
}

#[test]
fn test_multiple_mints() {
    new_test_ext().execute_with(|| {
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 1, 500 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 2, 300 * ONE_TOKEN));
        assert_ok!(DWTToken::mint(RuntimeOrigin::root(), 3, 200 * ONE_TOKEN));

        assert_eq!(DWTToken::total_supply(), 1000 * ONE_TOKEN);
        assert_eq!(DWTToken::balance(1).free, 500 * ONE_TOKEN);
        assert_eq!(DWTToken::balance(2).free, 300 * ONE_TOKEN);
        assert_eq!(DWTToken::balance(3).free, 200 * ONE_TOKEN);
    });
}

#[test]
fn test_genesis_mint() {
    new_test_ext_with_mint().execute_with(|| {
        // Verify genesis mint worked
        assert_eq!(DWTToken::balance(1).free, 1000 * ONE_TOKEN);
        assert_eq!(DWTToken::balance(2).free, 500 * ONE_TOKEN);
        assert_eq!(DWTToken::total_supply(), 1500 * ONE_TOKEN);
    });
}

#[test]
fn test_transfer_preserves_total_supply() {
    new_test_ext_with_mint().execute_with(|| {
        let initial_supply = DWTToken::total_supply();

        // Perform multiple transfers
        assert_ok!(DWTToken::transfer(RuntimeOrigin::signed(1), 3, 100 * ONE_TOKEN));
        assert_ok!(DWTToken::transfer(RuntimeOrigin::signed(2), 3, 50 * ONE_TOKEN));
        assert_ok!(DWTToken::transfer(RuntimeOrigin::signed(3), 1, 25 * ONE_TOKEN));

        // Total supply should remain unchanged
        assert_eq!(DWTToken::total_supply(), initial_supply);
    });
}

#[test]
fn test_burn_reduces_total_supply() {
    new_test_ext_with_mint().execute_with(|| {
        let initial_supply = DWTToken::total_supply();

        // Burn from multiple accounts
        assert_ok!(DWTToken::burn(RuntimeOrigin::signed(1), 100 * ONE_TOKEN));
        assert_ok!(DWTToken::burn(RuntimeOrigin::signed(2), 50 * ONE_TOKEN));

        // Total supply should be reduced
        assert_eq!(DWTToken::total_supply(), initial_supply - 150 * ONE_TOKEN);
    });
}
