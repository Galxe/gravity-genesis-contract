// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "forge-std/Test.sol";
import "@src/jwk/JWKManager.sol";
import "@src/interfaces/IJWKManager.sol";
import "@src/interfaces/IValidatorManager.sol";
import "@src/interfaces/IDelegation.sol";
import "@test/utils/TestConstants.sol";
import "@test/mocks/JWKManagerMock.sol";
import "@test/mocks/EpochManagerMock.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract JWKManagerTest is Test, TestConstants {
    JWKManager public jwkManager;
    JWKManager public implementation;
    JWKManagerMock public validatorManagerMock;
    JWKManagerMock public delegationMock;
    EpochManagerMock public epochManagerMock;

    // Test constants
    address private constant TEST_USER = 0x1234567890123456789012345678901234567890;
    address private constant TEST_VALIDATOR = 0x2234567890123456789012345678901234567890;
    address private constant TEST_TARGET_VALIDATOR = 0x3334567890123456789012345678901234567890;
    
    uint256 private constant TEST_STAKE_AMOUNT = 1 ether;
    
    bytes private constant TEST_CONSENSUS_KEY = 
        hex"123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456";
    bytes private constant TEST_BLS_PROOF = 
        hex"12345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456";
    
    string private constant TEST_MONIKER = "TestValidator";
    string private constant TEST_ISSUER = "https://test.issuer.com";

    function setUp() public {
        // Deploy mock contracts
        validatorManagerMock = new JWKManagerMock();
        delegationMock = new JWKManagerMock();
        epochManagerMock = new EpochManagerMock();

        // Deploy JWKManager implementation
        implementation = new JWKManager();

        // Deploy proxy
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), "");
        jwkManager = JWKManager(address(proxy));

        // Deploy mock contracts to system addresses
        vm.etch(VALIDATOR_MANAGER_ADDR, address(validatorManagerMock).code);
        vm.etch(DELEGATION_ADDR, address(delegationMock).code);
        vm.etch(EPOCH_MANAGER_ADDR, address(epochManagerMock).code);

        // Initialize JWKManager
        vm.prank(GENESIS_ADDR);
        jwkManager.initialize();

        // Add OIDC providers for testing
        vm.prank(GOV_HUB_ADDR);
        jwkManager.upsertOIDCProvider(TEST_ISSUER, "https://test.issuer.com/.well-known/openid_configuration");
        vm.prank(GOV_HUB_ADDR);
        jwkManager.upsertOIDCProvider("https://provider1.com", "https://provider1.com/.well-known/openid_configuration");
        vm.prank(GOV_HUB_ADDR);
        jwkManager.upsertOIDCProvider("https://provider2.com", "https://provider2.com/.well-known/openid_configuration");

        // Provide ETH to JWKManager for stake operations
        vm.deal(address(jwkManager), 100 ether);

        // Set up mock data
        _getValidatorManagerMock().setValidatorExists(TEST_TARGET_VALIDATOR, true);
        _getValidatorManagerMock().setValidatorStakeCredit(TEST_TARGET_VALIDATOR, address(0x123));
    }

    // ============ STAKE EVENT JWK PROCESSING TESTS ============

    /// @notice Test processing validator stake event JWK, should call register validator method
    function test_processStakeEventJWKs_validatorStakeEvent_shouldCallRegisterValidator() public {
        // Arrange
        IValidatorManager.Commission memory commission = IValidatorManager.Commission({
            rate: 1000, // 10%
            maxRate: 5000, // 50%
            maxChangeRate: 500 // 5%
        });

        IValidatorManager.ValidatorRegistrationParams memory params = IValidatorManager.ValidatorRegistrationParams({
            consensusPublicKey: TEST_CONSENSUS_KEY,
            blsProof: TEST_BLS_PROOF,
            commission: commission,
            moniker: TEST_MONIKER,
            initialOperator: TEST_USER,
            initialBeneficiary: TEST_USER,
            validatorNetworkAddresses: "",
            fullnodeNetworkAddresses: "",
            aptosAddress: ""
        });

        bytes memory validatorParams = abi.encode(params);
        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, validatorParams);

        IJWKManager.JWK memory stakeJWK = IJWKManager.JWK({
            variant: 2, // StakeRegisterValidatorEvent
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = stakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Create CrossChainParams for validator stake event
        IJWKManager.CrossChainParams[] memory crossChainParams = new IJWKManager.CrossChainParams[](1);
        crossChainParams[0] = IJWKManager.CrossChainParams({
            id: bytes("1"), // StakeRegisterValidatorEvent
            validatorParams: params,
            targetValidator: address(0),
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: TEST_ISSUER
        });

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, crossChainParams);

        // Assert
        assertTrue(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertEq(_getValidatorManagerMock().lastStakeUser(), TEST_USER);
        assertEq(_getValidatorManagerMock().lastStakeAmount(), TEST_STAKE_AMOUNT);
        assertEq(_getValidatorManagerMock().lastValidatorParams(), validatorParams);
    }

    /// @notice Test processing delegation stake event JWK, should call delegate method
    function test_processStakeEventJWKs_delegationStakeEvent_shouldCallDelegate() public {
        // Arrange
        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, TEST_TARGET_VALIDATOR);

        IJWKManager.JWK memory stakeJWK = IJWKManager.JWK({
            variant: 3, // StakeEvent
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = stakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Create CrossChainParams for delegation stake event
        IJWKManager.CrossChainParams[] memory crossChainParams = new IJWKManager.CrossChainParams[](1);
        crossChainParams[0] = IJWKManager.CrossChainParams({
            id: bytes("2"), // StakeEvent
            validatorParams: IValidatorManager.ValidatorRegistrationParams({
                consensusPublicKey: "",
                blsProof: "",
                commission: IValidatorManager.Commission({rate: 0, maxRate: 0, maxChangeRate: 0}),
                moniker: "",
                initialOperator: address(0),
                initialBeneficiary: address(0),
                validatorNetworkAddresses: "",
                fullnodeNetworkAddresses: "",
                aptosAddress: ""
            }),
            targetValidator: TEST_TARGET_VALIDATOR,
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: TEST_ISSUER
        });

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, crossChainParams);

        // Assert
        assertTrue(_getDelegationMock().stakeEventEmitted());
        assertEq(_getDelegationMock().lastStakeUser(), TEST_USER);
        assertEq(_getDelegationMock().lastStakeAmount(), TEST_STAKE_AMOUNT);
        assertEq(_getDelegationMock().lastTargetValidator(), TEST_TARGET_VALIDATOR);
    }

    /// @notice Test processing normal JWK (non-stake event), should not process stake
    function test_processStakeEventJWKs_normalJWK_shouldNotProcess() public {
        // Arrange - Create a normal RSA JWK (variant = 0)
        IJWKManager.RSA_JWK memory rsaJWK = IJWKManager.RSA_JWK({
            kid: "test-key-id",
            kty: "RSA",
            alg: "RS256",
            e: "AQAB",
            n: "test-modulus"
        });

        IJWKManager.JWK memory normalJWK = IJWKManager.JWK({
            variant: 0, // RSA_JWK
            data: abi.encode(rsaJWK)
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = normalJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));

        // Assert - Should not emit stake events
        assertFalse(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertFalse(_getDelegationMock().stakeEventEmitted());
    }

    /// @notice Test processing provider with multiple JWKs, should not process (only process single JWK providers)
    function test_processStakeEventJWKs_multipleJWKs_shouldProcessOnlySingleJWK() public {
        // Arrange - Create provider with multiple JWKs (should not process)
        IJWKManager.RSA_JWK memory rsaJWK = IJWKManager.RSA_JWK({
            kid: "test-key-id",
            kty: "RSA",
            alg: "RS256",
            e: "AQAB",
            n: "test-modulus"
        });

        IJWKManager.JWK memory normalJWK = IJWKManager.JWK({
            variant: 0, // RSA_JWK
            data: abi.encode(rsaJWK)
        });

        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, TEST_TARGET_VALIDATOR);
        IJWKManager.JWK memory stakeJWK = IJWKManager.JWK({
            variant: 3, // StakeEvent
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](2)
        });
        providerJWKs.jwks[0] = normalJWK;
        providerJWKs.jwks[1] = stakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));

        // Assert - Should not process because array has more than 1 element
        assertFalse(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertFalse(_getDelegationMock().stakeEventEmitted());
    }

    /// @notice Test processing unsupported JWK type, should not process stake
    function test_processStakeEventJWKs_unsupportedJWK_shouldNotProcess() public {
        // Arrange - Create an unsupported JWK (variant = 1)
        IJWKManager.UnsupportedJWK memory unsupportedJWK = IJWKManager.UnsupportedJWK({
            id: "unsupported-id",
            payload: "unsupported-payload"
        });

        IJWKManager.JWK memory unsupportedJWKStruct = IJWKManager.JWK({
            variant: 1, // UnsupportedJWK
            data: abi.encode(unsupportedJWK)
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = unsupportedJWKStruct;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));

        // Assert - Should not emit stake events
        assertFalse(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertFalse(_getDelegationMock().stakeEventEmitted());
    }

    /// @notice Test processing multiple providers, each provider should be processed
    function test_processStakeEventJWKs_multipleProviders_shouldProcessEach() public {
        // Arrange - Create two providers, each with a single stake JWK
        bytes memory validatorStakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, abi.encode(
            IValidatorManager.ValidatorRegistrationParams({
                consensusPublicKey: TEST_CONSENSUS_KEY,
                blsProof: TEST_BLS_PROOF,
                commission: IValidatorManager.Commission({
                    rate: 1000,
                    maxRate: 5000,
                    maxChangeRate: 500
                }),
                moniker: TEST_MONIKER,
                initialOperator: TEST_USER,
                initialBeneficiary: TEST_USER,
                validatorNetworkAddresses: "",
                fullnodeNetworkAddresses: "",
                aptosAddress: ""
            })
        ));

        bytes memory delegationStakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, TEST_TARGET_VALIDATOR);

        IJWKManager.JWK memory validatorStakeJWK = IJWKManager.JWK({
            variant: 2, // StakeRegisterValidatorEvent
            data: validatorStakeData
        });

        IJWKManager.JWK memory delegationStakeJWK = IJWKManager.JWK({
            variant: 3, // StakeEvent
            data: delegationStakeData
        });

        IJWKManager.ProviderJWKs memory provider1 = IJWKManager.ProviderJWKs({
            issuer: "https://provider1.com",
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        provider1.jwks[0] = validatorStakeJWK;

        IJWKManager.ProviderJWKs memory provider2 = IJWKManager.ProviderJWKs({
            issuer: "https://provider2.com",
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        provider2.jwks[0] = delegationStakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](2);
        providerJWKsArray[0] = provider1;
        providerJWKsArray[1] = provider2;

        // Create CrossChainParams for both events
        IJWKManager.CrossChainParams[] memory crossChainParams = new IJWKManager.CrossChainParams[](2);
        crossChainParams[0] = IJWKManager.CrossChainParams({
            id: bytes("1"), // StakeRegisterValidatorEvent
            validatorParams: IValidatorManager.ValidatorRegistrationParams({
                consensusPublicKey: TEST_CONSENSUS_KEY,
                blsProof: TEST_BLS_PROOF,
                commission: IValidatorManager.Commission({rate: 1000, maxRate: 5000, maxChangeRate: 500}),
                moniker: TEST_MONIKER,
                initialOperator: TEST_USER,
                initialBeneficiary: TEST_USER,
                validatorNetworkAddresses: "",
                fullnodeNetworkAddresses: "",
                aptosAddress: ""
            }),
            targetValidator: address(0),
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: "https://provider1.com"
        });
        crossChainParams[1] = IJWKManager.CrossChainParams({
            id: bytes("2"), // StakeEvent
            validatorParams: IValidatorManager.ValidatorRegistrationParams({
                consensusPublicKey: "",
                blsProof: "",
                commission: IValidatorManager.Commission({rate: 0, maxRate: 0, maxChangeRate: 0}),
                moniker: "",
                initialOperator: address(0),
                initialBeneficiary: address(0),
                validatorNetworkAddresses: "",
                fullnodeNetworkAddresses: "",
                aptosAddress: ""
            }),
            targetValidator: TEST_TARGET_VALIDATOR,
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: "https://provider2.com"
        });

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, crossChainParams);

        // Assert - Both should be processed
        assertTrue(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertTrue(_getDelegationMock().stakeEventEmitted());
    }

    // ============ PARAMETER EXTRACTION TESTS ============

    /// @notice Test extracting validator stake parameters, valid data should be extracted correctly
    function test_extractValidatorStakeParams_validData_shouldExtractCorrectly() public {
        // Arrange
        IValidatorManager.ValidatorRegistrationParams memory params = IValidatorManager.ValidatorRegistrationParams({
            consensusPublicKey: TEST_CONSENSUS_KEY,
            blsProof: TEST_BLS_PROOF,
            commission: IValidatorManager.Commission({
                rate: 1000,
                maxRate: 5000,
                maxChangeRate: 500
            }),
            moniker: TEST_MONIKER,
            initialOperator: TEST_USER,
            initialBeneficiary: TEST_USER,
            validatorNetworkAddresses: "",
            fullnodeNetworkAddresses: "",
            aptosAddress: ""
        });

        bytes memory validatorParams = abi.encode(params);
        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, validatorParams);

        // Act - This is tested indirectly through the main function
        IJWKManager.JWK memory stakeJWK = IJWKManager.JWK({
            variant: 2,
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = stakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Create CrossChainParams for validator stake event
        IJWKManager.CrossChainParams[] memory crossChainParams = new IJWKManager.CrossChainParams[](1);
        crossChainParams[0] = IJWKManager.CrossChainParams({
            id: bytes("1"), // StakeRegisterValidatorEvent
            validatorParams: params,
            targetValidator: address(0),
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: TEST_ISSUER
        });

        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, crossChainParams);

        // Assert - Parameters should be extracted correctly
        assertEq(_getValidatorManagerMock().lastStakeUser(), TEST_USER);
        assertEq(_getValidatorManagerMock().lastStakeAmount(), TEST_STAKE_AMOUNT);
        assertEq(_getValidatorManagerMock().lastValidatorParams(), validatorParams);
    }

    /// @notice Test extracting delegation stake parameters, valid data should be extracted correctly
    function test_extractDelegationStakeParams_validData_shouldExtractCorrectly() public {
        // Arrange
        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, TEST_TARGET_VALIDATOR);

        // Act - This is tested indirectly through the main function
        IJWKManager.JWK memory stakeJWK = IJWKManager.JWK({
            variant: 3,
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = stakeJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Create CrossChainParams for delegation stake event
        IJWKManager.CrossChainParams[] memory crossChainParams = new IJWKManager.CrossChainParams[](1);
        crossChainParams[0] = IJWKManager.CrossChainParams({
            id: bytes("2"), // StakeEvent
            validatorParams: IValidatorManager.ValidatorRegistrationParams({
                consensusPublicKey: "",
                blsProof: "",
                commission: IValidatorManager.Commission({rate: 0, maxRate: 0, maxChangeRate: 0}),
                moniker: "",
                initialOperator: address(0),
                initialBeneficiary: address(0),
                validatorNetworkAddresses: "",
                fullnodeNetworkAddresses: "",
                aptosAddress: ""
            }),
            targetValidator: TEST_TARGET_VALIDATOR,
            shares: TEST_STAKE_AMOUNT,
            blockNumber: block.number,
            issuer: TEST_ISSUER
        });

        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, crossChainParams);

        // Assert - Parameters should be extracted correctly
        assertEq(_getDelegationMock().lastStakeUser(), TEST_USER);
        assertEq(_getDelegationMock().lastStakeAmount(), TEST_STAKE_AMOUNT);
        assertEq(_getDelegationMock().lastTargetValidator(), TEST_TARGET_VALIDATOR);
    }

    // ============ ACCESS CONTROL TESTS ============

    /// @notice Test non-system caller calling upsertObservedJWKs, should revert
    function test_upsertObservedJWKs_nonSystemCaller_shouldRevert() public {
        // Arrange
        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](0);

        // Act & Assert
        vm.expectRevert();
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));
    }

    // ============ EDGE CASE TESTS ============

    /// @notice Test processing empty provider array, should not process anything
    function test_processStakeEventJWKs_emptyProviderArray_shouldNotProcess() public {
        // Arrange
        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](0);

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));

        // Assert - Should not emit any events
        assertFalse(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertFalse(_getDelegationMock().stakeEventEmitted());
    }

    /// @notice Test processing JWK with invalid variant, should not process stake
    function test_processStakeEventJWKs_invalidVariant_shouldNotProcess() public {
        // Arrange - Create JWK with invalid variant (4)
        bytes memory stakeData = abi.encode(TEST_USER, TEST_STAKE_AMOUNT, TEST_TARGET_VALIDATOR);

        IJWKManager.JWK memory invalidJWK = IJWKManager.JWK({
            variant: 4, // Invalid variant
            data: stakeData
        });

        IJWKManager.ProviderJWKs memory providerJWKs = IJWKManager.ProviderJWKs({
            issuer: TEST_ISSUER,
            version: 1,
            jwks: new IJWKManager.JWK[](1)
        });
        providerJWKs.jwks[0] = invalidJWK;

        IJWKManager.ProviderJWKs[] memory providerJWKsArray = new IJWKManager.ProviderJWKs[](1);
        providerJWKsArray[0] = providerJWKs;

        // Act
        vm.prank(SYSTEM_CALLER);
        jwkManager.upsertObservedJWKs(providerJWKsArray, new IJWKManager.CrossChainParams[](0));

        // Assert - Should not emit any events
        assertFalse(_getValidatorManagerMock().stakeRegisterValidatorEventEmitted());
        assertFalse(_getDelegationMock().stakeEventEmitted());
    }

    // Helper functions to avoid type conversion issues
    function _getValidatorManagerMock() internal pure returns (JWKManagerMock) {
        return JWKManagerMock(payable(VALIDATOR_MANAGER_ADDR));
    }

    function _getDelegationMock() internal pure returns (JWKManagerMock) {
        return JWKManagerMock(payable(DELEGATION_ADDR));
    }
}
