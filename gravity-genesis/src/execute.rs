use crate::storage::InMemoryDB;
use alloy_primitives::address;
use metrics_util::debugging::{DebugValue, DebuggingRecorder};

use alloy_chains::NamedChain;

use alloy_sol_macro::sol;
use alloy_sol_types::{SolCall, SolConstructor, SolError};
use grevm::{ParallelState, ParallelTakeBundle, Scheduler};
use revm::{
    CacheState, DatabaseCommit, DatabaseRef, EvmBuilder, StateBuilder, TransitionState,
    db::{
        AccountRevert, BundleAccount, BundleState, PlainAccount,
        states::{StorageSlot, bundle_state::BundleRetention},
    },
    primitives::{
        AccountInfo, Address, B256, Bytecode, EVMError, Env, ExecutionResult, KECCAK_EMPTY, SpecId,
        TxEnv, U256, alloy_primitives::U160, uint,
    },
};
use revm_primitives::{Bytes, EnvWithHandlerCfg, ResultAndState, TxKind, hex, keccak256};
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    fs::{self, File},
    io::{BufReader, BufWriter},
    sync::Arc,
    time::Instant,
};



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

// 简化的revert跟踪函数
fn analyze_revert_reason(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Revert { gas_used, output } => {
            let mut reason = format!("Revert with gas used: {}", gas_used);
            
            // 尝试解析revert原因
            if let Some(selector) = output.get(0..4) {
                reason.push_str(&format!("\nFunction selector: 0x{}", hex::encode(selector)));
                
                // 检查常见的错误选择器
                match selector {
                    [0x97, 0xb8, 0x83, 0x54] => reason.push_str(" (OnlySystemCaller)"),
                    [0x0a, 0x5a, 0x60, 0x41] => reason.push_str(" (UnknownParam)"),
                    [0x11, 0x6c, 0x64, 0xa8] => reason.push_str(" (InvalidValue)"),
                    [0x83, 0xf1, 0xb1, 0xd3] => reason.push_str(" (OnlyCoinbase)"),
                    [0xf2, 0x2c, 0x43, 0x90] => reason.push_str(" (OnlyZeroGasPrice)"),
                    _ => reason.push_str(" (Unknown error selector)"),
                }
            }
            
            if output.len() > 4 {
                reason.push_str(&format!("\nAdditional data: 0x{}", hex::encode(&output[4..])));
            }
            
            reason
        }
        ExecutionResult::Success { gas_used, .. } => {
            format!("Success with gas used: {}", gas_used)
        }
        ExecutionResult::Halt { reason, gas_used } => {
            format!("Halt: {:?} with gas used: {}", reason, gas_used)
        }
    }
}

pub const MINER_ADDRESS: usize = 999;

/// Simulate the sequential execution of transactions in reth
pub(crate) fn execute_revm_sequential<DB>(
    db: DB,
    spec_id: SpecId,
    env: Env,
    txs: &[TxEnv],
) -> Result<(Vec<ExecutionResult>, BundleState), EVMError<DB::Error>>
where
    DB: DatabaseRef,
    DB::Error: Debug,
{
    let db = StateBuilder::new()
        .with_bundle_update()
        .with_database_ref(db)
        .build();
    let mut evm = EvmBuilder::default()
        .with_db(db)
        .with_spec_id(spec_id)
        .with_env(Box::new(env))
        .build();

    let mut results = Vec::with_capacity(txs.len());
    for tx in txs {
        *evm.tx_mut() = tx.clone();
        let result_and_state = evm.transact()?;
        evm.db_mut().commit(result_and_state.state);
        results.push(result_and_state.result);
    }
    evm.db_mut().merge_transitions(BundleRetention::Reverts);

    Ok((results, evm.db_mut().take_bundle()))
}

/// Simulate the sequential execution of transactions with detailed logging
pub(crate) fn execute_revm_sequential_with_logging<DB>(
    db: DB,
    spec_id: SpecId,
    env: Env,
    txs: &[TxEnv],
) -> Result<(Vec<ExecutionResult>, BundleState), EVMError<DB::Error>>
where
    DB: DatabaseRef,
    DB::Error: Debug,
{
    let db = StateBuilder::new()
        .with_bundle_update()
        .with_database_ref(db)
        .build();
    let mut evm = EvmBuilder::default()
        .with_db(db)
        .with_spec_id(spec_id)
        .with_env(Box::new(env))
        .build();

    let mut results = Vec::with_capacity(txs.len());
    for (i, tx) in txs.iter().enumerate() {
        println!("=== Executing transaction {} ===", i + 1);
        println!("Transaction details:");
        println!("  Caller: {:?}", tx.caller);
        println!("  To: {:?}", tx.transact_to);
        println!("  Data length: {}", tx.data.len());
        if tx.data.len() >= 4 {
            println!("  Function selector: 0x{}", hex::encode(&tx.data[0..4]));
        }
        
        *evm.tx_mut() = tx.clone();
        let result_and_state = evm.transact()?;
        evm.db_mut().commit(result_and_state.state);
        
        println!("Transaction result: {}", analyze_revert_reason(&result_and_state.result));
        results.push(result_and_state.result);
        println!("=== Transaction {} completed ===", i + 1);
    }
    evm.db_mut().merge_transitions(BundleRetention::Reverts);

    Ok((results, evm.db_mut().take_bundle()))
}

// const SYSTEM_ADDRESS: Address = address!("00000000000000000000000000000000000000ff");
const SYSTEM_ADDRESS: Address = address!("0000000000000000000000000000000000000000");
const RECONF_ADDRESS: Address = address!("00000000000000000000000000000000000000f0");
const BLOCK_ADDRESS: Address = address!("00000000000000000000000000000000000000f1");
const CONSENSUS_CONFIG_ADDRESS: Address = address!("00000000000000000000000000000000000000f2");
const VALIDATOR_SET_ADDRESS: Address = address!("00000000000000000000000000000000000000f3");

fn new_system_call_txn(contract: Address, input: Bytes) -> TxEnv {
    TxEnv {
        caller: SYSTEM_ADDRESS,
        gas_limit: 30_000_000,
        gas_price: U256::ZERO,
        transact_to: TxKind::Call(contract),
        value: U256::ZERO,
        data: input,
        ..Default::default()
    }
}

fn new_system_create_txn(hex_code: &str, args: Bytes) -> TxEnv {
    let mut data = hex::decode(hex_code).expect("Invalid hex string");
    data.extend_from_slice(&args);
    TxEnv {
        caller: SYSTEM_ADDRESS,
        gas_limit: 30_000_000,
        gas_price: U256::ZERO,
        transact_to: TxKind::Create,
        value: U256::ZERO,
        data: data.into(),
        ..Default::default()
    }
}

fn read_hex_from_file(path: &str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to open {}", path))
}

fn deploy_system_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_system_reward_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_stake_config_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_validator_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_validator_performance_tracker_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_epoch_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_gov_token_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_timelock_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_gravity_governor_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_jwk_manager_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_keyless_account_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_block_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_timestamp_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_genesis_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_stake_credit_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_delegation_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_gov_hub_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_groth16_verifier_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_jwk_utils_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_protectable_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn deploy_bytes_contract(byte_code_dir: &str) -> (TxEnv, Address, String) {
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

fn match_execution_revert_reason(r: &ExecutionResult) -> String {
    sol! {
        error OnlySystemCaller();
        // @notice signature: 0x97b88354
        error UnknownParam(string key, bytes value);
        // @notice signature: 0x0a5a6041
        error InvalidValue(string key, bytes value);
        // @notice signature: 0x116c64a8
        error OnlyCoinbase();
        // @notice signature: 0x83f1b1d3
        error OnlyZeroGasPrice();
        // @notice signature: 0xf22c4390
        error OnlySystemContract(address systemContract);
    }
    match r {
        ExecutionResult::Revert { gas_used, output } => match output.as_ref() {
            b if b == OnlySystemCaller::SELECTOR => {
                return "Revert Reason: OnlySystemCaller()".to_string();
            }
            b if b == UnknownParam::SELECTOR => {
                return "Revert Reason: UnknownParam()".to_string();
            }
            b if b == InvalidValue::SELECTOR => {
                return "Revert Reason: InvalidValue()".to_string();
            }
            b if b == OnlyCoinbase::SELECTOR => {
                return "Revert Reason: OnlyCoinbase()".to_string();
            }
            b if b == OnlyZeroGasPrice::SELECTOR => {
                return "Revert Reason: OnlyZeroGasPrice()".to_string();
            }
            b if b == OnlySystemContract::SELECTOR => {
                return "Revert Reason: OnlySystemContract()".to_string();
            }
            _ => {
                return format!("Unknown revert reason: {:?}", output);
            }
        },
        _ => "Unknown revert reason".to_string(),
    }
}

pub fn genesis_generate(byte_code_dir: &str, output_dir: &str, config: GenesisConfig) {
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

    // 1. 部署 System 合约
    let (system_txn, system_address, _) = deploy_system_contract(byte_code_dir);
    println!("System contract address: {:?}", system_address);
    txs.push(system_txn);

    // 2. 部署 SystemReward 合约
    let (system_reward_txn, system_reward_address, _) =
        deploy_system_reward_contract(byte_code_dir);
    println!("SystemReward contract address: {:?}", system_reward_address);
    txs.push(system_reward_txn);

    // 3. 部署 StakeConfig 合约
    let (stake_config_txn, stake_config_address, _) = deploy_stake_config_contract(byte_code_dir);
    println!("StakeConfig contract address: {:?}", stake_config_address);
    txs.push(stake_config_txn);

    // 4. 部署 ValidatorManager 合约
    let (validator_manager_txn, validator_manager_address, _) =
        deploy_validator_manager_contract(byte_code_dir);
    println!(
        "ValidatorManager contract address: {:?}",
        validator_manager_address
    );
    txs.push(validator_manager_txn);

    // 5. 部署 ValidatorPerformanceTracker 合约
    let (validator_performance_tracker_txn, validator_performance_tracker_address, _) =
        deploy_validator_performance_tracker_contract(byte_code_dir);
    println!(
        "ValidatorPerformanceTracker contract address: {:?}",
        validator_performance_tracker_address
    );
    txs.push(validator_performance_tracker_txn);

    // 6. 部署 EpochManager 合约
    let (epoch_manager_txn, epoch_manager_address, _) =
        deploy_epoch_manager_contract(byte_code_dir);
    println!("EpochManager contract address: {:?}", epoch_manager_address);
    txs.push(epoch_manager_txn);

    // 7. 部署 GovToken 合约
    let (gov_token_txn, gov_token_address, _) = deploy_gov_token_contract(byte_code_dir);
    println!("GovToken contract address: {:?}", gov_token_address);
    txs.push(gov_token_txn);

    // 8. 部署 Timelock 合约
    let (timelock_txn, timelock_address, _) = deploy_timelock_contract(byte_code_dir);
    println!("Timelock contract address: {:?}", timelock_address);
    txs.push(timelock_txn);

    // 9. 部署 GravityGovernor 合约
    let (gravity_governor_txn, gravity_governor_address, _) =
        deploy_gravity_governor_contract(byte_code_dir);
    println!(
        "GravityGovernor contract address: {:?}",
        gravity_governor_address
    );
    txs.push(gravity_governor_txn);

    // 10. 部署 JWKManager 合约
    let (jwk_manager_txn, jwk_manager_address, _) = deploy_jwk_manager_contract(byte_code_dir);
    println!("JWKManager contract address: {:?}", jwk_manager_address);
    txs.push(jwk_manager_txn);

    // 11. 部署 KeylessAccount 合约
    let (keyless_account_txn, keyless_account_address, _) =
        deploy_keyless_account_contract(byte_code_dir);
    println!(
        "KeylessAccount contract address: {:?}",
        keyless_account_address
    );
    txs.push(keyless_account_txn);

    // 12. 部署 Block 合约
    let (block_txn, block_address, _) = deploy_block_contract(byte_code_dir);
    println!("Block contract address: {:?}", block_address);
    txs.push(block_txn);

    // 13. 部署 Timestamp 合约
    let (timestamp_txn, timestamp_address, _) = deploy_timestamp_contract(byte_code_dir);
    println!("Timestamp contract address: {:?}", timestamp_address);
    txs.push(timestamp_txn);

    // 14. 部署 Genesis 合约
    let (genesis_txn, genesis_address, _) = deploy_genesis_contract(byte_code_dir);
    println!("Genesis contract address: {:?}", genesis_address);
    txs.push(genesis_txn);

    // 15. 部署 StakeCredit 合约
    let (stake_credit_txn, stake_credit_address, _) = deploy_stake_credit_contract(byte_code_dir);
    println!("StakeCredit contract address: {:?}", stake_credit_address);
    txs.push(stake_credit_txn);

    // 16. 部署 Delegation 合约
    let (delegation_txn, delegation_address, _) = deploy_delegation_contract(byte_code_dir);
    println!("Delegation contract address: {:?}", delegation_address);
    txs.push(delegation_txn);

    // 17. 部署 GovHub 合约
    let (gov_hub_txn, gov_hub_address, _) = deploy_gov_hub_contract(byte_code_dir);
    println!("GovHub contract address: {:?}", gov_hub_address);
    txs.push(gov_hub_txn);

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

    // 调用Genesis初始化函数
    println!("Calling Genesis initialize function...");
    let genesis_init_txn = call_genesis_initialize(genesis_address, &config);
    txs.push(genesis_init_txn);

    // 执行所有交易（包括部署和初始化）
    println!("=== Starting Genesis deployment and initialization ===");
    let (result, mut bundle_state) =
        execute_revm_sequential_with_logging(db, SpecId::LATEST, env, &txs).unwrap();
    let mut success_count = 0;
    for (i, r) in result.iter().enumerate() {
        if !r.is_success() {
            println!("=== Transaction {} failed ===", i + 1);
            println!("Detailed analysis: {}", analyze_revert_reason(r));
            println!("Revert reason: {}", match_execution_revert_reason(r));
            panic!("Genesis transaction {} failed", i + 1);
        }
        println!("Transaction {}: {}", i + 1, analyze_revert_reason(r));
        success_count += 1;
    }
    println!("=== All {} transactions completed successfully ===", success_count);
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
        BufWriter::new(
            File::create(format!("{output_dir}/gravity/genesis_accounts.json")).unwrap(),
        ),
        &genesis_state,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        BufWriter::new(
            File::create(format!("{output_dir}/gravity/genesis_contracts.json")).unwrap(),
        ),
        &bundle_state
            .contracts
            .iter()
            .map(|(k, v)| (k, v.bytecode()))
            .collect::<HashMap<_, _>>(),
    )
    .unwrap();
}
