## Why

当 Immutable Record 需要修正时，必须通过Correction Record机制记录修正历史，而不是直接修改原记录。这保证了GMP合规所需的数据不可篡改性，同时允许通过合规渠道进行数据修正。

## What Changes

- 新增 Correction Chain 模块，实现数据修正链
- 新增 `correction_records` 表结构
- 实现修正记录创建、验证、查询API
- 与电子签名集成，确保修正操作经过授权审批
- 与审计链集成，记录修正操作的完整性

## Capabilities

### New Capabilities

- `correction-record`: 修正记录核心数据结构，支持 original_id, corrected_id, correction_reason, corrector_id, authorization 等字段
- `correction-chain`: 修正链管理，维护记录间的引用关系和完整性校验
- `correction-workflow`: 修正审批工作流，与 ApprovalPolicy 集成
- `correction-api`: 修正操作 API，支持创建、查询、验证修正记录

## Impact

- 新增 `crates/gmp/src/correction.rs` 模块
- 新增 `crates/gmp/src/correction_chain.rs` 模块
- 修改 `crates/gmp/src/lib.rs` 导出新模块
- 与 `electronic_signature` 模块深度集成
- 与 `audit_chain` 模块集成进行完整性校验
