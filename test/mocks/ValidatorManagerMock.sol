// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "@src/interfaces/IValidatorManager.sol";

contract ValidatorManagerMock {
    mapping(address => bool) public isCurrentEpochValidatorMap;
    mapping(address => uint64) public validatorIndexMap;
    mapping(address => bool) public validatorExistsMap;
    mapping(address => address) public validatorStakeCreditMap;
    mapping(address => IValidatorManager.ValidatorStatus) public validatorStatusMap;
    mapping(address => uint256) public validatorStakeMap;
    bool public initialized;

    function initialize(
        IValidatorManager.InitializationParams calldata params
    ) external {
        initialized = true;
        // Store the validators for testing
        for (uint256 i = 0; i < params.validatorAddresses.length; i++) {
            isCurrentEpochValidatorMap[params.validatorAddresses[i]] = true;
            validatorIndexMap[params.validatorAddresses[i]] = uint64(i);
            validatorExistsMap[params.validatorAddresses[i]] = true;
            validatorStatusMap[params.validatorAddresses[i]] = IValidatorManager.ValidatorStatus.ACTIVE;
        }
    }

    function setIsCurrentEpochValidator(address validator, bool isValidator) external {
        isCurrentEpochValidatorMap[validator] = isValidator;
    }

    function setValidatorIndex(address validator, uint64 index) external {
        validatorIndexMap[validator] = index;
    }

    function isCurrentEpochValidator(
        address validator
    ) external view returns (bool) {
        return isCurrentEpochValidatorMap[validator];
    }

    function getValidatorIndex(
        address validator
    ) external view returns (uint64) {
        require(isCurrentEpochValidatorMap[validator], "ValidatorNotActive");
        return validatorIndexMap[validator];
    }

    function setIsValidatorExists(address validator, bool exists) external {
        validatorExistsMap[validator] = exists;
    }

    function isValidatorExists(
        address validator
    ) external view returns (bool) {
        return validatorExistsMap[validator];
    }

    function setValidatorStakeCredit(address validator, address stakeCredit) external {
        validatorStakeCreditMap[validator] = stakeCredit;
    }

    function getValidatorStakeCredit(
        address validator
    ) external view returns (address) {
        return validatorStakeCreditMap[validator];
    }

    function setValidatorStatus(address validator, IValidatorManager.ValidatorStatus status) external {
        validatorStatusMap[validator] = status;
    }

    function getValidatorStatus(
        address validator
    ) external view returns (IValidatorManager.ValidatorStatus) {
        return validatorStatusMap[validator];
    }

    function setValidatorStake(address validator, uint256 stake) external {
        validatorStakeMap[validator] = stake;
    }

    function getValidatorStake(
        address validator
    ) external view returns (uint256) {
        return validatorStakeMap[validator];
    }

    function checkVotingPowerIncrease(
        uint256 /* amount */
    ) external pure {
        // Do nothing - mock implementation
    }

    function checkValidatorMinStake(
        address /* validator */
    ) external pure {
        // Do nothing - mock implementation
    }

    function getTotalStake() external pure returns (uint256) {
        return 1000 ether; // Mock value
    }

    function getValidatorCount() external pure returns (uint256) {
        return 10; // Mock value
    }

    function getActiveValidatorCount() external pure returns (uint256) {
        return 8; // Mock value
    }

    function setupValidator(
        address validator,
        address stakeCredit,
        IValidatorManager.ValidatorStatus status,
        uint256 stake
    ) external {
        validatorExistsMap[validator] = true;
        validatorStakeCreditMap[validator] = stakeCredit;
        validatorStatusMap[validator] = status;
        validatorStakeMap[validator] = stake;
        isCurrentEpochValidatorMap[validator] = (status == IValidatorManager.ValidatorStatus.ACTIVE);
    }

    function removeValidator(
        address validator
    ) external {
        validatorExistsMap[validator] = false;
        validatorStakeCreditMap[validator] = address(0);
        validatorStatusMap[validator] = IValidatorManager.ValidatorStatus.INACTIVE;
        validatorStakeMap[validator] = 0;
        isCurrentEpochValidatorMap[validator] = false;
    }

    function getValidatorSet() external pure returns (IValidatorManager.ValidatorSet memory) {
        return IValidatorManager.ValidatorSet({
            activeValidators: new IValidatorManager.ValidatorInfo[](0),
            pendingInactive: new IValidatorManager.ValidatorInfo[](0),
            pendingActive: new IValidatorManager.ValidatorInfo[](0),
            totalVotingPower: 0,
            totalJoiningPower: 0
        });
    }

    function onNewEpoch() external {
        // Mock implementation - do nothing
    }
}
