## Why

v3.2.0 Alpha 阶段即将完成，需要规划 RC/GA 阶段的工作。RC Gate 要求通过 32 项检查，GA Gate 要求通过 42 项检查。需要明确的设计文档、开发和测试改进计划来确保顺利完成门禁。

## What Changes

- 创建 RC 阶段设计文档和开发任务计划
- 创建 GA 阶段设计文档和开发任务计划  
- 定义 QA 增强测试的验收标准
- 明确稳定性测试要求

## Capabilities

### New Capabilities

- `rc-gate-readiness`: RC 门禁就绪计划，包含 R1-R16 核心检查和 R-S1~S16 稳定性测试的验收标准
- `ga-gate-readiness`: GA 门禁就绪计划，包含 G1-G12 核心检查和 G-QA1~QA10 QA 增强测试的验收标准
- `sql-compat-enhancement`: SQL 兼容性增强计划（MERTGE、Event Scheduler）
- `gmp-workflow-engine`: GMP 工作流引擎状态机实现
- `gmp-mobile-trusted-collection`: 移动端可信采集协议
- `gmp-sop-training`: SOP/培训绑定检查
- `gmp-device-calibration`: 设备校准管理
- `perf-qps-optimization`: QPS 优化（PERF-1）
- `perf-tpch-sf10`: TPC-H SF=10 性能测试
- `stability-testing`: 72h 稳定性测试框架
- `formal-proofs`: TLA+ 形式化验证（≥30 proofs）

### Modified Capabilities

- `tpch-real-query-testing`: 扩展 TPC-H 测试覆盖范围

## Impact

- 影响模块: gmp, executor, storage, transaction, network
- 门禁脚本: scripts/gate/check_rc_v320.sh, scripts/gate/check_ga_v320.sh
- 文档: docs/releases/v3.2.0/RC_GATE_CHECKLIST.md, docs/releases/v3.2.0/GA_GATE_CHECKLIST.md
