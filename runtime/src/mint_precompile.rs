use fp_evm::{
    LinearCostPrecompile, PrecompileFailure, PrecompileHandle, ExitRevert, ExitSucceed,
};
use sp_core::{H160, U256};
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{marker::PhantomData, vec::Vec};
use frame_support::traits::Currency;

pub struct MintPrecompile<R>(PhantomData<R>);

impl<R> MintPrecompile<R>
where
    R: pallet_evm::Config + pallet_balances::Config,
    R::AccountId: From<H160>,
    R::Balance: TryFrom<U256> + Zero,
    <R::Balance as TryFrom<U256>>::Error: core::fmt::Debug,
{
    fn parse_input(input: &[u8]) -> Result<(H160, U256), PrecompileFailure> {
        if input.len() != 52 {
            return Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: b"length not matched".to_vec(),
            });
        }

        let evm_address = H160::from_slice(&input[0..20]);
        let amount_to_mint = U256::from_big_endian(&input[20..52]);

        Ok((evm_address, amount_to_mint))
    }

    fn convert_to_balance(amount: U256) -> Result<R::Balance, PrecompileFailure> {
        amount.try_into().map_err(|_| PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"Failed to convert U256 to Balance".to_vec(),
        })
    }

    fn mint_to_account(account_id: &R::AccountId, amount: R::Balance) {
        R::Currency::deposit_creating(account_id, amount);
    }
}

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

        let (evm_address, amount_to_mint) = Self::parse_input(input)?;

        let account_id: R::AccountId = evm_address.into();

        let amount_to_mint_balance = Self::convert_to_balance(amount_to_mint)?;

        Self::mint_to_account(&account_id, amount_to_mint_balance);

        Ok((ExitSucceed::Returned, Vec::new()))
    }
}
