//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

benchmarks! {
    do_something {
        let value = 100u32.into();
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), value)
    verify {
        assert_eq!(Something::<T>::get(), Some(value));
    }

    cause_error {
        Something::<T>::put(100u32);
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
    verify {
        assert_eq!(Something::<T>::get(), Some(101u32));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            do_something::<Test>();
            cause_error::<Test>();
        });
    }
}
