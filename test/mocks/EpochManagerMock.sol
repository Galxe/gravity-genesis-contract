// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

contract EpochManagerMock {
    bool public canTriggerEpochTransitionFlag;
    uint256 public triggerEpochTransitionCallCount;
    bool public initialized;

    function initialize() external {
        initialized = true;
        canTriggerEpochTransitionFlag = true; // Default to true for testing
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

    function currentEpoch() external pure returns (uint256) {
        return 1; // Return a default epoch for testing
    }
}
