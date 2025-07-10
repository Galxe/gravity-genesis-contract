use crate::{
    contracts::*,
    storage::InMemoryDB,
    utils::{SYSTEM_ADDRESS, new_system_call_txn},
};

use alloy_chains::NamedChain;

use crate::utils::{analyze_revert_reason, execute_revm_sequential_with_logging};
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use revm::{
    db::PlainAccount,
    primitives::{AccountInfo, Address, Env, KECCAK_EMPTY, SpecId, TxEnv, U256, uint},
};
use revm_primitives::{Bytes, hex};
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
            panic!("Genesis transaction {} failed", i + 1);
        }
        println!("Transaction {}: {}", i + 1, analyze_revert_reason(r));
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
