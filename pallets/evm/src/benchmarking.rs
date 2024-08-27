#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::traits::Bounded;
use sp_runtime::U256;
const SEED: u32 = 0;

benchmarks! {
    mint_evm_tokens {
        let caller: T::AccountId = account("caller", 0, SEED);
        let evm_address: H160 = H160::repeat_byte(1);
        let amount: U256 = U256::from(1000);

        LastMintingBlock::<T>::put(T::BlockNumber::min_value());
    }: _(RawOrigin::Root, evm_address, amount)
    verify {
        assert!(LastMintingBlock::<T>::get() > T::BlockNumber::min_value());
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
