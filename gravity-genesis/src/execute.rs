use crate::{
    contracts::*,
    storage::InMemoryDB,
    utils::{new_system_call_txn, read_hex_from_file, BLOCK_ADDR, DELEGATION_ADDR, EPOCH_MANAGER_ADDR, GENESIS_ADDR, GOVERNOR_ADDR, GOV_HUB_ADDR, GOV_TOKEN_ADDR, JWK_MANAGER_ADDR, KEYLESS_ACCOUNT_ADDR, STAKE_CONFIG_ADDR, STAKE_CREDIT_ADDR, SYSTEM_ADDRESS, SYSTEM_CALLER, SYSTEM_REWARD_ADDR, TIMELOCK_ADDR, TIMESTAMP_ADDR, VALIDATOR_MANAGER_ADDR, VALIDATOR_PERFORMANCE_TRACKER_ADDR},
};

use alloy_chains::NamedChain;

use crate::utils::{analyze_revert_reason, execute_revm_sequential_with_logging};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use revm::{
    db::PlainAccount,
    primitives::{AccountInfo, Address, Env, KECCAK_EMPTY, SpecId, TxEnv, U256, uint},
};
use revm_primitives::{hex, Bytecode, Bytes};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, fs::File, io::BufWriter};

#[derive(Debug, Deserialize, Serialize)]
pub struct GenesisConfig {
    #[serde(rename = "validatorAddresses")]
    pub validator_addresses: Vec<String>,
    #[serde(rename = "consensusAddresses")]
    pub consensus_addresses: Vec<String>,
    #[serde(rename = "feeAddresses")]
    pub fee_addresses: Vec<String>,
    #[serde(rename = "votingPowers")]
    pub voting_powers: Vec<String>,
    #[serde(rename = "voteAddresses")]
    pub vote_addresses: Vec<String>,
}

fn call_genesis_initialize(genesis_address: Address, config: &GenesisConfig) -> TxEnv {
    // Convert string addresses to Address type
    let validator_addresses: Vec<Address> = config
        .validator_addresses
        .iter()
        .map(|addr| addr.parse::<Address>().expect("Invalid validator address"))
        .collect();

    let consensus_addresses: Vec<Address> = config
        .consensus_addresses
        .iter()
        .map(|addr| addr.parse::<Address>().expect("Invalid consensus address"))
        .collect();

    let fee_addresses: Vec<Address> = config
        .fee_addresses
        .iter()
        .map(|addr| addr.parse::<Address>().expect("Invalid fee address"))
        .collect();

    let voting_powers: Vec<U256> = config
        .voting_powers
        .iter()
        .map(|power| power.parse::<U256>().expect("Invalid voting power"))
        .collect();

    let vote_addresses: Vec<Bytes> = config
        .vote_addresses
        .iter()
        .map(|addr| hex::decode(addr).expect("Invalid vote address").into())
        .collect();

    println!("=== Genesis Initialize Parameters ===");
    println!("Genesis address: {:?}", genesis_address);
    println!("Validator addresses: {:?}", validator_addresses);
    println!("Consensus addresses: {:?}", consensus_addresses);
    println!("Fee addresses: {:?}", fee_addresses);
    println!("Voting powers: {:?}", voting_powers);
    println!("Vote addresses count: {}", vote_addresses.len());

    // 草 这里根本就没拿到Genesis地址实例啊 要用GENESIS_ADDRESS来初始化出来的
    sol! {
        contract Genesis {
            function initialize(
                address[] calldata validatorAddresses,
                address[] calldata consensusAddresses,
                address payable[] calldata feeAddresses,
                uint256[] calldata votingPowers,
                bytes[] calldata voteAddresses
            ) external;
        }
    }

    let call_data = Genesis::initializeCall {
        validatorAddresses: validator_addresses,
        consensusAddresses: consensus_addresses,
        feeAddresses: fee_addresses,
        votingPowers: voting_powers,
        voteAddresses: vote_addresses,
    }
    .abi_encode();

    println!("Call data length: {}", call_data.len());
    println!("Call data: 0x{}", hex::encode(&call_data));

    let txn = new_system_call_txn(genesis_address, call_data.into());
    txn
}

fn only_deploy_bytecode(byte_code_dir: &str) -> HashMap<Address, PlainAccount> {
    let mut map = HashMap::new();
    let hex_path = format!("{}/System.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(SYSTEM_CALLER, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/SystemReward.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(SYSTEM_REWARD_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/StakeConfig.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(STAKE_CONFIG_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/ValidatorManager.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(VALIDATOR_MANAGER_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/ValidatorPerformanceTracker.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(VALIDATOR_PERFORMANCE_TRACKER_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/EpochManager.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(EPOCH_MANAGER_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/GovToken.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(GOV_TOKEN_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/Timelock.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(TIMELOCK_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/GravityGovernor.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(GOVERNOR_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/JWKManager.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(JWK_MANAGER_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/KeylessAccount.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(KEYLESS_ACCOUNT_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/Block.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(BLOCK_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/Timestamp.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(TIMESTAMP_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/Genesis.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(GENESIS_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/StakeCredit.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(STAKE_CREDIT_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/Delegation.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(DELEGATION_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    let hex_path = format!("{}/GovHub.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    map.insert(GOV_HUB_ADDR, PlainAccount {
        info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
        storage: Default::default(),
    });

    // let hex_path = format!("{}/Groth16Verifier.hex", byte_code_dir);
    // let bytes_sol_hex = read_hex_from_file(&hex_path);
    // map.insert(GROTH16_VERIFIER_ADDR, PlainAccount {
    //     info: AccountInfo::from_bytecode(Bytecode::LegacyRaw(Bytes::from(bytes_sol_hex))),
    //     storage: Default::default(),
    // });
    map.insert(SYSTEM_ADDRESS, PlainAccount {
        info: AccountInfo {
            balance: uint!(1_000_000_000_000_000_000_U256),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
        },
        storage: Default::default(),
    });

    map
}

fn load_genesis_state(byte_code_dir: &str) -> HashMap<Address, PlainAccount> {
    let mut map = HashMap::new();
    map.insert(SYSTEM_ADDRESS, PlainAccount {
        info: AccountInfo {
            balance: uint!(1_000_000_000_000_000_000_U256),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
        },
        storage: Default::default(),
    });

    map
}

pub fn genesis_generate(byte_code_dir: &str, output_dir: &str, config: GenesisConfig) {
    // let deployed_state = None;
    // let map = only_deploy_bytecode(byte_code_dir);
    
    let deployed_state = Some(deploy_and_constrcut_all(byte_code_dir));
    let map = load_genesis_state(byte_code_dir);
    
    let mut env = Env::default();
    env.cfg.chain_id = NamedChain::Mainnet.into();
    let db = InMemoryDB::new(
        map,
        Default::default(),
        Default::default(),
    );

    let mut txs = Vec::new();
    // 调用Genesis初始化函数
    println!("Calling Genesis initialize function...");
    let genesis_init_txn = call_genesis_initialize(GENESIS_ADDR, &config);
    txs.push(genesis_init_txn);

    println!("=== Starting Genesis deployment and initialization ===");
    let (result, mut bundle_state) =
        execute_revm_sequential_with_logging(db, SpecId::LATEST, env, &txs, deployed_state)
            .unwrap();
    let mut success_count = 0;
    for (i, r) in result.iter().enumerate() {
        if !r.is_success() {
            println!("=== Transaction {} failed ===", i + 1);
            println!("Detailed analysis: {}", analyze_revert_reason(r));
            panic!("Genesis transaction {} failed", i + 1);
        } else {
            println!("Transaction {}: succeed", i + 1);
        }
        success_count += 1;
    }
    println!(
        "=== All {} transactions completed successfully ===",
        success_count
    );
    bundle_state.state.remove(&SYSTEM_ADDRESS);
    let genesis_state = bundle_state
        .state
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                PlainAccount {
                    info: v.info.unwrap(),
                    storage: v
                        .storage
                        .into_iter()
                        .map(|(k, v)| (k, v.present_value()))
                        .collect(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_accounts.json")).unwrap()),
        &genesis_state,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_contracts.json")).unwrap()),
        &bundle_state
            .contracts
            .iter()
            .map(|(k, v)| (k, v.bytecode()))
            .collect::<HashMap<_, _>>(),
    )
    .unwrap();
}
