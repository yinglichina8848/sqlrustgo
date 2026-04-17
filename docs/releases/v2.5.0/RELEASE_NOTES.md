# SQLRustGo v2.5.0 发布说明

**发布日期**: 2026-04-16
**代号**: Full Integration + GMP

---

## 概述

SQLRustGo v2.5.0 是一个重大版本，包含全面的SQL数据库功能，包括生产级别的特性：MVCC事务隔离、TPC-H基准测试、图引擎支持、以及统一的SQL+向量+图查询能力。

---

## 版本信息

| 组件 | 版本 |
|------|------|
| Workspace版本 | v1.8.0 |
| 发布版本 | v2.5.0 |
| Rust版本 | 2021 |
| Workspace crates | 36 |

---

## 已关闭的Issue (v2.5.0)

| Issue | 标题 | PR(s) | 状态 |
|-------|------|-------|------|
| #1424 | Sysbench OLTP工作负载 | #1463 | 已关闭 |
| #1385 | 基于成本的优化器 (CBO) | #1465 | 已关闭 |
| #1390 | PITR时间点恢复 | #1467 | 已关闭 |
| #1423 | MySQL生产差距 - 编译错误修复 | #1469 | 已关闭 |
| #1379 | 表级外键 | #1436, #1442 | 已关闭 |
| #1384 | 预处理语句 | #1421 | 已关闭 |
| #1382 | 子查询支持 (EXISTS/ANY/ALL/IN) | #1420, #1422, #1426 | 已关闭 |
| #1383 | 带超时控制的连接池 | #1418 | 已关闭 |
| #1378 | 带磁盘持久化的图引擎 | #1413 | 已关闭 |
| #1388 | WAL崩溃恢复 | #1406 | 已关闭 |
| #1342 | TPC-H基准测试 (SF=1, SF=10) | #1411, #1412 | 已关闭 |
| #1380 | JOIN增强 (SEMI/ANTI) | #1448 | 已关闭 |
| #1389 | MVCC快照隔离 | #1447, #1449, #1450 | 已关闭 |
| #1377 | MVCC + WAL集成 | #1450 | 已关闭 |
| #1381 | Cypher Phase-1 (图查询) | #1445 | 已关闭 |
| #1326 | 统一查询API (SQL+向量+图) | #1408 | 已关闭 |
| #1394 | 表达式类型统一 | #1464, #1466 | 已关闭 |

---

## 主要功能

### 1. 事务与并发

#### MVCC (多版本并发控制)
- **Issue**: #1389
- **PRs**: #1447, #1449, #1450
- **描述**: 实现快照隔离以支持并发事务处理
- **组件**:
  - `VersionChainMap`: 只追加版本的存储
  - `MVCCStorage`: 事务感知的存储引擎
  - `TransactionManager`: 事务协调
  - GC/Vacuum: 版本清理

#### WAL (预写日志)
- **Issues**: #1388, #1390
- **PRs**: #1406, #1467
- **描述**: 崩溃恢复和时间点恢复支持
- **组件**:
  - `WalManager`: WAL条目日志记录
  - `WalArchiveManager`: WAL归档
  - `recover_to_timestamp()`: 基于时间戳的恢复
  - `recover_table_to_timestamp()`: 表级PITR

#### 事务集成
- **Issue**: #1377
- **PR**: #1450
- **描述**: MVCC + WAL集成到SQL执行路径
- **组件**:
  - `TransactionalExecutor`: 事务感知执行器
  - `WalTransactionalExecutor`: WAL集成事务

### 2. SQL功能

#### 外键约束
- **Issue**: #1379
- **PRs**: #1436, #1442
- **支持的动作**:
  - ON DELETE: CASCADE, SET NULL, RESTRICT
  - ON UPDATE: CASCADE, SET NULL, RESTRICT
- **特性**:
  - 自引用外键
  - CONSTRAINT关键字解析

#### 预处理语句
- **Issue**: #1384
- **PR**: #1421
- **SQL语法**:
  ```sql
  PREPARE stmt AS SELECT * FROM t WHERE id = @param;
  EXECUTE stmt USING @param = 1;
  DEALLOCATE PREPARE stmt;
  ```

#### 子查询支持
- **Issue**: #1382
- **PRs**: #1420, #1422, #1426
- **支持类型**:
  - EXISTS子查询
  - ANY/ALL比较
  - IN子查询
  - 相关子查询

#### JOIN增强
- **Issue**: #1380
- **PR**: #1448
- **支持的JOIN**:
  - LEFT SEMI JOIN
  - LEFT ANTI JOIN
  - RIGHT JOIN
  - FULL OUTER JOIN (部分)

### 3. 性能优化

#### 基于成本的优化器 (CBO)
- **Issue**: #1385
- **PR**: #1465
- **特性**:
  - 基于成本的连接重排
  - 基于统计数据的计划选择

#### BloomFilter优化
- **PRs**: #1402, #1404
- **特性**:
  - IN谓词BloomFilter优化
  - AND谓词块跳过

#### 列式存储
- **PR**: #1398
- **特性**:
  - 块级跳过优化
  - 压缩支持 (LZ4, Zstd)

#### 向量索引
- **组件**:
  - HNSW (分层可导航小世界)
  - IVF (倒排文件索引)
  - IVFPQ (乘积量化)
  - SIMD加速 (AVX2, AVX-512)
  - 并行KNN搜索

### 4. 基准测试

#### TPC-H合规
- **Issue**: #1342
- **PRs**: #1411, #1412, #1415
- **覆盖**: Q1-Q22完整
- **规模因子**: SF=0.1, SF=1, SF=10

#### Sysbench OLTP工作负载
- **Issue**: #1424
- **PR**: #1463
- **工作负载**:
  - `oltp_index_scan`: 主键查找
  - `oltp_range_scan`: 范围查询
  - `oltp_insert`: 批量插入
  - `oltp_update_index`: 索引更新
  - `oltp_update_non_index`: 非索引更新
  - `oltp_delete`: 删除操作
  - `oltp_mixed`: 混合读写
  - `oltp_write_only`: 只写

### 5. 图引擎

#### Cypher Phase-1
- **Issue**: #1381
- **PR**: #1445
- **支持**:
  - `MATCH ... WHERE ... RETURN`
  - BFS/DFS遍历
  - 多跳查询
- **存储**: `DiskGraphStore` 带WAL持久化

### 6. 统一查询API

- **Issue**: #1326
- **PR**: #1408
- **描述**: 混合SQL+向量+图查询
- **特性**:
  - 跨引擎并行执行
  - 统一评分和结果融合

### 7. 表达式类型统一

- **Issue**: #1394
- **PRs**: #1464, #1466
- **描述**: `expression.rs` 现在从 `parser.rs` 导入Expression

---

## 性能目标

### OLTP工作负载

| 场景 | 并发 | 目标TPS | 目标P99延迟 |
|------|------|---------|-------------|
| 点查 | 32 | > 50,000 | < 5ms |
| 索引扫描 | 32 | > 10,000 | < 20ms |
| 插入 | 16 | > 20,000 | < 10ms |
| 更新 | 16 | > 15,000 | < 15ms |
| 混合OLTP | 32 | > 10,000 | < 30ms |

### TPC-H基准测试

| 场景 | 目标 | vs MySQL |
|------|------|----------|
| TPC-H Q1 (SF=1) | < 500ms | > 5x更快 |
| TPC-H All (SF=1) | < 10s | > 3x更快 |
| 向量搜索10K | < 100ms | N/A |

### 存储性能

| 操作 | 提升 |
|------|------|
| B+树点查询 | 6203x更快 |
| B+树索引 | 8000x更快 |
| 二进制导入 (SF=1) | 7700x比JSON快 |
| TPC-H数据导入 | 5.4x (Binary vs JSON) |

---

## Crate结构

### 核心数据库Crate

| Crate | 描述 | 关键模块 |
|-------|------|----------|
| `sqlrustgo-parser` | SQL解析 | 词法分析器、解析器、表达式 |
| `sqlrustgo-planner` | 查询规划 | 逻辑计划、物理计划 |
| `sqlrustgo-optimizer` | 查询优化 | CBO、规则、成本模型 |
| `sqlrustgo-executor` | 查询执行 | Volcano执行器、向量化 |
| `sqlrustgo-storage` | 数据存储 | BufferPool、B+树、WAL、FileStorage |
| `sqlrustgo-catalog` | 元数据管理 | Schema、表、认证 |
| `sqlrustgo-transaction` | 事务处理 | MVCC、锁管理器、恢复 |
| `sqlrustgo-types` | 数据类型 | Value、SqlError |
| `sqlrustgo-common` | 公共工具 | 配置、日志 |

### 扩展Crate

| Crate | 描述 |
|-------|------|
| `sqlrustgo-graph` | Cypher解析器、图引擎 |
| `sqlrustgo-vector` | HNSW、IVF、IVFPQ、SIMD |
| `sqlrustgo-unified-query` | SQL+向量+图融合 |
| `sqlrustgo-unified-storage` | 文档、GraphLink存储 |
| `sqlrustgo-gmp` | 文档管理、嵌入向量 |
| `sqlrustgo-rag` | 检索增强生成 |

### 服务器与工具Crate

| Crate | 描述 |
|-------|------|
| `sqlrustgo-server` | HTTP服务器、连接池 |
| `sqlrustgo-mysql-server` | MySQL协议服务器 |
| `sqlrustgo-agentsql` | Agent SQL接口 |
| `sqlrustgo-bench` | 基准测试框架 |
| `sqlrustgo-tools` | 备份、恢复、mysqldump |

---

## 测试覆盖

### 单元测试

| Crate | 测试数 | 状态 |
|-------|--------|------|
| `sqlrustgo-parser` | 316 | ✅ 通过 |
| `sqlrustgo-catalog` | 120 | ✅ 通过 |
| `sqlrustgo-storage` | 513 | ✅ 通过 |

### 集成测试

| 测试 | 目的 | 状态 |
|------|------|------|
| `foreign_key_test` | 外键约束 | ✅ |
| `prepared_statement_test` | 预处理语句 | ✅ |
| `subquery_test` | 子查询执行 | ✅ |
| `mvcc_snapshot_isolation_test` | MVCC隔离 | ✅ |
| `tpch_sf1_benchmark` | TPC-H SF=1 | ✅ |
| `crash_recovery_test` | WAL恢复 | ✅ |

### 回归测试框架

- **位置**: `tests/regression_test.rs`
- **覆盖**: 所有主要功能
- **CI集成**: ✅

---

## 已知限制

| 功能 | 状态 | 备注 |
|------|------|------|
| MVCC可串行化 | 需Phase 2/3 | 仅支持快照隔离 |
| MVCC索引 | 进行中 | 索引集成待完成 |
| FULL OUTER JOIN | 部分 | 仅HashJoin/SortMergeJoin |
| PITR | 已实现 | 需要WAL归档才能完整PITR |
| CBO | 基础版 | 成本模型持续优化中 |
