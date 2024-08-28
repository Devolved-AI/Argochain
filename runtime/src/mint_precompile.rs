use fp_evm::LinearCostPrecompile; 

use fp_evm::{
    PrecompileFailure, PrecompileHandle, PrecompileOutput, ExitRevert, ExitSucceed,
};
use sp_core::{H160, U256};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{marker::PhantomData, vec::Vec};
use frame_support::traits::Currency;

pub struct MintPrecompile<R>(PhantomData<R>);

impl<R> LinearCostPrecompile for MintPrecompile<R>
where
    R: pallet_evm::Config + pallet_balances::Config,
    R::AccountId: From<H160>,
    R::Balance: TryFrom<U256> + Zero,
    <R::Balance as TryFrom<U256>>::Error: core::fmt::Debug,
{
    const BASE: u64 = 10;
    const WORD: u64 = 1;

    fn execute(
        handle: &mut impl PrecompileHandle,
    ) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let input = handle.input();

        if input.len() != 52 {
            return Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"Invalid input length".to_vec(),
            });
        }

        let evm_address = H160::from_slice(&input[0..20]);
        let amount_to_mint = U256::from_big_endian(&input[20..52]);

        let account_id: R::AccountId = evm_address.into();

        let amount_to_mint_balance: R::Balance = amount_to_mint.try_into().map_err(|_| PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"Failed to convert U256 to Balance".to_vec(),
        })?;

        let current_balance: R::Balance = R::Currency::free_balance(&account_id);

        let new_balance = current_balance.saturating_add(amount_to_mint_balance);

        R::Currency::deposit_creating(&account_id, new_balance);

        Ok((ExitSucceed::Returned, Vec::new()))
    }
}
