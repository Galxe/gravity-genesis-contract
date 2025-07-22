use alloy_primitives::address;

use revm::{
    db::{states::bundle_state::BundleRetention, BundleState, State}, inspector_handle_register, inspectors::TracerEip3155, primitives::{Address, EVMError, Env, ExecutionResult, SpecId, TxEnv, U256}, Database, DatabaseCommit, DatabaseRef, EvmBuilder, StateBuilder
};
use revm_primitives::{Bytes, TxKind, hex};
use std::u64;
use tracing::info;

pub const DEAD_ADDRESS: Address = address!("000000000000000000000000000000000000dEaD");
pub const GENESIS_ADDR: Address = address!("0000000000000000000000000000000000001008");
pub const SYSTEM_CALLER: Address = address!("00000000000000000000000000000000000000ff");
pub const PERFORMANCE_TRACKER_ADDR: Address = address!("00000000000000000000000000000000000000f1");
pub const EPOCH_MANAGER_ADDR: Address = address!("00000000000000000000000000000000000000f3");
pub const STAKE_CONFIG_ADDR: Address = address!("0000000000000000000000000000000000002008");
pub const DELEGATION_ADDR: Address = address!("0000000000000000000000000000000000002009");
pub const VALIDATOR_MANAGER_ADDR: Address = address!("0000000000000000000000000000000000002010");
pub const VALIDATOR_MANAGER_UTILS_ADDR: Address =
    address!("000000000000000000000000000000000000200c");
pub const VALIDATOR_PERFORMANCE_TRACKER_ADDR: Address =
    address!("000000000000000000000000000000000000200b");
pub const BLOCK_ADDR: Address = address!("0000000000000000000000000000000000002001");
pub const TIMESTAMP_ADDR: Address = address!("0000000000000000000000000000000000002004");
pub const JWK_MANAGER_ADDR: Address = address!("0000000000000000000000000000000000002002");
pub const KEYLESS_ACCOUNT_ADDR: Address = address!("000000000000000000000000000000000000200A");
pub const SYSTEM_REWARD_ADDR: Address = address!("0000000000000000000000000000000000001002");
pub const GOV_HUB_ADDR: Address = address!("0000000000000000000000000000000000001007");
pub const STAKE_CREDIT_ADDR: Address = address!("0000000000000000000000000000000000002003");
pub const GOV_TOKEN_ADDR: Address = address!("0000000000000000000000000000000000002005");
pub const GOVERNOR_ADDR: Address = address!("0000000000000000000000000000000000002006");
pub const TIMELOCK_ADDR: Address = address!("0000000000000000000000000000000000002007");

// const SYSTEM_ADDRESS: Address = address!("00000000000000000000000000000000000000ff");
// this address is used to call evm. It's not used for gravity pre compile contract
pub const SYSTEM_ADDRESS: Address = address!("0000000000000000000000000000000000000000");

pub fn analyze_revert_reason(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Revert { gas_used, output } => {
            let mut reason = format!("Revert with gas used: {}", gas_used);

            if let Some(selector) = output.get(0..4) {
                reason.push_str(&format!("\nFunction selector: 0x{}", hex::encode(selector)));

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
                reason.push_str(&format!(
                    "\nAdditional data: 0x{}",
                    hex::encode(&output[4..])
                ));
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

/// Simulate the sequential execution of transactions with detailed logging
pub(crate) fn execute_revm_sequential_with_logging<DB>(
    db: DB,
    spec_id: SpecId,
    env: Env,
    txs: &[TxEnv],
    pre_bundle_state: Option<BundleState>,
) -> Result<(Vec<ExecutionResult>, BundleState), EVMError<DB::Error>>
where
    DB: DatabaseRef,
{
    let db = if pre_bundle_state.is_some() {
        StateBuilder::new()
            .with_bundle_prestate(pre_bundle_state.unwrap())
            .with_database_ref(db)
            .build()
    } else {
        StateBuilder::new()
            .with_bundle_update()
            .with_database_ref(db)
            .build()
    };
    let mut evm = EvmBuilder::default()
        .with_db(db)
        .with_spec_id(spec_id)
        .with_env(Box::new(env))
        .build();

    let mut evm = evm
        .modify()
        .reset_handler_with_external_context(TracerEip3155::new(Box::new(std::io::stdout())))
        .append_handler_register(inspector_handle_register)
        .build();

    let mut results = Vec::with_capacity(txs.len());
    for (i, tx) in txs.iter().enumerate() {
        info!("=== Executing transaction {} ===", i + 1);
        info!("Transaction details:");
        info!("  Caller: {:?}", tx.caller);
        info!("  To: {:?}", tx.transact_to);
        info!("  Data length: {}", tx.data.len());
        if tx.data.len() >= 4 {
            info!("  Function selector: 0x{}", hex::encode(&tx.data[0..4]));
        }

        *evm.tx_mut() = tx.clone();

        let result_and_state = evm.transact()?;
        evm.db_mut().commit(result_and_state.state);

        info!(
            "Transaction result: {}",
            analyze_revert_reason(&result_and_state.result)
        );
        results.push(result_and_state.result);
        info!("=== Transaction {} completed ===", i + 1);
    }
    evm.db_mut().merge_transitions(BundleRetention::Reverts);

    Ok((results, evm.db_mut().take_bundle()))
}

/// Simulate the sequential execution of transactions with detailed logging
pub(crate) fn execute_revm_sequential<DB>(
    db: &mut State<DB>,
    spec_id: SpecId,
    env: Env,
    txs: &[TxEnv],
) -> Result<(Vec<ExecutionResult>, BundleState), EVMError<DB::Error>>
where
    DB: Database,
{
    let mut evm = EvmBuilder::default()
        .with_db(db)
        .with_spec_id(spec_id)
        .with_env(Box::new(env))
        .build();

    let mut evm = evm
        .modify()
        .reset_handler_with_external_context(TracerEip3155::new(Box::new(std::io::stdout())))
        .append_handler_register(inspector_handle_register)
        .build();

    let mut results = Vec::with_capacity(txs.len());
    for (i, tx) in txs.iter().enumerate() {
        info!("=== Executing transaction {} ===", i + 1);
        info!("Transaction details:");
        info!("  Caller: {:?}", tx.caller);
        info!("  To: {:?}", tx.transact_to);
        info!("  Data length: {}", tx.data.len());
        if tx.data.len() >= 4 {
            info!("  Function selector: 0x{}", hex::encode(&tx.data[0..4]));
        }

        *evm.tx_mut() = tx.clone();

        let result_and_state = evm.transact()?;
        info!("transaction evm state {:?}", result_and_state.state);
        evm.db_mut().commit(result_and_state.state);

        info!(
            "Transaction result: {}",
            analyze_revert_reason(&result_and_state.result)
        );
        results.push(result_and_state.result);
        info!("=== Transaction {} completed ===", i + 1);
    }
    evm.db_mut().merge_transitions(BundleRetention::Reverts);

    Ok((results, evm.db_mut().take_bundle()))
}

pub fn new_system_call_txn(contract: Address, input: Bytes) -> TxEnv {
    TxEnv {
        caller: SYSTEM_ADDRESS,
        gas_limit: u64::MAX,
        gas_price: U256::ZERO,
        transact_to: TxKind::Call(contract),
        value: U256::ZERO,
        data: input,
        ..Default::default()
    }
}

pub fn new_system_create_txn(hex_code: &str, args: Bytes) -> TxEnv {
    let mut data = hex::decode(hex_code).expect("Invalid hex string");
    data.extend_from_slice(&args);
    TxEnv {
        caller: SYSTEM_ADDRESS,
        gas_limit: u64::MAX,
        gas_price: U256::ZERO,
        transact_to: TxKind::Create,
        value: U256::ZERO,
        data: data.into(),
        ..Default::default()
    }
}

pub fn read_hex_from_file(path: &str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to open {}", path))
}
