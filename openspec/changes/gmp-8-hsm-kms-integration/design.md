## Context

企业级GMP部署需要硬件级密钥保护。HSM/KMS集成将签名操作卸载到硬件安全模块，满足以下需求：
- 密钥安全存储（不可导出）
- 签名操作在硬件内完成
- 企业级密钥管理策略
- 符合GMP/FDA 21 CFR Part 11 要求

## Goals / Non-Goals

**Goals:**
- 提供统一的HSM接口抽象
- 支持TPM 2.0本地硬件
- 支持PKCS#11标准HSM
- 支持云KMS（AWS/Azure/GCP）
- 提供软件TPM模拟器用于开发测试
- 实现密钥轮换机制

**Non-Goals:**
- 不实现完整的KMS服务器（使用云服务）
- 不实现密钥生命周期管理（仅轮换）
- 不实现多租户隔离

## Decisions

### Decision 1: Provider Trait 抽象

**选择**: 定义统一的 `HsmProvider` trait，所有provider实现此trait

**理由**:
- 解耦具体实现
- 便于切换provider
- 统一接口便于集成

### Decision 2: 软件TPM作为默认

**选择**: SoftwareTpm作为默认provider，用于开发和测试

**理由**:
- 无需硬件即可开发
- 便于CI/CD测试
- 生产环境可切换到真实HSM

### Decision 3: 密钥轮换策略

**选择**: 基于时间的自动轮换 + 手动触发

**理由**:
- 满足合规要求
- 提供灵活性
- 记录轮换历史

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| HSM厂商锁定 | 切换HSM需要改代码 | 抽象provider trait隔离 |
| 云KMS延迟 | 签名操作变慢 | 本地缓存签名结果 |
| 密钥轮换期间服务中断 | 暂时无法签名 | 双密钥并行期 |

## Open Questions

1. 是否需要支持HSM集群（高可用）？
2. 密钥轮换时如何处理未完成的签名请求？
3. 是否需要支持离线HSM（冷存储）？
