use codec::Codec;
// Substrate
use sc_executor::WasmExecutor;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay};

use crate::eth::EthCompatRuntimeApiCollection;

/// Full backend.
pub type FullBackend<BlockT> = sc_service::TFullBackend<BlockT>;
/// Full client.
pub type FullClient<BlockT, RA, HF> = sc_service::TFullClient<BlockT, RA, WasmExecutor<HF>>;

/// A set of APIs that every runtime must implement.
pub trait BaseRuntimeApiCollection<Block: BlockT>:
	sp_api::ApiExt<Block>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
{
}

impl<Block, Api> BaseRuntimeApiCollection<Block> for Api
where
	Block: BlockT,
	Api: sp_api::ApiExt<Block>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>,
{
}

/// A set of APIs that template runtime must implement.
pub trait RuntimeApiCollection<
	Block: BlockT,
	// AuraId: Codec,
	AccountId: Codec,
	Nonce: Codec,
	Balance: Codec + MaybeDisplay,
>:
	BaseRuntimeApiCollection<Block>
	+ EthCompatRuntimeApiCollection<Block>
	// + sp_consensus_aura::AuraApi<Block, AuraId>
	+ sp_consensus_grandpa::GrandpaApi<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
{
}

impl<Block, AccountId, Nonce, Balance, Api>
	RuntimeApiCollection<Block, AccountId, Nonce, Balance> for Api
where
	Block: BlockT,
	// AuraId: Codec,
	AccountId: Codec,
	Nonce: Codec,
	Balance: Codec + MaybeDisplay,
	Api: BaseRuntimeApiCollection<Block>
		+ EthCompatRuntimeApiCollection<Block>
		// + sp_consensus_aura::AuraApi<Block, AuraId>
		+ sp_consensus_grandpa::GrandpaApi<Block>
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>,
{
}
