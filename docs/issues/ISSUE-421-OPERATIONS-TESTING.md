# ISSUE-421: 运维功能完善与测试补全

## 基本信息

| 字段 | 内容 |
|------|------|
| **Issue ID** | ISSUE-421 |
| **标题** | 运维功能完善与测试补全 |
| **优先级** | P1 |
| **类型** | Feature / Testing |
| **创建日期** | 2026-05-08 |
| **目标版本** | v3.0.0 |
| **状态** | Open |

---

## 一、背景

根据 v3.0.0 RC 测试改进计划和 SQL Corpus 评估分析，发现以下问题：

1. **SQL Corpus 现状**: 80 个 SKIP 标记分布在 71 个文件中，大量运维相关功能未测试
2. **运维功能实现**: 部分运维功能已有基础实现，但缺乏测试覆盖
3. **Beta 门禁**: 需要添加运维测试检查项到 Beta 门禁脚本

---

## 二、现状分析

### 2.1 运维功能实现状态

| 功能 | 实现程度 | 关键文件 |
|------|---------|---------|
| **备份恢复** | 部分实现 | `backup.rs`, `backup_storage.rs`, `backup_restore.rs` |
| **表维护** | **未实现** | 无相关代码 |
| **事务隔离** | 部分实现 | `transaction_manager.rs`, `mvcc.rs` |
| **监控诊断** | 完全实现 | `explain.rs`, `information-schema/lib.rs` |
| **配置管理** | **未实现** | 无相关代码 |

### 2.2 SQL Corpus 运维测试覆盖

已创建测试文件:

```
sql_corpus/OPERATIONS/
├── BACKUP/
│   └── backup_restore.sql     # 备份恢复测试
├── MAINTENANCE/
│   └── table_maintenance.sql  # 表维护测试
├── MONITORING/
│   └── explain_queries.sql    # 监控诊断测试
└── TRANSACTION/
    └── isolation_levels.sql    # 事务隔离测试
```

---

## 三、待完成任务

### T1: 表维护功能实现 [P1]

**缺口**: `vacuum`, `analyze`, `optimize`, `check` 命令未实现

**实现计划**:
1. 在 `storage` crate 添加表维护命令
2. 实现 `ANALYZE TABLE` - 更新表统计信息
3. 实现 `CHECK TABLE` - 检查表完整性
4. 实现 `OPTIMIZE TABLE` - 整理表碎片
5. 实现 `VACUUM TABLE` - 清理垃圾数据

**相关文件**:
- `crates/storage/src/table_manager.rs`
- `crates/executor/src/commands/`

**预计时间**: 8 小时

### T2: 配置管理系统 [P2]

**缺口**: `SET VARIABLE`, `SHOW VARIABLES` 未实现

**实现计划**:
1. 定义系统变量存储结构
2. 实现 `SHOW [GLOBAL|SESSION] VARIABLES`
3. 实现 `SET [GLOBAL|SESSION] var_name = value`
4. 添加配置持久化

**相关文件**:
- `crates/server/src/config.rs`
- `crates/executor/src/commands/`

**预计时间**: 6 小时

### T3: 备份恢复增强 [P1]

**缺口**: 增量备份逻辑不完整，SQL 格式解析需完善

**改进计划**:
1. 完善增量备份逻辑 (基于 WAL 日志序号)
2. 实现真正的 SQL INSERT 语句解析
3. 添加备份校验和验证
4. 实现时间点恢复 (PITR)

**相关文件**:
- `crates/storage/src/backup.rs`
- `crates/storage/src/backup_storage.rs`

**预计时间**: 6 小时

### T4: 事务隔离级别扩展 [P1]

**缺口**: 标准 SQL 隔离级别 (READ COMMITTED, REPEATABLE READ) 未实现

**改进计划**:
1. 添加 `READ COMMITTED` 隔离级别支持
2. 添加 `REPEATABLE READ` 隔离级别支持
3. 验证 MVCC 实现与标准隔离级别的一致性
4. 添加隔离级别相关测试

**相关文件**:
- `crates/transaction/src/transaction_manager.rs`
- `crates/transaction/src/mvcc.rs`

**预计时间**: 8 小时

---

## 四、Beta 门禁检查项

已在 `scripts/gate/check_beta_v300.sh` 中添加:

| 检查项 | 描述 |
|--------|------|
| B-S7 | Backup/Restore 测试 |
| B-S8 | EXPLAIN/Monitoring 测试 |
| B-S9 | Information Schema 测试 |
| B-S10 | SQL Corpus Operations 类别测试 |

---

## 五、验收标准

### 5.1 功能验收

- [ ] 表维护命令 (ANALYZE/CHECK/OPTIMIZE/VACUUM) 可正常执行
- [ ] SHOW/INFORMATION_SCHEMA 查询返回正确元数据
- [ ] EXPLAIN 和 EXPLAIN ANALYZE 输出正确执行计划

### 5.2 测试验收

- [ ] SQL Corpus Operations 类别测试文件创建完成
- [ ] 新增运维测试用例 ≥30 个
- [ ] Beta 门禁脚本运维检查项通过

### 5.3 覆盖率验收

- [ ] 运维功能代码覆盖率 ≥75%
- [ ] SQL Corpus 整体通过率 ≥90%

---

## 六、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 表维护实现复杂 | 可能影响发布计划 | 优先实现 ANALYZE/CHECK，OPTIMIZE/VACUUM P2 |
| 配置管理涉及核心模块 | 风险较高 | 先做调研，明确影响范围 |
| 测试环境依赖存储 | CI 配置复杂 | 使用临时目录，自动清理 |

---

## 七、相关文档

- [SQL Corpus 评估文档](./SQL_CORPUS_AND_OPS_TESTING.md)
- [RC 测试改进计划](./RC_TEST_IMPROVEMENT_PLAN.md)
- [测试指南](../../../docs/guides/TESTING_GUIDE.md)

---

## 八、修改历史

| 日期 | 修改人 | 修改内容 |
|------|--------|----------|
| 2026-05-08 | Claude | 初始创建 Issue |

---

*本文档由 SQLRustGo Team 维护*
