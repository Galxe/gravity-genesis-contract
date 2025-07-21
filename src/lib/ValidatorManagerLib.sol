// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "../System.sol";
import "@src/interfaces/IStakeConfig.sol";
import "@src/interfaces/IEpochManager.sol";
import "@src/interfaces/IValidatorManager.sol";
import "@src/interfaces/ITimestamp.sol";
import "@src/interfaces/IValidatorPerformanceTracker.sol";
import "@src/stake/StakeCredit.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

/**
 * @title ValidatorManagerLib
 * @dev Library containing helper functions for ValidatorManager to reduce contract size
 */
library ValidatorManagerLib {
    using EnumerableSet for EnumerableSet.AddressSet;

    uint256 private constant BLS_PUBKEY_LENGTH = 48;
    uint256 private constant BLS_SIG_LENGTH = 96;

    /**
     * @dev Verify BLS vote address and proof
     * @param operatorAddress Operator address
     * @param voteAddress BLS vote address
     * @param blsProof BLS proof
     * @return Whether verification succeeded
     */
    function checkVoteAddress(
        address operatorAddress,
        bytes calldata voteAddress,
        bytes calldata blsProof
    ) external view returns (bool) {
        // check lengths
        if (voteAddress.length != BLS_PUBKEY_LENGTH || blsProof.length != BLS_SIG_LENGTH) {
            return false;
        }

        // generate message hash
        bytes32 msgHash = keccak256(abi.encodePacked(operatorAddress, voteAddress, block.chainid));
        bytes memory msgBz = new bytes(32);
        assembly {
            mstore(add(msgBz, 32), msgHash)
        }

        // call precompiled contract to verify BLS signature
        // precompiled contract address is 0x66
        bytes memory input = bytes.concat(msgBz, blsProof, voteAddress); // length: 32 + 96 + 48 = 176
        bytes memory output = new bytes(1);
        assembly {
            let len := mload(input)
            if iszero(staticcall(not(0), 0x66, add(input, 0x20), len, add(output, 0x20), 0x01)) { revert(0, 0) }
        }
        uint8 result = uint8(output[0]);
        if (result != uint8(1)) {
            return false;
        }
        return true;
    }

    /**
     * @dev Check if validator name is valid
     * @param moniker Validator name
     * @return Whether the name is valid
     */
    function checkMoniker(
        string memory moniker
    ) external pure returns (bool) {
        bytes memory bz = bytes(moniker);

        // 1. moniker length should be between 3 and 9
        if (bz.length < 3 || bz.length > 9) {
            return false;
        }

        // 2. first character should be uppercase
        if (uint8(bz[0]) < 65 || uint8(bz[0]) > 90) {
            return false;
        }

        // 3. only alphanumeric characters are allowed
        for (uint256 i = 1; i < bz.length; ++i) {
            // Check if the ASCII value of the character falls outside the range of alphanumeric characters
            if (
                (uint8(bz[i]) < 48 || uint8(bz[i]) > 57) && (uint8(bz[i]) < 65 || uint8(bz[i]) > 90)
                    && (uint8(bz[i]) < 97 || uint8(bz[i]) > 122)
            ) {
                // Character is a special character
                return false;
            }
        }

        // No special characters found
        return true;
    }

    /**
     * @dev Check voting power increase limit
     */
    function checkVotingPowerIncrease(
        uint256 increaseAmount,
        uint256 totalVotingPower,
        EnumerableSet.AddressSet storage pendingActive,
        mapping(address => IValidatorManager.ValidatorInfo) storage validatorInfos
    ) external view {
        uint256 votingPowerIncreaseLimit = IStakeConfig(0x0000000000000000000000000000000000002008).votingPowerIncreaseLimit();

        if (totalVotingPower > 0) {
            // 计算所有pending验证人的实际下一个epoch投票权
            uint256 totalPendingPower = 0;
            address[] memory pendingVals = pendingActive.values();
            for (uint256 i = 0; i < pendingVals.length; i++) {
                address stakeCreditAddress = validatorInfos[pendingVals[i]].stakeCreditAddress;
                if (stakeCreditAddress != address(0)) {
                    totalPendingPower += StakeCredit(payable(stakeCreditAddress)).getNextEpochVotingPower();
                }
            }

            uint256 currentJoining = totalPendingPower + increaseAmount;

            if (currentJoining * 100 > totalVotingPower * votingPowerIncreaseLimit) {
                revert IValidatorManager.VotingPowerIncreaseExceedsLimit();
            }
        }
    }

    /**
     * @dev Validate registration params
     */
    function validateRegistrationParams(
        address validator,
        IValidatorManager.ValidatorRegistrationParams calldata params,
        mapping(address => IValidatorManager.ValidatorInfo) storage validatorInfos,
        mapping(bytes => address) storage voteAddressToValidator,
        mapping(bytes => address) storage consensusToValidator,
        mapping(bytes32 => bool) storage monikerSet,
        mapping(address => address) storage operatorToValidator
    ) external view {
        if (validatorInfos[validator].registered) {
            revert IValidatorManager.ValidatorAlreadyExists(validator);
        }

        // check BLS vote address
        if (params.voteAddress.length > 0 && voteAddressToValidator[params.voteAddress] != address(0)) {
            revert IValidatorManager.DuplicateVoteAddress(params.voteAddress);
        }

        // check consensus address
        if (params.consensusPublicKey.length > 0 && consensusToValidator[params.consensusPublicKey] != address(0)) {
            revert IValidatorManager.DuplicateConsensusAddress(params.consensusPublicKey);
        }

        // check validator name
        if (!_checkMonikerInternal(params.moniker)) {
            revert IValidatorManager.InvalidMoniker(params.moniker);
        }

        bytes32 monikerHash = keccak256(abi.encodePacked(params.moniker));
        if (monikerSet[monikerHash]) {
            revert IValidatorManager.DuplicateMoniker(params.moniker);
        }

        // check commission settings
        if (
            params.commission.maxRate > IStakeConfig(0x0000000000000000000000000000000000002008).MAX_COMMISSION_RATE()
                || params.commission.rate > params.commission.maxRate
                || params.commission.maxChangeRate > params.commission.maxRate
        ) {
            revert IValidatorManager.InvalidCommission();
        }

        // check BLS proof
        if (params.voteAddress.length > 0 && !_checkVoteAddressInternal(validator, params.voteAddress, params.blsProof)) {
            revert IValidatorManager.InvalidVoteAddress();
        }

        // check address validity
        if (params.initialOperator == address(0)) {
            revert IValidatorManager.InvalidAddress(address(0));
        }

        // check address conflict
        if (operatorToValidator[params.initialOperator] != address(0)) {
            revert IValidatorManager.AddressAlreadyInUse(params.initialOperator, operatorToValidator[params.initialOperator]);
        }
    }

    /**
     * @dev Internal helper for checking moniker
     */
    function _checkMonikerInternal(
        string memory moniker
    ) internal pure returns (bool) {
        bytes memory bz = bytes(moniker);

        // 1. moniker length should be between 3 and 9
        if (bz.length < 3 || bz.length > 9) {
            return false;
        }

        // 2. first character should be uppercase
        if (uint8(bz[0]) < 65 || uint8(bz[0]) > 90) {
            return false;
        }

        // 3. only alphanumeric characters are allowed
        for (uint256 i = 1; i < bz.length; ++i) {
            // Check if the ASCII value of the character falls outside the range of alphanumeric characters
            if (
                (uint8(bz[i]) < 48 || uint8(bz[i]) > 57) && (uint8(bz[i]) < 65 || uint8(bz[i]) > 90)
                    && (uint8(bz[i]) < 97 || uint8(bz[i]) > 122)
            ) {
                // Character is a special character
                return false;
            }
        }

        // No special characters found
        return true;
    }

    /**
     * @dev Internal helper for checking vote address
     */
    function _checkVoteAddressInternal(
        address operatorAddress,
        bytes calldata voteAddress,
        bytes calldata blsProof
    ) internal view returns (bool) {
        // check lengths
        if (voteAddress.length != BLS_PUBKEY_LENGTH || blsProof.length != BLS_SIG_LENGTH) {
            return false;
        }

        // generate message hash
        bytes32 msgHash = keccak256(abi.encodePacked(operatorAddress, voteAddress, block.chainid));
        bytes memory msgBz = new bytes(32);
        assembly {
            mstore(add(msgBz, 32), msgHash)
        }

        // call precompiled contract to verify BLS signature
        // precompiled contract address is 0x66
        bytes memory input = bytes.concat(msgBz, blsProof, voteAddress); // length: 32 + 96 + 48 = 176
        bytes memory output = new bytes(1);
        assembly {
            let len := mload(input)
            if iszero(staticcall(not(0), 0x66, add(input, 0x20), len, add(output, 0x20), 0x01)) { revert(0, 0) }
        }
        uint8 result = uint8(output[0]);
        if (result != uint8(1)) {
            return false;
        }
        return true;
    }
}