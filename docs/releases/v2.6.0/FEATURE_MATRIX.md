# SQLRustGo v2.6.0 功能矩阵

**最后更新**: 2026-04-18

---

## 概述

本文档提供SQLRustGo v2.6.0所有功能的完整矩阵，包括实现状态、测试覆盖和性能指标。

---

## 1. SQL 语法支持

### 1.1 SQL-92 核心语法

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| SELECT | `parser/src/parser.rs` | `sql-corpus` | 59 | ✅ | 完整支持 |
| INSERT | `parser/src/parser.rs` | `sql-corpus` | - | ✅ | - |
| UPDATE | `parser/src/parser.rs` | `sql-corpus` | - | ✅ | - |
| DELETE | `parser/src/parser.rs` | #1557 | - | ✅ | 新增 |
| CREATE TABLE | `parser/src/parser.rs` | `sql-corpus` | - | ✅ | - |
| DROP TABLE | `parser/src/parser.rs` | `sql-corpus` | - | ✅ | - |
| CREATE INDEX | `parser/src/parser.rs` | - | - | ✅ | - |
| ALTER TABLE | `parser/src/parser.rs` | - | - | ✅ | - |

### 1.2 聚合函数

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| COUNT | `executor/src/aggregate.rs` | `sql-corpus` | 10+ | ✅ | #1545 |
| SUM | `executor/src/aggregate.rs` | `sql-corpus` | - | ✅ | - |
| AVG | `executor/src/aggregate.rs` | `sql-corpus` | - | ✅ | - |
| MIN | `executor/src/aggregate.rs` | `sql-corpus` | - | ✅ | - |
| MAX | `executor/src/aggregate.rs` | `sql-corpus` | - | ✅ | - |

### 1.3 JOIN 语法

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| INNER JOIN | `executor/src/join.rs` | `sql-corpus` | 8+ | ✅ | #1545 |
| LEFT JOIN | `executor/src/join.rs` | `sql-corpus` | - | ✅ | - |
| RIGHT JOIN | `executor/src/join.rs` | - | - | ✅ | - |
| CROSS JOIN | `executor/src/join.rs` | - | - | ✅ | - |
| FULL OUTER JOIN | - | - | - | ⏳ | 待开发 |

### 1.4 分组查询

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| GROUP BY | `planner/src/group_by.rs` | `sql-corpus` | - | ✅ | #1545 |
| HAVING | `planner/src/having.rs` | `sql-corpus` | - | ✅ | #1567 |
| GROUP BY + 聚合 | - | `sql-corpus` | - | ✅ | - |

### 1.5 子查询

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| EXISTS | `executor/src/subquery.rs` | - | - | ✅ | - |
| IN | `executor/src/subquery.rs` | - | - | ✅ | - |
| ANY/ALL | `executor/src/subquery.rs` | - | - | ✅ | - |
| 标量子查询 | `executor/src/subquery.rs` | - | - | ✅ | - |

---

## 2. 约束与完整性

### 2.1 外键约束

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| 表级外键 | `parser/src/table_constraint.rs` | - | - | ✅ | #1436 |
| 列级外键 | `parser/src/parser.rs` | - | - | ✅ | - |
| ON DELETE CASCADE | `storage/src/fk_validation.rs` | - | - | ✅ | #1436 |
| ON DELETE SET NULL | `storage/src/fk_validation.rs` | - | - | ✅ | - |
| ON DELETE RESTRICT | `storage/src/fk_validation.rs` | - | - | ✅ | - |
| ON UPDATE CASCADE | `storage/src/fk_validation.rs` | - | - | ✅ | - |
| 自引用外键 | `storage/src/fk_validation.rs` | - | - | ✅ | #1567 |

### 2.2 主键与唯一约束

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| PRIMARY KEY | `storage/src/engine.rs` | - | - | ✅ | - |
| UNIQUE | `storage/src/engine.rs` | - | - | ✅ | - |
| NOT NULL | `storage/src/engine.rs` | - | - | ✅ | - |

---

## 3. 存储引擎

### 3.1 核心存储

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| BufferPool | `storage/src/buffer_pool.rs` | - | - | ✅ | - |
| FileStorage | `storage/src/file_storage.rs` | - | - | ✅ | - |
| MemoryStorage | `storage/src/memory_storage.rs` | - | - | ✅ | - |
| ColumnarStorage | `storage/src/columnar.rs` | - | - | ✅ | - |

### 3.2 索引

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| B+Tree | `storage/src/btree.rs` | - | - | ✅ | - |
| Hash Index | `storage/src/hash_index.rs` | - | - | ✅ | - |
| Vector Index | `storage/src/vector_index.rs` | - | - | ✅ | - |

### 3.3 事务存储

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| MVCC | `transaction/src/mvcc.rs` | - | - | ✅ | SI |
| WAL | `storage/src/wal.rs` | - | - | ✅ | - |
| SSI | - | - | - | ⏳ | 待开发 |

---

## 4. 执行引擎

### 4.1 执行算子

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| SeqScan | `executor/src/seq_scan.rs` | - | - | ✅ | - |
| IndexScan | `executor/src/index_scan.rs` | - | - | ✅ | #1505 |
| Insert | `executor/src/insert.rs` | - | - | ✅ | - |
| Update | `executor/src/update.rs` | - | - | ✅ | - |
| Delete | `executor/src/delete.rs` | - | - | ✅ | #1557 |
| Sort | `executor/src/sort.rs` | - | - | ✅ | - |
| Limit | `executor/src/limit.rs` | - | - | ✅ | - |

### 4.2 高级功能

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| Prepared Statement | `executor/src/prepared.rs` | - | - | ✅ | - |
| 存储过程 | `executor/src/stored_proc.rs` | - | - | ⚠️ | 部分支持 |
| 触发器 | `executor/src/trigger.rs` | - | - | ⚠️ | 部分支持 |

---

## 5. API 接口

### 5.1 核心 API

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| ExecutionEngine | `src/execution_engine.rs` | - | - | ✅ | #1566 |
| Storage Engine | `storage/src/engine.rs` | - | - | ✅ | - |
| Transaction Manager | `transaction/src/transaction.rs` | - | - | ✅ | - |

### 5.2 网络协议

| 功能 | 实现位置 | 测试文件 | 测试数 | 状态 | 备注 |
|------|----------|----------|--------|------|------|
| MySQL Protocol | `mysql-server/src/lib.rs` | - | - | ✅ | - |
| REPL | `src/repl.rs` | - | - | ✅ | - |

---

## 6. 测试覆盖

### 6.1 测试套件

| 测试类型 | 测试数 | 状态 | 备注 |
|----------|--------|------|------|
| SQL Corpus | 59/59 | ✅ | 100% 通过 |
| 单元测试 | - | ⏳ | 覆盖率待测 |
| 集成测试 | - | ✅ | #1561 修复 |

### 6.2 代码质量

| 检查项 | 状态 | 备注 |
|--------|------|------|
| Clippy | ✅ | -D warnings 通过 |
| 格式化 | ✅ | cargo fmt 通过 |
| 文档 | ⏳ | 待测 |

---

## 7. 性能指标

### 7.1 OLTP 性能

| 场景 | 目标 | 当前 | 状态 |
|------|------|------|------|
| 点查 (32并发) | 75,000 TPS | - | ⏳ |
| 索引扫描 (32并发) | 15,000 TPS | - | ⏳ |
| 插入 (16并发) | 30,000 TPS | - | ⏳ |

### 7.2 TPC-H 性能

| 场景 | 目标 | 当前 | 状态 |
|------|------|------|------|
| Q1 (SF=1) | < 200ms | - | ⏳ |
| All Q (SF=1) | < 5s | - | ⏳ |

---

## 8. 功能状态汇总

| 类别 | 总数 | 完成 | 进行中 | 待开发 | 完成率 |
|------|------|------|--------|--------|--------|
| SQL 语法 | 15 | 14 | 0 | 1 | 93% |
| 约束完整性 | 8 | 8 | 0 | 0 | 100% |
| 存储引擎 | 8 | 7 | 0 | 1 | 88% |
| 执行引擎 | 12 | 11 | 1 | 0 | 92% |
| API 接口 | 6 | 6 | 0 | 0 | 100% |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
