#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use pallet_evm::Pallet as EvmPallet;
    use sp_core::{H160, U256, H256, ecdsa};
    use sp_runtime::traits::SaturatedConversion;
    use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
    use sp_io::hashing::keccak_256;
    use scale_info::prelude::format;
    use sp_std::vec::Vec;
    use sp_std::str::FromStr;
    use hex_literal::hex;
    use sp_core::crypto::ByteArray;
    use sp_runtime::traits::StaticLookup;
    use pallet_evm::{AddressMapping, PrecompileSet, Vicinity};
    use sp_io::crypto::secp256k1_ecdsa_recover;
    use scale_info::prelude::string::String;




    type SubstrateBalanceOf<T> = <<T as Config>::SubstrateCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type EvmBalanceOf<T> = <<T as Config>::EvmCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_evm::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type SubstrateCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type EvmCurrency: Currency<Self::AccountId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn locked_balance)]
    pub type LockedBalance<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, SubstrateBalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Minted { who: T::AccountId, amount: SubstrateBalanceOf<T> },
        Burned { who: T::AccountId, amount: SubstrateBalanceOf<T> },
        Locked { who: T::AccountId, amount: SubstrateBalanceOf<T> },
        Unlocked { who: T::AccountId, amount: SubstrateBalanceOf<T> },
        EvmBalanceChecked(H160, U256),
        EvmBalanceMutated(H160, U256, bool),
        EvmToSubstrateTransfer(H160, T::AccountId, u128),
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        LockNotFound,
        UnlockNotPossible,
        Unauthorized,
        AmountConversionFailed,
        OperationNotAllowed,
        InvalidSignature,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(origin: OriginFor<T>, account: T::AccountId, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;

            T::SubstrateCurrency::deposit_creating(&account, amount);
            Self::deposit_event(Event::Minted { who: account, amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn burn(origin: OriginFor<T>, account: T::AccountId, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;

            T::SubstrateCurrency::withdraw(
                &account,
                amount,
                frame_support::traits::WithdrawReasons::TRANSFER,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            Self::deposit_event(Event::Burned { who: account, amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn lock(origin: OriginFor<T>, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::SubstrateCurrency::reserve(&who, amount)?;
            <LockedBalance<T>>::insert(&who, amount);
            Self::deposit_event(Event::Locked { who: who.clone(), amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn unlock(origin: OriginFor<T>, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let locked_amount = <LockedBalance<T>>::get(&who);
            ensure!(locked_amount >= amount, Error::<T>::UnlockNotPossible);

            T::SubstrateCurrency::unreserve(&who, amount);
            <LockedBalance<T>>::mutate(&who, |balance| *balance -= amount);
            Self::deposit_event(Event::Unlocked { who: who.clone(), amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn check_evm_balance(origin: OriginFor<T>, evm_address: H160) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let (account, _) = EvmPallet::<T>::account_basic(&evm_address);

            Self::deposit_event(Event::EvmBalanceChecked(evm_address, account.balance));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn substrate_to_evm(
            origin: OriginFor<T>,
            evm_address: H160,
            amount: SubstrateBalanceOf<T>,
            add: bool,
        ) -> DispatchResult {
            ensure!(add, Error::<T>::OperationNotAllowed);

            let substrate_account = ensure_signed(origin)?;

            let transferable_balance = T::SubstrateCurrency::free_balance(&substrate_account);
            ensure!(transferable_balance >= amount, Error::<T>::InsufficientBalance);

            T::SubstrateCurrency::withdraw(
                &substrate_account,
                amount,
                frame_support::traits::WithdrawReasons::TRANSFER,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let amount_u256 = U256::from(amount.saturated_into::<u128>());

            EvmPallet::<T>::mutate_balance(evm_address, amount_u256, add);

            Self::deposit_event(Event::EvmBalanceMutated(evm_address, amount_u256, add));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn evm_to_substrate(
            origin: OriginFor<T>,
            evm_address: H160,
            // substrate_account: T::AccountId,
            amount: U256,
            subtract: bool,
            signature: ecdsa::Signature, 
        ) -> DispatchResult {
            ensure!(!subtract, Error::<T>::Unauthorized);
            // let _who = ensure_signed(origin)?;
            let substrate_account = ensure_signed(origin)?;
            frame_support::log::info!("amount:{}", amount);
            frame_support::log::info!("signature:{:?}", signature);
        
            let amount_u128: u128 = amount.try_into().map_err(|_| Error::<T>::AmountConversionFailed)?;
            let message = format!("Transfer {} AGC from 0x{:x} to Substrate", amount_u128, evm_address);
            frame_support::log::info!("amount in u128:{}", amount_u128);
            frame_support::log::info!("message:{}", message);
            frame_support::log::info!("message length:{}", message.len());
        
            let prefix = "\x19Ethereum Signed Message:\n";
            let message_len = format!("{}", message.len());
            let message_to_sign = format!("{}{}{}", prefix, message_len, message);
        
            let message_hash = keccak_256(message_to_sign.as_bytes());
            frame_support::log::info!("message_to_sign:{}", message_to_sign);
            frame_support::log::info!("message_hash:{:?}", message_hash);
        
            let r = &signature.0[..32];
            let s = &signature.0[32..64];
            let v = &signature.0[64..65];
            
            let mut sig_array = [0u8; 65];
            sig_array[..32].copy_from_slice(r);
            sig_array[32..64].copy_from_slice(s);
            sig_array[64..65].copy_from_slice(v);
        
            let recovered_pubkey = secp256k1_ecdsa_recover(&sig_array, &message_hash)
                .map_err(|_| Error::<T>::InvalidSignature)?;
        
            frame_support::log::info!("recovered_pubkey: {:?}", recovered_pubkey);
        
            let recovered_key_hash = keccak_256(&recovered_pubkey);
            let recovered_address = H160::from_slice(&recovered_key_hash[12..]);
        
            frame_support::log::info!("Recovered Ethereum Address: {:?}", recovered_address);
        
            ensure!(recovered_address == evm_address, Error::<T>::Unauthorized);
        
            let (account, _) = EvmPallet::<T>::account_basic(&evm_address);
            ensure!(account.balance >= amount, Error::<T>::InsufficientBalance);
        
            EvmPallet::<T>::mutate_balance(evm_address, amount, false);
        
            let substrate_amount = SubstrateBalanceOf::<T>::saturated_from(amount_u128);
        
            T::SubstrateCurrency::deposit_creating(&substrate_account, substrate_amount);
        
            Self::deposit_event(Event::Minted {
                who: substrate_account.clone(),
                amount: substrate_amount,
            });
            Self::deposit_event(Event::EvmToSubstrateTransfer(evm_address, substrate_account, amount_u128));
        
            Ok(())
        }
        
        

        
        
    }
}
