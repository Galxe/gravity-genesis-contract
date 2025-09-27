// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

contract EpochManagerMock {
    bool public canTriggerEpochTransitionFlag;
    uint256 public triggerEpochTransitionCallCount;
    bool public initialized;
    uint256 public mockCurrentEpoch;

    function initialize() external {
        initialized = true;
        canTriggerEpochTransitionFlag = true; // Default to true for testing
        mockCurrentEpoch = 1; // Default epoch
    }

    function setCurrentEpoch(uint256 epoch) external {
        mockCurrentEpoch = epoch;
    }

    function setCanTriggerEpochTransition(
        bool canTrigger
    ) external {
        canTriggerEpochTransitionFlag = canTrigger;
    }

    function canTriggerEpochTransition() external view returns (bool) {
        return canTriggerEpochTransitionFlag;
    }

    function triggerEpochTransition() external {
        triggerEpochTransitionCallCount++;
    }

    function reset() external {
        canTriggerEpochTransitionFlag = false;
        triggerEpochTransitionCallCount = 0;
    }

    function currentEpoch() external view returns (uint256) {
        return mockCurrentEpoch;
    }
}
