#![cfg_attr(not(feature = "std"), no_std)]

//! # Layer 0: Protocol Registry Pallet
//!
//! Central address registry for all 10 security layers.
//! Manages layer registration, timelock updates, and genesis phase.

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Zero;

pub use pallet::*;

/// Timelock period for layer updates (48 hours in blocks)
/// Assuming 6-second block time: 48 * 60 * 60 / 6 = 28,800 blocks
pub const TIMELOCK_PERIOD: u32 = 28800;

/// Genesis phase duration (24 hours in blocks)
/// 24 * 60 * 60 / 6 = 14,400 blocks
pub const GENESIS_DURATION: u32 = 14400;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Layer registry information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct LayerInfo<BlockNumber, AccountId> {
        /// Layer ID (0-10)
        pub layer_id: u8,
        /// Layer account/address
        pub address: AccountId,
        /// Block number when layer was registered
        pub registered_at: BlockNumber,
        /// Block number of last update
        pub last_updated: BlockNumber,
        /// Earliest block when next update is allowed
        pub update_timelock: BlockNumber,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Origin that can register layers (typically Root or specific authority)
        type RegistryOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// Storage map: Layer ID -> Layer Info
    #[pallet::storage]
    #[pallet::getter(fn get_layer_info)]
    pub type LayerRegistry<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u8, // Layer ID
        LayerInfo<T::BlockNumber, T::AccountId>,
    >;

    /// Whether the protocol is in genesis phase
    #[pallet::storage]
    #[pallet::getter(fn is_genesis_phase)]
    pub type GenesisPhase<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Block number when genesis phase started
    #[pallet::storage]
    #[pallet::getter(fn genesis_start_block)]
    pub type GenesisStartBlock<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;

    /// Track pending layer updates (layer_id -> (new_address, available_from_block))
    #[pallet::storage]
    #[pallet::getter(fn pending_update)]
    pub type PendingUpdates<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u8,
        (T::AccountId, T::BlockNumber),
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_layers: Vec<(u8, T::AccountId)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_layers: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // Start in genesis phase
            GenesisPhase::<T>::put(true);
            GenesisStartBlock::<T>::put(T::BlockNumber::zero());

            // Register initial layers
            for (layer_id, address) in &self.initial_layers {
                let info = LayerInfo {
                    layer_id: *layer_id,
                    address: address.clone(),
                    registered_at: T::BlockNumber::zero(),
                    last_updated: T::BlockNumber::zero(),
                    update_timelock: T::BlockNumber::zero(),
                };
                LayerRegistry::<T>::insert(layer_id, info);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new layer has been registered
        LayerRegistered {
            layer_id: u8,
            address: T::AccountId,
        },
        /// A layer update has been initiated (timelock started)
        LayerUpdateInitiated {
            layer_id: u8,
            old_address: T::AccountId,
            new_address: T::AccountId,
            available_from_block: T::BlockNumber,
        },
        /// A pending layer update has been executed
        LayerUpdateExecuted {
            layer_id: u8,
            old_address: T::AccountId,
            new_address: T::AccountId,
        },
        /// Genesis phase has ended
        GenesisEnded,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Layer is already registered
        LayerAlreadyRegistered,
        /// Layer is not registered
        LayerNotRegistered,
        /// Caller is not authorized to register layers
        NotAuthorized,
        /// Timelock period has not expired yet
        TimelockNotExpired,
        /// Genesis phase is still active
        GenesisPhaseActive,
        /// Genesis phase has already ended
        GenesisPhaseEnded,
        /// No pending update exists for this layer
        NoPendingUpdate,
        /// Invalid layer ID (must be 0-10)
        InvalidLayerId,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new layer (only during genesis or by authorized origin)
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_layer(
            origin: OriginFor<T>,
            layer_id: u8,
            address: T::AccountId,
        ) -> DispatchResult {
            T::RegistryOrigin::ensure_origin(origin)?;

            // Validate layer ID
            ensure!(layer_id <= 10, Error::<T>::InvalidLayerId);

            // Check if layer is already registered
            ensure!(
                !LayerRegistry::<T>::contains_key(layer_id),
                Error::<T>::LayerAlreadyRegistered
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            // Create layer info
            let info = LayerInfo {
                layer_id,
                address: address.clone(),
                registered_at: current_block,
                last_updated: current_block,
                update_timelock: current_block, // Can update immediately after genesis
            };

            LayerRegistry::<T>::insert(layer_id, info);

            Self::deposit_event(Event::LayerRegistered { layer_id, address });

            Ok(())
        }

        /// Initiate a layer address update (starts 48-hour timelock)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn initiate_layer_update(
            origin: OriginFor<T>,
            layer_id: u8,
            new_address: T::AccountId,
        ) -> DispatchResult {
            T::RegistryOrigin::ensure_origin(origin)?;

            // Validate layer ID
            ensure!(layer_id <= 10, Error::<T>::InvalidLayerId);

            // Check if layer exists
            let layer_info = LayerRegistry::<T>::get(layer_id)
                .ok_or(Error::<T>::LayerNotRegistered)?;

            // Cannot update during genesis phase
            ensure!(!GenesisPhase::<T>::get(), Error::<T>::GenesisPhaseActive);

            let current_block = <frame_system::Pallet<T>>::block_number();
            let available_from = current_block + T::BlockNumber::from(TIMELOCK_PERIOD);

            // Store pending update
            PendingUpdates::<T>::insert(layer_id, (new_address.clone(), available_from));

            Self::deposit_event(Event::LayerUpdateInitiated {
                layer_id,
                old_address: layer_info.address,
                new_address,
                available_from_block: available_from,
            });

            Ok(())
        }

        /// Execute a pending layer update (after timelock expires)
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn execute_layer_update(
            origin: OriginFor<T>,
            layer_id: u8,
        ) -> DispatchResult {
            T::RegistryOrigin::ensure_origin(origin)?;

            // Validate layer ID
            ensure!(layer_id <= 10, Error::<T>::InvalidLayerId);

            // Check if layer exists
            let mut layer_info = LayerRegistry::<T>::get(layer_id)
                .ok_or(Error::<T>::LayerNotRegistered)?;

            // Check if there's a pending update
            let (new_address, available_from) = PendingUpdates::<T>::get(layer_id)
                .ok_or(Error::<T>::NoPendingUpdate)?;

            // Check if timelock has expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(
                current_block >= available_from,
                Error::<T>::TimelockNotExpired
            );

            let old_address = layer_info.address.clone();

            // Update layer info
            layer_info.address = new_address.clone();
            layer_info.last_updated = current_block;
            layer_info.update_timelock = current_block + T::BlockNumber::from(TIMELOCK_PERIOD);

            LayerRegistry::<T>::insert(layer_id, layer_info);
            PendingUpdates::<T>::remove(layer_id);

            Self::deposit_event(Event::LayerUpdateExecuted {
                layer_id,
                old_address,
                new_address,
            });

            Ok(())
        }

        /// End the genesis phase (can only be called once)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn end_genesis_phase(origin: OriginFor<T>) -> DispatchResult {
            T::RegistryOrigin::ensure_origin(origin)?;

            ensure!(GenesisPhase::<T>::get(), Error::<T>::GenesisPhaseEnded);

            // Check if genesis duration has passed
            if let Some(start_block) = GenesisStartBlock::<T>::get() {
                let current_block = <frame_system::Pallet<T>>::block_number();
                let elapsed = current_block - start_block;
                ensure!(
                    elapsed >= T::BlockNumber::from(GENESIS_DURATION),
                    Error::<T>::GenesisPhaseActive
                );
            }

            GenesisPhase::<T>::put(false);

            Self::deposit_event(Event::GenesisEnded);

            Ok(())
        }
    }

    /// Helper functions
    impl<T: Config> Pallet<T> {
        /// Get the address for a specific layer
        pub fn get_layer_address(layer_id: u8) -> Option<T::AccountId> {
            LayerRegistry::<T>::get(layer_id).map(|info| info.address)
        }

        /// Check if a layer is registered
        pub fn is_layer_registered(layer_id: u8) -> bool {
            LayerRegistry::<T>::contains_key(layer_id)
        }

        /// Get all registered layer IDs
        pub fn get_registered_layers() -> Vec<u8> {
            LayerRegistry::<T>::iter_keys().collect()
        }

        /// Check if update timelock has expired for a layer
        pub fn can_update_layer(layer_id: u8) -> bool {
            if let Some(layer_info) = LayerRegistry::<T>::get(layer_id) {
                let current_block = <frame_system::Pallet<T>>::block_number();
                current_block >= layer_info.update_timelock
            } else {
                false
            }
        }
    }
}
