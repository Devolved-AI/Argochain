//# Author: Argo Network (Christian Kessler)
//# Description: This is a chain spec file for the Argo Network. It is used to configure the chain and its genesis block.
//# Date: 2024-04-18
//# Market: CoreComponents 

#![cfg(feature = "unified-accounts")]
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::FromStr};
// Substrate Modules
use sc_chain_spec::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;

#![cfg(not(feature = "unified-accounts"))]

use sp_core::sr25519;
use sp_core::{Storage, Pair, Public, H160, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sp_state_machine::BasicExternalities;
use std::collections::BTreeMap;
use frontier_template_runtime::{
    AccountId, Account, AuraConfig, BalancesConfig, EnableManualSeal, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
    SS58Prefix, SystemConfig, WASM_BINARY, EVMConfig, EthereumConfig,
};

pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

pub type DevChainSpec = sc_service::GenericChainSpec<DevelopmentGenesisConfig>;

#![derive(Serialize, Deserialize)]
pub struct DevArgoExt {
    genesis_config: RuntimeGenesisConfig,
    enable_manual_seal: bool,
}

impl sp_runtime::BuildStorage for DevelopmentGenesisConfig {
    fn simulate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        BasicExternalities::execute_with_storage(storage, || {
            if let Some(enable_manual_seal) = &self.enable_manual_seal {
                EnableManualSeal::set(*enable_manual_seal);
            }
        });
        self.genesis_config.simulate_storage(storage)
    }
}

/// Generate key pairs for the authorities.
pub fn get_from_seed<Tpublic: Public>(seed: &str) -> <Tpublic::Pair as Pair>::Public {
    Tpublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()       
    }

#[allow(dead_code)]
type AccountPublic = <Signature as Verify>::Signer;

#[allow(dead_code)]
pub fn get_account_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<TPublic>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}


//// Generate an Aura authority key pair.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), SS58Prefix::get().into());
    properties
}

const UNITS: Balance = 1_000_000_000_000_000_000;

#![cfg(feature = "unified-accounts")]
pub fn development_config(enable_manual_seal: Option<bool>) -> DevChainSpec {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available")?;

    let from_genesis = DevelopmentGenesisConfig::from_genesis(
        "DevArgoNet", 
        "Local",
        ChainType::Development, 
        move || { 
            DevArgoExt {
                genesis_config: DevArgoNet_genesis(
                    wasm_binary,
                    // Sudo Account,
                Account::AccountId::from_hex("0x7f8f3e0e4b0f3b3d7f3f47f8f3e0e4b0f3b3d7f3f4"));
                // Pre-funded accounts
                vec![
                    AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
                    AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
                    AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
                    AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
                    AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
                    AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
    
                    ],
                    // Initial Authorities for PoA Protocol.
                    vec![authority_keys_from_seed("Alice"),
                    // Ethereum Chain ID
                    SS58Prefix::get() as u64,   
                    ],
                    enable_manual_seal.unwrap_or(false),
                )
            },
            // Bootnodes
            vec![],
            // Telemetry
            None,
            // Protocol ID
            '333',
            // Fork ID
            None,
            // Properties
            Some(properties()),
            // Extensions
            None,
        });

    
    
    
    
    #![cfg(feature = "unified-accounts")]
    pub fn devargonet_config(enable_manual_seal: Option<bool>) -> DevChainSpec {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available")?;
    
    DevChainSpec::from_genesis(
        //name
        "DevArgoNet",
         // ID
        "42",
        ChainType::Development,
        move || {
            DevArgoExt {
                DevArgoNet_genesis(
                    wasm_binary,
                    // Sudo Account
                    get_account_id_from_seed::<sr25519::Public>("Athena"),
                    // Pre-funded accounts
                    vec![
                        get_account_id_from_seed::<sr25519::Public>("Alith"),
                        get_account_id_from_seed::<sr25519::Public>("Baltathar"),
                        get_account_id_from_seed::<sr25519::Public>("Charleth"),
                        get_account_id_from_seed::<sr25519::Public>("Dorothy"),
                        get_account_id_from_seed::<sr25519::Public>("Ethan"),
                        get_account_id_from_seed::<sr25519::Public>("Faith"),
                    ],
                    // Initial Authorities
                    vec![
                        authority_keys_from_seed("Athena")],
                        // Ethereum Chain ID
                        SS58Prefix::get() as u64 == 42,
                        enable_manual_seal.unwrap_or(false),
        // Bootnodes
        vec![],
            "/dns/p2p/localhost/tcp/30333/p2p/12D3KooWQv9",
            "dns/p2p/localhost/tcp/30334/p2p/12D3KooWQv9",
        // Telemetry
        "https://telemetry.polkadot.io/#/local_testnet"
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    )
    }
    
        #![cfg(feature = "unified-accounts")]
        pub fn local_testnet_config() -> ChainSpec {
            let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available")?;
    
            ChainSpec::from_genesis(
                // Name
                "Local Testnetr#",
                // ID
                "local_testnet#",
                ChainType::Local,
                move || {
                    let argo_net_genesis = ArgoNet_genesis(
                            wasm_binary,
                            // Sudo Account
                            AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),  // Alith    
                            // Pre-funded accounts
                            vec![
                                AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
                                AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
                                AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
                                AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
                                AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
                                AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
                            ],
                            // Initial Authorities
                            vec![
                                authority_keys_from_seed("Alice"),
                                authority_keys_from_seed("Bob"),
                            ],
                            42, // Ethereum Chain ID
                        );
                    DevArgoExt {
                        argo_net_genesis
                    }
                },
                // Bootnodes
                vec![],
                // Telemetry
                None,
                // Protocol ID
                "333",
                // Fork ID
                None,
                // Properties
                Some(properties()),
                // Extensions
                None,
            )
        }
    
        #![cfg(not(feature = "unified-accounts"))]
        pub fn devargonet_config() -> ChainSpec {
            let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available")?;
    
            let from_genesis = ChainSpec::from_genesis(
                // Name
                "ArgoNet",
                // ID
                "argochain",
                ChainType::Local,
                move || {
                    testnet_genesis(
                        wasm_binary,
                        // Sudo Account
                        get_account_id_from_seed::<sr25519::Public>("Athena"),
                        // Pre-funded accounts
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alith"),
                            get_account_id_from_seed::<sr25519::Public>("Baltathar"),
                            get_account_id_from_seed::<sr25519::Public>("Charleth"),
                            get_account_id_from_seed::<sr25519::Public>("Dorothy"),
                            get_account_id_from_seed::<sr25519::Public>("Ethan"),
                            get_account_id_from_seed::<sr25519::Public>("Faith"),
                        ],
                        // Initial Authorities
                        vec![
                            authority_keys_from_seed("Athena"),
                            authority_keys_from_seed("Bob"),
                        ],
                        42,
                    )
                },
                // Bootnodes
                vec![],
                // Telemetry
                None,
                // Protocol ID
                None,
                // Fork ID
                None,
                // Properties
                Some(properties()),
                // Extensions
                None,
            );
            from_genesis
        }
    
    /// Initial storage state for the genesis block.
    fn testnet_genesis(
    wasm_binary: &[u8],
    sudo_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    chain_id: u64,
    ) -> GenesisConfig {
    use frontier_template_runtime::{
        AccountId, AuraConfig, BalancesConfig, EVMChainIdConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
    };
    
    RuntimeGenesisConfig {
        // System Configuration
        system: SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        sudo: SudoConfig { 
            key: Some(sudo_key), 
        },
        // Financial Components
        balances: BalancesConfig {
            balances: endowed_accounts
            .iter()
            .cloned()
            .map(|k|(k, UNITS))
            .collect(),
        },
        transaction_payment: Default::default(),
    
        // Consensus Components
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        },
        // EVM Components
        evm_chain_id: EVMChainIdConfig {chain_id},
        evm: EVMConfig {
            accounts: {
                let mut map = BTreeMap::new();
                for (account, balance) in &endowed_accounts {
                    map.insert(account.clone(), balance.clone());
                }
                map
            },
        },; }; }
    let from_genesis = from_genesis