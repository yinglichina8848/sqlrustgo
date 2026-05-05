# SQLRustGo v2.9.0 弱项全面分析报告

> **版本**: v2.9.0  
> **基准**: develop/v2.9.0 (da80c704)  
> **日期**: 2026-05-05  
> **状态**: 内部分析文档 — 非发布物

---

## 摘要

| 维度 | v2.9.0 评分 | MySQL 5.7 | 差距 |
|------|------------|-----------|------|
| SQL 语言覆盖 | 72/100 | 100 | -28 |
| 存储引擎 | 65/100 | 100 | -35 |
| 事务 ACID | 70/100 | 100 | -30 |
| 性能 (QPS) | 45/100 | 100 | -55 |
| 安全 | 62/100 | 100 | -38 |
| 运维生态 | 35/100 | 100 | -65 |
| 测试体系 | 70/100 | 100 | -30 |
| **综合** | **56.7/100** | **100** | **-43.3** |

---

## 一、功能弱项 (Functionality)

### 1.1 SQL 语言 — 关键缺口

#### P0 (严重影响兼容性的缺失)

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| **事件调度器 (Event Scheduler)** | CREATE EVENT / DROP EVENT | ❌ 完全缺失 | `grep -r "EVENT" crates/` 无相关实现 |
| **全文索引 (Full-Text Index)** | MATCH...AGAINST + FULLTEXT | ❌ 完全缺失 | storage engine 无 FULLTEXT 实现 |
| **空间数据类型 (GIS)** | GEOMETRY/POINT/LINESTRING + ST_* 函数 | ❌ 完全缺失 | parser 有 `ST_` 前缀检测，但无 spatial index |
| **INSERT...SELECT** | 完整支持 | ❌ 尚未实现 | 2.9.0 README P1 延期项 |

#### P1 (影响主流 SQL 兼容性的不完整实现)

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| **窗口函数** | 完整 (ROW_NUMBER/RANK/DENSE_RANK/NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE) | ⚠️ 仅 ROW_NUMBER/RANK/DENSE_RANK | docs/releases/v2.9.0/README.md:430 |
| **存储过程游标/异常** | 完整 (DECLARE CURSOR + HANDLER) | ⚠️ 仅 IF/WHILE/LOOP，无游标 | `crates/executor/src/stored_proc.rs` 存在但 planner 不规划 |
| **触发器链** | BEFORE/AFTER/INSTEAD OF + 触发器级联 | ⚠️ 基础存在，storage 返回 "not supported" | `file_storage.rs:1432`: "Triggers not supported in FileStorage" |
| **CTE 执行** | WITH 递归 CTE | ⚠️ Parser 有 `WithClause`，但 planner 无执行 | `crates/planner/src/planner.rs` 无 CTE logical plan 处理 |

#### P2 (影响特定场景的缺失)

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| JSON 函数完整套件 | JSON_EXTRACT/JSON_VALUE/JSON_ARRAY/JSON_OBJECT/JSON_MERGE 等 | ⚠️ 仅 JSON_EXTRACT/JSON_OBJECT 基础 | backup module 有 `json_value` 但无完整函数集 |
| 列级权限 DML | GRANT/REVOKE col_priv | ⚠️ 部分支持 | docs/releases/v2.9.0/README.md:388 |
| 行级安全 (RLS) | 完整 | ❌ 完全缺失 | security crate 无 RLS 实现 |
| CREATE SEQUENCE | 完整 | ❌ 完全缺失 | catalog 无 sequence 支持 |

### 1.2 存储引擎 — 关键缺口

| 功能 | MySQL (InnoDB) | v2.9.0 | 差距 |
|------|----------------|--------|------|
| **聚簇索引 (Clustered Index)** | 主键索引即数据 | ❌ B+Tree 无聚簇 | storage crate 无聚簇实现 |
| **自适应哈希索引 (AHI)** | 热数据自动哈希加速 | ❌ 完全缺失 | 无 adaptive hash |
| **Change Buffer** | 辅助索引变更缓存合并 | ⚠️ 仅简单 insert_buffer | `file_storage.rs:26` 有但非 InnoDB 风格 |
| **双写缓冲 (Doublewrite)** | 防止部分写入 | ❌ 完全缺失 | WAL 无 doublewrite 机制 |
| **Redo Log (InnoDB 风格)** | 页级 redo，crash safe | ⚠️ WAL 存在但非 page-level redo | `crates/storage/src/wal.rs` 是日志级，非页级 |
| **Undo Log (MVCC Undo)** | InnoDB undo slots | ⚠️ 有 UndoRecord 但非 slots | `transaction/src/savepoint.rs:4` |
| **表压缩** | PAGE_COMPRESSED | ❌ 完全缺失 | 无压缩实现 |
| **行格式 Compressed** | DYNAMIC/COMPRESSED | ❌ 仅 REDUNDANT/COMPACT | 无动态行格式 |

### 1.3 事务与并发 — 关键缺口

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| **SERIALIZABLE 隔离级别** | 完整支持 | ❌ 仅 RC/SI 两种 | docs/releases/v2.9.0/README.md:460 |
| **Gap Locking** | Next-Key Locking (防止幻读) | ❌ 无 gap lock | `lock_manager.rs:175` 无 gap locking 实现 |
| **XA 两阶段提交** | XA START/END/PREPARE/COMMIT | ⚠️ 存在但未验证 | 无 `xa_start` 等协议处理 |
| **SERIALIZABLE SSI 性能** | 优化过的 SSI | ⚠️ SSI 存在但可能性能差 | `transaction/src/ssi.rs:287` TLA+ cycle 注释暗示复杂/脆弱 |

### 1.4 网络协议 — 关键缺口

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| **SSL/TLS 加密连接** | SSL_REQUIRE + MYSQL41 + SHA256 | ❌ 无 TLS | docs/releases/v2.9.0/README.md:487 |
| **Prepared Statement 二进制协议** | COM_STMT_PREPARE/COM_STMT_EXECUTE | ⚠️ 有基础但可能不完整 | `mysql-server/src/lib.rs:1199` 有 `PreparedStatementManager` |
| **多语句执行** | COM_MULTI | 可能缺失 | 无明确测试 |

### 1.5 运维生态 — 关键缺口

| 功能 | MySQL | v2.9.0 | 证据 |
|------|-------|--------|------|
| **INFORMATION_SCHEMA** | 完整 (FILES/TABLES/COLUMNS/STATISTICS) | ❌ 完全缺失 | docs/releases/v2.9.0/README.md:419 |
| **performance_schema** | 完整 (instrumentation) | ❌ 完全缺失 | 同上 |
| **慢查询日志 (Slow Query Log)** | 完整配置 | ⚠️ 存在 `/metrics` 但非 SQL 级别慢查询 | `crates/server/src/` 无 slow_query_log 实现 |
| **EXPLAIN 输出** | 完整执行计划 | ⚠️ 存在但不完整 | executor 无完整 CBO 输出 |
| **系统变量体系** | 完整 `SHOW VARIABLES` | ⚠️ 部分实现 | sysbench 需要 `SHOW VARIABLES` |

---

## 二、性能弱项 (Performance)

### 2.1 基准数据

| 基准 | MySQL 8.0 | PostgreSQL 16 | SQLite 3.45 | v2.9.0 |
|------|-----------|---------------|-------------|---------|
| Point Select TPS | 224,931 | 285,128 | 13,617 | **~2,000** |
| P99 Latency | 0.16ms | 0.13ms | 2.51ms | **未知** |
| TPC-H SF=1 | 22/22 | 22/22 | 22/22 | **22/22 (但 Q17/Q18 慢)** |
| TPC-H SF=0.3 | — | — | 基线 | **30x 慢于 SQLite** |
| TPC-H SF=1 | — | — | 基线 | **OOM** |
| JOIN QPS | — | — | — | **12,617 (已优化)** |

**结论**: QPS 为 MySQL 的 **0.9%**，差距 112 倍。

### 2.2 性能瓶颈根因

#### P0 瓶颈 (导致 QPS ~2,000)

**1. 无查询计划器优化 (No CBO)**
- 证据: `crates/optimizer/src/rules.rs` — PredicatePushdown/ProjectionPruning/ConstantFolding 全部为 TODO stub
- 影响: 所有查询全表扫描，O(n) 而非 O(log n)

**2. 无查询缓存 (Query Cache 基础存在但不完整)**
- 证据: `crates/executor/src/query_cache.rs` 有 LRU 缓存，但无 DML 自动失效机制
- 影响: 写入后缓存不失效，数据陈旧

**3. 无连接池 (Connection Pool)**
- 证据: `crates/mysql-server/src/lib.rs` 无连接池，每个查询重新打开文件
- 影响: 连接 overhead 大，高并发下性能衰减

**4. 无并行查询 (Partition-based Parallel Query)**
- 证据: `crates/executor/src/parallel_executor.rs` 存在但仅适用于简单并行
- 影响: 多核不能充分利用

**5. 无批量提交优化 (Group Commit)**
- 证据: WAL 逐条 commit，无 group commit 优化
- 影响: fsync 次数过多

#### P1 瓶颈 (影响复杂查询)

| 瓶颈 | 证据 | 影响 |
|------|------|------|
| **无向量化和 SIMD** | executor 无 Volcano-Columnar hybrid | 聚合/排序慢 10-100x |
| **无索引合并 (Index Merge)** | optimizer 无多索引合并逻辑 | 多条件查询需全表扫描 |
| **无分区裁剪 (Partition Pruning)** | `PartitionInfo` 存在但无裁剪逻辑 | 分区表全扫描 |
| **TPC-H Q17/Q18 相关子查询** | correlated subquery 解析但执行不优化 | 指数级复杂度 |

### 2.3 UPDATE/DELETE 性能

| 操作 | MySQL 8.0 | v2.9.0 | 差距 |
|------|-----------|--------|------|
| UPDATE QPS | 10,000+ | ~950 | **10x+** |
| DELETE QPS | 10,000+ | ~206 | **50x+** |
| Batch INSERT | 50,000+ | 有但未优化 | 待测 |

---

## 三、设计弱项 (Design)

### 3.1 架构层面的设计债务

#### A. 优化器 — 核心基础设施为空

所有 CBO 规则都是 TODO stub：

```
crates/optimizer/src/rules.rs:
  Line 67:  PredicatePushdown::apply  →  // TODO: Implement predicate pushdown logic
  Line 95:  ProjectionPruning::apply   →  // TODO: Implement projection pruning logic
  Line 122: ConstantFolding::apply     →  // TODO: Implement constant folding logic
```

这意味着 **查询计划完全依赖规则而非代价**，无法生成高效执行计划。

#### B. 存储引擎抽象 — 无聚簇索引导致性能天花板

MySQL InnoDB 的核心性能优势是主键即数据（聚簇索引），所有 secondary index 指向主键。SQLRustGo 的 B+Tree 无论 primary 还是 secondary 都需额外一次随机读。

#### C. Planner 与 Executor 的 CTE 断连

Parser 有 `WithClause`，但 `planner.rs` 的 `create_physical_plan_internal` 无 CTE 处理分支。CTE 在语法层通过但执行层绕过。

#### D. Trigger 在 storage 层的"假实现"

`crates/executor/src/trigger.rs` 存在 68KB 代码，但 `file_storage.rs` 和 `columnar/storage.rs` 均返回 "Triggers not supported"。这意味着触发器是假实现。

#### E. MVCC SSI 的脆弱性

`transaction/src/ssi.rs` 包含 TLA+ 注释提及 "Classic 3-cycle"，暗示 SSI 实现依赖复杂的序列化图检测，可能在高并发下有性能问题或正确性问题。

### 3.2 API 和扩展点设计问题

| 问题 | 描述 |
|------|------|
| **Extension Point 不清晰** | ARCHITECTURE_EVOLUTION.md 指出 3.0→4.0 的核心危险是 "Module boundaries not truly stable, APIs frozen too early, wrong extension points" |
| **Trait Boundaries 模糊** | executor crate 的 VolcanoExecutor trait 和 StorageEngine trait 边界不清晰，影响并行执行 |
| **AI 耦合** | ARCHITECTURE_EVOLUTION.md 指出 "2.0 → 3.0 transition risks: AI deeply coupled to kernel" |

---

## 四、测试弱项 (Testing)

### 4.1 测试覆盖缺口

| 模块 | 当前覆盖率 | 目标 | 缺口 |
|------|-----------|------|------|
| optimizer | ~55% | 75% | **-20%** (最差) |
| planner | ~65% | 80% | **-15%** |
| network | ~60% | 75% | **-15%** |
| executor | ~72% | 80% | **-8%** |
| storage | ~78% | 85% | **-7%** |

### 4.2 测试场景缺口 (Critical Gaps)

#### 崩溃恢复测试 — 仅 in-memory

| 测试项 | 状态 | 问题 |
|--------|------|------|
| `crash_recovery_test.rs` | ⚠️ 仅 in-memory engine | 无真实进程 kill -9 测试 |
| WAL replay after hard shutdown | ❌ 未测试 | 无磁盘 persistence 验证 |
| PITR (Point-In-Time Recovery) | ❌ 未测试 | 无时间点恢复测试 |

#### 混沌工程测试 — 仅在 GitHub Actions

| 场景 | Gitea CI | GitHub Actions |
|------|----------|---------------|
| Deadlock injection | ❌ | ✅ |
| CPU 80% stress | ❌ | ✅ |
| Network 30% packet loss | ❌ | ✅ |
| Memory fault injection | ❌ | ❌ |
| Disk I/O delay simulation | ❌ | ❌ |
| Process kill -9 mid-transaction | ❌ | ❌ |

#### MySQL 兼容性测试 — 无真实 MySQL

| 测试项 | 状态 |
|--------|------|
| 真实 MySQL wire protocol 兼容性 | ❌ 仅测 embedded crate |
| `mysql_server_tests.rs` | 仅测试嵌入式，非真实 MySQL |
| SQLancer (differential fuzz) | ✅ 存在 |

#### 性能回归测试 — 存在但无效

| 脚本 | 问题 |
|------|------|
| `scripts/gate/check_perf_baseline.sh` | 仅检查 baseline 文件存在，然后 SKIP — 无实际比较 |
| `scripts/benchmark/check_regression.sh` | 存在但未集成到 CI gate |
| TPC-H in CI | ❌ 不在 CI gate 中 |
| Sysbench OLTP in CI | ❌ 不在 CI gate 中 |

### 4.3 覆盖率趋势跟踪 — 完全缺失

- Coverage pipeline 仅检查当前 % vs 75% 阈值
- 无历史趋势存储（每次 run 覆盖覆盖，不记录历史）
- 无 coverage 下降自动告警
- 无 per-module coverage SLO 追踪

---

## 五、运维与监控弱项 (Operations & Monitoring)

### 5.1 监控端点现状

| 端点 | 状态 | 说明 |
|------|------|------|
| `/health/live` | ✅ 已实现 | Liveness probe |
| `/health/ready` | ✅ 已实现 | Readiness probe |
| `/metrics` (Prometheus) | ✅ 已实现 | M-004 完成 |
| `/health` (综合) | ✅ 已实现 | 聚合健康检查 |
| `INFORMATION_SCHEMA` | ❌ 完全缺失 | MySQL 兼容性缺失 |
| `performance_schema` | ❌ 完全缺失 | instrumentation 缺失 |
| 慢查询日志 | ❌ 缺失 | 仅 /metrics 无 SQL 级别慢查询 |

### 5.2 运维工具缺口

| 工具 | MySQL | v2.9.0 |
|------|-------|--------|
| `mysqldump` | ✅ | ⚠️ `crates/tools/src/mysqldump.rs` 存在但功能不明 |
| `mysqladmin` | ✅ | ❌ 无等效工具 |
| `mysqlbinlog` | ✅ | ❌ WAL 无 binlog 格式 |
| 在线 DDL | ✅ | ❌ ALTER TABLE 阻塞 |
| 在线添加索引 | ✅ | ❌ 需重建表 |

### 5.3 高可用性缺口

| 功能 | MySQL | v2.9.0 |
|------|-------|--------|
| 自动故障转移 | ✅ | ❌ |
| 组复制 (Group Replication) | ✅ | ❌ |
| 半同步复制 + GTID | ✅ | ✅ 存在 |
| 多源复制 | ✅ | ✅ 存在 |

---

## 六、文档弱项 (Documentation)

### 6.1 缺失的文档

| 文档 | 状态 | 说明 |
|------|------|------|
| **弱项分析** (本文档) | ⚠️ 已有但分散 | 需整合到单一文档 |
| 运维手册 (Operations Manual) | ❌ 缺失 | 无慢查询分析、备份恢复、监控配置 |
| 性能调优指南 | ❌ 缺失 | 无 B+Tree/buffer pool/WAL 调优建议 |
| 升级兼容性指南 | ⚠️ 有 MIGRATION_GUIDE 但基础 | 无 breaking change 清单 |
| 架构决策记录 (ADR) | ⚠️ 分散在 ARCHITECTURE_DECISIONS.md | 无正式 ADR 格式 |
| 开发者贡献指南 | ⚠️ 有 AI_COLLABORATION_GUIDE | 无正式 CODEOWNERS |

### 6.2 现有文档的问题

| 问题 | 描述 |
|------|------|
| **文档与实现脱节** | README 声明的功能可能实际未实现 (trigger 在 storage 层) |
| **Benchmark 结果未整合** | TPC-H/Sysbench 报告分散在多处，无统一基准页面 |
| **版本间弱项对比缺失** | 无 v2.8.0 vs v2.9.0 弱项变化追踪 |

---

## 七、安全弱项 (Security)

| 功能 | MySQL 5.7 | v2.9.0 | 优先级 |
|------|-----------|--------|--------|
| TLS 加密连接 | ✅ | ❌ | **P0** |
| AES-256 存储加密 | ✅ | ❌ | **P1** |
| 行级安全 (RLS) | ✅ | ❌ | **P1** |
| 列级权限 DML | ✅ | ⚠️ 部分 | P2 |
| 密码轮转 | ✅ | ❌ | P2 |
| 审计日志 | ✅ | ✅ | — |

---

## 八、CI/CD 弱项

| 缺口 | 影响 |
|------|------|
| Coverage 趋势未存储/可视化 | 无法发现 coverage 退化 |
| Benchmark 回归检测未实现 | `check_perf_baseline.sh` 是 stub |
| TPC-H 未进入 CI gate | 性能指标不受 CI 保护 |
| Sysbench OLTP 未进入 CI gate | QPS 指标不受 CI 保护 |
| MySQL wire protocol 兼容性未测 | 协议兼容性无保证 |
| Chaos 测试仅在 GitHub Actions | Gitea CI 无混沌工程 |
| 形式化验证未集成到 CI | S-01~S-05 Phase B 成果未自动化 |

---

## 九、总结 — 关键弱项优先级矩阵

```
严重度 →
优先级 ↓  功能 (F)   性能 (P)   设计 (D)   测试 (T)   运维 (O)
─────────────────────────────────────────────────────────────
P0       事件调度器  QPS<10K   优化器空   TPC-H/sysbench 监控端点
P0       全文索引    UPDATE慢   (CBO=0)   崩溃恢复真  INFORMATION
P0       空间数据              SSI脆弱性             _SCHEMA
P1       窗口函数    TPC-H慢   CTE断连   混沌工程    慢查询日志
P1       INSERT...S  (Q17/Q18) Trigger假  覆盖趋势    mysqldump
P1       SERIALIZABLE Gap锁    实现      MySQL兼容   在线DDL
P2       游标/异常    XA未验证   AI耦合    性能回归    密码轮转
P2       JSON完整    组提交     Extension  测试        TLS
                     缺失       Point模糊
```

---

## 十、v2.9.0 vs MySQL 功能矩阵摘要

| 类别 | MySQL 5.7 | v2.9.0 | 完整度 |
|------|-----------|--------|--------|
| DDL (CREATE/ALTER/DROP) | 完整 | 基础 | 85% |
| DML (INSERT/UPDATE/DELETE) | 完整 | 基础，UPDATE/DELETE 已修复 | 90% |
| SELECT + JOIN | 完整 | 完整 (JOIN 修复完成) | 95% |
| 子查询 + CTE | 完整 | CTE parser 有，planner 无 | 75% |
| 窗口函数 | 完整 | 部分 | 50% |
| 存储过程 | 完整 | 基础 | 60% |
| 触发器 | 完整 | 假实现 | 30% |
| 事件调度器 | 完整 | 无 | 0% |
| 全文索引 | 完整 | 无 | 0% |
| 空间数据 | 完整 | 无 | 0% |
| MVCC + WAL | 完整 | 基础 | 80% |
| 隔离级别 (4种) | 完整 | 2种 (RC/SI) | 50% |
| XA 事务 | 完整 | 有但不完整 | 70% |
| 备份恢复 | 完整 | 基础 | 60% |
| 复制 | 完整 | 半同步+GTID+MTS | 80% |
| 用户+RBAC | 完整 | 基础 | 85% |
| SSL/TLS | 完整 | 无 | 0% |
| INFORMATION_SCHEMA | 完整 | 无 | 0% |
| performance_schema | 完整 | 无 | 0% |
| 慢查询日志 | 完整 | 无 | 0% |
| EXPLAIN | 完整 | 基础 | 50% |

---

*本文档为内部分析材料，汇总了代码分析、benchmark 结果、CI/CD 现状和文档审查的发现。*
