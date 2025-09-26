// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.30;

import "@src/System.sol";
import "@src/access/Protectable.sol";
import "@src/interfaces/IReconfigurationWithDKG.sol";
import "@src/interfaces/IDKG.sol";
import "@src/interfaces/IEpochManager.sol";
import "@src/interfaces/IValidatorManager.sol";

/**
 * @title ReconfigurationWithDKG
 * @dev Reconfiguration with DKG helper functions
 * @notice This contract manages reconfiguration processes that involve DKG operations
 */
contract ReconfigurationWithDKG is System, Protectable, IReconfigurationWithDKG {
    // DKG contract address - using system constant
    // address public constant DKG_ADDR = 0x000000000000000000000000000000000000200E;

    // State variables
    bool private _initialized;

    // Modifiers
    modifier onlyAuthorizedCallers() {
        if (
            msg.sender != SYSTEM_CALLER && 
            msg.sender != BLOCK_ADDR && 
            msg.sender != GENESIS_ADDR
        ) {
            revert NotAuthorized(msg.sender);
        }
        _;
    }

    modifier onlyInitialized() {
        if (!_initialized) revert ReconfigurationNotInProgress();
        _;
    }

    /// @inheritdoc IReconfigurationWithDKG
    function initialize() external onlyGenesis {
        if (_initialized) revert ReconfigurationNotInProgress();
        _initialized = true;
        // Contract initialization logic
    }

    /// @inheritdoc IReconfigurationWithDKG
    function tryStart() external onlyAuthorizedCallers whenNotPaused onlyInitialized {
        uint256 currentEpoch = IEpochManager(EPOCH_MANAGER_ADDR).currentEpoch();
        
        // Check if there's an incomplete DKG session
        (bool hasIncompleteSession, IDKG.DKGSessionState memory session) = IDKG(DKG_ADDR).incompleteSession();
        
        if (hasIncompleteSession) {
            uint64 sessionDealerEpoch = IDKG(DKG_ADDR).sessionDealerEpoch(session);
            
            // If the incomplete session is for the current epoch, return without starting new one
            if (sessionDealerEpoch == currentEpoch) {
                return;
            }
            
            // Clear the old session if it's for a different epoch
            IDKG(DKG_ADDR).tryClearIncompleteSession();
        }

        // Start reconfiguration process
        _onReconfigStart();
        
        // Get current and next validator consensus infos
        IDKG.ValidatorConsensusInfo[] memory currentValidators = _getCurrentValidatorConsensusInfos();
        IDKG.ValidatorConsensusInfo[] memory nextValidators = _getNextValidatorConsensusInfos();
        
        // Get current randomness config
        IDKG.RandomnessConfig memory randomnessConfig = _getCurrentRandomnessConfig();
        
        // Start DKG session
        IDKG(DKG_ADDR).start(
            uint64(currentEpoch),
            randomnessConfig,
            currentValidators,
            nextValidators
        );

    }

    /// @inheritdoc IReconfigurationWithDKG
    function finish() external onlyAuthorizedCallers whenNotPaused onlyInitialized {
        _finishReconfiguration();
    }

    /**
     * @dev Internal function to finish reconfiguration
     */
    function _finishReconfiguration() internal {
        // Clear incomplete DKG session if it exists
        IDKG(DKG_ADDR).tryClearIncompleteSession();
        
        // Apply buffered on-chain configs for new epoch
        _applyOnNewEpochConfigs();
        
        // Trigger epoch transition
        if (IEpochManager(EPOCH_MANAGER_ADDR).canTriggerEpochTransition()) {
            IEpochManager(EPOCH_MANAGER_ADDR).triggerEpochTransition();
        }
    }

    /// @inheritdoc IReconfigurationWithDKG
    function finishWithDkgResult(bytes calldata dkgResult) external onlyAuthorizedCallers whenNotPaused onlyInitialized {
        // Finish DKG with the provided result
        IDKG(DKG_ADDR).finish(dkgResult);
        
        // Complete the reconfiguration - call internal function directly
        _finishReconfiguration();
    }

    /// @inheritdoc IReconfigurationWithDKG
    function isReconfigurationInProgress() external view onlyInitialized returns (bool) {
        return IDKG(DKG_ADDR).isDKGInProgress();
    }

    /**
     * @dev Internal function to handle reconfiguration start
     */
    function _onReconfigStart() internal {
        // Add any necessary logic for reconfiguration start
        // This could include updating state, emitting events, etc.
    }

    /**
     * @dev Apply all necessary configurations for the new epoch
     */
    function _applyOnNewEpochConfigs() internal {
        // Apply various on-chain configurations for the new epoch
        // This includes:
        // - Consensus config updates
        // - Execution config updates  
        // - Gas schedule updates
        // - Version updates
        // - Feature updates
        // - JWK consensus config updates
        // - JWKs updates
        // - Keyless account updates
        // - Randomness config updates
        // - Randomness API config updates
        
        // For now, we'll implement a placeholder that can be extended
        // based on the specific requirements of each module
    }

    /**
     * @dev Get current validator consensus infos
     * @return Array of current validator consensus information
     */
    function _getCurrentValidatorConsensusInfos() internal view returns (IDKG.ValidatorConsensusInfo[] memory) {
        // This should be implemented to get current validator set from ValidatorManager
        // For now, return empty array as placeholder
        return new IDKG.ValidatorConsensusInfo[](0);
    }

    /**
     * @dev Get next validator consensus infos
     * @return Array of next validator consensus information
     */
    function _getNextValidatorConsensusInfos() internal view returns (IDKG.ValidatorConsensusInfo[] memory) {
        // This should be implemented to get next validator set from ValidatorManager
        // For now, return empty array as placeholder
        return new IDKG.ValidatorConsensusInfo[](0);
    }

    /**
     * @dev Get current randomness config
     * @return Current randomness configuration
     */
    function _getCurrentRandomnessConfig() internal pure returns (IDKG.RandomnessConfig memory) {
        // This should be implemented to get current randomness config
        // For now, return default config as placeholder
        return IDKG.RandomnessConfig({
            config: IDKG.Config({
                secrecyThreshold: IDKG.FixedPoint64({value: 0}),
                reconstructionThreshold: IDKG.FixedPoint64({value: 0}),
                fastPathSecrecyThreshold: IDKG.FixedPoint64({value: 0})
            })
        });
    }
}
