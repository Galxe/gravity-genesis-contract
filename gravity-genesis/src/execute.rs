use crate::{
    genesis::{
        GenesisConfig, call_genesis_initialize, call_get_validator_set, print_validator_set_result,
    },
    utils::{
        CONTRACTS, GENESIS_ADDR, SYSTEM_ACCOUNT_INFO, SYSTEM_ADDRESS, analyze_txn_result,
        execute_revm_sequential, read_hex_from_file,
    },
};

use alloy_chains::NamedChain;

use revm::{
    DatabaseRef, InMemoryDB, StateBuilder,
    db::{CacheDB, PlainAccount},
    primitives::{AccountInfo, Env, SpecId},
};
use revm_primitives::{Bytecode, Bytes, hex};
use std::{collections::HashMap, fs::File, io::BufWriter};
use tracing::{debug, error, info, warn};

// Alternative approach: Use BSC-style direct bytecode deployment
fn deploy_bsc_style(byte_code_dir: &str) -> impl DatabaseRef {
    let revm_db = InMemoryDB::default();
    let mut db = CacheDB::new(revm_db);

    // Add system address with balance
    db.insert_account_info(SYSTEM_ADDRESS, SYSTEM_ACCOUNT_INFO);

    for (contract_name, target_address) in CONTRACTS {
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

        info!(
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
        warn!("   [!] Warning: Using constructor bytecode as runtime bytecode");
        bytes
    } else {
        // This looks like runtime bytecode already
        bytes
    }
}

pub fn genesis_generate(byte_code_dir: &str, output_dir: &str, config: GenesisConfig) {
    info!("=== Starting Genesis deployment and initialization ===");

    // Try BSC-style deployment first
    let mut db = deploy_bsc_style(byte_code_dir);
    let mut wrapped_db = StateBuilder::new().with_database_ref(db).build();

    let mut env = Env::default();
    env.cfg.chain_id = NamedChain::Mainnet.into();

    // Set higher gas limit for genesis transactions
    env.tx.gas_limit = 30_000_000;

    let mut txs = Vec::new();
    // Call Genesis initialize function
    info!("Calling Genesis initialize function...");
    let genesis_init_txn = call_genesis_initialize(GENESIS_ADDR, &config);
    txs.push(genesis_init_txn);
    let get_validator_set_txn = call_get_validator_set();
    txs.push(get_validator_set_txn);

    let r = execute_revm_sequential(&mut wrapped_db, SpecId::LATEST, env.clone(), &txs);
    let (result, mut bundle_state) = match r {
        Ok((result, bundle_state)) => {
            info!("=== Genesis initialization successful ===");

            if let Some(validator_set_result) = result.last() {
                print_validator_set_result(validator_set_result, &config);
            }

            (result, bundle_state)
        }
        Err(e) => {
            let error_msg = format!("{:?}", e.map_db_err(|_| "Database error".to_string()));
            panic!("Error: {}", error_msg);
        }
    };

    info!("=== Bundle state ===");
    info!("{:?}", bundle_state);

    let mut success_count = 0;
    for (i, r) in result.iter().enumerate() {
        if !r.is_success() {
            error!("=== Transaction {} failed ===", i + 1);
            info!("Detailed analysis: {}", analyze_txn_result(r));
            panic!("Genesis transaction {} failed", i + 1);
        } else {
            info!("Transaction {}: succeed", i + 1);
            info!("Detailed analysis: {}", analyze_txn_result(r));
        }
        success_count += 1;
    }
    info!(
        "=== All {} transactions completed successfully ===",
        success_count
    );

    // Add deployed contracts to the final state
    let mut genesis_state = HashMap::new();

    for (contract_name, contract_address) in CONTRACTS {
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

        info!(
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
        debug!("Address: {:?}, account: {:?}", address, account);
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

    info!("=== Final Genesis State ===");
    info!("Total accounts: {}", genesis_state.len());
    for (address, account) in &genesis_state {
        debug!("Address: {:?}", address);
        debug!("  Balance: {}", account.info.balance);
        debug!("  Nonce: {}", account.info.nonce);
        debug!(
            "  Code: {}",
            account
                .info
                .code
                .as_ref()
                .map_or("None".to_string(), |c| format!("{} bytes", c.len()))
        );
        debug!("  Storage slots: {}", account.storage.len());
    }

    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_accounts.json")).unwrap()),
        &genesis_state,
    )
    .unwrap();

    // Create contracts JSON with bytecode
    let contracts_json: HashMap<_, _> = genesis_state
        .iter()
        .filter_map(|(addr, account)| {
            account
                .info
                .code
                .as_ref()
                .map(|code| (*addr, code.bytecode()))
        })
        .collect();

    serde_json::to_writer_pretty(
        BufWriter::new(File::create(format!("{output_dir}/genesis_contracts.json")).unwrap()),
        &contracts_json,
    )
    .unwrap();

    info!("=== Genesis files generated successfully ===");
    info!("- genesis_accounts.json: {} accounts", genesis_state.len());
    info!(
        "- genesis_contracts.json: {} contracts",
        contracts_json.len()
    );
}
