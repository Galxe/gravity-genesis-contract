use std::collections::HashMap;

use crate::{storage::InMemoryDB, utils::*};
use alloy_chains::NamedChain;
use alloy_primitives::Address;
use alloy_sol_macro::sol;
// use alloy_contract::SolCallBuilder;
use revm::{db::{BundleState, PlainAccount}, primitives::Bytes};
use revm_primitives::{AccountInfo, Env, KECCAK_EMPTY, SpecId, TxEnv, uint};

pub fn deploy_system_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/System.hex", byte_code_dir);
    let system_sol_hex = read_hex_from_file(&hex_path);
    let system_address = SYSTEM_ADDRESS.create(1);
    sol! {
        contract System {
        }
    }
    let txn = new_system_create_txn(&system_sol_hex, Bytes::default());
    (txn, system_address, system_sol_hex)
}

pub fn deploy_system_reward_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/SystemReward.hex", byte_code_dir);
    let system_reward_sol_hex = read_hex_from_file(&hex_path);
    let system_reward_address = SYSTEM_ADDRESS.create(2);
    sol! {
        contract SystemReward {
        }
    }
    let txn = new_system_create_txn(&system_reward_sol_hex, Bytes::default());
    (txn, system_reward_address, system_reward_sol_hex)
}

pub fn deploy_stake_config_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/StakeConfig.hex", byte_code_dir);
    let stake_config_sol_hex = read_hex_from_file(&hex_path);
    let stake_config_address = SYSTEM_ADDRESS.create(3);
    sol! {
        contract StakeConfig {
            constructor();
        }
    }
    let txn = new_system_create_txn(&stake_config_sol_hex, Bytes::default());
    (txn, stake_config_address, stake_config_sol_hex)
}

pub fn deploy_validator_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/ValidatorManager.hex", byte_code_dir);
    let validator_manager_sol_hex = read_hex_from_file(&hex_path);
    let validator_manager_address = SYSTEM_ADDRESS.create(4);
    sol! {
        contract ValidatorManager {
        }
    }
    let txn = new_system_create_txn(&validator_manager_sol_hex, Bytes::default());
    (txn, validator_manager_address, validator_manager_sol_hex)
}

pub fn deploy_validator_performance_tracker_contract(
    byte_code_dir: &str,
) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/ValidatorPerformanceTracker.hex", byte_code_dir);
    let validator_performance_tracker_sol_hex = read_hex_from_file(&hex_path);
    let validator_performance_tracker_address = SYSTEM_ADDRESS.create(5);
    sol! {
        contract ValidatorPerformanceTracker {
        }
    }
    let txn = new_system_create_txn(&validator_performance_tracker_sol_hex, Bytes::default());
    (
        txn,
        validator_performance_tracker_address,
        validator_performance_tracker_sol_hex,
    )
}

pub fn deploy_epoch_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/EpochManager.hex", byte_code_dir);
    let epoch_manager_sol_hex = read_hex_from_file(&hex_path);
    let epoch_manager_address = SYSTEM_ADDRESS.create(6);
    sol! {
        contract EpochManager {
            constructor();
        }
    }
    let txn = new_system_create_txn(&epoch_manager_sol_hex, Bytes::default());
    (txn, epoch_manager_address, epoch_manager_sol_hex)
}

pub fn deploy_gov_token_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/GovToken.hex", byte_code_dir);
    let gov_token_sol_hex = read_hex_from_file(&hex_path);
    let gov_token_address = SYSTEM_ADDRESS.create(7);
    sol! {
        contract GovToken {
        }
    }
    let txn = new_system_create_txn(&gov_token_sol_hex, Bytes::default());
    (txn, gov_token_address, gov_token_sol_hex)
}

pub fn deploy_timelock_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Timelock.hex", byte_code_dir);
    let timelock_sol_hex = read_hex_from_file(&hex_path);
    let timelock_address = SYSTEM_ADDRESS.create(8);
    sol! {
        contract Timelock {
        }
    }
    let txn = new_system_create_txn(&timelock_sol_hex, Bytes::default());
    (txn, timelock_address, timelock_sol_hex)
}

pub fn deploy_gravity_governor_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/GravityGovernor.hex", byte_code_dir);
    let gravity_governor_sol_hex = read_hex_from_file(&hex_path);
    let gravity_governor_address = SYSTEM_ADDRESS.create(9);
    sol! {
        contract GravityGovernor {
        }
    }
    let txn = new_system_create_txn(&gravity_governor_sol_hex, Bytes::default());
    (txn, gravity_governor_address, gravity_governor_sol_hex)
}

pub fn deploy_jwk_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/JWKManager.hex", byte_code_dir);
    let jwk_manager_sol_hex = read_hex_from_file(&hex_path);
    let jwk_manager_address = SYSTEM_ADDRESS.create(10);
    sol! {
        contract JWKManager {
            constructor();
        }
    }
    let txn = new_system_create_txn(&jwk_manager_sol_hex, Bytes::default());
    (txn, jwk_manager_address, jwk_manager_sol_hex)
}

pub fn deploy_keyless_account_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/KeylessAccount.hex", byte_code_dir);
    let keyless_account_sol_hex = read_hex_from_file(&hex_path);
    let keyless_account_address = SYSTEM_ADDRESS.create(11);
    sol! {
        contract KeylessAccount {
            constructor();
        }
    }
    let txn = new_system_create_txn(&keyless_account_sol_hex, Bytes::default());
    (txn, keyless_account_address, keyless_account_sol_hex)
}

pub fn deploy_block_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Block.hex", byte_code_dir);
    let block_sol_hex = read_hex_from_file(&hex_path);
    let block_address = SYSTEM_ADDRESS.create(12);
    sol! {
        contract Block {
        }
    }
    let txn = new_system_create_txn(&block_sol_hex, Bytes::default());
    (txn, block_address, block_sol_hex)
}

pub fn deploy_timestamp_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Timestamp.hex", byte_code_dir);
    let timestamp_sol_hex = read_hex_from_file(&hex_path);
    let timestamp_address = SYSTEM_ADDRESS.create(13);
    sol! {
        contract Timestamp {
        }
    }
    let txn = new_system_create_txn(&timestamp_sol_hex, Bytes::default());
    (txn, timestamp_address, timestamp_sol_hex)
}

pub fn deploy_genesis_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Genesis.hex", byte_code_dir);
    let genesis_sol_hex = read_hex_from_file(&hex_path);
    let genesis_address = SYSTEM_ADDRESS.create(14);
    sol! {
        contract Genesis {
        }
    }
    let txn = new_system_create_txn(&genesis_sol_hex, Bytes::default());
    (txn, genesis_address, genesis_sol_hex)
}

pub fn deploy_stake_credit_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/StakeCredit.hex", byte_code_dir);
    let stake_credit_sol_hex = read_hex_from_file(&hex_path);
    let stake_credit_address = SYSTEM_ADDRESS.create(15);
    sol! {
        contract StakeCredit {
            constructor();
        }
    }
    let txn = new_system_create_txn(&stake_credit_sol_hex, Bytes::default());
    (txn, stake_credit_address, stake_credit_sol_hex)
}

pub fn deploy_delegation_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Delegation.hex", byte_code_dir);
    let delegation_sol_hex = read_hex_from_file(&hex_path);
    let delegation_address = SYSTEM_ADDRESS.create(16);
    sol! {
        contract Delegation {
        }
    }
    let txn = new_system_create_txn(&delegation_sol_hex, Bytes::default());
    (txn, delegation_address, delegation_sol_hex)
}

pub fn deploy_gov_hub_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/GovHub.hex", byte_code_dir);
    let gov_hub_sol_hex = read_hex_from_file(&hex_path);
    let gov_hub_address = SYSTEM_ADDRESS.create(17);
    sol! {
        contract GovHub {
        }
    }
    let txn = new_system_create_txn(&gov_hub_sol_hex, Bytes::default());
    (txn, gov_hub_address, gov_hub_sol_hex)
}

pub fn deploy_groth16_verifier_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Groth16Verifier.hex", byte_code_dir);
    let groth16_verifier_sol_hex = read_hex_from_file(&hex_path);
    let groth16_verifier_address = SYSTEM_ADDRESS.create(18);
    sol! {
        // #[sol(rpc)]
        // #[sol(bytecode = groth16_verifier_sol_hex)]
        contract Groth16Verifier {
        }
    }
    
    let txn = new_system_create_txn(&groth16_verifier_sol_hex, Bytes::default());
    (txn, groth16_verifier_address, groth16_verifier_sol_hex)
}

pub fn deploy_jwk_utils_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/JWKUtils.hex", byte_code_dir);
    let jwk_utils_sol_hex = read_hex_from_file(&hex_path);
    let jwk_utils_address = SYSTEM_ADDRESS.create(19);
    sol! {
        library JWKUtils {
        }
    }
    let txn = new_system_create_txn(&jwk_utils_sol_hex, Bytes::default());
    (txn, jwk_utils_address, jwk_utils_sol_hex)
}

pub fn deploy_protectable_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Protectable.hex", byte_code_dir);
    let protectable_sol_hex = read_hex_from_file(&hex_path);
    let protectable_address = SYSTEM_ADDRESS.create(20);
    sol! {
        abstract contract Protectable {
        }
    }
    let txn = new_system_create_txn(&protectable_sol_hex, Bytes::default());
    (txn, protectable_address, protectable_sol_hex)
}

pub fn deploy_bytes_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/Bytes.hex", byte_code_dir);
    let bytes_sol_hex = read_hex_from_file(&hex_path);
    let bytes_address = SYSTEM_ADDRESS.create(21);
    sol! {
        library Bytes {
        }
    }
    let txn = new_system_create_txn(&bytes_sol_hex, revm_primitives::Bytes::default());
    (txn, bytes_address, bytes_sol_hex)
}

pub fn deploy_and_constrcut_all(byte_code_dir: &str) -> BundleState {
    let mut env = Env::default();
    env.cfg.chain_id = NamedChain::Mainnet.into();
    let db = InMemoryDB::new(
        HashMap::from([(
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
        )]),
        Default::default(),
        Default::default(),
    );

    let mut txs = Vec::new();
    let mut addr_map = HashMap::new();

    // 1. 部署 System 合约
    let (system_txn, system_address, _) = deploy_system_contract(byte_code_dir);
    println!("System contract address: {:?}", system_address);
    txs.push(system_txn);
    addr_map.insert(system_address, SYSTEM_CALLER);

    // 2. 部署 SystemReward 合约
    let (system_reward_txn, system_reward_address, _) =
        deploy_system_reward_contract(byte_code_dir);
    println!("SystemReward contract address: {:?}", system_reward_address);
    txs.push(system_reward_txn);
    addr_map.insert(system_reward_address, SYSTEM_REWARD_ADDR);

    // 3. 部署 StakeConfig 合约
    let (stake_config_txn, stake_config_address, _) = deploy_stake_config_contract(byte_code_dir);
    println!("StakeConfig contract address: {:?}", stake_config_address);
    txs.push(stake_config_txn);
    addr_map.insert(stake_config_address, STAKE_CONFIG_ADDR);

    // 4. 部署 ValidatorManager 合约
    let (validator_manager_txn, validator_manager_address, _) =
        deploy_validator_manager_contract(byte_code_dir);
    println!(
        "ValidatorManager contract address: {:?}",
        validator_manager_address
    );
    txs.push(validator_manager_txn);
    addr_map.insert(validator_manager_address, VALIDATOR_MANAGER_ADDR);

    // 5. 部署 ValidatorPerformanceTracker 合约
    let (validator_performance_tracker_txn, validator_performance_tracker_address, _) =
        deploy_validator_performance_tracker_contract(byte_code_dir);
    println!(
        "ValidatorPerformanceTracker contract address: {:?}",
        validator_performance_tracker_address
    );
    txs.push(validator_performance_tracker_txn);
    addr_map.insert(
        validator_performance_tracker_address,
        VALIDATOR_PERFORMANCE_TRACKER_ADDR,
    );

    // 6. 部署 EpochManager 合约
    let (epoch_manager_txn, epoch_manager_address, _) =
        deploy_epoch_manager_contract(byte_code_dir);
    println!("EpochManager contract address: {:?}", epoch_manager_address);
    txs.push(epoch_manager_txn);
    addr_map.insert(epoch_manager_address, EPOCH_MANAGER_ADDR);

    // 7. 部署 GovToken 合约
    let (gov_token_txn, gov_token_address, _) = deploy_gov_token_contract(byte_code_dir);
    println!("GovToken contract address: {:?}", gov_token_address);
    txs.push(gov_token_txn);
    addr_map.insert(gov_token_address, GOV_TOKEN_ADDR);

    // 8. 部署 Timelock 合约
    let (timelock_txn, timelock_address, _) = deploy_timelock_contract(byte_code_dir);
    println!("Timelock contract address: {:?}", timelock_address);
    txs.push(timelock_txn);
    addr_map.insert(timelock_address, TIMELOCK_ADDR);

    // 9. 部署 GravityGovernor 合约
    let (gravity_governor_txn, gravity_governor_address, _) =
        deploy_gravity_governor_contract(byte_code_dir);
    println!(
        "GravityGovernor contract address: {:?}",
        gravity_governor_address
    );
    txs.push(gravity_governor_txn);
    addr_map.insert(gravity_governor_address, GOVERNOR_ADDR);

    // 10. 部署 JWKManager 合约
    let (jwk_manager_txn, jwk_manager_address, _) = deploy_jwk_manager_contract(byte_code_dir);
    println!("JWKManager contract address: {:?}", jwk_manager_address);
    txs.push(jwk_manager_txn);
    addr_map.insert(jwk_manager_address, JWK_MANAGER_ADDR);

    // 11. 部署 KeylessAccount 合约
    let (keyless_account_txn, keyless_account_address, _) =
        deploy_keyless_account_contract(byte_code_dir);
    println!(
        "KeylessAccount contract address: {:?}",
        keyless_account_address
    );
    txs.push(keyless_account_txn);
    addr_map.insert(keyless_account_address, KEYLESS_ACCOUNT_ADDR);

    // 12. 部署 Block 合约
    let (block_txn, block_address, _) = deploy_block_contract(byte_code_dir);
    println!("Block contract address: {:?}", block_address);
    txs.push(block_txn);
    addr_map.insert(block_address, BLOCK_ADDR);

    // 13. 部署 Timestamp 合约
    let (timestamp_txn, timestamp_address, _) = deploy_timestamp_contract(byte_code_dir);
    println!("Timestamp contract address: {:?}", timestamp_address);
    txs.push(timestamp_txn);
    addr_map.insert(timestamp_address, TIMESTAMP_ADDR);

    // 14. 部署 Genesis 合约
    let (genesis_txn, genesis_address, _) = deploy_genesis_contract(byte_code_dir);
    println!("Genesis contract address: {:?}", genesis_address);
    txs.push(genesis_txn);
    addr_map.insert(genesis_address, GENESIS_ADDR);

    // 15. 部署 StakeCredit 合约
    let (stake_credit_txn, stake_credit_address, _) = deploy_stake_credit_contract(byte_code_dir);
    println!("StakeCredit contract address: {:?}", stake_credit_address);
    txs.push(stake_credit_txn);
    addr_map.insert(stake_credit_address, STAKE_CREDIT_ADDR);

    // 16. 部署 Delegation 合约
    let (delegation_txn, delegation_address, _) = deploy_delegation_contract(byte_code_dir);
    println!("Delegation contract address: {:?}", delegation_address);
    txs.push(delegation_txn);
    addr_map.insert(delegation_address, DELEGATION_ADDR);

    // 17. 部署 GovHub 合约
    let (gov_hub_txn, gov_hub_address, _) = deploy_gov_hub_contract(byte_code_dir);
    println!("GovHub contract address: {:?}", gov_hub_address);
    txs.push(gov_hub_txn);
    addr_map.insert(gov_hub_address, GOV_HUB_ADDR);

    // 18. 部署 Groth16Verifier 合约
    let (groth16_verifier_txn, groth16_verifier_address, _) =
        deploy_groth16_verifier_contract(byte_code_dir);
    println!(
        "Groth16Verifier contract address: {:?}",
        groth16_verifier_address
    );
    txs.push(groth16_verifier_txn);

    // 19. 部署 JWKUtils 合约
    let (jwk_utils_txn, jwk_utils_address, _) = deploy_jwk_utils_contract(byte_code_dir);
    println!("JWKUtils contract address: {:?}", jwk_utils_address);
    txs.push(jwk_utils_txn);

    // 20. 部署 Protectable 合约
    let (protectable_txn, protectable_address, _) = deploy_protectable_contract(byte_code_dir);
    println!("Protectable contract address: {:?}", protectable_address);
    txs.push(protectable_txn);

    // 21. 部署 Bytes 合约
    let (bytes_txn, bytes_address, _) = deploy_bytes_contract(byte_code_dir);
    println!("Bytes contract address: {:?}", bytes_address);
    txs.push(bytes_txn);

    // 执行所有交易（包括部署和初始化）
    println!("=== Starting deployment and initialization ===");
    let (result, mut bundle_state) =
        execute_revm_sequential_with_logging(db, SpecId::LATEST, env, &txs, None).unwrap();
    let mut success_count = 0;
    for (i, r) in result.iter().enumerate() {
        if !r.is_success() {
            println!("=== Transaction {} failed ===", i + 1);
            println!("Detailed analysis: {}", analyze_revert_reason(r));
            panic!("transaction {} failed", i + 1);
        }
        println!("Transaction {}: {}", i + 1, analyze_revert_reason(r));
        success_count += 1;
    }
    println!(
        "=== All {} transactions completed successfully ===",
        success_count
    );
    bundle_state.state.remove(&SYSTEM_ADDRESS);
    let state = bundle_state
        .state
        .into_iter()
        .map(|(address, account)| {
            if let Some(addr) = addr_map.get(&address) {
                (addr.clone(), account)
            } else {
                (address, account)
            }
        })
        .collect();
    bundle_state.state = state;
    bundle_state
}
