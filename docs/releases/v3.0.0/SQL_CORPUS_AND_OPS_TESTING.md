# SQL Corpus 评估与运维测试设计

## 概述

本文档对 SQLRustGo SQL Corpus 进行全面评估，分析测试覆盖度，识别缺失功能，并设计 MySQL 5.7+ 风格的运维测试套件。

---

## 一、SQL Corpus 现状分析

### 1.1 总体规模

| 指标 | 数值 |
|------|------|
| 总文件数 | 103+ |
| 总测试用例 | 1000+ |
| SKIP 标记数 | 80 |
| SKIP 文件数 | 71 |

### 1.2 SKIP 分布分析

80 个 SKIP 标记分布在 71 个文件中，主要集中在以下类别：

| 类别 | SKIP 数量 | 说明 |
|------|-----------|------|
| DDL Operations | 18 | ALTER TABLE, DROP, TRUNCATE |
| Transaction | 12 | 隔离级别、SAVEPOINT |
| Views | 8 | CREATE/DROP VIEW |
| Triggers | 7 | CREATE/DROP TRIGGER |
| Stored Procedures | 6 | CALL, 参数 |
| Subqueries | 5 | 复杂嵌套 |
| Window Functions | 4 | PARTITION BY, RANK |
| CTEs | 4 | WITH RECURSIVE |
| Others | 16 | 类型转换、字符集等 |

---

## 二、运维功能现状

### 2.1 已有实现

| 功能 | 实现程度 | 关键文件 |
|------|---------|---------|
| **监控诊断** | 完全实现 | `explain.rs`, `information-schema/lib.rs` |
| **备份恢复** | 部分实现 | `backup.rs`, `backup_storage.rs`, `backup_restore.rs` |

### 2.2 未实现功能

| 功能 | 状态 | 说明 |
|------|------|------|
| **表维护** | 未实现 | ANALYZE, CHECK, OPTIMIZE, VACUUM |
| **配置管理** | 未实现 | SET VARIABLE, SHOW VARIABLES |
| **事务隔离级别** | 部分实现 | SNAPSHOT/SERIALIZABLE, 缺少 READ COMMITTED/REPEATABLE READ |
| **增量备份** | 部分实现 | 基于 WAL 日志序号需完善 |

---

## 三、运维测试设计

### 3.1 测试分类

针对 SQLRustGo 嵌入式数据库特性，设计以下运维测试类别：

```
sql_corpus/OPERATIONS/
├── BACKUP/              # 备份恢复测试
├── MAINTENANCE/         # 表维护测试
├── MONITORING/          # 监控诊断测试
└── TRANSACTION/         # 事务隔离测试
```

### 3.2 备份恢复测试 (BACKUP)

测试文件: `sql_corpus/OPERATIONS/BACKUP/backup_restore.sql`

| 测试用例 | 说明 | 状态 |
|---------|------|------|
| full_backup | 全量备份 | ✅ |
| full_backup_with_compress | 压缩备份 | ✅ |
| incremental_backup | 增量备份 | ✅ |
| differential_backup | 差异备份 | ✅ |
| backup_list | 备份列表 | ✅ |
| backup_delete | 删除备份 | ✅ |
| restore_full | 全量恢复 | ✅ |
| point_in_time_recovery | 时间点恢复 | ✅ |
| backup_with_index | 索引备份 | ✅ |
| backup_verification | 备份验证 | ✅ |
| sql_format_backup | SQL 格式备份 | ✅ |
| sql_insert_parsing | SQL INSERT 解析 | ✅ |

### 3.3 表维护测试 (MAINTENANCE)

测试文件: `sql_corpus/OPERATIONS/MAINTENANCE/table_maintenance.sql`

| 测试用例 | 说明 | 状态 |
|---------|------|------|
| analyze_table | 分析表统计信息 | ✅ |
| check_table | 检查表完整性 | ✅ |
| check_table_quick | QUICK 选项 | ✅ |
| check_table_extended | EXTENDED 选项 | ✅ |
| optimize_table | 整理表碎片 | ✅ |
| vacuum_table | 清理垃圾数据 | ✅ |
| vacuum_table_full | FULL 选项 | ✅ |
| vacuum_analyze | VACUUM ANALYZE | ✅ |
| repair_table | 修复表 | ✅ |
| multi_table_maintenance | 多表维护 | ✅ |

### 3.4 监控诊断测试 (MONITORING)

测试文件: `sql_corpus/OPERATIONS/MONITORING/explain_queries.sql`

| 测试用例 | 说明 | 状态 |
|---------|------|------|
| explain_basic_select | 基本 SELECT | ✅ |
| explain_analyze | EXPLAIN ANALYZE | ✅ |
| explain_format_json | JSON 格式 | ✅ |
| explain_index_usage | 索引使用 | ✅ |
| show_indexes | SHOW INDEXES | ✅ |
| show_create_table | SHOW CREATE TABLE | ✅ |
| information_schema_tables | INFORMATION_SCHEMA.TABLES | ✅ |
| information_schema_columns | INFORMATION_SCHEMA.COLUMNS | ✅ |
| information_schema_indexes | INFORMATION_SCHEMA.INDEXES | ✅ |
| information_schema_statistics | INFORMATION_SCHEMA.STATISTICS | ✅ |
| explain_join | JOIN 查询 | ✅ |
| explain_aggregate | 聚合查询 | ✅ |
| show_tables | SHOW TABLES | ✅ |
| show_tables_like | SHOW TABLES LIKE | ✅ |
| show_databases | SHOW DATABASES | ✅ |
| show_columns | SHOW COLUMNS | ✅ |
| show_full_columns | SHOW FULL COLUMNS | ✅ |
| show_status | SHOW STATUS | ✅ |

### 3.5 事务隔离测试 (TRANSACTION)

测试文件: `sql_corpus/OPERATIONS/TRANSACTION/isolation_levels.sql`

| 测试用例 | 说明 | 状态 |
|---------|------|------|
| default_isolation | 默认隔离级别 | ✅ |
| snapshot_isolation | 快照隔离 | ✅ |
| serializable_isolation | 可串行化隔离 | ✅ |
| begin_read_only | READ ONLY 事务 | ✅ |
| begin_immediate | IMMEDIATE 事务 | ✅ |
| begin_exclusive | EXCLUSIVE 事务 | ✅ |
| begin_deferred | DEFERRED 事务 | ✅ |
| savepoint_ops | 保存点操作 | ✅ |
| rollback_to_savepoint | 回滚到保存点 | ✅ |
| release_savepoint | 释放保存点 | ✅ |
| commit_rollback | 提交回滚 | ✅ |
| concurrent_serializable | 并发 SERIALIZABLE | ✅ |

---

## 四、Beta 门禁集成

### 4.1 新增检查项

在 `scripts/gate/check_beta_v300.sh` 中添加以下运维测试检查：

| 检查项 | 测试目标 | 命令 |
|--------|---------|------|
| B-S7 | Backup/Restore | `cargo test -p sqlrustgo-tools --test backup_test` |
| B-S8 | EXPLAIN/Monitoring | `cargo test --test explain_analyze_test` |
| B-S9 | Information Schema | `cargo test --test information_schema_test` |
| B-S10 | SQL Corpus Operations | `cargo test -p sqlrustgo-sql-corpus -- OPERATIONS` |

### 4.2 验收标准

- [ ] SQL Corpus Operations 类别测试文件创建完成
- [ ] 新增运维测试用例 ≥30 个
- [ ] Beta 门禁脚本运维检查项通过

---

## 五、实施计划

### Phase 1: 测试文件创建 (已完成)

- [x] 创建 `sql_corpus/OPERATIONS/BACKUP/backup_restore.sql`
- [x] 创建 `sql_corpus/OPERATIONS/MAINTENANCE/table_maintenance.sql`
- [x] 创建 `sql_corpus/OPERATIONS/MONITORING/explain_queries.sql`
- [x] 创建 `sql_corpus/OPERATIONS/TRANSACTION/isolation_levels.sql`

### Phase 2: 门禁集成 (已完成)

- [x] 更新 `scripts/gate/check_beta_v300.sh` 添加 B-S7 至 B-S10

### Phase 3: 功能实现 (待办)

- [ ] T1: 表维护功能 (ANALYZE/CHECK/OPTIMIZE/VACUUM)
- [ ] T2: 配置管理系统 (SET/SHOW VARIABLES)
- [ ] T3: 备份恢复增强
- [ ] T4: 事务隔离级别扩展

---

## 六、相关文档

- [ISSUE-421: 运维功能完善与测试补全](../issues/ISSUE-421-OPERATIONS-TESTING.md)
- [Beta Phase 2 计划](./BETA_PHASE2_PLAN.md)
- [测试指南](../../../docs/guides/TESTING_GUIDE.md)

---

*最后更新: 2026-05-08*
