// pallets/mint/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use pallet_balances::BalanceOf;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CoinsMinted(T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotSudo,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint_coins(origin: OriginFor<T>, to: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            // Ensure the origin is from the Sudo account
            ensure_root(origin)?;

            // Mint coins to the specified account
            pallet_balances::Pallet::<T>::mutate_account(to.clone(), |balance| {
                *balance += amount;
            });

            // Emit an event
            Self::deposit_event(Event::CoinsMinted(to, amount));

            Ok(())
        }
    }
}
