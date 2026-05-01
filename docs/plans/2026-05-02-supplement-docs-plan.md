# v2.8.0 缺失文档补全计划

> **目标**: 补齐 v2.8.0 缺失的 6 份核心文档
> **依据**: 基于 v2.8.0 develop 分支实际代码/测试状态，严格参照 v2.7.0 文档模板格式
> **数据来源**: cargo test 实际运行结果 + 源码分析 + 现有 v2.8.0 TEST_REPORT.md

---

## 数据事实汇总

### 测试数据 (2026-05-01, cargo test --all-features)

| 指标 | 值 |
|------|-----|
| 单元测试通过 | 258 (0 failed, 33 ignored) |
| 分布式测试通过 | 658 (100%) |
| SQL Corpus 通过 | 174/426 (40.8%) |
| 总测试通过 | 1090 |
| 测试文件 | 31 个 |

### 模块概览

- **存储层**: backup.rs, pitr_recovery.rs, backup_scheduler.rs, backup_storage.rs, WAL (8 entry types + 2PC)
- **安全层**: audit.rs, firewall.rs, encryption.rs, tls.rs, session.rs, alert.rs, cancel.rs
- **分布式**: partition (75), replication (79), failover (55), read_write (27)
- **稳定性**: crash_recovery_test (8), long_run_stability_72h_test (6), long_run_stability_test (8)

---

## 任务列表

### 任务 1: BACKUP_RESTORE_REPORT.md
**依据**: crates/storage/src/backup.rs, pitr_recovery.rs, backup_scheduler.rs, backup_storage.rs, crates/tools/src/backup_restore.rs
**内容**: 导出格式 (CSV/JSON/SQL), 备份类型 (Full/Incremental/Differential), PITR (LSN/Timestamp/TransactionId), 调度器, 存储后端子模块

### 任务 2: COVERAGE_REPORT.md
**依据**: cargo test 实际输出, TEST_REPORT.md, v2.7.0 格式
**内容**: 基于 cargo-tarpaulin (如果可用) 或 测试用例覆盖统计

### 任务 3: INTEGRATION_STATUS.md
**依据**: TEST_REPORT.md + 模块源码分析 + CHANGELOG
**内容**: 各 Phase (A-E) 集成状态, 阻塞依赖, 验证命令

### 任务 4: TEST_MANUAL.md
**依据**: 实际测试命令, 测试分类, 环境要求
**内容**: 环境搭建, 测试分类详解, 常见问题

### 任务 5: SECURITY_ANALYSIS.md
**依据**: crates/security/src/ (audit, firewall, encryption, TLS, session, alert)
**内容**: 安全特性矩阵, 风险分析, 加固建议

### 任务 6: STABILITY_REPORT.md
**依据**: crash_recovery_test, long_run_stability tests, WAL tests
**内容**: 长稳测试, 崩溃恢复, 备份恢复演练, 并发压力

---

## 执行顺序

1. BACKUP_RESTORE_REPORT.md — 代码证据最完整
2. TEST_MANUAL.md — 依赖测试数据
3. INTEGRATION_STATUS.md — 依赖模块分析
4. STABILITY_REPORT.md — 依赖测试结果
5. COVERAGE_REPORT.md — 需要额外数据分析
6. SECURITY_ANALYSIS.md — 依赖安全模块分析
