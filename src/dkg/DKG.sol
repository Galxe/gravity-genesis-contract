// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.30;

import "@src/System.sol";
import "@src/access/Protectable.sol";
import "@src/interfaces/IDKG.sol";
import "@src/interfaces/ITimestamp.sol";

/**
 * @title DKG
 * @dev DKG on-chain states and helper functions
 * @notice This contract manages DKG sessions for validator set transitions
 */
contract DKG is System, Protectable, IDKG {
    // Error codes
    uint64 constant EDKG_IN_PROGRESS = 1;
    uint64 constant EDKG_NOT_IN_PROGRESS = 2;

    // State variables
    DKGSessionState private _lastCompleted;
    bool private _hasLastCompleted;
    DKGSessionState private _inProgress;
    bool private _hasInProgress;
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

    modifier whenDKGNotInProgress() {
        if (_hasInProgress) revert DKGInProgress();
        _;
    }

    modifier whenDKGInProgress() {
        if (!_hasInProgress) revert DKGNotInProgress();
        _;
    }

    modifier onlyInitialized() {
        if (!_initialized) revert DKGNotInitialized();
        _;
    }

    /// @inheritdoc IDKG
    function initialize() external onlyGenesis {
        if (_initialized) revert DKGNotInitialized();
        _initialized = true;
        _hasLastCompleted = false;
        _hasInProgress = false;
    }

    /// @inheritdoc IDKG
    function start(
        uint64 dealerEpoch,
        RandomnessConfig memory randomnessConfig,
        ValidatorConsensusInfo[] memory dealerValidatorSet,
        ValidatorConsensusInfo[] memory targetValidatorSet
    ) external onlyAuthorizedCallers whenNotPaused whenDKGNotInProgress onlyInitialized {
        DKGSessionMetadata memory newSessionMetadata = DKGSessionMetadata({
            dealerEpoch: dealerEpoch,
            randomnessConfig: randomnessConfig,
            dealerValidatorSet: dealerValidatorSet,
            targetValidatorSet: targetValidatorSet
        });

        uint64 startTimeUs = uint64(ITimestamp(TIMESTAMP_ADDR).nowMicroseconds());

        _inProgress = DKGSessionState({
            metadata: newSessionMetadata,
            startTimeUs: startTimeUs,
            transcript: ""
        });
        _hasInProgress = true;

        emit DKGStartEvent(newSessionMetadata, startTimeUs);
    }

    /// @inheritdoc IDKG
    function finish(bytes memory transcript) external onlyAuthorizedCallers whenNotPaused whenDKGInProgress onlyInitialized {
        // Move in-progress session to completed
        _lastCompleted = _inProgress;
        _lastCompleted.transcript = transcript;
        _hasLastCompleted = true;

        // Clear in-progress session
        _hasInProgress = false;
    }

    /// @inheritdoc IDKG
    function tryClearIncompleteSession() external onlyAuthorizedCallers whenNotPaused onlyInitialized {
        if (_hasInProgress) {
            _hasInProgress = false;
        }
    }

    /// @inheritdoc IDKG
    function incompleteSession() external view onlyInitialized returns (bool hasSession, DKGSessionState memory session) {
        hasSession = _hasInProgress;
        if (hasSession) {
            session = _inProgress;
        }
    }

    /// @inheritdoc IDKG
    function sessionDealerEpoch(DKGSessionState memory session) external pure returns (uint64) {
        return session.metadata.dealerEpoch;
    }

    /// @inheritdoc IDKG
    function isDKGInProgress() external view onlyInitialized returns (bool) {
        return _hasInProgress;
    }

    /// @inheritdoc IDKG
    function lastCompletedSession() external view onlyInitialized returns (bool hasSession, DKGSessionState memory session) {
        hasSession = _hasLastCompleted;
        if (hasSession) {
            session = _lastCompleted;
        }
    }

    /**
     * @dev Get current DKG state for debugging
     * @return hasLastCompleted Whether there is a last completed session
     * @return hasInProgress Whether there is an in-progress session
     */
    function getDKGState() external view returns (bool hasLastCompleted, bool hasInProgress) {
        hasLastCompleted = _hasLastCompleted;
        hasInProgress = _hasInProgress;
    }
}
