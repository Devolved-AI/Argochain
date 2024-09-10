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

    // Storage to hold the string
    #[pallet::storage]
    #[pallet::getter(fn stored_string)]
    pub type StoredString<T> = StorageValue<_, Vec<u8>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) = true)]
    pub enum Event<T: Config> {
        // Event emitted when a string is successfully stored
        StringStored(Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        // Error when trying to store an invalid string
        NoneValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Function to store a string, callable only by the sudo account
        #[pallet::weight(10_000)]
        pub fn store_string(origin: OriginFor<T>, input_string: Vec<u8>) -> DispatchResult {
            // Ensure the caller is the root (sudo) account
            ensure_root(origin)?;

            // Store the string
            StoredString::<T>::put(&input_string);

            // Emit an event
            Self::deposit_event(Event::StringStored(input_string));

            Ok(())
        }

        // Function to retrieve the stored string (publicly callable)
        #[pallet::weight(10_000)]
        pub fn get_stored_string(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let stored_string = StoredString::<T>::get().ok_or(Error::<T>::NoneValue)?;

            // Optionally, return the string in the event or through other mechanisms
            Ok(Some(stored_string).into())
        }
    }
}
