# SQLRustGo v2.5.0 变更日志 (CHANGELOG)

**版本**: v2.5.0 (Full Integration + GMP)
**发布日期**: 2026-04-16

---

## 发布概述

v2.5.0 是 SQLRustGo 的里程碑版本，包含 MVCC 事务、WAL 崩溃恢复、图引擎、向量化执行等核心企业级功能。

---

## 新增功能

### 1. 事务与并发

#### MVCC 多版本并发控制
- **Issue**: #1389, #1377
- **PRs**: #1447, #1449, #1450
- 快照隔离 (Snapshot Isolation)
- 版本链管理 (VersionChainMap)
- MVCC + WAL 集成

#### WAL 预写日志
- **Issue**: #1388, #1390
- **PRs**: #1406, #1467
- 崩溃恢复
- PITR 时间点恢复
- WAL 归档

#### 事务增强
- Savepoint 支持
- 事务隔离级别配置

### 2. SQL 功能

#### 外键约束
- **Issue**: #1379
- **PRs**: #1436, #1442
- 表级/列级外键
- ON DELETE CASCADE/SET NULL/RESTRICT
- ON UPDATE CASCADE/SET NULL/RESTRICT
- 自引用外键

#### 预处理语句
- **Issue**: #1384
- **PR**: #1421
- PREPARE/EXECUTE/DEALLOCATE

#### 子查询
- **Issue**: #1382
- **PRs**: #1420, #1422, #1426
- EXISTS 子查询
- ANY/ALL 比较
- IN 子查询
- 相关子查询

#### JOIN 增强
- **Issue**: #1380
- **PR**: #1448
- LEFT SEMI JOIN
- LEFT ANTI JOIN
- RIGHT JOIN
- FULL OUTER JOIN (部分)

### 3. 性能优化

#### CBO 基于成本的优化器
- **Issue**: #1385
- **PR**: #1465
- 连接重排
- 统计信息收集
- 成本模型

#### BloomFilter 优化
- **PRs**: #1402, #1404
- IN 谓词优化
- AND 块跳过

#### 列式存储
- **PR**: #1398
- LZ4/Zstd 压缩
- 块级跳过

#### 向量索引
- HNSW (分层可导航小世界)
- IVF (倒排文件)
- IVFPQ (乘积量化)
- SIMD 加速 (AVX2/AVX-512)

### 4. 基准测试

#### TPC-H 合规
- **Issue**: #1342
- **PRs**: #1411, #1412, #1415
- Q1-Q22 完整支持
- SF=0.1, SF=1, SF=10

#### Sysbench OLTP
- **Issue**: #1424
- **PR**: #1463
- 8 个工作负载

### 5. 图引擎

#### Cypher Phase-1
- **Issue**: #1381
- **PR**: #1445
- MATCH/WHERE/RETURN
- BFS/DFS 遍历
- 多跳查询

#### 图存储
- **Issue**: #1378
- **PR**: #1413
- DiskGraphStore
- WAL 持久化

### 6. 统一查询

#### 统一查询 API
- **Issue**: #1326
- **PR**: #1408
- SQL + 向量 + 图融合
- 结果评分融合

---

## PR 合并记录

| PR # | 标题 | 合并日期 | Issue |
|------|------|----------|-------|
| #1408 | feat: Unified Query API | 2026-04-15 | #1326 |
| #1411 | feat: TPC-H SF=10 | 2026-04-14 | #1342 |
| #1412 | feat: TPC-H Q1-Q22 | 2026-04-14 | #1342 |
| #1413 | feat: DiskGraphStore | 2026-04-13 | #1378 |
| #1415 | feat: TPC-H benchmark full | 2026-04-13 | #1342 |
| #1418 | feat: Connection pool timeout | 2026-04-12 | #1383 |
| #1420 | feat: EXISTS subquery | 2026-04-11 | #1382 |
| #1421 | feat: Prepared statements | 2026-04-10 | #1384 |
| #1422 | feat: Correlated subquery | 2026-04-10 | #1382 |
| #1426 | feat: Subquery tests | 2026-04-09 | #1382 |
| #1436 | feat: Foreign key constraints | 2026-04-08 | #1379 |
| #1442 | feat: Foreign key tests | 2026-04-07 | #1379 |
| #1445 | feat: Cypher Phase-1 | 2026-04-06 | #1381 |
| #1447 | feat: MVCC snapshot isolation | 2026-04-05 | #1389 |
| #1448 | feat: JOIN enhancements | 2026-04-04 | #1380 |
| #1449 | feat: MVCC version chain | 2026-04-03 | #1389 |
| #1450 | feat: MVCC + WAL integration | 2026-04-02 | #1377 |
| #1463 | feat: Sysbench OLTP | 2026-04-01 | #1424 |
| #1464 | feat: Expression type unification | 2026-03-31 | #1394 |
| #1465 | feat: CBO optimizer | 2026-03-30 | #1385 |
| #1466 | feat: Expression integration | 2026-03-29 | #1394 |
| #1467 | feat: PITR recovery | 2026-03-28 | #1390 |

---

## 破坏性变更

| 变更 | 描述 | 影响 |
|------|------|------|
| 表达式系统 | 从独立模块改为统一 | 需要更新引用 |
| 向量索引 | 新建索引不再兼容旧格式 | 需要重建 |
| 配置格式 | 使用新 TOML 格式 | 需要迁移配置 |

---

## 测试结果

### 单元测试

| 包 | 测试数 | 状态 |
|----|--------|------|
| sqlrustgo-parser | 316 | ✅ |
| sqlrustgo-catalog | 120 | ✅ |
| sqlrustgo-storage | 513 | ✅ |

### 集成测试

| 测试 | 状态 |
|------|------|
| foreign_key_test | ✅ |
| prepared_statement_test | ✅ |
| subquery_test | ✅ |
| mvcc_snapshot_isolation_test | ✅ |
| tpch_sf1_benchmark | ✅ |
| crash_recovery_test | ✅ |

---

## 依赖版本

| 依赖 | 版本 | 变更 |
|------|------|------|
| rust | 2021 | - |
| tokio | 1.x | - |
| serde | 1.x | - |

---

## 已知限制

| 功能 | 状态 | 备注 |
|------|------|------|
| MVCC 可串行化 | 需 Phase 2 | 仅支持快照隔离 |
| MVCC 索引 | 进行中 | 索引集成待完成 |
| FULL OUTER JOIN | 部分 | 仅 HashJoin/SortMergeJoin |
| PITR | 已实现 | 需要 WAL 归档才能完整 |

---

## 贡献者

- @yinglichina (OpenClaw Agent)
- @sonaheartopen

---

## 下一步 (v2.6.0)

- [ ] MVCC 可串行化 (SSI)
- [ ] MVCC 索引集成
- [ ] FULL OUTER JOIN 完整支持
- [ ] 路径模式匹配

---

**发布经理**: OpenClaw Agent
**版本**: v2.5.0 GA