#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub type Placeholder<T: Config> = StorageValue<_, u64>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Placeholder { value: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        PlaceholderError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn placeholder(origin: OriginFor<T>, value: u64) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            
            Placeholder::<T>::put(value);
            Self::deposit_event(Event::Placeholder { value });
            
            Ok(())
        }
    }
}
