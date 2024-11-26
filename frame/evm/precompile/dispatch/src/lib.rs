#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_crate_dependencies)]

extern crate alloc;

use alloc::format;
use core::marker::PhantomData;

use fp_evm::{
    ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use parity_scale_codec::{Decode, DecodeLimit as _};
use frame_support::{
    dispatch::{DispatchClass, GetDispatchInfo, Pays, PostDispatchInfo},
    traits::{ConstU32, Get},
};
use sp_runtime::traits::Dispatchable;
use pallet_evm::{AddressMapping, GasWeightMapping};

// Limit for decoding call depth, defaulting to 8 for safety.
pub struct Dispatch<T, DispatchValidator = (), DecodeLimit = ConstU32<8>> {
    _marker: PhantomData<(T, DispatchValidator, DecodeLimit)>,
}

impl<T, DispatchValidator, DecodeLimit> Precompile for Dispatch<T, DispatchValidator, DecodeLimit>
where
    T: pallet_evm::Config,
    T::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
    <T::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<T::AccountId>>,
    DispatchValidator: DispatchValidateT<T::AccountId, T::RuntimeCall>,
    DecodeLimit: Get<u32>,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let input = handle.input();
        let target_gas = handle.gas_limit();
        let context = handle.context();

        let call = T::RuntimeCall::decode_with_depth_limit(DecodeLimit::get(), &mut &*input)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("decode failed".into()),
            })?;
        let info = call.get_dispatch_info();

        if let Some(gas) = target_gas {
            let valid_weight =
                info.weight.ref_time() <= T::GasWeightMapping::gas_to_weight(gas, false).ref_time();
            if !valid_weight {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::OutOfGas,
                });
            }
        }

        let origin = T::AddressMapping::into_account_id(context.caller);

        if let Some(err) = DispatchValidator::validate_before_dispatch(&origin, &call) {
            return Err(err);
        }

        handle.record_external_cost(
            Some(info.weight.ref_time()),
            Some(info.weight.proof_size()),
        )?;

        match call.dispatch(Some(origin).into()) {
            Ok(post_info) => {
                if post_info.pays_fee(&info) == Pays::Yes {
                    let actual_weight = post_info.actual_weight.unwrap_or(info.weight);
                    let cost = T::GasWeightMapping::weight_to_gas(actual_weight);
                    handle.record_cost(cost)?;

                    handle.refund_external_cost(
                        Some(
                            info.weight
                                .ref_time()
                                .saturating_sub(actual_weight.ref_time()),
                        ),
                        Some(
                            info.weight
                                .proof_size()
                                .saturating_sub(actual_weight.proof_size()),
                        ),
                    );
                }

                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Stopped,
                    output: Default::default(),
                })
            }
            Err(e) => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other(
                    format!("dispatch execution failed: {}", <&'static str>::from(e)).into(),
                ),
            }),
        }
    }
}

/// Dispatch validation trait.
pub trait DispatchValidateT<AccountId, RuntimeCall> {
    fn validate_before_dispatch(
        origin: &AccountId,
        call: &RuntimeCall,
    ) -> Option<PrecompileFailure>;
}

/// Default implementation of `DispatchValidateT`.
impl<AccountId, RuntimeCall> DispatchValidateT<AccountId, RuntimeCall> for ()
where
    RuntimeCall: GetDispatchInfo,
{
    fn validate_before_dispatch(
        _origin: &AccountId,
        call: &RuntimeCall,
    ) -> Option<PrecompileFailure> {
        let info = call.get_dispatch_info();
        if !(info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal) {
            return Some(PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid call".into()),
            });
        }
        None
    }
}
