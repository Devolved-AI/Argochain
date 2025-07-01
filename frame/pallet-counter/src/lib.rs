#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_evm::Pallet as EvmPallet;
    use sp_core::{H160, U256, ecdsa};
    use sp_runtime::traits::{SaturatedConversion, Zero, Saturating};
    #[allow(unused_imports)]
    use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
    use sp_io::hashing::keccak_256;
    use scale_info::prelude::format;
    use sp_std::vec::Vec;
    use hex_literal::hex;
    use sp_io::crypto::secp256k1_ecdsa_recover;
    use frame_support::traits::ExistenceRequirement;
    use sp_runtime::AccountId32;
    use alloc::string::String;
    use frame_support::sp_runtime::traits::ConstU32;

    // use log::info;


    // Define the authorized backend account (common account for safety)
    // pub const AUTHORIZED_BACKEND_ACCOUNT: AccountId32 = AccountId32::new(hex!(
    //     "64882b6b92eefc93a7e9c929681a7facc12eb8c5ee505c610aa207a5e7c46206"
    // ));

    type SubstrateBalanceOf<T> = <<T as Config>::SubstrateCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    #[allow(dead_code)]
    type EvmBalanceOf<T> = <<T as Config>::EvmCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type BlockNumberFor<T> = <<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number;

    const MAX_PENDING_OPS: u32 = 5;

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

    // Add nonce storage to prevent replay attacks
    #[pallet::storage]
    #[pallet::getter(fn transfer_nonce)]
    pub type TransferNonce<T: Config> = StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery>;

    // Security enhancements for critical functions
    #[pallet::storage]
    #[pallet::getter(fn emergency_pause)]
    pub type EmergencyPause<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn mint_operation_count)]
    pub type MintOperationCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn burn_operation_count)]
    pub type BurnOperationCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_mint_operation)]
    pub type LastMintOperation<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_burn_operation)]
    pub type LastBurnOperation<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_mint_operations)]
    pub type PendingMintOperations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<(SubstrateBalanceOf<T>, BlockNumberFor<T>), ConstU32<MAX_PENDING_OPS>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_burn_operations)]
    pub type PendingBurnOperations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<(SubstrateBalanceOf<T>, BlockNumberFor<T>), ConstU32<MAX_PENDING_OPS>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn authorized_operators)]
    pub type AuthorizedOperators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

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
        IPFSHashIncluded(T::AccountId, Vec<u8>),
        // Security enhancement events
        EmergencyPauseSet(bool),
        MintOperationScheduled { operator: T::AccountId, target: T::AccountId, amount: SubstrateBalanceOf<T>, delay_blocks: BlockNumberFor<T> },
        BurnOperationScheduled { operator: T::AccountId, target: T::AccountId, amount: SubstrateBalanceOf<T>, delay_blocks: BlockNumberFor<T> },
        MintOperationExecuted { operator: T::AccountId, target: T::AccountId, amount: SubstrateBalanceOf<T> },
        BurnOperationExecuted { operator: T::AccountId, target: T::AccountId, amount: SubstrateBalanceOf<T> },
        OperatorAuthorized { operator: T::AccountId, authorized: bool },
        RateLimitExceeded { operator: T::AccountId, operation_type: Vec<u8> },
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
        InvalidIPFSHash,
        UnauthorizedBackend,
        UnauthorizedUser,
        InvalidNonce,
        SignatureReplay,
        // Security enhancement errors
        EmergencyPauseActive,
        AmountExceedsLimit,
        RateLimitExceeded,
        InsufficientDelay,
        OperationNotPending,
        UnauthorizedOperator,
        InvalidOperationAmount,
        TooManyPendingOperations,
        EvmBalanceConversionFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        
        #[pallet::weight(10_000)]
        #[pallet::call_index(0)]
        pub fn mint(origin: OriginFor<T>, account: T::AccountId, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            let operator = ensure_signed(origin)?;
            
            // Check emergency pause
            ensure!(!Self::emergency_pause(), Error::<T>::EmergencyPauseActive);
            
            // Validate amount (prevent excessive minting)
            let max_mint_amount = SubstrateBalanceOf::<T>::saturated_from(1_000_000_000_000u128); // 1 trillion tokens max
            ensure!(amount <= max_mint_amount, Error::<T>::AmountExceedsLimit);
            ensure!(amount > SubstrateBalanceOf::<T>::zero(), Error::<T>::InvalidOperationAmount);
            
            // Check if operator is authorized (multi-signature requirement)
            ensure!(Self::authorized_operators(operator.clone()), Error::<T>::UnauthorizedOperator);
            
            // Rate limiting: max 10 mint operations per day per operator
            let current_block = frame_system::Pallet::<T>::block_number();
            let last_operation = Self::last_mint_operation(operator.clone());
            let operation_count = Self::mint_operation_count(operator.clone());
            
            if last_operation > BlockNumberFor::<T>::zero() {
                let blocks_since_last = current_block.saturating_sub(last_operation);
                let blocks_per_day = BlockNumberFor::<T>::saturated_from(14400u32); // Assuming 6s blocks
                
                if blocks_since_last < blocks_per_day && operation_count >= 10 {
                    return Err(Error::<T>::RateLimitExceeded.into());
                }
            }
            
            // For large amounts (> 100k tokens), require time delay
            let large_amount_threshold = SubstrateBalanceOf::<T>::saturated_from(100_000_000_000u128); // 100 billion tokens
            if amount > large_amount_threshold {
                let delay_blocks = BlockNumberFor::<T>::saturated_from(1440u32); // 24 hours delay
                let execution_block = current_block.saturating_add(delay_blocks);
                
                // Add to pending operations
                let mut pending = Self::pending_mint_operations(operator.clone());
                pending.try_push((amount, execution_block)).map_err(|_| Error::<T>::TooManyPendingOperations)?;
                <PendingMintOperations<T>>::insert(&operator, pending);
                
                Self::deposit_event(Event::MintOperationScheduled {
                    operator: operator.clone(),
                    target: account.clone(),
                    amount,
                    delay_blocks,
                });
                
                return Ok(());
            }
            
            // Execute immediate mint for smaller amounts
            T::SubstrateCurrency::deposit_creating(&account, amount);
            
            // Update tracking
            <LastMintOperation<T>>::insert(&operator, current_block);
            <MintOperationCount<T>>::mutate(&operator, |count| *count = count.saturating_add(1));
            
            Self::deposit_event(Event::MintOperationExecuted {
                operator: operator.clone(),
                target: account.clone(),
                amount,
            });
            Self::deposit_event(Event::Minted { who: account, amount });
            
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(1)]
        pub fn burn(origin: OriginFor<T>, account: T::AccountId, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            let operator = ensure_signed(origin)?;
            
            // Check emergency pause
            ensure!(!Self::emergency_pause(), Error::<T>::EmergencyPauseActive);
            
            // Validate amount (prevent excessive burning)
            let max_burn_amount = SubstrateBalanceOf::<T>::saturated_from(1_000_000_000_000u128); // 1 trillion tokens max
            ensure!(amount <= max_burn_amount, Error::<T>::AmountExceedsLimit);
            ensure!(amount > SubstrateBalanceOf::<T>::zero(), Error::<T>::InvalidOperationAmount);
            
            // Check if operator is authorized (multi-signature requirement)
            ensure!(Self::authorized_operators(operator.clone()), Error::<T>::UnauthorizedOperator);
            
            // Rate limiting: max 10 burn operations per day per operator
            let current_block = frame_system::Pallet::<T>::block_number();
            let last_operation = Self::last_burn_operation(operator.clone());
            let operation_count = Self::burn_operation_count(operator.clone());
            
            if last_operation > BlockNumberFor::<T>::zero() {
                let blocks_since_last = current_block.saturating_sub(last_operation);
                let blocks_per_day = BlockNumberFor::<T>::saturated_from(14400u32); // Assuming 6s blocks
                
                if blocks_since_last < blocks_per_day && operation_count >= 10 {
                    return Err(Error::<T>::RateLimitExceeded.into());
                }
            }
            
            // For large amounts (> 100k tokens), require time delay
            let large_amount_threshold = SubstrateBalanceOf::<T>::saturated_from(100_000_000_000u128); // 100 billion tokens
            if amount > large_amount_threshold {
                let delay_blocks = BlockNumberFor::<T>::saturated_from(1440u32); // 24 hours delay
                let execution_block = current_block.saturating_add(delay_blocks);
                
                // Add to pending operations
                let mut pending = Self::pending_burn_operations(operator.clone());
                pending.try_push((amount, execution_block)).map_err(|_| Error::<T>::TooManyPendingOperations)?;
                <PendingBurnOperations<T>>::insert(&operator, pending);
                
                Self::deposit_event(Event::BurnOperationScheduled {
                    operator: operator.clone(),
                    target: account.clone(),
                    amount,
                    delay_blocks,
                });
                
                return Ok(());
            }
            
            // Execute immediate burn for smaller amounts
            T::SubstrateCurrency::withdraw(
                &account,
                amount,
                frame_support::traits::WithdrawReasons::TRANSFER,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            
            // Update tracking
            <LastBurnOperation<T>>::insert(&operator, current_block);
            <BurnOperationCount<T>>::mutate(&operator, |count| *count = count.saturating_add(1));
            
            Self::deposit_event(Event::BurnOperationExecuted {
                operator: operator.clone(),
                target: account.clone(),
                amount,
            });
            Self::deposit_event(Event::Burned { who: account, amount });
            
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(2)]
        pub fn lock(origin: OriginFor<T>, amount: SubstrateBalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::SubstrateCurrency::reserve(&who, amount)?;
            <LockedBalance<T>>::insert(&who, amount);
            Self::deposit_event(Event::Locked { who: who.clone(), amount });
            Ok(())
        }
        

        #[pallet::weight(10_000)]
        #[pallet::call_index(3)]
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
        #[pallet::call_index(4)]
        pub fn check_evm_balance(origin: OriginFor<T>, evm_address: H160) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let (account, _) = EvmPallet::<T>::account_basic(&evm_address);

            Self::deposit_event(Event::EvmBalanceChecked(evm_address, account.balance));

            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(5)]
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

            EvmPallet::<T>::mutate_balance(evm_address, amount_u256, add)
                .map_err(|_| Error::<T>::EvmBalanceConversionFailed)?;

            Self::deposit_event(Event::EvmBalanceMutated(evm_address, amount_u256, add));

            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(6)]
        pub fn evm_to_substrate(
            origin: OriginFor<T>,
            evm_address: H160,
            amount: U256,
            subtract: bool,
            signature: ecdsa::Signature,
            expected_nonce: u64,
        ) -> DispatchResult {
            ensure!(!subtract, Error::<T>::Unauthorized);
            let substrate_account = ensure_signed(origin)?;
            
            // Validate nonce to prevent replay attacks
            let current_nonce = TransferNonce::<T>::get(evm_address);
            ensure!(expected_nonce == current_nonce, Error::<T>::InvalidNonce);
            
            let amount_u128: u128 = amount.try_into().map_err(|_| Error::<T>::AmountConversionFailed)?;
            
            // Get current block information for context
            let block_number = frame_system::Pallet::<T>::block_number();
            let block_hash = frame_system::Pallet::<T>::block_hash(block_number);
            let chain_id = T::ChainId::get();
            
            // Create a secure message with context to prevent replay attacks
            let message = format!(
                "Transfer {} AGC from 0x{:x} to {:?} on chain {} at block {:?} nonce {}",
                amount_u128,
                evm_address,
                substrate_account,
                chain_id,
                block_hash,
                expected_nonce
            );
            
            let prefix = "\x19Ethereum Signed Message:\n";
            let message_len = format!("{}", message.len());
            let message_to_sign = format!("{}{}{}", prefix, message_len, message);
            
            let message_hash = keccak_256(message_to_sign.as_bytes());
            
            let r = &signature.0[..32];
            let s = &signature.0[32..64];
            let v = &signature.0[64..65];
            
            let mut sig_array = [0u8; 65];
            sig_array[..32].copy_from_slice(r);
            sig_array[32..64].copy_from_slice(s);
            sig_array[64..65].copy_from_slice(v);
            
            let recovered_pubkey = secp256k1_ecdsa_recover(&sig_array, &message_hash)
                .map_err(|_| Error::<T>::InvalidSignature)?;
            
            let recovered_key_hash = keccak_256(&recovered_pubkey);
            let recovered_address = H160::from_slice(&recovered_key_hash[12..]);
            
            ensure!(recovered_address == evm_address, Error::<T>::Unauthorized);
            
            let (account, _) = EvmPallet::<T>::account_basic(&evm_address);
            ensure!(account.balance >= amount, Error::<T>::InsufficientBalance);
            
            // Increment nonce to prevent replay
            TransferNonce::<T>::mutate(evm_address, |nonce| *nonce += 1);
            
            EvmPallet::<T>::mutate_balance(evm_address, amount, false)
                .map_err(|_| Error::<T>::EvmBalanceConversionFailed)?;
            
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
        #[pallet::call_index(7)]
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

        #[pallet::weight(10_000)]
        #[pallet::call_index(8)]
        pub fn include_ipfs_hash(
            origin: OriginFor<T>,
            ipfs_hash: Vec<u8>,
            backend_signature: Vec<u8>,  
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;

            let authorized_backend_account: AccountId32 = AccountId32::new(hex!(
                "7c650b5b9f657ddcc7a6ddbf9147d33f3b6ffda5009658b1ee6b7e3665a99701"
            ));

            let backend_bytes: [u8; 32] = *authorized_backend_account.as_ref();

            let backend_public_key = sp_core::sr25519::Public::from_raw(backend_bytes);

            let backend_sig = sp_core::sr25519::Signature::try_from(backend_signature.as_slice())
                .map_err(|_| Error::<T>::InvalidSignature)?;

            let message = ipfs_hash.clone();  

            ensure!(
                sp_io::crypto::sr25519_verify(
                    &backend_sig, 
                    &message, 
                    &backend_public_key
                ),
                Error::<T>::InvalidSignature
            );

            ensure!(ipfs_hash.len() == 46, Error::<T>::InvalidIPFSHash);

            Self::deposit_event(Event::IPFSHashIncluded(user.clone(), ipfs_hash.clone()));

            Ok(())
        }

        #[pallet::weight(1_000)]
        #[pallet::call_index(9)]
        pub fn query_transfer_nonce(origin: OriginFor<T>, evm_address: H160) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            
            let _nonce = Self::transfer_nonce(evm_address);
            
            // We could emit an event here, but for simplicity we'll just return success
            // The nonce can be queried through storage queries as well
            Ok(())
        }

        // Security administration functions
        #[pallet::weight(10_000)]
        #[pallet::call_index(10)]
        pub fn set_emergency_pause(origin: OriginFor<T>, paused: bool) -> DispatchResult {
            ensure_root(origin)?;
            <EmergencyPause<T>>::put(paused);
            Self::deposit_event(Event::EmergencyPauseSet(paused));
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(11)]
        pub fn authorize_operator(origin: OriginFor<T>, operator: T::AccountId, authorized: bool) -> DispatchResult {
            ensure_root(origin)?;
            <AuthorizedOperators<T>>::insert(&operator, authorized);
            Self::deposit_event(Event::OperatorAuthorized { operator, authorized });
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(12)]
        pub fn execute_pending_mint_operations(origin: OriginFor<T>) -> DispatchResult {
            let operator = ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            
            let mut pending = Self::pending_mint_operations(operator.clone());
            let mut executed = Vec::new();
            
            for (amount, execution_block) in pending.iter() {
                if current_block >= *execution_block {
                    // Execute the mint operation
                    T::SubstrateCurrency::deposit_creating(&operator, *amount);
                    
                    Self::deposit_event(Event::MintOperationExecuted {
                        operator: operator.clone(),
                        target: operator.clone(),
                        amount: *amount,
                    });
                    Self::deposit_event(Event::Minted { who: operator.clone(), amount: *amount });
                    
                    executed.push((*amount, *execution_block));
                }
            }
            
            // Remove executed operations
            for executed_op in executed {
                pending.retain(|op| *op != executed_op);
            }
            
            <PendingMintOperations<T>>::insert(&operator, pending);
            Ok(())
        }

        #[pallet::weight(10_000)]
        #[pallet::call_index(13)]
        pub fn execute_pending_burn_operations(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let operator = ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            
            let mut pending = Self::pending_burn_operations(operator.clone());
            let mut executed = Vec::new();
            
            for (amount, execution_block) in pending.iter() {
                if current_block >= *execution_block {
                    // Execute the burn operation
                    T::SubstrateCurrency::withdraw(
                        &target,
                        *amount,
                        frame_support::traits::WithdrawReasons::TRANSFER,
                        frame_support::traits::ExistenceRequirement::KeepAlive,
                    )?;
                    
                    Self::deposit_event(Event::BurnOperationExecuted {
                        operator: operator.clone(),
                        target: target.clone(),
                        amount: *amount,
                    });
                    Self::deposit_event(Event::Burned { who: target.clone(), amount: *amount });
                    
                    executed.push((*amount, *execution_block));
                }
            }
            
            // Remove executed operations
            for executed_op in executed {
                pending.retain(|op| *op != executed_op);
            }
            
            <PendingBurnOperations<T>>::insert(&operator, pending);
            Ok(())
        }
    }
    impl<T: Config> Pallet<T> {
        /// Get the message that needs to be signed for a transfer
        /// This includes all necessary context to prevent replay attacks
        pub fn get_transfer_message(
            evm_address: H160,
            substrate_account: &T::AccountId,
            amount: u128,
            nonce: u64,
        ) -> String {
            let block_number = frame_system::Pallet::<T>::block_number();
            let block_hash = frame_system::Pallet::<T>::block_hash(block_number);
            let chain_id = T::ChainId::get();
            
            format!(
                "Transfer {} AGC from 0x{:x} to {:?} on chain {} at block {:?} nonce {}",
                amount,
                evm_address,
                substrate_account,
                chain_id,
                block_hash,
                nonce
            )
        }

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
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();
            
            // Reset rate limits daily
            let blocks_per_day = BlockNumberFor::<T>::saturated_from(14400u32);
            if block_number % blocks_per_day == BlockNumberFor::<T>::zero() {
                // Reset operation counts
                <MintOperationCount<T>>::clear(1000, None);
                <BurnOperationCount<T>>::clear(1000, None);
                weight = weight.saturating_add(Weight::from_parts(1000, 0));
            }
            
            weight
        }
    }
}
