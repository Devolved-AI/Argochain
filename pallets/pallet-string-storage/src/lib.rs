#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{pallet_prelude::*, ensure_root};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) = Storage)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::storage]
    #[pallet::getter(fn stored_string)]
    pub type StoredString<T> = StorageValue<_, Vec<u8>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) = true)]
    pub enum Event<T: Config> {
        StringStored(Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn store_string(origin: OriginFor<T>, input_string: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;

            StoredString::<T>::put(&input_string);

            Self::deposit_event(Event::StringStored(input_string));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn get_stored_string(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let stored_string = StoredString::<T>::get().ok_or(Error::<T>::NoneValue)?;

            Ok(Some(stored_string).into())
        }
    }
}
