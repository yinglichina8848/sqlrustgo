# v2.9.0 功能矩阵

> **版本**: v2.9.0
> **阶段**: Alpha

## 1. SQL 功能

### 1.1 DDL 命令

| 功能 | 状态 | 备注 |
|------|------|------|
| CREATE TABLE | ✅ | |
| DROP TABLE | ✅ | |
| CREATE TABLE IF NOT EXISTS | ✅ | 新增 |
| DROP TABLE IF EXISTS | ✅ | 新增 |
| ALTER TABLE ADD COLUMN | ✅ | |
| ALTER TABLE DROP COLUMN | ✅ | 新增 |
| ALTER TABLE MODIFY COLUMN | ✅ | 新增 |
| ALTER TABLE RENAME TO | ✅ | |
| CREATE VIEW | ✅ | 新增 |
| DROP VIEW | ✅ | 新增 |
| DROP VIEW IF EXISTS | ✅ | 新增 |
| CREATE INDEX | ✅ | |
| CREATE UNIQUE INDEX | ✅ | 新增 |
| DROP INDEX | ✅ | 新增 |
| DROP INDEX IF EXISTS | ✅ | 新增 |
| TRUNCATE TABLE | ✅ | |

### 1.2 DML 命令

| 功能 | 状态 | 备注 |
|------|------|------|
| SELECT | ✅ | |
| INSERT | ✅ | |
| INSERT ON DUPLICATE KEY UPDATE | ✅ | 新增 |
| REPLACE INTO | ✅ | |
| UPDATE | ✅ | |
| DELETE | ✅ | |
| INSERT INTO ... SELECT | ❌ | 待实现 |

### 1.3 子查询与CTE

| 功能 | 状态 | 备注 |
|------|------|------|
| EXISTS / NOT EXISTS | ✅ | |
| IN / NOT IN (subquery) | ✅ | |
| ANY / ALL / SOME | ✅ | |
| CTE / WITH | ✅ | 新增 |
| 递归 CTE | ✅ | 新增 |

### 1.4 表达式与函数

| 功能 | 状态 | 备注 |
|------|------|------|
| CASE WHEN | ✅ | 新增 |
| COALESCE | ✅ | |
| IFNULL | ✅ | |
| NULLIF | ✅ | |
| IF() | ✅ | |
| CAST / CONVERT | ✅ | |
| JSON_EXTRACT | ✅ | 新增 |
| 聚合函数 | ✅ | |

### 1.5 窗口函数

| 功能 | 状态 | 备注 |
|------|------|------|
| ROW_NUMBER | ✅ | 新增 |
| RANK | ✅ | 新增 |
| DENSE_RANK | ✅ | 新增 |
| PARTITION BY | ✅ | 新增 |

## 2. 存储引擎

| 功能 | 状态 | 备注 |
|------|------|------|
| Buffer Pool | ✅ | |
| FileStorage | ✅ | |
| MemoryStorage | ✅ | |
| ColumnarStorage | ✅ | |
| B+ Tree 索引 | ✅ | |
| MVCC | ✅ | |
| WAL | ✅ | |

## 3. 分布式

| 功能 | 状态 | 备注 |
|------|------|------|
| Semi-sync 复制 | ✅ | 新增 |
| MTS 并行复制 | ✅ | 新增 |
| Multi-source | ✅ | 新增 |
| XA 事务 | ✅ | 新增 |

## 4. 安全

| 功能 | 状态 | 备注 |
|------|------|------|
| RBAC | ✅ | |
| GRANT/REVOKE | ✅ | |
| AES-256 加密 | ✅ | |
| 安全审计 | ✅ | |

## 5. API 接口

| 功能 | 状态 | 备注 |
|------|------|------|
| TCP Server | ✅ | |
| MySQL 协议 | ✅ | |
| REPL | ✅ | |
| Prepared Statement | ✅ | |