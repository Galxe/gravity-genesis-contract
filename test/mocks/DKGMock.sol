// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.30;

import "@src/interfaces/IDKG.sol";

contract DKGMock is IDKG {
    // State variables
    DKGSessionState private _lastCompleted;
    bool private _hasLastCompleted;
    DKGSessionState private _inProgress;
    bool private _hasInProgress;
    bool private _initialized;

    // Mock control variables
    bool public shouldFailStart;
    bool public shouldFailFinish;

    function initialize() external override {
        _initialized = true;
        _hasLastCompleted = false;
        _hasInProgress = false;
    }

    function start(
        uint64 dealerEpoch,
        RandomnessConfig memory randomnessConfig,
        ValidatorConsensusInfo[] memory dealerValidatorSet,
        ValidatorConsensusInfo[] memory targetValidatorSet
    ) external override {
        require(!shouldFailStart, "DKGMock: Start failed");
        require(!_hasInProgress, "DKG already in progress");

        DKGSessionMetadata memory metadata = DKGSessionMetadata({
            dealerEpoch: dealerEpoch,
            randomnessConfig: randomnessConfig,
            dealerValidatorSet: dealerValidatorSet,
            targetValidatorSet: targetValidatorSet
        });

        _inProgress = DKGSessionState({
            metadata: metadata,
            startTimeUs: uint64(block.timestamp * 1000000),
            transcript: ""
        });
        _hasInProgress = true;

        emit DKGStartEvent(metadata, uint64(block.timestamp * 1000000));
    }

    function finish(bytes memory transcript) external override {
        require(!shouldFailFinish, "DKGMock: Finish failed");
        require(_hasInProgress, "DKG not in progress");

        _lastCompleted = _inProgress;
        _lastCompleted.transcript = transcript;
        _hasLastCompleted = true;

        _hasInProgress = false;
    }

    function tryClearIncompleteSession() external override {
        if (_hasInProgress) {
            _hasInProgress = false;
        }
    }

    function incompleteSession() external view override returns (bool hasSession, DKGSessionState memory session) {
        hasSession = _hasInProgress;
        if (hasSession) {
            session = _inProgress;
        }
    }

    function sessionDealerEpoch(DKGSessionState memory session) external pure override returns (uint64) {
        return session.metadata.dealerEpoch;
    }

    function isDKGInProgress() external view override returns (bool) {
        return _hasInProgress;
    }

    function lastCompletedSession() external view override returns (bool hasSession, DKGSessionState memory session) {
        hasSession = _hasLastCompleted;
        if (hasSession) {
            session = _lastCompleted;
        }
    }

    // Mock control functions
    function setShouldFailStart(bool _shouldFail) external {
        shouldFailStart = _shouldFail;
    }

    function setShouldFailFinish(bool _shouldFail) external {
        shouldFailFinish = _shouldFail;
    }

    function setInProgressSession(DKGSessionState memory session) external {
        _inProgress = session;
        _hasInProgress = true;
    }

    function setLastCompletedSession(DKGSessionState memory session) external {
        _lastCompleted = session;
        _hasLastCompleted = true;
    }

    function clearSessions() external {
        _hasInProgress = false;
        _hasLastCompleted = false;
    }
}
