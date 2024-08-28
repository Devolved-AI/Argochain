use pallet_evm::{
    IsPrecompileResult, Precompile, PrecompileHandle, PrecompileOutput, PrecompileResult, PrecompileSet, Pallet,
};
use sp_core::{H160, U256};
use sp_std::{marker::PhantomData, vec::Vec};

use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn used_addresses() -> [H160; 8] { // Updated size to 8
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(1024),
            hash(1025),
            hash(2048), // Address for the new balance mutation precompile
        ]
    }
}

impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles:
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles:
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            // Custom balance mutation precompile:
            a if a == hash(2048) => Some(BalanceMutationPrecompile::<R>::execute(handle)),
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

// Define the BalanceMutationPrecompile
pub struct BalanceMutationPrecompile<R>(PhantomData<R>);

impl<R> BalanceMutationPrecompile<R>
where
    R: pallet_evm::Config,
{
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let input = handle.input();

        if input.len() != 53 {
            return Err(pallet_evm::PrecompileFailure::Revert {
                exit_status: pallet_evm::ExitRevert::Reverted,
                output: b"Invalid input length".to_vec(),
            });
        }

        let evm_address = H160::from_slice(&input[0..20]);
        let amount = U256::from_big_endian(&input[20..52]);
        let add_flag = input[52] == 1;

        Pallet::<R>::mutate_balance(evm_address, amount, add_flag);

        Ok(PrecompileOutput {
            exit_status: pallet_evm::ExitSucceed::Returned,
            output: Vec::new(),
        })
    }
}
