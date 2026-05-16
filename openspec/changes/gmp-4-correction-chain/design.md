## Context

GMP合规要求数据记录不可篡改，但在实际运营中错误数据需要被修正。Correction Chain通过创建新的修正记录而非修改原记录来解决这一矛盾。

当前GMP模块已实现：
- AuditChain: 审计链完整性校验
- ElectronicSignature: 电子签名和审批策略
- ImmutableRecord: 不可变记录机制（待完善）

Correction Chain需要与这些模块深度集成。

## Goals / Non-Goals

**Goals:**
- 实现Correction Record数据结构，包含原始记录引用、修正内容、修正原因、授权信息
- 实现Correction Chain维护记录间的引用关系和完整性
- 实现修正审批工作流，与ApprovalPolicy集成
- 与AuditChain集成，记录修正操作的完整性

**Non-Goals:**
- 不实现自动修正（必须人工审批）
- 不实现修正冲突处理（后续版本）
- 不实现跨数据库修正（单库范围）

## Decisions

### Decision 1: Correction Record 存储结构

**选择**: 专用表 `gmp_correction_records`，包含完整溯源信息

**理由**:
- 与业务数据分离，便于审计
- 支持CKKS直接查询修正历史
- 与Immutable Record机制解耦

### Decision 2: Chain 完整性校验

**选择**: 继承 AuditChain 的链式校验机制

**理由**:
- 复用现有完整性校验代码
- 与审计链无缝集成
- 支持增量验证

### Decision 3: 审批工作流集成

**选择**: 复用 ApprovalPolicyEvaluator

**理由**:
- 避免重复实现审批逻辑
- 与电子签名统一
- 配置可复用

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 修正链过长影响性能 | 查询修正历史时需要遍历链 | 限制链深度，或缓存热点路径 |
| 修正记录存储膨胀 | 频繁修正导致存储增长 | 定期归档或压缩历史 |
| 与现有事务模型冲突 | 修正操作需要在事务内完成 | 设计为单对象事务，避免分布式 |

## Open Questions

1. 修正链是否需要支持分支（一个原始记录多个修正版本）？
2. 修正记录是否需要单独的数字签名？
3. 是否需要支持修正过期/撤销机制？
