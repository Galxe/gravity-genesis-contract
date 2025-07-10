use crate::utils::{SYSTEM_ADDRESS, new_system_create_txn, read_hex_from_file};
use alloy_primitives::Address;
use alloy_sol_macro::sol;
use revm::primitives::Bytes;
use revm_primitives::TxEnv;

pub fn deploy_system_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
    let hex_path = format!("{}/System.hex", byte_code_dir);
    let system_sol_hex = read_hex_from_file(&hex_path);
    let system_address = SYSTEM_ADDRESS.create(1);
    sol! {
        contract System {
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
            constructor();
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
        contract Groth16Verifier {
            constructor();
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
        contract JWKUtils {
            constructor();
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
        contract Protectable {
            constructor();
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
        contract Bytes {
            constructor();
        }
    }
    let txn = new_system_create_txn(&bytes_sol_hex, revm_primitives::Bytes::default());
    (txn, bytes_address, bytes_sol_hex)
}
