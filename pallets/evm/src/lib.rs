#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::PalletId;
use pallet_evm;

#[frame_support::pallet]

pub mod pallet{

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config:frame_system::Config{
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type PalletId:Get<PalletId> ;
    }


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T:Config>{
        MintedAmount(H160,U256)
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

        // Mint tokens using the EVM pallet
        <T as pallet_evm::Config>::Currency::deposit_creating(&evm_address, amount);

        // Emit an event if desired
        Self::deposit_event(Event::MintedAmount(evm_address, amount));

        Ok(())
    }
}

}

