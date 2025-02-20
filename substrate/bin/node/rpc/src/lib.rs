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
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]
#![warn(unused_crate_dependencies)]



use sp_inherents::CreateInherentDataProviders;
use std::sync::Arc;

use jsonrpsee::RpcModule;
use futures::channel::mpsc;
use sp_consensus_babe::BabeApi;
use fc_rpc::pending::ConsensusDataProvider;
use node_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Nonce};
use sc_client_api::{Backend,backend::StorageProvider, client::BlockchainEvents, AuxStore, UsageProvider};
use sc_consensus_manual_seal::rpc::EngineCommand;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sc_consensus_babe::BabeWorkerHandle;
use sc_consensus_beefy::communication::notification::{
	BeefyBestBlockStream, BeefyVersionedFinalityProofStream,
};
use sc_consensus_grandpa::{
	FinalityProofProvider, GrandpaApi, GrandpaJustificationStream, SharedAuthoritySet,
    SharedVoterState,
};
pub use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_service::TransactionPool;
// use sc_transaction_pool_api::TransactionPool;
use sc_transaction_pool::{ChainApi, Pool};
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_application_crypto::RuntimeAppPublic;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
    Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_consensus::SelectChain;
use sp_consensus_beefy::AuthorityIdBound;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{
    Block as BlockT, Hash as HashT, HashingFor, Header as HeaderT, NumberFor,
};
// Frontier Compatilbe 

mod eth;
pub use self::eth::{create_eth, EthDeps};
/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// A handle to the BABE worker for issuing requests.
    pub babe_worker_handle: BabeWorkerHandle<Block>,
    /// The keystore that manages the keys of the node.
    pub keystore: KeystorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Dependencies for BEEFY
pub struct BeefyDeps<AuthorityId: AuthorityIdBound> {
    /// Receives notifications about finality proof events from BEEFY.
    pub beefy_finality_proof_stream: BeefyVersionedFinalityProofStream<Block, AuthorityId>,
    /// Receives notifications about best block events from BEEFY.
    pub beefy_best_block_stream: BeefyBestBlockStream<Block>,
    /// Executor to drive the subscription manager in the BEEFY RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B, AuthorityId: AuthorityIdBound, A: ChainApi, CT, CIDP> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
    /// BEEFY specific dependencies.
    pub beefy: BeefyDeps<AuthorityId>,
    /// Shared statement store reference.
    pub statement_store: Arc<dyn sp_statement_store::StatementStore>,
    /// The backend used by the node.
    pub backend: Arc<B>,
    /// Mixnet API.
    // pub mixnet_api: Option<sc_mixnet::Api>,
    pub eth: EthDeps<C, P, A, CT, CIDP>,
}

pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<B, C, BE> fc_rpc::EthConfig<B, C> for DefaultEthConfig<C, BE>
where
    B: BlockT,
    C: StorageProvider<B, BE> + Sync + Send + 'static,
    BE: sc_client_api::Backend<B> + 'static,
{
    type EstimateGasAdapter = ();
    type RuntimeStorageOverride =
        fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<B, C, BE>;
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, B, AuthorityId, A, CT, CIDP>(
    deps: FullDeps<C, P, SC, B, AuthorityId, A, CT, CIDP>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<Block>,
        >,
    >,
    pending_consenus_data_provider: Box<dyn ConsensusDataProvider<Block>>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    // Block: BlockT,
    C: ProvideRuntimeApi<Block>
        + sc_client_api::BlockBackend<Block>
        + CallApiAt<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        // + RpcFinalityProofProvider<Block>
        + Sync
        + Send
        + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    // C::Api: sp_consensus_aura::AuraApi<Block, AuraId>,
    C::Api: sc_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    C::Api: sp_api::ApiExt<Block>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C: BlockchainEvents<Block> + UsageProvider<Block> + StorageProvider<Block, B>,
    P: TransactionPool<Block = Block> + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashingFor<Block>>,
    B::Blockchain: BlockchainBackend<Block>,
    AuthorityId: AuthorityIdBound,
    <AuthorityId as RuntimeAppPublic>::Signature: Send + Sync,
    A: ChainApi<Block = Block> + 'static,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + 'static,
    CT: fp_rpc::ConvertTransaction<<Block as BlockT>::Extrinsic> + Send + Sync + 'static,
{
    use mmr_rpc::{Mmr, MmrApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_beefy_rpc::{Beefy, BeefyApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use sc_rpc::{
        dev::{Dev, DevApiServer},
        mixnet::MixnetApiServer,
        statement::StatementApiServer,
    };
    use sc_rpc_spec_v2::chain_spec::{ChainSpec, ChainSpecApiServer};
    use sc_sync_state_rpc::{SyncState, SyncStateApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};
    let FullDeps {
        client,
        pool,
        select_chain,
        chain_spec,
        deny_unsafe,
        babe,
        grandpa,
        beefy,
        statement_store,
        backend,
        // mixnet_api,
        eth,
    } = deps;
    let mut io = RpcModule::new(());

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    let chain_name = chain_spec.name().to_string();
    let genesis_hash = client
        .block_hash(0u32.into())
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let properties = chain_spec.properties();
    io.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;

    io.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
    // Making synchronous calls in light client freezes the browser currently,
    // more context: https://github.com/paritytech/substrate/pull/3480
    // These RPCs should use an asynchronous caller instead.
    io.merge(
        Mmr::new(
            client.clone(),
            backend
                .offchain_storage()
                .ok_or_else(|| "Backend doesn't provide an offchain storage")?,
        )
        .into_rpc(),
    )?;

    io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    let BabeDeps {
        keystore,
        babe_worker_handle,
    } = babe;
    io.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle.clone(),
            keystore,
            select_chain,
            deny_unsafe,
        )
        .into_rpc(),
    )?;
    io.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;

    io.merge(
        SyncState::new(
            chain_spec,
            client.clone(),
            shared_authority_set,
            babe_worker_handle,
        )?
        .into_rpc(),
    )?;

    io.merge(StateMigration::new(client.clone(), backend, deny_unsafe).into_rpc())?;
    io.merge(Dev::new(client, deny_unsafe).into_rpc())?;
    let statement_store =
        sc_rpc::statement::StatementStore::new(statement_store, deny_unsafe).into_rpc();
    io.merge(statement_store)?;

    // if let Some(mixnet_api) = mixnet_api {
    // 	let mixnet = sc_rpc::mixnet::Mixnet::new(mixnet_api).into_rpc();
    // 	io.merge(mixnet)?;
    // }

    io.merge(
        Beefy::<Block, AuthorityId>::new(
            beefy.beefy_finality_proof_stream,
            beefy.beefy_best_block_stream,
            beefy.subscription_executor,
        )?
        .into_rpc(),
    )?;

    // Ethereum compatibility RPCs
    let io = create_eth::<_, _, _, _, _, _, DefaultEthConfig<C, B>>(
        io,
        eth,
        subscription_task_executor,
        pubsub_notification_sinks,
        pending_consenus_data_provider,
    )?;

    Ok(io)
}