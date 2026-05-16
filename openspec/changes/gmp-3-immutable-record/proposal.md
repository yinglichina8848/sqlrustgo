## Why

GMP合规要求数据记录不可篡改，需要提供完整的操作证据链。现有的 evidence.rs 已实现 EvidenceChain 核心功能，但未被导出和使用。GMP-3 需要将 EvidenceChain 模块激活、导出，并提供与 GMP 工作流的集成。

## What Changes

- 导出并激活现有的 EvidenceChain 模块
- 创建 `gmp_evidence_records` 表结构存储证据链
- 实现 EvidenceChain 与 AuditChain 的集成
- 提供证据链创建、验证、查询 API
- 与电子签名模块集成，确保证据创建经过授权

## Capabilities

### New Capabilities

- `immutable-record`: 不可变记录核心接口，封装 EvidenceChain 为 GMP 工作流服务
- `evidence-storage`: 证据持久化存储，支持数据库表和 WAL
- `evidence-verification`: 证据链完整性验证，与 AuditChain 协同校验
- `evidence-api`: 证据操作 API，支持创建、查询、验证证据链

### Modified Capabilities

- `audit-chain`: 扩展 AuditChain 以支持与 EvidenceChain 的双向验证

## Impact

- 修改 `crates/gmp/src/lib.rs` 导出 evidence 模块
- 新增 `crates/gmp/src/immutable_record.rs` 封装层
- 新增 `gmp_evidence_records` 表
- 与 `electronic_signature` 模块集成
- 与 `audit_chain` 模块深度集成