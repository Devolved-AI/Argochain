
use crate::{
	AccountId,
	RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig, EXISTENTIAL_DEPOSIT,
};
use alloc::{vec, vec::Vec};
use serde_json::Value;
use sp_core::sr25519;
use sp_genesis_builder::PresetId;


/// The default XCM version to set in genesis config.

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).


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
) -> Value {
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

    initial_authorities
        .iter()
        .map(|x| &x.0)
        .chain(initial_nominators.iter())
        .for_each(|x| {
            if !endowed_accounts.contains(x) {
                endowed_accounts.push(x.clone());
            }
        });

    const ENDOWMENT: Balance = 2_000_000 * ARGO;
    const STASH: Balance = ENDOWMENT / 1000;

    json!({
        "system": {
            "code": wasm_binary_unwrap().to_vec(),
        },
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, ENDOWMENT))
                .collect::<Vec<_>>(),
        },
        "indices": {
            "indices": [],
        },
        "session": {
            "keys": initial_authorities
                .iter()
                .map(|x| {
                    json!({
                        "stash": x.0,
                        "controller": x.0,
                        "session_keys": {
                            "grandpa": x.2,
                            "babe": x.3,
                            "im_online": x.4,
                            "authority_discovery": x.5,
                        },
                    })
                })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "validator_count": initial_authorities.len() as u32,
            "minimum_validator_count": initial_authorities.len() as u32,
            "invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            "slash_reward_fraction": "10%",
            "stakers": initial_authorities
                .iter()
                .map(|x| {
                    json!({
                        "stash": x.0,
                        "controller": x.1,
                        "balance": STASH,
                        "status": "Validator",
                    })
                })
                .chain(initial_nominators.iter().map(|x| {
                    json!({
                        "stash": x,
                        "controller": x,
                        "balance": STASH,
                        "status": "Nominator",
                    })
                }))
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": root_key,
        },
        "babe": {
            "epoch_config": argochain_runtime::BABE_GENESIS_EPOCH_CONFIG,
        },
        "im_online": {
            "keys": [],
        },
        "authority_discovery": {
            "keys": [],
        },
        "grandpa": {
            "authorities": [],
        },
        "technical_membership": {},
        "treasury": {},
        "society": {},
        "vesting": {},
        "assets": {},
        "pool_assets": {},
        "transaction_storage": {},
        "transaction_payment": {},
        "nomination_pools": {
            "min_create_bond": 10 * ARGO,
            "min_join_bond": ARGO,
        },
        "glutton": {},
        "evm": {},
        "ethereum": {
            "_marker": null,
        },
        "dynamic_fee": {},
        "base_fee": {},
    })
}



fn development_config_genesis() -> Value {
	development_config(
        vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		42,   //passing chain_id = 42.  Need to change??
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<vec::Vec<u8>> {
	let patch = match id.try_into() {
		Ok(sp_genesis_builder::DEV_RUNTIME_PRESET) => development_config_genesis(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
	]
}