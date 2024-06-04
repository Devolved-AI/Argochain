#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;
        #[pallet::constant]
        type MintingAccount: Get<Self::AccountId>;
        #[pallet::constant]
        type MintAmount: Get<BalanceOf<Self>>;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            Self::mint_units();
        }
    }

    impl<T: Config> Pallet<T> {
        fn mint_units() {
            let minting_account = T::MintingAccount::get();
            let mint_amount: BalanceOf<T> = T::MintAmount::get();
            T::Currency::deposit_creating(&minting_account, mint_amount);
        }
    }
}
