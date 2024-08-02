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

use fp_evm::GenesisAccount;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use argochain_runtime::{
	constants::currency::*, wasm_binary_unwrap, BabeConfig, BalancesConfig, Block, CouncilConfig,
	DemocracyConfig, ElectionsConfig, ImOnlineConfig, IndicesConfig, MaxNominations,
	NominationPoolsConfig, SessionConfig, SessionKeys, SocietyConfig, StakerStatus, StakingConfig,
	SudoConfig, SystemConfig, TechnicalCommitteeConfig,GrandpaConfig,AuthorityDiscoveryConfig,EthereumConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public, H160, U256, storage::Storage};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use std::{collections::BTreeMap, str::FromStr};
use sc_service::Properties;
pub use argochain_runtime::{RuntimeGenesisConfig, EVMConfig, GenesisConfig};
pub use node_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;
/// Flaming Fir testnet generator
// pub fn flaming_fir_config() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
// }

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> RuntimeGenesisConfig {
	#[rustfmt::skip]
		let initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![
		(
			// Validator 01
			// Stash Account
			// 5GP6QhAFgC2AGqk4SjBxjo8QbsyFSqLerFXkeei3Ja4ub6yC
			array_bytes::hex_n_into_unchecked("9078b5ab7ab0a30cd3dc7499a2d307f4cb3c78825950be5d96b283ed6c357d22"),
			// Controller account
			// 5ERnkSHvhvENYfpwKvHwDvb9RUy1g9o8P5399KU5jhgSpdoh
			array_bytes::hex_n_into_unchecked("a6618c462340747d1bd3d231ca69070e3dce2c0ebc292540fab0097b0b1a0048"),
			// Grandpa account
			// 5HP6y6LSUorMRUi4hktTkX7dBKfgfDHWQaaeS4sXqSe4noSg
			array_bytes::hex2array_unchecked("26c0ed226ac7bcedc03c4ddb80ba983a73afed4c289fd4d069ad7a4773df4713")
				.unchecked_into(),
			// Babe Account
			// 5Gp5Ldj57bvPvnSVG36KiBCB8kmUZNyczoMDfXHhLv1u8dCM
			array_bytes::hex2array_unchecked("cc03fa28f6affe614b98b6aa3effbbeea604eff5e6c241f92999d7d7f4e24970")
				.unchecked_into(),
			// imonline AccountJ
			// 5Dq6zmeYS1r2kNTwpxriMfikn7J33Xdxqs3GmX3AYXZix56o
			array_bytes::hex2array_unchecked("b0184e91829c18d0c6c53cf52ecc4e34201fc0948b6c37a86b58ee793bbb8e0c")
				.unchecked_into(),
			// authority discovery account
			// 5EJ1tvbXDNMnMSQt5N1LHMuhZ74UodGRsHLkEhmL1U9gbdGL
			array_bytes::hex2array_unchecked("f629db0ea8668e52598ff077c7613a01a560d2ece06f421667dbcfb38546287f")
				.unchecked_into(),
		),
		(
			// Validator 02
			// Stash Account
			// 5GzcnkD9ToM3eZ5bfLPzg87wyAFTaxQ5U1q4qaAUjWCpu26L
			array_bytes::hex_n_into_unchecked("5a39211ee03a0d20c8999f9166e18c554b1a335b174e5627da2b195e0df32225"),
			// Controller account
			// 5DLz2wQuesTXp8sy6Tr9f4G9WTGjrMUy2L3ZSu9qftLNdec3
			array_bytes::hex_n_into_unchecked("c440f736eb0f4efeb6fd406a6190fc090d6c710381741395e9dc6a7ee7064e05"),
			// Grandpa account
			// 5DFY3Q1RRcQHC3NCjCA5Dfad7BNJHNsdko7jcW8DMY5yadfu
			array_bytes::hex2array_unchecked("9fc1386bc35009a6845644a5092afc505ff065eb5f4cce4b2de856fef4b9a91e")
				.unchecked_into(),
			// Babe Account
			// 5CZqANK5Wz5CcKCX1KgsiBcHzcWA5Y9tYKBX7HHb6ej2pztY
			array_bytes::hex2array_unchecked("a22acb5d57afbc601b5c504bff9240d01589f0ee65bd1337e0b57e6606394c05")
				.unchecked_into(),
			// imonline Account
			// 5EKtGMfGy4AHG5C18y3DGtxJJAbQUa2SdxE8bTs4AEXWwNXd
			array_bytes::hex2array_unchecked("bc94beffcf4f6764d0964aaced70c048af7c2085cf5c0e989d15e182bb44483e")
				.unchecked_into(),
			// authority discovery account
			// 5F9WL65F6Vb6iTzhpGFDnKrQzhn7zwoU4eyNPX4LwfD6wkXR
			array_bytes::hex2array_unchecked("20bb86bbe2be4384924b683d191831b99397245e8a59f80f3774c46581469e35")
				.unchecked_into(),
		),
		(	
			// Validator 03
			// Stash Account
			// 5Cct2po3wyn4VbQ3jLAyttkBmuazxFBE1x7LLyTW5y9e93BT
			array_bytes::hex_n_into_unchecked("28434ff225784c73a446d4e60131cb9ad733bea6079b219329d50eb578fde435"),
			// Controller account
			// 5FF6zkZaG7afDJ3NmfqKLT1D185u37ReQkDKsVSqpEbwM6mK
			array_bytes::hex_n_into_unchecked("5093df3ad92bd606c123d62cfa9373210a0154d6f1b103777c2e8c3933795753"),
			// Grandpa account
			// 5DqsMyXw69D7f8XmVXMFvFHCh4C96GVpceSFH6YDcv4MbPxZ
			array_bytes::hex2array_unchecked("937ea882d717326397d9d87207db0b3af22f2bdd436b6b727cc57472774bc935")
				.unchecked_into(),
			// Babe Account
			// 5Cr17ma3dyLYoqizHGCEES2nyTqa3i6a7AaX6G4xTbccwKkA
			array_bytes::hex2array_unchecked("0aa299a921c1ab46d4f149e82e098e4ec3bb5da2f4b6af18ecb82ce415131f65")
				.unchecked_into(),
			// imonline Account
			// 5H44wWfdUAbRwz1U3cYUPCDoabahFAarKZBmMLh2DdZ6EWVi
			array_bytes::hex2array_unchecked("328659ad3ab97b9073fc3c509ab335fa1e101073db3c69b13060b3a966f7c155")
				.unchecked_into(),
			// authority discovery account
			// 5HozsyRsJDe4pGcsYPWP7cppwcRSRyEdtEF8fcdEiUc37HCc
			array_bytes::hex2array_unchecked("04c07a219802eb1d52039d1b94a269adff00e1b2aecb0fd1b3b1423fd44cb91c")
				.unchecked_into(),
		),
		(
			// Validator 04
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("5cff7487edca95e2eb0ae3399a09b023125bee6c734071aee51f8ad43dfd0a72"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("a84369109e5aa34eca11b0b8c0f378dde4d08e1e48a0d577ef2a483cc57b9c0b"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("71451ccef53edaf497611dc09834922dd819234bb56668de5dcdded9843bb004")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("96e8e8074f6d61c2708730fb22fcb9e735b22138fe2a3a6cc6b53daec58ebc2e")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("7a9bd742c87f07b9f782e275828a124d2bb485efd74d02a4828e4b166e05b57b")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("0xba85ac5cd071005a183132b34c8b7200a3342233638ceea2ddcb125faa0dae59")
				.unchecked_into(),
		),
		(
			// Validator 05
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("0e733c936e77d79505359546a20be34deb9ead14cef7de302de85e7d8bcaad63"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("f2fffdf2ef5b5df86f9b3972f90593e35140e44a421c8459a11dac43d6b6a64d"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("77564ca78c0688bb86767ff32f46cfaa7d741262145a1cb2bec27a07e940b5b1")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("36cb3494d69e1bfbcb4cdab64a33eccb1d4daf699f988c735cd32e6272d9166b")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("1ea6238440998511bf17bba4661d6511fdd4e94338baff70968ecae45e560b5e")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("78a115f06938584b443a65b5596bdacdd42078221c52c568dd3ea78286a07734")
				.unchecked_into(),
		),
		(
			// Validator 06
			// Stash Account
			// 5FRQCEfqfy1KPk7sEwzvab2m91rtEWkZTguHnWyZh8GmdUM2
			array_bytes::hex_n_into_unchecked("02a093dced748d34da2499916e2f43c4e7a1605c7ebd62b91fb93df29c5a657f"),
			// Controller account
			// 5HbMBXtxGNKBEvaTKqaJVTVhtiQWffa3zerZdg7pjJRQHn5S
			array_bytes::hex_n_into_unchecked("cefd9b75cd6f8faad9cd0d8964b422c00a8ef1c687b0e874be3c7acda7ee5578"),
			// Grandpa account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("0a8ccdf08b9ad16bede30b9a09eca50e02ad845bcc70b5d42d30115922f4a08a")
				.unchecked_into(),
			// Babe Account
			// 5DFq8RqSncqJx5E1X1xnybUTUTmxAGr8jBTkMa3zKEwLww8X
			array_bytes::hex2array_unchecked("e429bc829742f85f2aa630f642d0e9b0b59dcd469e3a0d1f591ce23271e4f246")
				.unchecked_into(),
			// imonline Account
			// 5FbQNWabp5o8oJEVrM4oJ7aBSHSr2NPDSFc6xNxLZvtgkSb5
			array_bytes::hex2array_unchecked("00af4277bd6f650547ba3347759decabf2586f41767e83b2f6f6bf01a3e21007")
				.unchecked_into(),
			// authority discovery account
			// 5FjFSpmHEy2sBzCqhmAi67PaSURTU9Gou3kguwf49qjDFGmW
			array_bytes::hex2array_unchecked("3c693558f39c8707cf81bbe43cd2c1842b6d5a67e459df6d0b6a6c0845eeac4a")
				.unchecked_into(),
		),
	];

	let root_key: AccountId = array_bytes::hex_n_into_unchecked(
		// Sudo Account
		// 5GErqqnpPaXJWdwb9EobdQwcPsD38ijaMUGV6mJkjhZJkzwd
		"5e630fa21c71aec95e0a0df72bfa1a1d67de75c354f16f9bff86f3b08128c816",
		
	);

	// let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	let mut endowed_accounts: Vec<(AccountId, Balance)> = vec![
		(root_key.clone(), 1_880_000 * ARGO),
		//1880000
	];

	initial_authorities.iter().for_each(|x| {
		endowed_accounts.push((x.0.clone(), 20_000 * ARGO));
	});

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "AGC".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 33.into());
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"ArgoChain",
		"argochain",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		Some(
			serde_json::from_str("{\"tokenDecimals\": 18, \"tokenSymbol\": \"AGC\"}")
				.expect("Provided valid json map"),
		),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create RuntimeGenesisConfig for testing
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
) -> RuntimeGenesisConfig {
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
			endowed_accounts.push((x.0.clone(), 20_000 * ARGO));
		}
	});
	initial_nominators.iter().for_each(|x| {
		if !endowed_accounts.iter().any(|(a, _)| a == x) {
			endowed_accounts.push((x.clone(), 0 * ARGO));
		}
	});
	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), 20_000 * ARGO, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			(
				x.clone(),
				x.clone(),
				10_000 * ARGO,
				StakerStatus::Nominator(initial_authorities.iter().map(|a| a.0.clone()).collect()),
			)
		}))
		.collect::<Vec<_>>();

	// let num_endowed_accounts = endowed_accounts.len();
	const STASH: Balance = 20_000 * ARGO;
	let technical_committee_members: Vec<AccountId> = vec![
		root_key.clone(),
		array_bytes::hex_n_into_unchecked("9078b5ab7ab0a30cd3dc7499a2d307f4cb3c78825950be5d96b283ed6c357d22"),
		array_bytes::hex_n_into_unchecked("5a39211ee03a0d20c8999f9166e18c554b1a335b174e5627da2b195e0df32225"),
		array_bytes::hex_n_into_unchecked("28434ff225784c73a446d4e60131cb9ad733bea6079b219329d50eb578fde435"),
		array_bytes::hex_n_into_unchecked("5cff7487edca95e2eb0ae3399a09b023125bee6c734071aee51f8ad43dfd0a72"),
		array_bytes::hex_n_into_unchecked("0e733c936e77d79505359546a20be34deb9ead14cef7de302de85e7d8bcaad63"),
		array_bytes::hex_n_into_unchecked("02a093dced748d34da2499916e2f43c4e7a1605c7ebd62b91fb93df29c5a657f"),
	];


	let elections_members: Vec<(AccountId,Balance)> = vec![
		(root_key.clone(), STASH),
        (array_bytes::hex_n_into_unchecked("9078b5ab7ab0a30cd3dc7499a2d307f4cb3c78825950be5d96b283ed6c357d22"), STASH),
        (array_bytes::hex_n_into_unchecked("5a39211ee03a0d20c8999f9166e18c554b1a335b174e5627da2b195e0df32225"), STASH),
        (array_bytes::hex_n_into_unchecked("28434ff225784c73a446d4e60131cb9ad733bea6079b219329d50eb578fde435"), STASH),
        (array_bytes::hex_n_into_unchecked("5cff7487edca95e2eb0ae3399a09b023125bee6c734071aee51f8ad43dfd0a72"), STASH),
		(array_bytes::hex_n_into_unchecked("0e733c936e77d79505359546a20be34deb9ead14cef7de302de85e7d8bcaad63"), STASH),
		(array_bytes::hex_n_into_unchecked("02a093dced748d34da2499916e2f43c4e7a1605c7ebd62b91fb93df29c5a657f"), STASH),
	];


	// const ENDOWMENT: Balance = 400_000 * ARGO;
	

	RuntimeGenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec(), ..Default::default() },
		balances: BalancesConfig {
			balances: endowed_accounts,
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: 100,
			// minimum_validator_count: initial_authorities.len() as u32,
			minimum_validator_count:4,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			max_nominator_count:Some(256),
			max_validator_count:Some(2000),
			stakers,
			// min_validator_bond:20_000_000_000_000_000_000_000,
			min_validator_bond:20_000 * ARGO,
			min_nominator_bond:40_00 * ARGO,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: elections_members,
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: technical_committee_members,
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			epoch_config: Some(argochain_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: Default::default(),
		grandpa: Default::default(),
		technical_membership: Default::default(),
		treasury: Default::default(),
		society: SocietyConfig { pot: 0 },
		vesting: Default::default(),
		assets: Default::default(),
		pool_assets: Default::default(),
		transaction_storage: Default::default(),
		transaction_payment: Default::default(),
		alliance: Default::default(),
		alliance_motion: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * ARGO,
			min_join_bond: 1 * ARGO,
			..Default::default()
		},
		glutton: Default::default(),
		// EVM compatibility
		// EVM compatibility
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(

					H160::from_str("976EA74026E726554dB657fA54763abd0C3a0aa9")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					H160::from_str("8eaf04151687736326c9fea17e25fc5287613693")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					H160::from_str("05E053aB0f66422d243C1F14Da2091CD56F51F73")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
			_marker: Default::default(),
		},
		ethereum: EthereumConfig {
			_marker: Default::default(),
		},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}

/// Helper function to create RuntimeGenesisConfig for development
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
	_chain_id: u64,
) -> RuntimeGenesisConfig {
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
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 2_000_000 * ARGO;
	const STASH: Balance = ENDOWMENT / 1000;

	RuntimeGenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec(), ..Default::default() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			epoch_config: Some(argochain_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig {
			keys: vec![],
			..Default::default()
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
			..Default::default()
		},
		technical_membership: Default::default(),
		treasury: Default::default(),

		society: Default::default(),
		vesting: Default::default(),
		assets: Default::default(),
		pool_assets: Default::default(),
		transaction_storage: Default::default(),

		transaction_payment: Default::default(),
		alliance: Default::default(),
		alliance_motion: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * ARGO,
			min_join_bond: ARGO,
			..Default::default()
		},
		glutton: Default::default(),

		// EVM compatibility
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(

					H160::from_str("976EA74026E726554dB657fA54763abd0C3a0aa9")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					H160::from_str("8eaf04151687736326c9fea17e25fc5287613693")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					H160::from_str("05E053aB0f66422d243C1F14Da2091CD56F51F73")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("1000000000000000000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
			_marker: Default::default(),
		},
		ethereum: EthereumConfig {
			_marker: Default::default(),
		},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}

fn development_config_genesis() -> RuntimeGenesisConfig {
	development_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		42,   //passing chain_id = 42.  Need to change??
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "AGC".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 33.into());
	ChainSpec::from_genesis(
		"Argochain Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Some(
			serde_json::from_str("{\"tokenDecimals\": 18, \"tokenSymbol\": \"AGC\"}")
				.expect("Provided valid json map"),
		),
		Default::default(),
	)
}

fn local_testnet_genesis() -> RuntimeGenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"ArgoChain local",
		"argochain_local",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		None,
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> RuntimeGenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, sync, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
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
