#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::dispatch::DispatchResult;
use frame_system::ensure_signed;
use sp_runtime::traits::Zero;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn locked_balance)]
    pub type LockedBalance<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Minted { who: T::AccountId, amount: BalanceOf<T> },
        Burned { who: T::AccountId, amount: BalanceOf<T> },
        Locked { who: T::AccountId, amount: BalanceOf<T> },
        Unlocked { who: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        LockNotFound,
        UnlockNotPossible,
        Unauthorized,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(origin: OriginFor<T>, account: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;

            T::Currency::deposit_creating(&account, amount);
            Self::deposit_event(Event::Minted { who: account, amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn burn(origin: OriginFor<T>, account: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;

            T::Currency::withdraw(
                &account,
                amount,
                frame_support::traits::WithdrawReasons::TRANSFER,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            Self::deposit_event(Event::Burned { who: account, amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn lock(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::Currency::reserve(&who, amount)?;
            <LockedBalance<T>>::insert(&who, amount);
            Self::deposit_event(Event::Locked { who: who.clone(), amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn unlock(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let locked_amount = <LockedBalance<T>>::get(&who);
            ensure!(locked_amount >= amount, Error::<T>::UnlockNotPossible);

            T::Currency::unreserve(&who, amount);
            <LockedBalance<T>>::mutate(&who, |balance| *balance -= amount);
            Self::deposit_event(Event::Unlocked { who: who.clone(), amount });
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_to_evm_account(
            origin: OriginFor<T>,
            evm_account: H160,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            // Ensure the Substrate account has enough balance
            ensure!(<Balances<T>>::free_balance(&who) >= amount, "Insufficient balance");
        
            // Deduct the amount from the Substrate account
            <Balances<T>>::mutate(&who, |balance| *balance -= amount);
        
            // Add the amount to the EVM account
            <EVMAccounts<T>>::mutate(&evm_account, |balance| *balance += amount);
        
            // Emit an event for the transfer
            Self::deposit_event(Event::TransferredToEvmAccount(who, evm_account, amount));
            Ok(())
        }

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