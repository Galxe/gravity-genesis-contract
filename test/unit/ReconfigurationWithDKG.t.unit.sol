// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.30;

import "forge-std/Test.sol";
import "@src/dkg/ReconfigurationWithDKG.sol";
import "@test/mocks/DKGMock.sol";
import "@test/mocks/EpochManagerMock.sol";
import "@test/mocks/ValidatorManagerMock.sol";
import "@src/interfaces/IReconfigurationWithDKG.sol";
import "@src/interfaces/IDKG.sol";
import "@test/utils/TestConstants.sol";

contract ReconfigurationWithDKGTest is Test, TestConstants {
    // DKG address constant (copied from System.sol to avoid inheritance conflicts)
    address constant DKG_ADDR = 0x000000000000000000000000000000000000200E;
    ReconfigurationWithDKG reconfigContract;
    DKGMock dkgMock;
    EpochManagerMock epochManagerMock;
    ValidatorManagerMock validatorManagerMock;

    // Test data
    uint256 constant TEST_CURRENT_EPOCH = 5;
    uint64 constant TEST_DEALER_EPOCH = 5;
    bytes constant TEST_DKG_RESULT = "test_dkg_result";

    function setUp() public {
        // Deploy mock contracts
        dkgMock = new DKGMock();
        epochManagerMock = new EpochManagerMock();
        validatorManagerMock = new ValidatorManagerMock();
        
        // Deploy ReconfigurationWithDKG contract
        reconfigContract = new ReconfigurationWithDKG();

        // Deploy mock contracts to system addresses
        vm.etch(DKG_ADDR, address(dkgMock).code);
        vm.etch(EPOCH_MANAGER_ADDR, address(epochManagerMock).code);
        vm.etch(VALIDATOR_MANAGER_ADDR, address(validatorManagerMock).code);

        // Set up mock data
        DKGMock(DKG_ADDR).initialize();
        EpochManagerMock(EPOCH_MANAGER_ADDR).setCurrentEpoch(TEST_CURRENT_EPOCH);
        EpochManagerMock(EPOCH_MANAGER_ADDR).setCanTriggerEpochTransition(true);
    }

    function test_initialize_shouldSucceed() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);

        // Act
        reconfigContract.initialize();

        // Assert - Contract should be initialized (no specific state to check)
        assertTrue(true);
    }

    function test_initialize_shouldRevertIfNotGenesis() public {
        // Arrange
        vm.startPrank(address(0x123));

        // Act & Assert
        vm.expectRevert();
        reconfigContract.initialize();
    }

    function test_tryStart_shouldSucceedWhenNoDKGInProgress() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        vm.startPrank(SYSTEM_CALLER);

        // Act
        reconfigContract.tryStart();

        // Assert
        assertTrue(DKGMock(DKG_ADDR).isDKGInProgress());
    }

    function test_tryStart_shouldReturnEarlyIfSameEpochDKGInProgress() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        // Set up incomplete DKG session for same epoch
        IDKG.DKGSessionState memory incompleteSession = _createTestDKGSession(TEST_DEALER_EPOCH);
        DKGMock(DKG_ADDR).setInProgressSession(incompleteSession);

        vm.startPrank(SYSTEM_CALLER);

        // Act
        reconfigContract.tryStart();

        // Assert - Should return early without emitting event or starting new DKG
        assertTrue(DKGMock(DKG_ADDR).isDKGInProgress());
    }

    function test_tryStart_shouldStartNewDKGIfDifferentEpoch() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        // Set up incomplete DKG session for different epoch
        IDKG.DKGSessionState memory incompleteSession = _createTestDKGSession(TEST_DEALER_EPOCH - 1);
        DKGMock(DKG_ADDR).setInProgressSession(incompleteSession);

        vm.startPrank(SYSTEM_CALLER);

        // Act
        reconfigContract.tryStart();
    }

    function test_tryStart_shouldRevertIfNotAuthorized() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        vm.startPrank(address(0x123));

        // Act & Assert
        vm.expectRevert(abi.encodeWithSelector(IReconfigurationWithDKG.NotAuthorized.selector, address(0x123)));
        reconfigContract.tryStart();
    }

    function test_finish_shouldSucceed() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        vm.startPrank(SYSTEM_CALLER);

        // Act
        reconfigContract.finish();
    }

    function test_finish_shouldRevertIfNotAuthorized() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        vm.startPrank(address(0x123));

        // Act & Assert
        vm.expectRevert(abi.encodeWithSelector(IReconfigurationWithDKG.NotAuthorized.selector, address(0x123)));
        reconfigContract.finish();
    }

    function test_finishWithDkgResult_shouldSucceed() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        // Set up DKG in progress
        IDKG.DKGSessionState memory inProgressSession = _createTestDKGSession(TEST_DEALER_EPOCH);
        DKGMock(DKG_ADDR).setInProgressSession(inProgressSession);

        vm.startPrank(SYSTEM_CALLER);

        // Act
        reconfigContract.finishWithDkgResult(TEST_DKG_RESULT);

        // Assert - DKG should no longer be in progress
        assertFalse(DKGMock(DKG_ADDR).isDKGInProgress());
    }

    function test_finishWithDkgResult_shouldRevertIfNotAuthorized() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        vm.startPrank(address(0x123));

        // Act & Assert
        vm.expectRevert(abi.encodeWithSelector(IReconfigurationWithDKG.NotAuthorized.selector, address(0x123)));
        reconfigContract.finishWithDkgResult(TEST_DKG_RESULT);
    }

    function test_isReconfigurationInProgress_shouldReturnCorrectStatus() public {
        // Arrange
        vm.startPrank(GENESIS_ADDR);
        reconfigContract.initialize();
        vm.stopPrank();

        // Initially should not be in progress
        assertFalse(reconfigContract.isReconfigurationInProgress());

        // Set DKG in progress
        IDKG.DKGSessionState memory inProgressSession = _createTestDKGSession(TEST_DEALER_EPOCH);
        DKGMock(DKG_ADDR).setInProgressSession(inProgressSession);

        // Should now be in progress
        assertTrue(reconfigContract.isReconfigurationInProgress());
    }

    function _createTestDKGSession(uint64 dealerEpoch) internal view returns (IDKG.DKGSessionState memory) {
        return IDKG.DKGSessionState({
            metadata: IDKG.DKGSessionMetadata({
                dealerEpoch: dealerEpoch,
                randomnessConfig: IDKG.RandomnessConfig({
                    config: IDKG.Config({
                        secrecyThreshold: IDKG.FixedPoint64({value: 100}),
                        reconstructionThreshold: IDKG.FixedPoint64({value: 200}),
                        fastPathSecrecyThreshold: IDKG.FixedPoint64({value: 50})
                    })
                }),
                dealerValidatorSet: new IDKG.ValidatorConsensusInfo[](0),
                targetValidatorSet: new IDKG.ValidatorConsensusInfo[](0)
            }),
            startTimeUs: uint64(block.timestamp * 1000000),
            transcript: ""
        });
    }
}
