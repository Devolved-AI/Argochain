#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::PalletId;
use pallet_evm;
use frame_system::{ensure_root, pallet_prelude::OriginFor};

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::constant]
        type MintingInterval: Get<Self::BlockNumber>;

        type Currency: pallet_evm::Config::Currency;
    }

    #[pallet::storage]
    #[pallet::getter(fn last_minting_block)]
    pub type LastMintingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MintedAmount(H160, U256),
        BlockMinting(H160, U256, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        TooEarlyToMint,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint_evm_tokens(
            origin: OriginFor<T>,
            evm_address: H160,    // EVM address to mint tokens to
            amount: U256          // Amount of tokens to mint
        ) -> DispatchResult {
            ensure_root(origin)?;  // Ensure this can only be called by root

            // Ensure minting interval has passed
            let current_block = <frame_system::Pallet<T>>::block_number();
            let last_mint = LastMintingBlock::<T>::get();

            ensure!(
                current_block >= last_mint + T::MintingInterval::get(),
                Error::<T>::TooEarlyToMint
            );

            // Mint tokens using the EVM pallet
            <T as pallet_evm::Config>::Currency::deposit_creating(&evm_address, amount);

            // Update the last minting block
            LastMintingBlock::<T>::put(current_block);

            // Emit an event
            Self::deposit_event(Event::MintedAmount(evm_address, amount));
            Self::deposit_event(Event::BlockMinting(evm_address, amount, current_block));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            let last_mint = Self::last_minting_block();
            if n >= last_mint + T::MintingInterval::get() {
                // Perform automated minting or trigger logic here
                // This example assumes no automatic minting, but you can add logic here if needed

                // Reset last minting block
                LastMintingBlock::<T>::put(n);
            }
            0
        }
    }
}
