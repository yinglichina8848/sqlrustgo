# v3.0.0 功能矩阵

> **版本**: v3.0.0
> **日期**: 2026-05-07
> **阶段**: GA (General Availability)

## 1. SQL 功能

### 1.1 DDL 命令

| 功能 | 状态 | 备注 |
|------|------|------|
| CREATE TABLE | ✅ | |
| DROP TABLE | ✅ | |
| CREATE TABLE IF NOT EXISTS | ✅ | |
| DROP TABLE IF EXISTS | ✅ | |
| ALTER TABLE ADD COLUMN | ✅ | |
| ALTER TABLE DROP COLUMN | ✅ | |
| ALTER TABLE MODIFY COLUMN | ✅ | |
| ALTER TABLE RENAME TO | ✅ | |
| CREATE VIEW | ✅ | |
| DROP VIEW | ✅ | |
| DROP VIEW IF EXISTS | ✅ | |
| CREATE INDEX | ✅ | |
| CREATE UNIQUE INDEX | ✅ | |
| DROP INDEX | ✅ | |
| DROP INDEX IF EXISTS | ✅ | |
| TRUNCATE TABLE | ✅ | |

### 1.2 DML 命令

| 功能 | 状态 | 备注 |
|------|------|------|
| SELECT | ✅ | |
| INSERT | ✅ | |
| INSERT ON DUPLICATE KEY UPDATE | ✅ | |
| REPLACE INTO | ✅ | |
| UPDATE | ✅ | |
| DELETE | ✅ | |
| **INSERT INTO ... SELECT** | ✅ | **v3.0.0 新增** |

### 1.3 子查询与CTE

| 功能 | 状态 | 备注 |
|------|------|------|
| EXISTS / NOT EXISTS | ✅ | |
| IN / NOT IN (subquery) | ✅ | |
| IN / NOT IN (value list) | ✅ | |
| ANY / ALL / SOME | ✅ | |
| **CTE / WITH** | ✅ | **v3.0.0 增强** |
| **递归 CTE** | ✅ | **v3.0.0 新增** |

### 1.4 表达式与函数

| 功能 | 状态 | 备注 |
|------|------|------|
| CASE WHEN | ✅ | |
| COALESCE | ✅ | |
| IFNULL | ✅ | |
| NULLIF | ✅ | |
| IF() | ✅ | |
| CAST / CONVERT | ✅ | |
| JSON_EXTRACT | ✅ | |
| 聚合函数 | ✅ | |

### 1.5 窗口函数

| 功能 | 状态 | 备注 |
|------|------|------|
| ROW_NUMBER | ✅ | |
| RANK | ✅ | |
| DENSE_RANK | ✅ | |
| NTILE | ✅ | **v3.0.0 新增** |
| LEAD | ✅ | **v3.0.0 新增** |
| LAG | ✅ | **v3.0.0 新增** |
| FIRST_VALUE | ✅ | **v3.0.0 新增** |
| LAST_VALUE | ✅ | **v3.0.0 新增** |
| NTH_VALUE | ✅ | **v3.0.0 新增** |
| PARTITION BY | ✅ | |
| ORDER BY within window | ✅ | |

## 2. 优化器

| 功能 | 状态 | 备注 |
|------|------|------|
| **PredicatePushdown** | ✅ | **v3.0.0 激活** |
| **ProjectionPruning** | ✅ | **v3.0.0 激活** |
| **ConstantFolding** | ✅ | **v3.0.0 激活** |
| **SimpleCostModel** | ⚠️ | 存在但未接入 planner |
| 索引选择 | ❌ | Beta 阶段 |
| Join 重排序 | ❌ | Beta 阶段 |

## 3. 存储引擎

| 功能 | 状态 | 备注 |
|------|------|------|
| Buffer Pool | ✅ | |
| FileStorage | ✅ | |
| MemoryStorage | ✅ | |
| ColumnarStorage | ✅ | |
| B+ Tree 索引 | ✅ | |
| MVCC | ✅ | |
| WAL | ✅ | |
| Group Commit | ✅ | **v3.0.0 新增** |

## 4. 事务隔离级别

| 隔离级别 | 状态 | 备注 |
|----------|------|------|
| Read Uncommitted | N/A | |
| Read Committed | ✅ | |
| Snapshot Isolation | ✅ | MVCC |
| Serializable (SSI) | ✅ | Proof-026 |

## 5. 性能组件

| 功能 | 状态 | 备注 |
|------|------|------|
| CBO 三规则 | ✅ | Alpha 激活 |
| **连接池** | ✅ | **v3.0.0 新增** |
| **查询缓存** | ✅ | **v3.0.0 新增** |
| **Group Commit** | ✅ | **v3.0.0 新增** |

## 6. API 与工具

| 功能 | 状态 | 备注 |
|------|------|------|
| TCP Server | ✅ | |
| MySQL 协议 | ✅ | |
| REPL | ✅ | |
| Prepared Statement | ✅ | |
| **EXPLAIN ANALYZE** | ✅ | **v3.0.0 新增** |
| **INFORMATION_SCHEMA** | ✅ | **v3.0.0 新增** |
| **SHOW VARIABLES** | ✅ | **v3.0.0 新增** |
| **慢查询日志** | ✅ | **v3.0.0 新增** |
| **SSL/TLS** | ✅ | **v3.0.0 新增** |

## 7. 安全

| 功能 | 状态 | 备注 |
|------|------|------|
| RBAC | ✅ | |
| GRANT/REVOKE | ✅ | |
| AES-256 加密 | ✅ | |
| **SSL/TLS** | ✅ | **v3.0.0 新增** |
| 安全审计 | ✅ | |

## 8. 分布式

| 功能 | 状态 | 备注 |
|------|------|------|
| Semi-sync 复制 | ✅ | |
| MTS 并行复制 | ✅ | |
| Multi-source | ✅ | |
| XA 事务 | ✅ | |

## 9. 测试覆盖率目标

| 模块 | Alpha 目标 | Beta 目标 | GA 目标 |
|------|------------|-----------|---------|
| executor | ≥45% | ≥60% | ≥70% |
| optimizer | ≥40% | ≥55% | ≥65% |
| parser | ≥50% | ≥60% | ≥70% |
| storage | ≥15% | ≥30% | ≥50% |
| catalog | ≥50% | ≥60% | ≥70% |
| **整体** | **≥50%** | **≥65%** | **≥85%** |