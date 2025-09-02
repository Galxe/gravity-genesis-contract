我现在的项目背景是这样的，我在src下面的代码 是gravity链的智能合约，然后我现在要做一个跨链映射的功能，我会在eth主网部署一系列gravity bridge合约. 核心的功能是stake和unstake合约. 我会在gravity这里有一个relayer，监控eth主网上的event来判断是否发生了stake和unstake。我需要通过这两个事件来在gravity chain上把对应用户的G token的余额转变成validator节点的信息，这样用户在stake的时候可以选择是否成为gravity的validtor节点候选人等待epoch切换后正式成为gravity的validator. 那么我在设计的时候就有些要点需要你帮我一起考虑了.

# 跨链映射功能完整设计

## 设计问题分析与修正

### **原设计问题**
1. **参数传递不完整**：原设计只传递 `ValidatorRegistrationParams`，但 `registerValidator` 需要 `msg.value` 作为初始质押
2. **状态同步缺失**：没有考虑跨链状态一致性和重试机制  
3. **权限控制缺失**：没有验证跨链操作的合法性
4. **ValidatorRegistrationParams 结构复杂**：包含多个字段需要正确序列化和反序列化

## Stake 功能设计

### **ETH 主网 Gravity Bridge 合约设计**

```solidity
// ETH 主网 Gravity Bridge 合约
contract GravityBridge {
    struct CrossChainStakeRequest {
        address user;
        uint256 amount;
        bool wantToBeValidator;
        bytes validatorParams; // 序列化后的 ValidatorRegistrationParams
        uint256 nonce;
        bytes signature; // 用户签名
    }
    
    function stake(CrossChainStakeRequest calldata request) external payable {
        require(msg.value == request.amount, "Amount mismatch");
        
        if (request.wantToBeValidator) {
            // 如果用户想成为 validator，emit 特殊事件
            emit StakeRegisterValidatorEvent(
                request.user, 
                request.amount,
                request.validatorParams,
                request.nonce
            );
        } else {
            // 普通质押事件
            emit StakeEvent(
                request.user, 
                request.amount,
                request.nonce
            );
        }
        
        // 锁定 ETH 到合约
        // 记录用户质押状态
    }
}
```

### **ValidatorRegistrationParams 序列化设计**

**序列化格式：**
```solidity
// 在 ETH 主网合约中序列化
function serializeValidatorParams(ValidatorRegistrationParams memory params) 
    internal pure returns (bytes memory) {
    return abi.encode(
        params.consensusPublicKey,
        params.blsProof,
        abi.encode(params.commission.rate, params.commission.maxRate, params.commission.maxChangeRate),
        params.moniker,
        params.initialOperator,
        params.initialVoter,
        params.initialBeneficiary,
        params.validatorNetworkAddresses,
        params.fullnodeNetworkAddresses,
        params.aptosAddress
    );
}

// 在 Gravity 链上反序列化
function deserializeValidatorParams(bytes memory data) 
    internal pure returns (ValidatorRegistrationParams memory params) {
    (
        params.consensusPublicKey,
        params.blsProof,
        bytes memory commissionData,
        params.moniker,
        params.initialOperator,
        params.initialVoter,
        params.initialBeneficiary,
        params.validatorNetworkAddresses,
        params.fullnodeNetworkAddresses,
        params.aptosAddress
    ) = abi.decode(data, (
        bytes, bytes, bytes, string, address, address, address, bytes, bytes, bytes
    ));
    
    // 解析 Commission 结构
    (params.commission.rate, params.commission.maxRate, params.commission.maxChangeRate) = 
        abi.decode(commissionData, (uint64, uint64, uint64));
}
```

### **Relayer 监控 Stake 事件**

```typescript
// Relayer 监控 ETH 主网事件
class GravityRelayer {
    async monitorStakeEvents() {
        // 监控 StakeRegisterValidatorEvent
        const filter = gravityBridge.filters.StakeRegisterValidatorEvent();
        
        gravityBridge.on(filter, async (user, amount, validatorParams, nonce) => {
            try {
                // 1. 验证事件有效性
                await this.validateStakeEvent(user, amount, nonce);
                
                // 2. 在 Gravity 链上执行跨链操作
                await this.executeCrossChainStake(user, amount, validatorParams);
                
                // 3. 记录处理状态
                await this.recordProcessedEvent(nonce);
                
            } catch (error) {
                console.error(`Failed to process stake event: ${error}`);
                // 重试机制
            }
        });
    }
}
```

### **Gravity 链上 Stake 处理**

```solidity
// Gravity 链上的跨链处理合约
contract CrossChainProcessor {
    mapping(bytes32 => bool) public processedEvents;
    
    function processStakeWithValidator(
        address user,
        uint256 amount,
        bytes calldata validatorParams,
        bytes32 eventHash,
        bytes calldata ethProof // ETH 主网事件证明
    ) external onlyRelayer {
        // 1. 验证事件未被处理
        require(!processedEvents[eventHash], "Event already processed");
        
        // 2. 验证 ETH 主网事件证明
        require(validateEthEventProof(ethProof), "Invalid proof");
        
        // 3. 反序列化 validator 参数
        ValidatorRegistrationParams memory params = deserializeValidatorParams(validatorParams);
        
        // 4. 调用 ValidatorManager 注册 validator
        // 注意：这里需要传递 amount 作为 msg.value
        IValidatorManager(VALIDATOR_MANAGER_ADDR).registerValidator{value: amount}(params);
        
        // 5. 标记事件已处理
        processedEvents[eventHash] = true;
        
        emit CrossChainStakeProcessed(user, amount, eventHash);
    }
}
```

### **完整 Stake 流程**

```
ETH 用户 → ETH Gravity Bridge → Relayer 监控 → Gravity 链处理 → Validator 注册
    ↓              ↓                ↓              ↓              ↓
发起质押    锁定 ETH + 发事件    解析事件      验证证明      注册 + 部署 StakeCredit
```

**详细步骤：**
1. **ETH 用户发起质押**：调用 `stake()` 函数，选择是否成为 validator
2. **ETH 合约处理**：锁定 ETH，emit 相应事件
3. **Relayer 监控**：捕获事件，验证有效性
4. **跨链处理**：在 Gravity 链上执行对应的质押操作
5. **Validator 注册**：如果选择成为 validator，调用 `registerValidator()`

## Unstake 功能设计

### **Unstake 场景分析**

基于现有的 ValidatorManager 实现，unstake 功能需要考虑以下几种情况：

1. **普通用户 unstake**：从 StakeCredit 合约中提取质押的资金
2. **Validator 退出**：validator 主动退出验证者集合
3. **强制退出**：validator 质押不足被系统强制退出

### **ETH 主网 Unstake 合约设计**

```solidity
// ETH 主网 Gravity Bridge 合约 - Unstake 功能
contract GravityBridge {
    // ... 现有的 stake 功能 ...
    
    struct CrossChainUnstakeRequest {
        address user;
        uint256 amount;
        bool isValidatorExit; // 是否是 validator 退出
        address validatorAddress; // 如果是 validator 退出，需要指定地址
        uint256 nonce;
        bytes signature;
    }
    
    function unstake(CrossChainUnstakeRequest calldata request) external {
        // 验证用户身份和签名
        require(validateUnstakeRequest(request), "Invalid unstake request");
        
        if (request.isValidatorExit) {
            // Validator 退出事件
            emit ValidatorExitRequested(
                request.user,
                request.validatorAddress,
                request.amount,
                request.nonce
            );
        } else {
            // 普通用户 unstake 事件
            emit UnstakeEvent(
                request.user,
                request.amount,
                request.nonce
            );
        }
        
        // 解锁 ETH 并转给用户
        // 更新用户质押状态
    }
    
    // 批量 unstake 支持
    function batchUnstake(CrossChainUnstakeRequest[] calldata requests) external {
        for (uint256 i = 0; i < requests.length; i++) {
            unstake(requests[i]);
        }
    }
}
```

### **Validator 退出流程设计**

#### **1. Validator 主动退出**

```solidity
// Gravity 链上的跨链处理合约 - Validator 退出
contract CrossChainProcessor {
    // ... 现有的 stake 处理功能 ...
    
    function processValidatorExit(
        address user,
        address validatorAddress,
        uint256 amount,
        bytes32 eventHash,
        bytes calldata ethProof
    ) external onlyRelayer {
        // 1. 验证事件未被处理
        require(!processedEvents[eventHash], "Event already processed");
        
        // 2. 验证 ETH 主网事件证明
        require(validateEthEventProof(ethProof), "Invalid proof");
        
        // 3. 检查 validator 状态
        ValidatorInfo memory validatorInfo = IValidatorManager(VALIDATOR_MANAGER_ADDR).getValidatorInfo(validatorAddress);
        require(validatorInfo.registered, "Validator not registered");
        
        // 4. 调用 ValidatorManager 退出验证者集合
        IValidatorManager(VALIDATOR_MANAGER_ADDR).leaveValidatorSet(validatorAddress);
        
        // 5. 标记事件已处理
        processedEvents[eventHash] = true;
        
        emit ValidatorExitProcessed(user, validatorAddress, amount, eventHash);
    }
}
```

#### **2. 系统强制退出处理**

```solidity
// 在 ValidatorManager 中的强制退出逻辑
contract ValidatorManager {
    // ... 现有代码 ...
    
    /**
     * @dev 处理跨链强制退出请求
     */
    function processCrossChainForceExit(
        address validator,
        bytes32 eventHash,
        bytes calldata ethProof
    ) external onlyRelayer {
        // 1. 验证事件证明
        require(validateEthEventProof(ethProof), "Invalid proof");
        
        // 2. 检查 validator 状态
        ValidatorInfo storage info = validatorInfos[validator];
        require(info.registered, "Validator not registered");
        
        // 3. 强制退出逻辑
        if (info.status == ValidatorStatus.ACTIVE) {
            // 从活跃集合中移除
            activeValidators.remove(validator);
            delete activeValidatorIndex[validator];
            
            // 添加到 pending_inactive
            pendingInactive.add(validator);
            pendingInactiveIndex[validator] = pendingInactive.length() - 1;
            
            // 更新状态
            info.status = ValidatorStatus.PENDING_INACTIVE;
            
            emit ValidatorStatusChanged(
                validator, 
                uint8(ValidatorStatus.ACTIVE), 
                uint8(ValidatorStatus.PENDING_INACTIVE), 
                uint64(IEpochManager(EPOCH_MANAGER_ADDR).currentEpoch())
            );
        }
        
        // 4. 标记事件已处理
        processedEvents[eventHash] = true;
    }
}
```

### **Unstake 状态流转**

```
Validator 状态流转：
ACTIVE → PENDING_INACTIVE → INACTIVE → 资金可提取

普通用户 unstake：
StakeCredit.unlock() → pendingInactive → inactive → claim()
```

### **Relayer 监控 Unstake 事件**

```typescript
// Relayer 监控 unstake 事件
class GravityRelayer {
    async monitorUnstakeEvents() {
        // 监控普通 unstake 事件
        const unstakeFilter = gravityBridge.filters.UnstakeEvent();
        gravityBridge.on(unstakeFilter, async (user, amount, nonce) => {
            await this.processUnstakeEvent(user, amount, nonce);
        });
        
        // 监控 validator 退出事件
        const validatorExitFilter = gravityBridge.filters.ValidatorExitRequested();
        gravityBridge.on(validatorExitFilter, async (user, validatorAddress, amount, nonce) => {
            await this.processValidatorExitEvent(user, validatorAddress, amount, nonce);
        });
    }
    
    async processUnstakeEvent(user: string, amount: string, nonce: string) {
        try {
            // 在 Gravity 链上执行 unstake
            await this.executeCrossChainUnstake(user, amount, nonce);
        } catch (error) {
            console.error(`Failed to process unstake event: ${error}`);
        }
    }
    
    async processValidatorExitEvent(user: string, validatorAddress: string, amount: string, nonce: string) {
        try {
            // 处理 validator 退出
            await this.executeValidatorExit(user, validatorAddress, amount, nonce);
        } catch (error) {
            console.error(`Failed to process validator exit event: ${error}`);
        }
        }
    }
}
```

### **跨链 Unstake 处理合约**

```solidity
// Gravity 链上的跨链 unstake 处理
contract CrossChainUnstakeProcessor {
    mapping(bytes32 => bool) public processedUnstakeEvents;
    
    function processUnstake(
        address user,
        uint256 amount,
        bytes32 eventHash,
        bytes calldata ethProof
    ) external onlyRelayer {
        // 1. 验证事件未被处理
        require(!processedUnstakeEvents[eventHash], "Event already processed");
        
        // 2. 验证 ETH 主网事件证明
        require(validateEthEventProof(ethProof), "Invalid proof");
        
        // 3. 查找用户的 StakeCredit 合约
        address stakeCreditAddress = _findUserStakeCredit(user);
        require(stakeCreditAddress != address(0), "No StakeCredit found");
        
        // 4. 执行 unstake 操作
        // 注意：这里需要根据具体的 StakeCredit 实现来调用相应的方法
        IStakeCredit(stakeCreditAddress).unlock(user, _calculateSharesForAmount(stakeCreditAddress, amount));
        
        // 5. 标记事件已处理
        processedUnstakeEvents[eventHash] = true;
        
        emit CrossChainUnstakeProcessed(user, amount, eventHash);
    }
    
    function _findUserStakeCredit(address user) internal view returns (address) {
        // 实现查找用户 StakeCredit 合约的逻辑
        // 可能需要遍历所有 validator 的 StakeCredit 合约
        // 或者维护一个用户到 StakeCredit 的映射
    }
    
    function _calculateSharesForAmount(address stakeCreditAddress, uint256 amount) internal view returns (uint256) {
        // 根据金额计算对应的份额
        return IStakeCredit(stakeCreditAddress).getSharesByPooledG(amount);
    }
}
```

### **完整 Unstake 流程**

```
ETH 用户 → ETH Gravity Bridge → Relayer 监控 → Gravity 链处理 → 资金提取
    ↓              ↓                ↓              ↓              ↓
发起 unstake   解锁 ETH + 发事件   解析事件      验证证明      执行 unstake
```

**详细步骤：**
1. **ETH 用户发起 unstake**：调用 `unstake()` 函数
2. **ETH 合约处理**：解锁 ETH，emit 相应事件
3. **Relayer 监控**：捕获事件，验证有效性
4. **跨链处理**：在 Gravity 链上执行对应的 unstake 操作
5. **资金提取**：根据 unstake 类型执行不同的退出逻辑

## 关键设计修正要点

**原设计问题修正：**
1. **完整参数传递**：包含质押金额、validator 参数和事件证明
2. **状态管理**：记录已处理事件，防止重复执行
3. **安全验证**：通过事件证明验证跨链操作的合法性
4. **错误处理**：包含重试机制和失败回滚
5. **ValidatorRegistrationParams 完整处理**：正确处理所有字段的序列化和反序列化

## 注意事项和最佳实践

1. **Gas 费用**：跨链操作需要足够的 G token 支付 gas
2. **时间窗口**：考虑跨链延迟和重试机制
3. **状态一致性**：确保 ETH 和 Gravity 链上的状态同步
4. **安全验证**：通过密码学证明确保跨链操作的真实性
5. **错误处理**：实现完善的错误处理和回滚机制
6. **事件去重**：防止重复处理同一事件

## Unstake 注意事项

1. **Validator 退出限制**：
   - 不能是最后一个活跃 validator
   - 需要等待 epoch 切换完成状态转换
   
2. **资金提取时机**：
   - 普通用户：通过 StakeCredit.unlock() → claim() 流程
   - Validator：需要等待 epoch 切换后状态变为 INACTIVE
   
3. **状态一致性**：
   - 确保 ETH 和 Gravity 链上的质押状态同步
   - 防止重复 unstake 或超额 unstake
   
4. **安全验证**：
   - 验证 unstake 请求的合法性
   - 确保只有质押者本人可以发起 unstake

## 与现有系统的集成

基于现有的 ValidatorManager 和 StakeCredit 实现：

### **Stake 相关：**
- **registerValidator()**：需要 `msg.value` 作为初始质押
- 会自动部署 `StakeCredit` 合约
- 设置 validator 状态为 `INACTIVE`
- 需要满足最小质押要求 `minValidatorStake()`
- 后续可以通过 `joinValidatorSet()` 申请加入活跃验证者集合

### **Unstake 相关：**
- **leaveValidatorSet()**：处理 validator 主动退出
- **onNewEpoch()**：处理 epoch 切换时的状态转换
- **StakeCredit.unlock()**：处理普通用户的 unstake 请求
- **StakeCredit.claim()**：处理资金提取

## 总结

这样的设计确保了跨链映射功能的完整性、安全性和可操作性，同时与现有的 ValidatorManager 实现完全兼容。Stake 和 Unstake 功能设计合理自洽，支持：

1. **完整的跨链流程**：从 ETH 主网到 Gravity 链的完整映射
2. **Validator 生命周期管理**：注册、激活、退出、资金提取
3. **状态一致性**：确保跨链操作的状态同步
4. **安全性**：通过事件证明和权限控制确保操作安全
5. **兼容性**：与现有系统完全兼容，无需修改核心逻辑