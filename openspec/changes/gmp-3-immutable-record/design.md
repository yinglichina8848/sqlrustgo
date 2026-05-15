## Context

GMP合规要求数据记录不可篡改，并提供完整的操作证据链。现有的 `evidence.rs` 已实现 `EvidenceChain` 核心功能：
- EvidenceNode: 单个证据节点，包含内容、哈希、引用
- EvidenceChain: 不可变、防篡改的证据链
- EvidenceChainBuilder: 构建器模式
- EvidenceMetadata: 元数据支持

但这些模块未被 lib.rs 导出，无法被外部使用。GMP-3 需要激活并集成这些模块。

## Goals / Non-Goals

**Goals:**
- 导出并激活 EvidenceChain 模块
- 实现证据持久化存储 (gmp_evidence_records 表)
- 与 AuditChain 集成，提供双向验证能力
- 与电子签名集成，确保证据创建经过授权
- 提供完整的 Immutable Record API

**Non-Goals:**
- 不实现跨库证据同步 (单库范围)
- 不实现实时证据计算 (批处理模式)
- 不实现证据压缩/归档 (后续版本)

## Decisions

### Decision 1: 证据存储结构

**选择**: 专用表 `gmp_evidence_records`，JSON 序列化 EvidenceChain

**理由**:
- 简化实现，复用现有 EvidenceChain 结构
- 便于审计查询
- 支持证据链的完整回放

**替代方案**: 单独字段存储
- 缺点: 增加复杂性，需要处理多表关联

### Decision 2: 与 AuditChain 集成方式

**选择**: 双向引用 - AuditChain 记录 EvidenceChain 的根哈希，EvidenceChain 引用 AuditChain 条目

**理由**:
- 支持跨链验证
- 保持模块独立性
- 便于增量验证

### Decision 3: 证据创建触发机制

**选择**: 显式 API 调用 + DML 钩子自动创建

**理由**:
- API 提供精细控制
- DML 钩子保证自动记录
- 灵活性与自动化兼顾

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 证据数据量大 | 存储成本增加 | 分级存储，历史证据归档 |
| 链验证性能 | 深层链验证慢 | 限制链深度，缓存验证结果 |
| 与现有模块冲突 | 集成复杂度高 | 保持模块边界清晰 |

## Open Questions

1. EvidenceChain 与 AuditChain 的验证时机：是同步还是异步？
2. 证据链是否需要支持分支（一个记录的多个版本链）？
3. 是否需要支持证据的过期/吊销机制？