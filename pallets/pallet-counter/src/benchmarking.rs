#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;

benchmarks! {
    mint {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 100u32.into();
    }: _(RawOrigin::Root, caller.clone(), amount)
    verify {
        assert_eq!(T::Currency::free_balance(&caller), amount);
    }

    burn {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 100u32.into();
        T::Currency::deposit_creating(&caller, amount);
    }: _(RawOrigin::Root, caller.clone(), amount)
    verify {
        assert_eq!(T::Currency::free_balance(&caller), 0u32.into());
    }

    lock {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 100u32.into();
        T::Currency::deposit_creating(&caller, amount);
    }: _(RawOrigin::Signed(caller.clone()), amount)
    verify {
        assert_eq!(T::Currency::reserved_balance(&caller), amount);
        assert_eq!(LockedBalance::<T>::get(&caller), amount);
    }

    unlock {
        let caller: T::AccountId = whitelisted_caller();
        let amount: BalanceOf<T> = 100u32.into();
        T::Currency::reserve(&caller, amount)?;
        <LockedBalance<T>>::insert(&caller, amount);
    }: _(RawOrigin::Signed(caller.clone()), amount)
    verify {
        assert_eq!(T::Currency::reserved_balance(&caller), 0u32.into());
        assert_eq!(LockedBalance::<T>::get(&caller), 0u32.into());
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
