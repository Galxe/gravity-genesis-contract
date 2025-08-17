use revm::{DatabaseRef, InMemoryDB, db::BundleState};
use revm_primitives::SpecId;
use tracing::error;

use crate::{
    execute::prepare_env,
    genesis::{
        GenesisConfig, call_get_current_epoch_info, call_get_validator_set,
        print_current_epoch_info_result, print_validator_set_result,
    },
    utils::execute_revm_sequential,
};

fn verify_validator_set(
    db: impl DatabaseRef,
    bundle_state: BundleState,
    config: &GenesisConfig,
) {
    let mut all_txs = vec![];
    let get_validator_set_txn = call_get_validator_set();
    all_txs.push(get_validator_set_txn.clone());
    let env = prepare_env();
    let r = execute_revm_sequential(db, SpecId::LATEST, env, &all_txs, Some(bundle_state));
    match r {
        Ok((result, _)) => {
            if let Some(validator_set_result) = result.get(0) {
                print_validator_set_result(validator_set_result, config);
            }
        }
        Err(e) => {
            error!(
                "verify validator set error: {:?}",
                e.map_db_err(|_| "Database error".to_string())
            );
        }
    }
}

fn verify_epoch_info(db: impl DatabaseRef, bundle_state: BundleState) {
    let mut all_txs = vec![];
    let get_epoch_info_txn = call_get_current_epoch_info();
    all_txs.push(get_epoch_info_txn.clone());
    let env = prepare_env();
    let r = execute_revm_sequential(db, SpecId::LATEST, env, &all_txs, Some(bundle_state));
    match r {
        Ok((result, _)) => {
            if let Some(epoch_info_result) = result.get(0) {
                print_current_epoch_info_result(epoch_info_result);
            }
        }
        Err(e) => {
            error!(
                "verify epoch info error: {:?}",
                e.map_db_err(|_| "Database error".to_string())
            );
        }
    }
}

pub fn verify_result(db: InMemoryDB, bundle_state: BundleState, config: &GenesisConfig) {
    verify_validator_set(db.clone(), bundle_state.clone(), config);
    verify_epoch_info(db.clone(), bundle_state.clone());
}
