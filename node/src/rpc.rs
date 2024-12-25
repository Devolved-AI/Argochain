// This file is part of Substrate.

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

//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use jsonrpsee::RpcModule;
use argochain_runtime::AccountId;
use node_primitives::Nonce;
use sc_transaction_pool_api::TransactionPool;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use std::sync::Arc;
use sp_runtime::generic::Block as GenericBlock;
use sp_runtime::OpaqueExtrinsic;
use sp_runtime::traits::BlakeTwo256;
use substrate_frame_rpc_system::{System, SystemApiServer};


pub use sc_rpc_api::DenyUnsafe;

type Header = sp_runtime::generic::Header<u32, BlakeTwo256>;
pub type OpaqueBlock = GenericBlock<Header, OpaqueExtrinsic>;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
    



}
use sp_block_builder::BlockBuilder;
use node_primitives::Block;
use sp_api::ProvideRuntimeApi;
pub type RpcExtension = jsonrpsee::RpcModule<()>;
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, node_primitives::Balance>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + Sync + Send + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    // Initialize an empty RpcModule<()>
    let mut module = RpcModule::new(());

    let FullDeps { client, pool } = deps;

    // Merge the System RPC into the module
    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;

    // Merge the TransactionPayment RPC into the module
    module.merge(TransactionPayment::new(client).into_rpc())?;

    Ok(module)
}


