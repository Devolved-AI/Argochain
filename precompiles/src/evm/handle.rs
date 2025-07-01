// This file is part of Frontier.

// Copyright (c) Moonsong Labs.
// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
	solidity::{
		codec::Reader,
		modifier::FunctionModifier,
		revert::{MayRevert, RevertReason},
	},
	EvmResult,
};
use fp_evm::{Log, PrecompileHandle};

pub trait PrecompileHandleExt: PrecompileHandle {
	/// Record cost of one DB read manually.
	/// The max encoded length of the data that will be read should be provided.
	fn record_db_read<Runtime: pallet_evm::Config>(
		&mut self,
		data_max_encoded_len: usize,
	) -> Result<(), evm::ExitError>;

	/// Record cost of a log manually.
	/// This can be useful to record log costs early when their content have static size.
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult;

	/// Record cost of logs.
	fn record_log_costs(&mut self, logs: &[&Log]) -> EvmResult;

	/// Check that a function call is compatible with the context it is
	/// called into.
	fn check_function_modifier(&self, modifier: FunctionModifier) -> MayRevert;

	/// Read the selector from the input data.
	fn read_u32_selector(&self) -> MayRevert<u32>;

	/// Returns a reader of the input, skipping the selector.
	fn read_after_selector(&self) -> MayRevert<Reader>;
}

impl<T: PrecompileHandle> PrecompileHandleExt for T {
	fn record_db_read<Runtime: pallet_evm::Config>(
		&mut self,
		data_max_encoded_len: usize,
	) -> Result<(), evm::ExitError> {
		self.record_cost(crate::prelude::RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		// TODO: record ref time when precompile will be benchmarked
		self.record_external_cost(None, Some(data_max_encoded_len as u64), None)
	}

	/// Record cost of a log manually.
	/// This can be useful to record log costs early when their content have static size.
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult {
		self.record_cost(crate::evm::costs::log_costs(topics, data_len)?)?;

		Ok(())
	}

	/// Record cost of logs.
	fn record_log_costs(&mut self, logs: &[&Log]) -> EvmResult {
		for log in logs {
			self.record_log_costs_manual(log.topics.len(), log.data.len())?;
		}

		Ok(())
	}

	/// Check that a function call is compatible with the context it is
	/// called into.
	fn check_function_modifier(&self, modifier: FunctionModifier) -> MayRevert {
		crate::solidity::modifier::check_function_modifier(
			self.context(),
			self.is_static(),
			modifier,
		)
	}

	/// Read the selector from the input data as u32.
	fn read_u32_selector(&self) -> MayRevert<u32> {
		crate::solidity::codec::selector(self.input())
			.ok_or(RevertReason::read_out_of_bounds("selector").into())
	}

	/// Returns a reader of the input, skipping the selector.
	fn read_after_selector(&self) -> MayRevert<Reader> {
		Reader::new_skip_selector(self.input())
	}
}

environmental::environmental!(EVM_CONTEXT: trait PrecompileHandle);

/// A wrapper that holds a reference to a PrecompileHandle with proper lifetime management
struct PrecompileHandleWrapper<'a> {
	handle: &'a mut dyn PrecompileHandle,
}

impl<'a> PrecompileHandle for PrecompileHandleWrapper<'a> {
	fn call(
		&mut self,
		address: sp_core::H160,
		transfer: Option<evm::Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: &evm::Context,
	) -> (evm::ExitReason, Vec<u8>) {
		self.handle.call(address, transfer, input, target_gas, is_static, context)
	}

	fn record_cost(&mut self, cost: u64) -> Result<(), evm::ExitError> {
		self.handle.record_cost(cost)
	}

	fn remaining_gas(&self) -> u64 {
		self.handle.remaining_gas()
	}

	fn log(
		&mut self,
		address: sp_core::H160,
		topics: Vec<sp_core::H256>,
		data: Vec<u8>,
	) -> Result<(), evm::ExitError> {
		self.handle.log(address, topics, data)
	}

	fn code_address(&self) -> sp_core::H160 {
		self.handle.code_address()
	}

	fn input(&self) -> &[u8] {
		self.handle.input()
	}

	fn context(&self) -> &evm::Context {
		self.handle.context()
	}

	fn is_static(&self) -> bool {
		self.handle.is_static()
	}

	fn gas_limit(&self) -> Option<u64> {
		self.handle.gas_limit()
	}

	fn record_external_cost(
		&mut self,
		ref_time: Option<u64>,
		proof_size: Option<u64>,
		storage_growth: Option<u64>,
	) -> Result<(), fp_evm::ExitError> {
		self.handle.record_external_cost(ref_time, proof_size, storage_growth)
	}

	fn refund_external_cost(&mut self, ref_time: Option<u64>, proof_size: Option<u64>) {
		self.handle.refund_external_cost(ref_time, proof_size)
	}
}

pub fn using_precompile_handle<'a, R, F: FnOnce() -> R>(
	precompile_handle: &'a mut dyn PrecompileHandle,
	mutator: F,
) -> R {
	// Create a wrapper that implements PrecompileHandle trait
	let mut wrapper = PrecompileHandleWrapper { handle: precompile_handle };
	
	// Use the environmental crate's safe using method with the wrapper
	// The wrapper ensures proper lifetime management without unsafe transmute
	EVM_CONTEXT::using(&mut wrapper, mutator)
}

pub fn with_precompile_handle<R, F: FnOnce(&mut dyn PrecompileHandle) -> R>(f: F) -> Option<R> {
	EVM_CONTEXT::with(|precompile_handle| f(precompile_handle))
}

#[cfg(test)]
mod tests {
	use super::*;

	struct MockPrecompileHandle;
	impl PrecompileHandle for MockPrecompileHandle {
		fn call(
			&mut self,
			_: sp_core::H160,
			_: Option<evm::Transfer>,
			_: Vec<u8>,
			_: Option<u64>,
			_: bool,
			_: &evm::Context,
		) -> (evm::ExitReason, Vec<u8>) {
			unimplemented!()
		}

		fn record_cost(&mut self, _: u64) -> Result<(), evm::ExitError> {
			unimplemented!()
		}

		fn remaining_gas(&self) -> u64 {
			unimplemented!()
		}

		fn log(
			&mut self,
			_: sp_core::H160,
			_: Vec<sp_core::H256>,
			_: Vec<u8>,
		) -> Result<(), evm::ExitError> {
			unimplemented!()
		}

		fn code_address(&self) -> sp_core::H160 {
			unimplemented!()
		}

		fn input(&self) -> &[u8] {
			unimplemented!()
		}

		fn context(&self) -> &evm::Context {
			unimplemented!()
		}

		fn is_static(&self) -> bool {
			true
		}

		fn gas_limit(&self) -> Option<u64> {
			unimplemented!()
		}

		fn record_external_cost(
			&mut self,
			_ref_time: Option<u64>,
			_proof_size: Option<u64>,
			_storage_growth: Option<u64>,
		) -> Result<(), fp_evm::ExitError> {
			Ok(())
		}

		fn refund_external_cost(&mut self, _ref_time: Option<u64>, _proof_size: Option<u64>) {}
	}

	#[test]
	fn with_precompile_handle_without_context() {
		assert_eq!(with_precompile_handle(|_| {}), None);
	}

	#[test]
	fn with_precompile_handle_with_context() {
		let mut precompile_handle = MockPrecompileHandle;

		assert_eq!(
			using_precompile_handle(&mut precompile_handle, || with_precompile_handle(
				|handle| handle.is_static()
			)),
			Some(true)
		);
	}
}
