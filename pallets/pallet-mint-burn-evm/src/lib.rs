// #![cfg_attr(not(feature = "std"), no_std)]

// pub use self::pallet::*;

// #[frame_support::pallet]
// pub mod pallet {
//     use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
//     use frame_system::pallet_prelude::*;
//     use frame_support::traits::{Currency, WithdrawReasons, ExistenceRequirement};
//     use sp_core::{H160, U256};
//     use sp_runtime::traits::SaturatedConversion;

//     pub trait EvmInterface {
//         fn get_evm_balance(evm_address: H160) -> Option<U256>;
//         fn set_evm_balance(evm_address: H160, balance: U256) -> DispatchResult;
//     }

//     #[pallet::pallet]
//     pub struct Pallet<T>(_);

//     #[pallet::config]
//     pub trait Config: frame_system::Config {
//         type Currency: Currency<Self::AccountId>;
//         type EvmInterface: EvmInterface;
//         type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
//     }

//     #[pallet::call]
//     impl<T: Config> Pallet<T> {
//         #[pallet::call_index(0)]
//         #[pallet::weight(10_000)]
//         pub fn substrate_to_evm(
//             origin: OriginFor<T>,
//             evm_address: H160,
//             amount: BalanceOf<T>,
//         ) -> DispatchResult {
//             let who = ensure_signed(origin)?;

//             T::Currency::withdraw(
//                 &who,
//                 amount,
//                 WithdrawReasons::TRANSFER,
//                 ExistenceRequirement::KeepAlive,
//             )?;

//             let evm_amount: U256 = U256::from(amount.saturated_into::<u128>());

//             let current_balance = T::EvmInterface::get_evm_balance(evm_address).unwrap_or(U256::zero());
//             let new_balance = current_balance.saturating_add(evm_amount);
//             T::EvmInterface::set_evm_balance(evm_address, new_balance)?;

//             Ok(())
//         }

//         #[pallet::call_index(1)]
//         #[pallet::weight(10_000)]
//         pub fn evm_to_substrate(
//             origin: OriginFor<T>,
//             evm_address: H160,
//             substrate_account: T::AccountId,
//             amount: U256,
//         ) -> DispatchResult {
//             let who = ensure_signed(origin)?;

//             let current_balance = T::EvmInterface::get_evm_balance(evm_address)
//                 .ok_or(Error::<T>::AccountNotFound)?;

//             ensure!(current_balance >= amount, Error::<T>::InsufficientBalance);

//             let new_balance = current_balance.saturating_sub(amount);
//             T::EvmInterface::set_evm_balance(evm_address, new_balance)?;

//             let substrate_amount: BalanceOf<T> = TryInto::<u128>::try_into(amount)
//                 .map_err(|_| Error::<T>::ConversionError)?
//                 .saturated_into();
//             T::Currency::deposit_creating(&substrate_account, substrate_amount);

//             Ok(())
//         }
//     }

//     #[pallet::error]
//     pub enum Error<T> {
//         AccountNotFound,
//         InsufficientBalance,
//         ConversionError,
//     }

//     #[pallet::event]
//     #[pallet::generate_deposit(pub(super) fn deposit_event)]
//     pub enum Event<T: Config> {}

//     pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
// }
#![cfg_attr(not(feature = "std"), no_std)]

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency, WithdrawReasons, ExistenceRequirement};
    use sp_core::{H160, U256};
    use sp_runtime::traits::SaturatedConversion;

    pub trait EvmInterface<AccountId> {
        fn get_evm_balance(evm_address: H160) -> Option<U256>;
        fn set_evm_balance(evm_address: H160, balance: U256) -> DispatchResult;
    }
    

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;
        type EvmInterface: EvmInterface<Self::AccountId>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }
    

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn substrate_to_evm(
            origin: OriginFor<T>,
            evm_address: H160,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::Currency::withdraw(
                &who,
                amount,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive,
            )?;

            let evm_amount: U256 = U256::from(amount.saturated_into::<u128>());

            let current_balance = T::EvmInterface::get_evm_balance(evm_address).unwrap_or(U256::zero());
            let new_balance = current_balance.saturating_add(evm_amount);
            T::EvmInterface::set_evm_balance(evm_address, new_balance)?;

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn evm_to_substrate(
            origin: OriginFor<T>,
            evm_address: H160,
            substrate_account: T::AccountId,
            amount: U256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_balance = T::EvmInterface::get_evm_balance(evm_address)
                .ok_or(Error::<T>::AccountNotFound)?;

            ensure!(current_balance >= amount, Error::<T>::InsufficientBalance);

            let new_balance = current_balance.saturating_sub(amount);
            T::EvmInterface::set_evm_balance(evm_address, new_balance)?;

            let substrate_amount: BalanceOf<T> = TryInto::<u128>::try_into(amount)
                .map_err(|_| Error::<T>::ConversionError)?
                .saturated_into();
            T::Currency::deposit_creating(&substrate_account, substrate_amount);

            Ok(())
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        AccountNotFound,
        InsufficientBalance,
        ConversionError,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
}
