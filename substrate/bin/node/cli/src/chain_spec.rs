// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use polkadot_sdk::{sc_rpc_spec_v2::chain_spec, *};

use argochain_runtime::{
	constants::currency::*, wasm_binary_unwrap, Block, MaxNominations, SessionKeys, StakerStatus,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sc_chain_spec::Properties;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

pub use argochain_runtime::RuntimeGenesisConfig;
pub use node_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const ENDOWMENT: Balance = 10_000_000 * ARGO;
// const STASH: Balance = ENDOWMENT / 1000;
const STASH: Balance = 20_000 * ARGO;


fn properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "AGC".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties
}

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,

) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn configure_accounts_for_staging_testnet() -> (
	Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,

	)>,
	AccountId,
	Vec<(AccountId, Balance)>
) {
	#[rustfmt::skip]
	// stash, controller, session-key, beefy id
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	//
	// and
	//
	// for i in 1 2 3 4 ; do for j in session; do subkey inspect --scheme ed25519 "$secret"//fir//$j//$i; done; done
	//
	// and
	//
	// for i in 1 2 3 4 ; do for j in session; do subkey inspect --scheme ecdsa "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
		// MixnetId,
		// BeefyId,
	)> = vec![
		(
			// Validator 01
			// Stash Account
			// 5GP6QhAFgC2AGqk4SjBxjo8QbsyFSqLerFXkeei3Ja4ub6yC
			array_bytes::hex_n_into_unchecked("bef744b4a41a91f56bf8ca2f5dfd92e3d55f2419a620e10bbc967a703708eb5e"),
			// Controller account
			// 5ERnkSHvhvENYfpwKvHwDvb9RUy1g9o8P5399KU5jhgSpdoh
			array_bytes::hex_n_into_unchecked("688d17178101de764a96e0da7fa9f3f3edbf14baa31c1a7d5e4bf6190562742e"),
			// Grandpa account
			// 5HP6y6LSUorMRUi4hktTkX7dBKfgfDHWQaaeS4sXqSe4noSg
			array_bytes::hex2array_unchecked("eb3561de12ca6cc4f57518c906e6926c87fcd915e040c072fa2f26169f2112cf")
				.unchecked_into(),
			// Babe Account
			// 5Gp5Ldj57bvPvnSVG36KiBCB8kmUZNyczoMDfXHhLv1u8dCM
			array_bytes::hex2array_unchecked("d204cd355787872dff1357af0e688e53c3555f2e50b0eb0056f2240c6db95274")
				.unchecked_into(),
			// imonline AccountJ
			// 5Dq6zmeYS1r2kNTwpxriMfikn7J33Xdxqs3GmX3AYXZix56o
			array_bytes::hex2array_unchecked("4e18ed26e8c176d2e03e86e0b3c384ec7c2ea97414b6e6f20160d74a3819ce7c")
				.unchecked_into(),
			// authority discovery account
			// 5EJ1tvbXDNMnMSQt5N1LHMuhZ74UodGRsHLkEhmL1U9gbdGL
			array_bytes::hex2array_unchecked("629f61aa89835bea084ad5cc02c65fe5043d6adc4dc214837dc941f6599a5e34")
				.unchecked_into(),
			
				
		),
		(
			// Validator 02
			// Stash Account
			// 5GzcnkD9ToM3eZ5bfLPzg87wyAFTaxQ5U1q4qaAUjWCpu26L
			array_bytes::hex_n_into_unchecked("da0f205fba369ea8d1a3dc925aaba2cc7fe691e44e351854ead006b2a926545b"),
			// Controller account
			// 5DLz2wQuesTXp8sy6Tr9f4G9WTGjrMUy2L3ZSu9qftLNdec3
			array_bytes::hex_n_into_unchecked("38a69de6d0e09071458f3f0c8b0298b463aaae1b227be41faa01ecbde6fd1861"),
			// Grandpa account
			// 5DFY3Q1RRcQHC3NCjCA5Dfad7BNJHNsdko7jcW8DMY5yadfu
			array_bytes::hex2array_unchecked("347ee480f480b06975d02e93e58b45572f1e31f3de3534c516916b550d83e0ab")
				.unchecked_into(),
			// Babe Account
			// 5CZqANK5Wz5CcKCX1KgsiBcHzcWA5Y9tYKBX7HHb6ej2pztY
			array_bytes::hex2array_unchecked("1636b54610e12dec6d4047071e470bb6be289f552a031b30b4e949158bffe628")
				.unchecked_into(),
			// imonline Account
			// 5EKtGMfGy4AHG5C18y3DGtxJJAbQUa2SdxE8bTs4AEXWwNXd
			array_bytes::hex2array_unchecked("640c2f92a2744b9ca2fc56b59b9446077c46b4a563df3206596553f735ce5f0c")
				.unchecked_into(),
			// authority discovery account
			// 5F9WL65F6Vb6iTzhpGFDnKrQzhn7zwoU4eyNPX4LwfD6wkXR
			array_bytes::hex2array_unchecked("885e1f8a0b2f3a1526d294f9030b9b9f7329cd2657a81f13d9eb1391dfd20415")
				.unchecked_into(),
			
		),
		(	
			// Validator 03
			// Stash Account
			// 5Cct2po3wyn4VbQ3jLAyttkBmuazxFBE1x7LLyTW5y9e93BT
			array_bytes::hex_n_into_unchecked("188a1afb495f13861bebbbb04ba71a22cadfab71bd79d54b848f4d71c7a6d64e"),
			// Controller account
			// 5FF6zkZaG7afDJ3NmfqKLT1D185u37ReQkDKsVSqpEbwM6mK
			array_bytes::hex_n_into_unchecked("8ca30c66f89299dbe94584578a433abd6a6fc32d35f08582a40e7dfdf5e0e650"),
			// Grandpa account
			// 5DqsMyXw69D7f8XmVXMFvFHCh4C96GVpceSFH6YDcv4MbPxZ
			array_bytes::hex2array_unchecked("4eae4638de005e4ad2c6a6baf70b994ced11b6cb7a6725776600da09aa47a817")
				.unchecked_into(),
			// Babe Account
			// 5Cr17ma3dyLYoqizHGCEES2nyTqa3i6a7AaX6G4xTbccwKkA
			array_bytes::hex2array_unchecked("228c258fd43646254ac8054c1be17fa64242b46557d3180c4946a83650b38010")
				.unchecked_into(),
			// imonline Account
			// 5H44wWfdUAbRwz1U3cYUPCDoabahFAarKZBmMLh2DdZ6EWVi
			array_bytes::hex2array_unchecked("dcb0e5311180866071e3c059579dc107f2985eb05106a80a54b4467ea859916c")
				.unchecked_into(),
			// authority discovery account
			// 5HozsyRsJDe4pGcsYPWP7cppwcRSRyEdtEF8fcdEiUc37HCc
			array_bytes::hex2array_unchecked("fe3205c22a92b36ac37380633f5c25b540ebb58390c0e6a91f3ac2a1b8040d5f")
				.unchecked_into(),
			
		),
		(
			// Validator 04
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("947d656a62e92c36c086ebc1b0f7473b1121f6cdd295cace4db7d99cdf24fc72"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("f48c2d6a3a24435117195c03fdd1dc32ca22fe133ac5d19f575d996141262828"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("cb889c5e38353ef45dd90da209e081cd2444548a67991a3161963d44925217ae")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("34b86966948aba8e078048ea31a31670923e2c6af70cb2b1f621fb573cfd5752")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("9c1e724ed3cd87eb7a0b6124f9880fc8a3f08fe36e52983aeb36ec43598cc307")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("a21a5d08177dce2ad9d2bd38000aa1b0db6b6f7b5b30c9cf67eb7a8cb681ba30")
				.unchecked_into(),
			
		),
		(
			// Validator 05
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("9ef74403465bf3b714c250b663ef2ddf759d36a5f7568dc3ad99b330365aed1b"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("4270bd2515ba814b8478cccf8183d257616d6628bb19ba82cc0863fdb38b303a"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("9a58de930caf823b1bfc60bfd565504e685c69bf73e4b0b5ff54eb46fa57d178")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("2439f49bbc53dc4cb624e8496457e558270363dc3c3c5af7c28a7098650fc303")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("aea7835285329d267c63c1ba29b00c2817e15f2ba6497b5dd735d844dd49af6b")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("a0e8f82617a4efe279794e3c5112853eaa03fd9f482d9d7c5d33400bb5c11077")
				.unchecked_into(),
			
		),
		(
			// Validator 06
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("cec881200692ec892a6032cabd2c27cb43f48f6a7b8c45f610cb7d8fab304370"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("e803fa8cc798c76ac82d9ca4a2277ee656887514f1a1ed6b0f95e68b3670e94a"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("dbccaebd776539664fc5eea115ce5e2557fe90414de059978630717e4aadd344")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("3af78e69eb6c75d416bc5a78ce77a9053feef4ecb2cffd5ff91b2dbb9c04525e")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("2ac3b71065cb515acdc93175a5dd7fbe4cd9e421da90d41008c89632a48ccf1e")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("244a90e9025622e1317f642b96233d51a9bd08855541e1b8b65dfa0e02b49c0c")
				.unchecked_into(),
			
		),
	];

	let root_key: AccountId = array_bytes::hex_n_into_unchecked(
		// Sudo Account
		// 5GErqqnpPaXJWdwb9EobdQwcPsD38ijaMUGV6mJkjhZJkzwd
		"b8afa2f67521bd80e4febceea9dd44a249744596b658b06e943ae265bf5be252",
		
	);

	let mut endowed_accounts: Vec<(AccountId, Balance)> = vec![
		(root_key.clone(), 1_880_000 * ARGO),
		//1880000
	];

	initial_authorities.iter().for_each(|x| {
		endowed_accounts.push((x.0.clone(), 20_000 * ARGO));
	});


	// let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];
	(initial_authorities, root_key, endowed_accounts)
}

fn staging_testnet_config_genesis() -> serde_json::Value {
	let (initial_authorities, root_key, endowed_accounts) =
		configure_accounts_for_staging_testnet();
	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	ChainSpec::builder(wasm_binary_unwrap(), Default::default())
		.with_name("Staging Testnet")
		.with_id("staging_testnet")
		.with_properties(properties())
		.with_chain_type(ChainType::Live)
		.with_genesis_config_patch(staging_testnet_config_genesis())
		.with_telemetry_endpoints(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		)
		.build()
}

/// Helper function to generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed.
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)
{
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),

	)
}

fn configure_accounts(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	endowed_accounts: Option<Vec<(AccountId,Balance)>>,
	stash: Balance,
) -> (
	Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
		// MixnetId,
		// BeefyId,
	)>,
	Vec<(AccountId, Balance)>,
	usize,
	Vec<(AccountId, AccountId, Balance, StakerStatus<AccountId>)>,
) {
	let mut endowed_accounts: Vec<(AccountId, Balance)> = endowed_accounts.unwrap_or_else(|| {
		vec![
			(get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Dave"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Eve"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Ferdie"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Alice//stash"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Bob//stash"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Charlie//stash"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Dave//stash"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Eve//stash"), 1_000_000 * ARGO),
			(get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"), 1_000_000 * ARGO),
		]
	});
	// endow all authorities and nominators.
	initial_authorities.iter().for_each(|x| {
		if !endowed_accounts.iter().any(|(a, _)| a == &x.0) {
			endowed_accounts.push((x.0.clone(), stash));
		}
	});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), stash, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(),  0 * ARGO, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	(initial_authorities, endowed_accounts, num_endowed_accounts, stakers)
}

fn dev_configure_accounts(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,

	)>,
	initial_nominators: Vec<AccountId>,
	endowed_accounts: Option<Vec<AccountId>>,
	stash: Balance,
) -> (
	Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	
	)>,
	Vec<AccountId>,
	usize,
	Vec<(AccountId, AccountId, Balance, StakerStatus<AccountId>)>,
) {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), stash, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), stash, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	(initial_authorities, endowed_accounts, num_endowed_accounts, stakers)
}

/// Helper function to create RuntimeGenesisConfig json patch for testing.
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,

	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<(AccountId,Balance)>>,
	
) -> serde_json::Value {
	let (initial_authorities, endowed_accounts, num_endowed_accounts, stakers) =
		configure_accounts(initial_authorities, initial_nominators, endowed_accounts, STASH);
		let technical_committee_members: Vec<AccountId> = vec![
			root_key.clone(),
			array_bytes::hex_n_into_unchecked("bef744b4a41a91f56bf8ca2f5dfd92e3d55f2419a620e10bbc967a703708eb5e"),
			array_bytes::hex_n_into_unchecked("da0f205fba369ea8d1a3dc925aaba2cc7fe691e44e351854ead006b2a926545b"),
			array_bytes::hex_n_into_unchecked("188a1afb495f13861bebbbb04ba71a22cadfab71bd79d54b848f4d71c7a6d64e"),
			array_bytes::hex_n_into_unchecked("947d656a62e92c36c086ebc1b0f7473b1121f6cdd295cace4db7d99cdf24fc72"),
			array_bytes::hex_n_into_unchecked("9ef74403465bf3b714c250b663ef2ddf759d36a5f7568dc3ad99b330365aed1b"),
			array_bytes::hex_n_into_unchecked("cec881200692ec892a6032cabd2c27cb43f48f6a7b8c45f610cb7d8fab304370"),
		];
	

		let elections_members: Vec<(AccountId,Balance)> = vec![
			(root_key.clone(), STASH),
			(array_bytes::hex_n_into_unchecked("bef744b4a41a91f56bf8ca2f5dfd92e3d55f2419a620e10bbc967a703708eb5e"), STASH),
			(array_bytes::hex_n_into_unchecked("da0f205fba369ea8d1a3dc925aaba2cc7fe691e44e351854ead006b2a926545b"), STASH),
			(array_bytes::hex_n_into_unchecked("188a1afb495f13861bebbbb04ba71a22cadfab71bd79d54b848f4d71c7a6d64e"), STASH),
			(array_bytes::hex_n_into_unchecked("947d656a62e92c36c086ebc1b0f7473b1121f6cdd295cace4db7d99cdf24fc72"), STASH),
			(array_bytes::hex_n_into_unchecked("9ef74403465bf3b714c250b663ef2ddf759d36a5f7568dc3ad99b330365aed1b"), STASH),
			(array_bytes::hex_n_into_unchecked("cec881200692ec892a6032cabd2c27cb43f48f6a7b8c45f610cb7d8fab304370"), STASH),
		];

	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().collect::<Vec<_>>(),
		},
		"session": {
			"keys": initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		"staking": {
			"validatorCount": 100,
			"minimumValidatorCount": 4,
			"invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
			"slashRewardFraction": Perbill::from_percent(10),
			"stakers": stakers.clone(),
		},
		"elections": {
			"members": elections_members
				
		},
		"technicalCommittee": {
			"members": technical_committee_members
				
		},
		"sudo": { "key": Some(root_key.clone()) },
		"babe": {
			"epochConfig": Some(argochain_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		"society": { "pot": 0 },
		"assets": {
			// This asset is used by the NIS pallet as counterpart currency.
			"assets": vec![(9, get_account_id_from_seed::<sr25519::Public>("Alice"), true, 1)],
		},
		"nominationPools": {
			"minCreateBond": 10 * ARGO,
			"minJoinBond": 1 * ARGO,
		},
	})
}

fn development_config_genesis_json() -> serde_json::Value {
	development_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Helper function to create RuntimeGenesisConfig json patch for testing.
pub fn development_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,

	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> serde_json::Value {
	let (initial_authorities, endowed_accounts, num_endowed_accounts, stakers) =
	dev_configure_accounts(initial_authorities, initial_nominators, endowed_accounts, STASH);

	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect::<Vec<_>>(),
		},
		"session": {
			"keys": initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		"staking": {
			"validatorCount": initial_authorities.len() as u32,
			"minimumValidatorCount": initial_authorities.len() as u32,
			"invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
			"slashRewardFraction": Perbill::from_percent(10),
			"stakers": stakers.clone(),
		},
		"elections": {
			"members": endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect::<Vec<_>>(),
		},
		"technicalCommittee": {
			"members": endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect::<Vec<_>>(),
		},
		"sudo": { "key": Some(root_key.clone()) },
		"babe": {
			"epochConfig": Some(argochain_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		"society": { "pot": 0 },
		"assets": {
			// This asset is used by the NIS pallet as counterpart currency.
			"assets": vec![(9, get_account_id_from_seed::<sr25519::Public>("Alice"), true, 1)],
		},
		"nominationPools": {
			"minCreateBond": 10 * ARGO,
			"minJoinBond": 1 * ARGO,
		},
	})
}

/// Development config (single validator Alice).
pub fn development_config() -> ChainSpec {
    ChainSpec::builder(wasm_binary_unwrap(), Default::default())
        .with_name("Development")
        .with_id("dev")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_patch(development_config_genesis_json())
        .with_properties(properties())  // Add this line
        .build()
}

fn local_testnet_genesis() -> serde_json::Value {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob).
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::builder(wasm_binary_unwrap(), Default::default())
		.with_name("Local Testnet")
		.with_id("local_testnet")
		.with_chain_type(ChainType::Local)
		.with_genesis_config_patch(local_testnet_genesis())
		.build()
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	/// Local testnet config (single validator - Alice).
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::builder(wasm_binary_unwrap(), Default::default())
			.with_name("Integration Test")
			.with_id("test")
			.with_chain_type(ChainType::Development)
			.with_genesis_config_patch(testnet_genesis(
				vec![authority_keys_from_seed("Alice")],
				vec![],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				None,
			))
			.build()
	}

	/// Local testnet config (multivalidator Alice + Bob).
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::builder(wasm_binary_unwrap(), Default::default())
			.with_name("Integration Test")
			.with_id("test")
			.with_chain_type(ChainType::Development)
			.with_genesis_config_patch(local_testnet_genesis())
			.build()
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, sync, transaction_pool, .. } =
				new_full_base::<sc_network::NetworkWorker<_, _>>(config, None, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				sync,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}