//! Substrate Node CLI library.
#![warn(missing_docs)]

mod network;
mod argo_bootstrap;
mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;
mod rpc;
mod eth;
mod client;

use libp2p::Multiaddr;
use tokio;
// mod command_helper;

#[tokio::main]
async fn main() -> sc_cli::Result<()> {
	let mut swarm = argo_bootstrap::build_swarm().await;

	let bootstrap_peers: Vec<Multiaddr> = vec![
		"/ip4/1.2.3.4/tcp/30333/p2p/12D3KooWRBN5aLQzf96TdeEUUqoktGeqNNz3iJqKN8cr6mbi1RKn".parse().unwrap,
		
	];

	argo_bootstrap::bootstrap(&mut swarm, bootstrap_peers).await;
	
	command::run()
}
