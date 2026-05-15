## Why

数据溯源(Provenance Tracking)是GMP合规的核心要求之一。需要追踪每条记录的来源、变化历史、责任人和时间戳，确保数据的完整性和可追溯性。

## What Changes

- 新增 Provenance Tracking 模块，实现数据血缘追踪
- 新增 `provenance_records` 表结构
- 实现记录来源跟踪、变化历史查询API
- 与电子签名集成，责任人签名验证
- 与审计链集成，溯源数据的完整性验证

## Capabilities

### New Capabilities

- `provenance-record`: 溯源记录结构，支持 source_id, lineage_path, creator_id, create_time, operation_type 等
- `provenance-lineage`: 血缘关系图维护，支持父子记录关系追踪
- `provenance-query`: 溯源查询API，支持按记录、时间范围、责任人等多种维度查询
- `provenance-verification`: 溯源数据验证，确保数据血缘的完整性和真实性

## Impact

- 新增 `crates/gmp/src/provenance.rs` 模块
- 新增 `crates/gmp/src/provenance_lineage.rs` 模块
- 修改 `crates/gmp/src/lib.rs` 导出新模块
- 与 `electronic_signature` 模块集成进行责任人验证
- 与 `audit_chain` 模块集成进行完整性校验
