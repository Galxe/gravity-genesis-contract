// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "@src/System.sol";
import "@src/interfaces/IValidatorManager.sol";
import "@src/interfaces/IStakeConfig.sol";
import "@src/interfaces/IEpochManager.sol";
import "@src/interfaces/ITimestamp.sol";
import "@src/interfaces/IBlock.sol";
import "@src/interfaces/IValidatorPerformanceTracker.sol";
import "@src/interfaces/IGovToken.sol";
import "@src/interfaces/IJWKManager.sol";
import "@src/interfaces/IKeylessAccount.sol";
import "@src/governance/GravityGovernor.sol";
import "@src/governance/Timelock.sol";

/**
 * @title Genesis
 * @dev Genesis initialization contract
 * Responsible for initializing all core components and initial validator set during chain startup
 */
contract Genesis is System {
    // Genesis completion flag
    bool private genesisCompleted;

    // Error definitions
    error GenesisAlreadyCompleted();
    error InvalidInitialValidators();

    event GenesisCompleted(uint256 timestamp, uint256 validatorCount);

    uint256 public genesisTotalIncoming;

    function getGenesisTotalIncoming() external view returns (uint256) {
        return genesisTotalIncoming;
    }

    /**
     * @dev Genesis initialization entry function
     */
    function initialize(
        address[] calldata validatorAddresses,
        bytes[] calldata consensusPublicKeys,
        uint256[] calldata votingPowers,
        bytes[] calldata validatorNetworkAddresses,
        bytes[] calldata fullnodeNetworkAddresses,
        bytes[] calldata aptosAddresses
    ) external onlySystemCaller {
        if (genesisCompleted) revert GenesisAlreadyCompleted();
        if (consensusPublicKeys.length == 0) revert InvalidInitialValidators();

        // 1. Initialize staking module
        _initializeStake(
            validatorAddresses,
            consensusPublicKeys,
            votingPowers,
            validatorNetworkAddresses,
            fullnodeNetworkAddresses,
            aptosAddresses
        );

        // 2. Initialize epoch module
        _initializeEpoch();

        // 3. Initialize governance module
        _initializeGovernance();

        // 4. Initialize JWK module
        _initializeJWK();

        // 5. Initialize Block contract
        IBlock(BLOCK_ADDR).initialize();

        genesisCompleted = true;

        // Trigger first epoch
        IEpochManager(EPOCH_MANAGER_ADDR).triggerEpochTransition();

        emit GenesisCompleted(block.timestamp, consensusPublicKeys.length);
    }

    /**
     * @dev Initialize the staking module
     */
    function _initializeStake(
        address[] calldata validatorAddresses,
        bytes[] calldata consensusPublicKeys,
        uint256[] calldata votingPowers,
        bytes[] calldata validatorNetworkAddresses,
        bytes[] calldata fullnodeNetworkAddresses,
        bytes[] calldata aptosAddresses
    ) internal {
        // Initialize StakeConfig
        IStakeConfig(STAKE_CONFIG_ADDR).initialize();

        // Initialize ValidatorManager with initial validator data
        IValidatorManager.InitializationParams memory initParams = IValidatorManager.InitializationParams({
            validatorAddresses: validatorAddresses,
            consensusPublicKeys: consensusPublicKeys,
            votingPowers: votingPowers,
            validatorNetworkAddresses: validatorNetworkAddresses,
            fullnodeNetworkAddresses: fullnodeNetworkAddresses,
            aptosAddresses: aptosAddresses
        });

        IValidatorManager(VALIDATOR_MANAGER_ADDR).initialize(initParams);

        // Initialize ValidatorPerformanceTracker
        IValidatorPerformanceTracker(VALIDATOR_PERFORMANCE_TRACKER_ADDR).initialize(validatorAddresses);
    }

    /**
     * @dev Initialize epoch module
     */
    function _initializeEpoch() internal {
        // Initialize EpochManager
        IEpochManager(EPOCH_MANAGER_ADDR).initialize();
    }

    /**
     * @dev Initialize governance module
     */
    function _initializeGovernance() internal {
        // Initialize GovToken
        IGovToken(GOV_TOKEN_ADDR).initialize();

        // Initialize Timelock
        Timelock(payable(TIMELOCK_ADDR)).initialize();

        // Initialize GravityGovernor
        GravityGovernor(payable(GOVERNOR_ADDR)).initialize();
    }

    /**
     * @dev Initialize JWK module
     */
    function _initializeJWK() internal {
        // Initialize JWKManager
        IJWKManager(JWK_MANAGER_ADDR).initialize();

        // Initialize KeylessAccount
        IKeylessAccount(KEYLESS_ACCOUNT_ADDR).initialize();
    }

    /**
     * @dev Check if genesis is completed
     */
    function isGenesisCompleted() external view returns (bool) {
        return genesisCompleted;
    }
}
