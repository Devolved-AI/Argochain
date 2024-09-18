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
    use frame_support::traits::ExistenceRequirement;





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
        TransferOfBalanceNew{ from: T::AccountId, to: T::AccountId, amount: SubstrateBalanceOf<T>, message: Vec<u8> },

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
        MessageTooLong,
        InvalidMessageContent,         
        SuspiciousContent,
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


        #[pallet::weight(5_000)]
        pub fn balance_transfer_new(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: SubstrateBalanceOf<T>,  
            message: Vec<u8>,               
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(core::str::from_utf8(&message).is_ok(), Error::<T>::InvalidMessageContent);
            ensure!(message.len() <= 64, Error::<T>::MessageTooLong);
            let message_str = core::str::from_utf8(&message).unwrap_or("");

            ensure!(
                message.iter().all(|&byte| (byte >= 32 && byte <= 126)), 
                Error::<T>::InvalidMessageContent
            );
            let url_patterns = ["http://", "https://", "www.", ".com", ".net", ".org", ".xyz", ".io", ".gov", ".edu", ".mil", ".info"];
            ensure!(
                !url_patterns.iter().any(|&pattern| message_str.contains(pattern)),
                Error::<T>::SuspiciousContent
            );
            let blacklisted_words = ["scam", "fraud", "hack", "illegal", "phishing"];
            ensure!(
                !blacklisted_words.iter().any(|&word| message_str.to_lowercase().contains(word)),
                Error::<T>::SuspiciousContent
            );
            ensure!(
                !Self::contains_ip_address(message_str),
                Error::<T>::SuspiciousContent
            );

            T::SubstrateCurrency::transfer(&who, &to, amount, ExistenceRequirement::KeepAlive)?;

            Self::deposit_event(Event::TransferOfBalanceNew {
                from: who.clone(),
                to: to.clone(),
                amount,
                message: message.clone(),
            });

            Ok(())
        }

        
    }
    impl<T: Config> Pallet<T> {
        pub fn contains_ip_address(message: &str) -> bool {
            if Self::contains_ipv4_address(message) {
                return true;
            }    
            if Self::contains_ipv6_address(message) {
                return true;
            }
            false
        }
    
        pub fn contains_ipv4_address(message: &str) -> bool {
            let parts: Vec<&str> = message.split('.').collect();
            if parts.len() == 4 {
                for part in parts {
                    if let Ok(num) = part.parse::<u8>() {
                        if !(0..=255).contains(&num) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                return true;
            }
            false
        }
    
        pub fn contains_ipv6_address(message: &str) -> bool {
            let parts: Vec<&str> = message.split(':').collect();
            if parts.len() > 1 && parts.len() <= 8 {
                for part in parts {
                    if part.is_empty() {
                        continue;
                    }
                    if part.len() > 4 || part.chars().any(|c| !c.is_digit(16)) {
                        return false;
                    }
                }
                return true;
            }
            false
        }
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        use frame_support::{assert_ok, assert_noop};
        use sp_runtime::AccountId32;

        // Utility function to simulate balance transfer
        fn setup_accounts() -> (AccountId32, AccountId32) {
            let sender = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([2u8; 32]);
            (sender, recipient)
        }

        #[test]
        fn test_balance_transfer_new_valid_message() {
            let (sender, recipient) = setup_accounts();

            // A valid message
            let message = b"Hello, this is a valid message!".to_vec();

            // Simulate a successful transfer
            assert_ok!(Pallet::<Test>::balance_transfer_new(
                Origin::signed(sender.clone()),
                recipient.clone(),
                1000,   // Example amount
                message.clone(),
            ));

            // Check if event was emitted correctly
            System::assert_has_event(Event::TransferOfBalanceNew {
                from: sender.clone(),
                to: recipient.clone(),
                amount: 1000,
                message: message.clone(),
            }.into());
        }

        #[test]
        fn test_balance_transfer_new_message_too_long() {
            let (sender, recipient) = setup_accounts();

            // A message exceeding 64 bytes in length
            let message = vec![b'A'; 65];

            // Expect an error due to message length
            assert_noop!(
                Pallet::<Test>::balance_transfer_new(
                    Origin::signed(sender),
                    recipient,
                    1000,
                    message,
                ),
                Error::<Test>::MessageTooLong
            );
        }

        #[test]
        fn test_balance_transfer_new_invalid_message_characters() {
            let (sender, recipient) = setup_accounts();

            // A message containing invalid UTF-8 characters
            let message = vec![0xFF, 0xFE, 0xFD];

            // Expect an error due to invalid message content
            assert_noop!(
                Pallet::<Test>::balance_transfer_new(
                    Origin::signed(sender),
                    recipient,
                    1000,
                    message,
                ),
                Error::<Test>::InvalidMessageContent
            );
        }

        #[test]
        fn test_balance_transfer_new_suspicious_url_content() {
            let (sender, recipient) = setup_accounts();

            // A message containing a URL pattern
            let message = b"Check this out: http://malicious.com".to_vec();

            // Expect an error due to suspicious content
            assert_noop!(
                Pallet::<Test>::balance_transfer_new(
                    Origin::signed(sender),
                    recipient,
                    1000,
                    message,
                ),
                Error::<Test>::SuspiciousContent
            );
        }

        #[test]
        fn test_balance_transfer_new_blacklisted_words() {
            let (sender, recipient) = setup_accounts();

            // A message containing a blacklisted word
            let message = b"This is a scam!".to_vec();

            // Expect an error due to suspicious content
            assert_noop!(
                Pallet::<Test>::balance_transfer_new(
                    Origin::signed(sender),
                    recipient,
                    1000,
                    message,
                ),
                Error::<Test>::SuspiciousContent
            );
        }

        #[test]
        fn test_balance_transfer_new_contains_ip_address() {
            let (sender, recipient) = setup_accounts();

            // A message containing an IP address
            let message = b"Contact me at 192.168.0.1".to_vec();

            // Expect an error due to suspicious content
            assert_noop!(
                Pallet::<Test>::balance_transfer_new(
                    Origin::signed(sender),
                    recipient,
                    1000,
                    message,
                ),
                Error::<Test>::SuspiciousContent
            );
        }
    }

    
}
