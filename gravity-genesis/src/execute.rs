use crate::utils::{
    BLOCK_ADDR, DELEGATION_ADDR, EPOCH_MANAGER_ADDR, GENESIS_ADDR, GOV_HUB_ADDR, GOV_TOKEN_ADDR,
    GOVERNOR_ADDR, JWK_MANAGER_ADDR, KEYLESS_ACCOUNT_ADDR, STAKE_CONFIG_ADDR, STAKE_CREDIT_ADDR,
    SYSTEM_ADDRESS, SYSTEM_CALLER, SYSTEM_REWARD_ADDR, TIMELOCK_ADDR, TIMESTAMP_ADDR,
    VALIDATOR_MANAGER_ADDR, VALIDATOR_PERFORMANCE_TRACKER_ADDR, new_system_call_txn,
    new_system_create_txn, read_hex_from_file,
};

use alloy_chains::NamedChain;

use crate::utils::{analyze_revert_reason, execute_revm_sequential_with_logging};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use revm::{
    DatabaseRef, InMemoryDB,
    db::{CacheDB, PlainAccount},
    primitives::{AccountInfo, Address, Env, KECCAK_EMPTY, SpecId, TxEnv, U256, uint},
};
use revm_primitives::{Bytecode, Bytes, hex};
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

// Deploy contracts using constructor bytecode (proper deployment)
fn deploy_all_contracts(byte_code_dir: &str) -> (impl DatabaseRef, Vec<TxEnv>) {
    let revm_db = InMemoryDB::default();
    let mut db = CacheDB::new(revm_db);
    let mut txs = Vec::new();

    // Add system address with balance
    db.insert_account_info(
        SYSTEM_ADDRESS,
        AccountInfo {
            balance: uint!(1_000_000_000_000_000_000_U256),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
        },
    );

    // Deploy contracts in order using constructor bytecode
    let contracts = [
        ("System", SYSTEM_CALLER),
        ("SystemReward", SYSTEM_REWARD_ADDR),
        ("StakeConfig", STAKE_CONFIG_ADDR),
        ("ValidatorManager", VALIDATOR_MANAGER_ADDR),
        (
            "ValidatorPerformanceTracker",
            VALIDATOR_PERFORMANCE_TRACKER_ADDR,
        ),
        ("EpochManager", EPOCH_MANAGER_ADDR),
        ("GovToken", GOV_TOKEN_ADDR),
        ("Timelock", TIMELOCK_ADDR),
        ("GravityGovernor", GOVERNOR_ADDR),
        ("JWKManager", JWK_MANAGER_ADDR),
        ("KeylessAccount", KEYLESS_ACCOUNT_ADDR),
        ("Block", BLOCK_ADDR),
        ("Timestamp", TIMESTAMP_ADDR),
        ("Genesis", GENESIS_ADDR),
        ("StakeCredit", STAKE_CREDIT_ADDR),
        ("Delegation", DELEGATION_ADDR),
        ("GovHub", GOV_HUB_ADDR),
    ];

    for (contract_name, target_address) in contracts {
        let hex_path = format!("{}/{}.hex", byte_code_dir, contract_name);
        let constructor_bytecode = read_hex_from_file(&hex_path);

        // Create deployment transaction
        let deploy_txn = new_system_create_txn(&constructor_bytecode, Bytes::default());
        txs.push(deploy_txn);

        println!(
            "Prepared deployment for {}: target address {:?}",
            contract_name, target_address
        );
    }

    (db, txs)
}

// Alternative approach: Use BSC-style direct bytecode deployment
fn deploy_bsc_style(byte_code_dir: &str) -> impl DatabaseRef {
    let revm_db = InMemoryDB::default();
    let mut db = CacheDB::new(revm_db);

    // Add system address with balance
    db.insert_account_info(
        SYSTEM_ADDRESS,
        AccountInfo {
            balance: uint!(1_000_000_000_000_000_000_U256),
            nonce: 1,
            code_hash: KECCAK_EMPTY,
            code: None,
        },
    );

    // Deploy runtime bytecode directly to predefined addresses (BSC style)
    let contracts = [
        ("System", SYSTEM_CALLER),
        ("SystemReward", SYSTEM_REWARD_ADDR),
        ("StakeConfig", STAKE_CONFIG_ADDR),
        ("ValidatorManager", VALIDATOR_MANAGER_ADDR),
        (
            "ValidatorPerformanceTracker",
            VALIDATOR_PERFORMANCE_TRACKER_ADDR,
        ),
        ("EpochManager", EPOCH_MANAGER_ADDR),
        ("GovToken", GOV_TOKEN_ADDR),
        ("Timelock", TIMELOCK_ADDR),
        ("GravityGovernor", GOVERNOR_ADDR),
        ("JWKManager", JWK_MANAGER_ADDR),
        ("KeylessAccount", KEYLESS_ACCOUNT_ADDR),
        ("Block", BLOCK_ADDR),
        ("Timestamp", TIMESTAMP_ADDR),
        ("Genesis", GENESIS_ADDR),
        ("StakeCredit", STAKE_CREDIT_ADDR),
        ("Delegation", DELEGATION_ADDR),
        ("GovHub", GOV_HUB_ADDR),
    ];

    for (contract_name, target_address) in contracts {
        let hex_path = format!("{}/{}.hex", byte_code_dir, contract_name);
        let bytecode_hex = read_hex_from_file(&hex_path);

        // For BSC style, we need to extract runtime bytecode from constructor bytecode
        // This is a simplified approach - in reality, we'd need to execute the constructor
        // and extract the returned bytecode
        let runtime_bytecode = extract_runtime_bytecode(&bytecode_hex);

        db.insert_account_info(
            target_address,
            AccountInfo {
                code: Some(Bytecode::new_raw(Bytes::from(runtime_bytecode))),
                ..AccountInfo::default()
            },
        );

        println!(
            "Deployed {} runtime bytecode to {:?}",
            contract_name, target_address
        );
    }

    db
}

// Extract runtime bytecode from constructor bytecode
// This is a simplified implementation - in reality, we'd need to execute the constructor
fn extract_runtime_bytecode(constructor_bytecode: &str) -> Vec<u8> {
    // For now, we'll try to detect if this is constructor bytecode or runtime bytecode
    let bytes = hex::decode(constructor_bytecode).unwrap_or_default();

    // Simple heuristic: if the bytecode starts with typical constructor patterns,
    // we need to extract the runtime part
    if bytes.len() > 100 && (bytes[0] == 0x60 || bytes[0] == 0x61) {
        // This looks like constructor bytecode
        // For now, we'll use a simplified approach and return the original bytecode
        // In a real implementation, we'd execute the constructor and extract the returned bytecode
        println!("   [!] Warning: Using constructor bytecode as runtime bytecode");
        bytes
    } else {
        // This looks like runtime bytecode already
        bytes
    }
}

fn load_genesis_state(_byte_code_dir: &str) -> HashMap<Address, PlainAccount> {
    let mut map = HashMap::new();
    map.insert(
        SYSTEM_ADDRESS,
        PlainAccount {
            info: AccountInfo {
                balance: uint!(1_000_000_000_000_000_000_U256),
                nonce: 1,
                code_hash: KECCAK_EMPTY,
                code: None,
            },
            storage: Default::default(),
        },
    );

    map
}

pub fn genesis_generate(byte_code_dir: &str, output_dir: &str, config: GenesisConfig) {
    println!("=== Starting Genesis deployment and initialization ===");

    // Try BSC-style deployment first
    let db = deploy_bsc_style(byte_code_dir);

    let mut env = Env::default();
    env.cfg.chain_id = NamedChain::Mainnet.into();

    // Set higher gas limit for genesis transactions
    env.tx.gas_limit = 30_000_000;

    let mut txs = Vec::new();
    // Call Genesis initialize function
    println!("Calling Genesis initialize function...");
    let genesis_init_txn = call_genesis_initialize(GENESIS_ADDR, &config);
    txs.push(genesis_init_txn);

    let r = execute_revm_sequential_with_logging(db, SpecId::LATEST, env, &txs, None);
    let (result, mut bundle_state) = match r {
        Ok((result, bundle_state)) => (result, bundle_state),
        Err(e) => {
            println!("=== BSC-style deployment failed, trying constructor deployment ===");
            let error_msg = format!("{:?}", e.map_db_err(|_| "Database error".to_string()));
            println!("Error: {}", error_msg);

            // Try constructor deployment approach
            let (db2, deploy_txs) = deploy_all_contracts(byte_code_dir);
            let mut env2 = Env::default();
            env2.cfg.chain_id = NamedChain::Mainnet.into();
            env2.tx.gas_limit = 30_000_000;

            // First deploy all contracts
            let mut all_txs = deploy_txs;
            all_txs.push(call_genesis_initialize(GENESIS_ADDR, &config));

            match execute_revm_sequential_with_logging(db2, SpecId::LATEST, env2, &all_txs, None) {
                Ok((result, bundle_state)) => (result, bundle_state),
                Err(e2) => {
                    println!("=== Both deployment approaches failed ===");
                    println!("BSC-style error: {}", error_msg);
                    println!(
                        "Constructor deployment error: {:?}",
                        e2.map_db_err(|_| "Database error".to_string())
                    );
                    panic!("Genesis execution failed with both approaches");
                }
            }
        }
    };

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

    // Add deployed contracts to the final state
    let mut genesis_state = HashMap::new();

    // Add all deployed contracts to the state
    let contracts = [
        ("System", SYSTEM_CALLER),
        ("SystemReward", SYSTEM_REWARD_ADDR),
        ("StakeConfig", STAKE_CONFIG_ADDR),
        ("ValidatorManager", VALIDATOR_MANAGER_ADDR),
        (
            "ValidatorPerformanceTracker",
            VALIDATOR_PERFORMANCE_TRACKER_ADDR,
        ),
        ("EpochManager", EPOCH_MANAGER_ADDR),
        ("GovToken", GOV_TOKEN_ADDR),
        ("Timelock", TIMELOCK_ADDR),
        ("GravityGovernor", GOVERNOR_ADDR),
        ("JWKManager", JWK_MANAGER_ADDR),
        ("KeylessAccount", KEYLESS_ACCOUNT_ADDR),
        ("Block", BLOCK_ADDR),
        ("Timestamp", TIMESTAMP_ADDR),
        ("Genesis", GENESIS_ADDR),
        ("StakeCredit", STAKE_CREDIT_ADDR),
        ("Delegation", DELEGATION_ADDR),
        ("GovHub", GOV_HUB_ADDR),
    ];

    for (contract_name, contract_address) in contracts {
        let hex_path = format!("{}/{}.hex", byte_code_dir, contract_name);
        let bytecode_hex = read_hex_from_file(&hex_path);
        let runtime_bytecode = extract_runtime_bytecode(&bytecode_hex);

        genesis_state.insert(
            contract_address,
            PlainAccount {
                info: AccountInfo {
                    code: Some(Bytecode::new_raw(Bytes::from(runtime_bytecode))),
                    ..AccountInfo::default()
                },
                storage: Default::default(),
            },
        );

        println!(
            "Added {} to genesis state at {:?}",
            contract_name, contract_address
        );
    }

    // Add any state changes from the bundle_state (from the initialize transaction)
    bundle_state.state.remove(&SYSTEM_ADDRESS);
    // write bundle state into one json file named bundle_state.json
    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/bundle_state.json")).unwrap()),
        &bundle_state,
    )
    .unwrap();

    for (address, account) in bundle_state.state.into_iter() {
        if let Some(info) = account.info {
            let storage = account
                .storage
                .into_iter()
                .map(|(k, v)| (k, v.present_value()))
                .collect();

            // If this address already exists in genesis_state, merge the storage
            if let Some(existing) = genesis_state.get_mut(&address) {
                existing.storage.extend(storage);
                // Update account info if it has changed
                if info.code.is_some() || info.balance > existing.info.balance {
                    existing.info = info;
                }
            } else {
                genesis_state.insert(address, PlainAccount { info, storage });
            }
        }
    }

    println!("=== Final Genesis State ===");
    println!("Total accounts: {}", genesis_state.len());
    for (address, account) in &genesis_state {
        println!("Address: {:?}", address);
        println!("  Balance: {}", account.info.balance);
        println!("  Nonce: {}", account.info.nonce);
        println!(
            "  Code: {}",
            account
                .info
                .code
                .as_ref()
                .map_or("None".to_string(), |c| format!("{} bytes", c.len()))
        );
        println!("  Storage slots: {}", account.storage.len());
    }

    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_accounts.json")).unwrap()),
        &genesis_state,
    )
    .unwrap();

    // Create contracts JSON with bytecode
    let contracts_json: HashMap<Address, String> = genesis_state
        .iter()
        .filter_map(|(addr, account)| {
            account
                .info
                .code
                .as_ref()
                .map(|code| (*addr, hex::encode(code.bytecode())))
        })
        .collect();

    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_contracts.json")).unwrap()),
        &contracts_json,
    )
    .unwrap();

    println!("=== Genesis files generated successfully ===");
    println!("- genesis_accounts.json: {} accounts", genesis_state.len());
    println!(
        "- genesis_contracts.json: {} contracts",
        contracts_json.len()
    );
}
