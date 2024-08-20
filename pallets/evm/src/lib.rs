#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_support::PalletId;
use pallet_evm;

// Configurations for our pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // Define a struct for the Pallet.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Define a constant for the weight of mint_evm_tokens function.
    const MINT_EVM_TOKENS_WEIGHT: u64 = 10_000;

    // Configuration for our pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    // Declare events that can be emitted by this Pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MintedAmount(H160, U256)
    }

    // Public function that can be called by other modules.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(MINT_EVM_TOKENS_WEIGHT)]
        pub fn mint_evm_tokens(
            origin: OriginFor<T>,    // Authorization origin.
            evm_address: H160,       // EVM address to mint tokens to.
            amount: U256,             // Number of tokens to mint.
        ) -> DispatchResult {
            ensure_root(origin)?;
            <T as pallet_evm::ConfigT>::Currency::deposit_creating(&evm_address, amount);
            Self::deposit_event(Event::MintedAmount(evm_address, amount));
            Ok(())
        }
    }
}
