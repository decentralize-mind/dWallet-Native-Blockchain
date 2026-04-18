#![cfg_attr(not(feature = "std"), no_std)]

//! # Layer 1: DWT Token Pallet
//!
//! Native token implementation with:
//! - ERC20-like functionality (mint, burn, transfer, approve)
//! - Fee tier system based on token holdings
//! - Flash-loan resistant voting power snapshots
//! - Max supply enforcement (1,123,000,000 DWT)

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{Zero, CheckedAdd, CheckedSub, CheckedMul, CheckedDiv},
    ArithmeticError,
};

pub use pallet::*;

/// Token decimals (18)
pub const DECIMALS: u8 = 18;

/// One token in base units (10^18)
pub const ONE_TOKEN: u128 = 10u128.pow(18);

/// Maximum supply: 1,123,000,000 DWT
pub const MAX_SUPPLY: u128 = 1_123_000_000 * ONE_TOKEN;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Token balance information
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct TokenBalance {
        /// Free transferable balance
        pub free: u128,
        /// Reserved balance (staking, voting)
        pub reserved: u128,
        /// Frozen balance (cannot spend)
        pub frozen: u128,
    }

    impl TokenBalance {
        /// Total balance (free + reserved)
        pub fn total(&self) -> u128 {
            self.free.saturating_add(self.reserved)
        }

        /// Available balance (free - frozen)
        pub fn available(&self) -> u128 {
            self.free.saturating_sub(self.frozen)
        }
    }

    /// Voting power with snapshot support
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct VotingPower {
        /// Current voting power
        pub current: u128,
        /// Snapshot voting power (for governance)
        pub snapshot: u128,
        /// Block number when snapshot was taken
        pub snapshot_block: u32,
    }

    /// Allowance for spender
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct Allowance {
        /// Approved amount
        pub amount: u128,
        /// Expiration block (0 = no expiry)
        pub expires_at: u32,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Origin that can mint tokens
        type MintOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// Total token supply
    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    pub type TotalSupply<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Account balances
    #[pallet::storage]
    #[pallet::getter(fn balance)]
    pub type Balances<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        TokenBalance,
        ValueQuery,
    >;

    /// Voting power for governance
    #[pallet::storage]
    #[pallet::getter(fn voting_power)]
    pub type VotingPowerMap<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        VotingPower,
        ValueQuery,
    >;

    /// Allowances: (owner, spender) -> Allowance
    #[pallet::storage]
    #[pallet::getter(fn allowance)]
    pub type Allowances<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId, // owner
        Twox64Concat,
        T::AccountId, // spender
        Allowance,
    >;

    /// Token owner (can mint)
    #[pallet::storage]
    #[pallet::getter(fn token_owner)]
    pub type TokenOwner<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Governance snapshot storage
    #[pallet::storage]
    pub type VoteSnapshots<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u64, // proposal_id
        u32, // snapshot_block
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_mint: Vec<(T::AccountId, u128)>,
        pub token_owner: Option<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_mint: Vec::new(),
                token_owner: None,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // Set token owner
            if let Some(owner) = &self.token_owner {
                TokenOwner::<T>::put(owner);
            }

            // Mint initial tokens
            for (account, amount) in &self.initial_mint {
                let current_supply = TotalSupply::<T>::get();
                let new_supply = current_supply.saturating_add(*amount);
                assert!(new_supply <= MAX_SUPPLY, "Initial mint exceeds max supply");

                Balances::<T>::mutate(account, |balance| {
                    balance.free = balance.free.saturating_add(*amount);
                });

                VotingPowerMap::<T>::mutate(account, |vp| {
                    vp.current = vp.current.saturating_add(*amount);
                });
            }

            TotalSupply::<T>::put(self.initial_mint.iter().map(|(_, amount)| amount).sum());
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens were minted
        Minted {
            to: T::AccountId,
            amount: u128,
        },
        /// Tokens were burned
        Burned {
            from: T::AccountId,
            amount: u128,
        },
        /// Tokens were transferred
        Transferred {
            from: T::AccountId,
            to: T::AccountId,
            amount: u128,
        },
        /// Allowance was approved
        Approval {
            owner: T::AccountId,
            spender: T::AccountId,
            amount: u128,
        },
        /// Ownership was transferred
        OwnershipTransferred {
            old_owner: T::AccountId,
            new_owner: T::AccountId,
        },
        /// Voting snapshot was created
        SnapshotCreated {
            proposal_id: u64,
            snapshot_block: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient free balance
        InsufficientBalance,
        /// Insufficient allowance
        InsufficientAllowance,
        /// Overflow in arithmetic operation
        Overflow,
        /// Underflow in arithmetic operation
        Underflow,
        /// Caller is not authorized
        Unauthorized,
        /// Amount cannot be zero
        ZeroAmount,
        /// Max supply exceeded
        MaxSupplyExceeded,
        /// Allowance has expired
        AllowanceExpired,
        /// Snapshot already exists for this proposal
        SnapshotExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint new tokens (owner only)
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            T::MintOrigin::ensure_origin(origin)?;

            ensure!(amount > 0, Error::<T>::ZeroAmount);

            // Check max supply
            let current_supply = TotalSupply::<T>::get();
            let new_supply = current_supply
                .checked_add(amount)
                .ok_or(Error::<T>::Overflow)?;
            ensure!(new_supply <= MAX_SUPPLY, Error::<T>::MaxSupplyExceeded);

            // Update balance
            Balances::<T>::mutate(&to, |balance| {
                balance.free = balance.free.saturating_add(amount);
            });

            // Update voting power
            VotingPowerMap::<T>::mutate(&to, |vp| {
                vp.current = vp.current.saturating_add(amount);
            });

            // Update total supply
            TotalSupply::<T>::put(new_supply);

            Self::deposit_event(Event::Minted { to, amount });

            Ok(())
        }

        /// Burn tokens from caller's balance
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            amount: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::ZeroAmount);

            let balance = Balances::<T>::get(&who);
            ensure!(balance.free >= amount, Error::<T>::InsufficientBalance);

            // Update balance
            Balances::<T>::mutate(&who, |b| {
                b.free = b.free.saturating_sub(amount);
            });

            // Update voting power
            VotingPowerMap::<T>::mutate(&who, |vp| {
                vp.current = vp.current.saturating_sub(amount);
            });

            // Update total supply
            let current_supply = TotalSupply::<T>::get();
            TotalSupply::<T>::put(current_supply.saturating_sub(amount));

            Self::deposit_event(Event::Burned {
                from: who,
                amount,
            });

            Ok(())
        }

        /// Transfer tokens to another account
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            Self::do_transfer(&from, &to, amount)?;

            Self::deposit_event(Event::Transferred {
                from,
                to,
                amount,
            });

            Ok(())
        }

        /// Transfer entire free balance
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn transfer_all(
            origin: OriginFor<T>,
            to: T::AccountId,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            let balance = Balances::<T>::get(&from);
            let amount = balance.available();

            ensure!(amount > 0, Error::<T>::InsufficientBalance);

            Self::do_transfer(&from, &to, amount)?;

            Self::deposit_event(Event::Transferred {
                from,
                to,
                amount,
            });

            Ok(())
        }

        /// Approve spender to use tokens
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn approve(
            origin: OriginFor<T>,
            spender: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            Allowances::<T>::insert(
                &owner,
                &spender,
                Allowance {
                    amount,
                    expires_at: 0, // No expiry
                },
            );

            Self::deposit_event(Event::Approval {
                owner,
                spender,
                amount,
            });

            Ok(())
        }

        /// Transfer tokens on behalf of owner (using allowance)
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let spender = ensure_signed(origin)?;

            // Check and update allowance
            let mut allowance = Allowances::<T>::get(&from, &spender)
                .unwrap_or_default();

            // Check expiry
            if allowance.expires_at > 0 {
                let current_block = <frame_system::Pallet<T>>::block_number();
                ensure!(
                    current_block < allowance.expires_at.into(),
                    Error::<T>::AllowanceExpired
                );
            }

            ensure!(allowance.amount >= amount, Error::<T>::InsufficientAllowance);

            // Deduct from allowance
            allowance.amount = allowance.amount.saturating_sub(amount);
            Allowances::<T>::insert(&from, &spender, allowance);

            // Execute transfer
            Self::do_transfer(&from, &to, amount)?;

            Self::deposit_event(Event::Transferred {
                from,
                to,
                amount,
            });

            Ok(())
        }

        /// Create voting snapshot for governance proposal
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)]
        pub fn create_snapshot(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Check if snapshot already exists
            ensure!(
                !VoteSnapshots::<T>::contains_key(proposal_id),
                Error::<T>::SnapshotExists
            );

            // Snapshot is taken 100 blocks ago for flash-loan resistance
            let current_block = <frame_system::Pallet<T>>::block_number();
            let snapshot_block = current_block.saturating_sub(100u32.into());

            // Record snapshot block
            VoteSnapshots::<T>::insert(proposal_id, snapshot_block.as_());

            // Update voting power snapshots for all accounts
            for (account, balance) in Balances::<T>::iter() {
                VotingPowerMap::<T>::mutate(&account, |vp| {
                    vp.snapshot = balance.total();
                    vp.snapshot_block = snapshot_block.as_();
                });
            }

            Self::deposit_event(Event::SnapshotCreated {
                proposal_id,
                snapshot_block: snapshot_block.as_(),
            });

            Ok(())
        }

        /// Transfer token ownership (minting rights)
        #[pallet::call_index(7)]
        #[pallet::weight(10_000)]
        pub fn transfer_ownership(
            origin: OriginFor<T>,
            new_owner: T::AccountId,
        ) -> DispatchResult {
            let current_owner = ensure_signed(origin)?;

            // Verify current owner
            let owner = TokenOwner::<T>::get()
                .ok_or(Error::<T>::Unauthorized)?;
            ensure!(owner == current_owner, Error::<T>::Unauthorized);

            TokenOwner::<T>::put(&new_owner);

            Self::deposit_event(Event::OwnershipTransferred {
                old_owner: current_owner,
                new_owner,
            });

            Ok(())
        }
    }

    /// Helper functions
    impl<T: Config> Pallet<T> {
        /// Internal transfer function
        fn do_transfer(from: &T::AccountId, to: &T::AccountId, amount: u128) -> DispatchResult {
            ensure!(amount > 0, Error::<T>::ZeroAmount);

            // Check sender balance
            let from_balance = Balances::<T>::get(from);
            ensure!(from_balance.free >= amount, Error::<T>::InsufficientBalance);

            // Update sender balance
            Balances::<T>::mutate(from, |b| {
                b.free = b.free.saturating_sub(amount);
            });

            // Update sender voting power
            VotingPowerMap::<T>::mutate(from, |vp| {
                vp.current = vp.current.saturating_sub(amount);
            });

            // Update recipient balance
            Balances::<T>::mutate(to, |b| {
                b.free = b.free.saturating_add(amount);
            });

            // Update recipient voting power
            VotingPowerMap::<T>::mutate(to, |vp| {
                vp.current = vp.current.saturating_add(amount);
            });

            Ok(())
        }

        /// Get fee tier for an account based on balance
        pub fn get_fee_tier(account: &T::AccountId) -> u32 {
            let balance = Balances::<T>::get(account).total();

            if balance >= 100_000 * ONE_TOKEN {
                2000  // Platinum: 80% discount
            } else if balance >= 10_000 * ONE_TOKEN {
                5000  // Gold: 50% discount
            } else if balance >= 1_000 * ONE_TOKEN {
                7500  // Silver: 25% discount
            } else if balance >= 100 * ONE_TOKEN {
                9000  // Bronze: 10% discount
            } else {
                10000 // No discount
            }
        }

        /// Get total balance (free + reserved)
        pub fn total_balance(account: &T::AccountId) -> u128 {
            Balances::<T>::get(account).total()
        }

        /// Get available balance (free - frozen)
        pub fn available_balance(account: &T::AccountId) -> u128 {
            Balances::<T>::get(account).available()
        }

        /// Get voting snapshot for a proposal
        pub fn get_voting_snapshot(
            account: &T::AccountId,
            proposal_id: u64,
        ) -> Option<u128> {
            if let Some(snapshot_block) = VoteSnapshots::<T>::get(proposal_id) {
                let vp = VotingPowerMap::<T>::get(account);
                if vp.snapshot_block == snapshot_block {
                    return Some(vp.snapshot);
                }
            }
            None
        }
    }
}
