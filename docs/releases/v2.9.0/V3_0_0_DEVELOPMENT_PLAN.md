# SQLRustGo v3.0.0 开发与测试计划

> **版本**: v3.0.0 (Draft)  
> **基于**: v2.9.0 弱项分析  
> **日期**: 2026-05-05  
> **状态**: 规划中 — 非正式发布文档

---

## 一、v3.0.0 愿景与目标

### 1.1 核心愿景

**"Production-Grade Performance — MySQL-Compatible Core"**

v3.0.0 是从 "能用" 到 "好用" 的分水岭版本。目标是：
- QPS 达到 MySQL 的 **20%+** (目标 ≥20,000 QPS point select)
- SQL 兼容性达到 MySQL 5.7 的 **85%+**
- 填补所有 P0/P1 功能缺口
- 完善 CI/CD 质量门禁
- 补充运维和监控基础设施

### 1.2 量化目标

| 指标 | v2.9.0 | v3.0.0 目标 | 差距 |
|------|---------|-------------|------|
| Point Select QPS | ~2,000 | ≥20,000 | **+900%** |
| UPDATE QPS | ~950 | ≥10,000 | **+953%** |
| DELETE QPS | ~206 | ≥5,000 | **+2427%** |
| TPC-H SF=1 可用查询 | 22/22 (慢) | 22/22 (P99<500ms) | 质量提升 |
| SQL Corpus 通过率 | 92.6% | ≥98% | +5.4% |
| MySQL 5.7 兼容性评分 | 56.7/100 | ≥75/100 | +18.3 |
| SERIALIZABLE 隔离级别 | ❌ | ✅ | 新增 |

---

## 二、阶段管理原则

### 2.1 版本阶段定义

| 阶段 | 性质 | 内容 |
|------|------|------|
| **Development** | **开发** | 所有功能开发、性能优化、重构、测试编写 |
| **Alpha** | 测试 | 功能验证测试、Beta 前问题修复 |
| **Beta** | 测试 | 候选版本测试、回归测试、问题修复 |
| **RC (Release Candidate)** | 测试 | 最终测试、门禁验证、文档完善 |
| **GA** | 发布 | 生产就绪发布 |

### 2.2 核心规则

> **所有功能和性能提升任务必须在 Development 阶段完成。ABRG 阶段只做测试和错误修复，不做新功能开发。**

```
Rule: No new features in ABRG
IF phase IN {Alpha, Beta, RC, GA}
THEN allowed = {bug fix, test fix, documentation, CI/CD improvement}
NOT allowed = {new feature, new performance optimization, architectural change}
```

---

## 三、功能路线图 (Feature Roadmap)

> **所有功能开发在 Development 阶段完成，ABRG 阶段不引入新功能。**

### Development 阶段任务

#### F-01: 完整 CBO 实现

```
优先级: P0
依赖: N/A
工时: 4 周
```

**实现内容**:

1. **Predicate Pushdown** (`crates/optimizer/src/rules.rs`)
   - 将 WHERE 条件下推到存储层
   - 减少数据传输量

2. **Projection Pruning**
   - 只读取需要的列
   - 减少 I/O

3. **Constant Folding**
   - 编译期计算常量表达式
   - 减少运行时计算

4. **Index Selection** (新增)
   - 基于代价选择最优索引
   - 多索引时合并结果

#### F-02: 查询缓存完善

```
优先级: P0
依赖: F-01
工时: 2 周
```

- DML 后自动失效相关缓存条目
- LRU 缓存容量可配置
- 缓存统计指标

#### F-03: 连接池

```
优先级: P0
依赖: N/A
工时: 2 周
```

- 实现连接复用，避免每次查询重新打开文件
- 配置项: `max_connections`, `connection_pool_size`

#### F-04: 批量提交优化 (Group Commit)

```
优先级: P1
依赖: F-01
工时: 2 周
```

- WAL 批量 fsync，减少 sync 次数
- 配置项: `group_commit_batch_size`, `group_commit_timeout_ms`

#### F-05: INSERT...SELECT

```
优先级: P1
依赖: F-01
工时: 2 周
```

- Parser 支持 `INSERT INTO ... SELECT ...`
- Executor 实现结果流式写入目标表

#### F-06: 窗口函数完善

```
优先级: P1
依赖: N/A
工时: 3 周
```

- 补全 NTILE(), LEAD(), LAG(), FIRST_VALUE(), LAST_VALUE(), NTH_VALUE()
- 支持 RANGE BETWEEN 和 ROWS BETWEEN 语法

#### F-07: CTE 执行

```
优先级: P1
依赖: Planner CTE support
工时: 3 周
```

- Planner 增加 `LogicalPlan::With` 处理分支
- 实现递归 CTE 执行
- 覆盖 WITH 和 WITH RECURSIVE

#### F-08: 存储过程游标 + 异常处理

```
优先级: P2
依赖: stored_proc.rs
工时: 4 周
```

- DECLARE CURSOR + OPEN/FETCH/CLOSE
- DECLARE HANDLER + GET DIAGNOSTICS
- 异常传播机制

#### F-09: SERIALIZABLE 隔离级别 + Gap Locking

```
优先级: P1
依赖: SSI 优化
工时: 3 周
```

- 完善 SSI 实现，优化 TLA+ cycle 检测性能
- Gap Locking (Next-Key Locking) 实现
- 支持 SERIALIZABLE 语义

#### F-10: INFORMATION_SCHEMA

```
优先级: P1
依赖: catalog 完善
工时: 2 周
```

- 实现 TABLES / COLUMNS / STATISTICS / FILES
- 兼容 MySQL 5.7 INFORMATION_SCHEMA 格式
- 提供 `SHOW TABLES`, `SHOW COLUMNS`, `SHOW INDEX` 等效功能

#### F-11: EXPLAIN 完善

```
优先级: P1
依赖: CBO
工时: 2 周
```

- 输出完整执行计划 (operator, rows estimate, cost)
- 支持 `EXPLAIN ANALYZE`
- 输出格式: traditional, JSON, tree

#### F-12: 慢查询日志

```
优先级: P2
依赖: query execution hook
工时: 2 周
```

- `long_query_time` 阈值配置
- 输出到文件或 syslog
- 格式兼容 MySQL slow query log

#### F-13: SSL/TLS 支持

```
优先级: P0 (安全)
依赖: network crate
工时: 3 周
```

#### F-14: 在线 DDL

```
优先级: P2
依赖: storage
工时: 4 周
```

#### F-15: XA 事务验证与优化

```
优先级: P2
依赖: transaction crate
工时: 2 周
```

---

## 四、性能优化路线图

> **所有性能优化在 Development 阶段完成。**

### 4.1 性能目标分解

```
Point SELECT:     2,000 QPS  →  20,000 QPS  (+900%)
UPDATE:              950 QPS  →  10,000 QPS  (+953%)
DELETE:              206 QPS  →   5,000 QPS  (+2,427%)
TPC-H SF=0.3:    6,400 ms  →     100 ms    (-98.4%)
TPC-H SF=1:         OOM     →   2,000 ms   (可运行)
```

### 4.2 开发阶段优化任务

| 优化项 | 预期收益 | 工时 | 阶段 |
|--------|---------|------|------|
| CBO Predicate Pushdown | 复杂查询 10-50x | 4 周 | Dev |
| Projection Pruning | I/O 减少 30-70% | 1 周 | Dev |
| Constant Folding | 编译期计算 5-10% | 1 周 | Dev |
| Index Selection | 多条件查询 5-20x | 2 周 | Dev |
| 连接池 | 高并发 3-5x | 2 周 | Dev |
| Group Commit | 写入吞吐 2-3x | 2 周 | Dev |
| 批量 Insert 优化 | Bulk insert 2-5x | 1 周 | Dev |
| Prepared Statement 缓存 | 解析开销减少 20-30% | 1 周 | Dev |
| 向量化执行 (基础) | 聚合 3-5x | 4 周 | Dev |
| SIMD 加速 (可选) | 特定操作 2-4x | 3 周 | Dev |

---

## 五、测试计划

### 5.1 测试分层模型

```
┌──────────────────────────────────────────────────────────────┐
│                    CI/CD Gate (Fast)                         │
│  Unit Tests (3000+) + Clippy + Fmt + Coverage ≥80%         │
├──────────────────────────────────────────────────────────────┤
│                    Integration Tests                         │
│  TPC-H (22 queries) + Sysbench OLTP + SQLite Fuzz          │
├──────────────────────────────────────────────────────────────┤
│                    System Tests                             │
│  Chaos Engineering + Crash Recovery + Performance Regression  │
├──────────────────────────────────────────────────────────────┤
│                    Benchmark Tracking                        │
│  Historical trending + SLO alerting                          │
└──────────────────────────────────────────────────────────────┘
```

### 5.2 Development 阶段 — 测试准备

在开发同时编写测试，确保开发完成时测试同步完成。

| 任务 | 说明 | 阶段 |
|------|------|------|
| T-01: TPC-H 测试用例完善 | Q17/Q18 相关子查询测试覆盖 | Dev |
| T-02: Sysbench OLTP 适配 | 确保 SQL 支持 sysbench 全场景 | Dev |
| T-03: 单元测试补全 | optimizer/planner 覆盖率提升至目标 | Dev |
| T-04: SQLancer Fuzz 测试 | differential fuzz 覆盖新功能 | Dev |

### 5.3 Alpha 阶段 — 测试验证

**目标**: 发现功能问题，验证核心功能正确性

| 测试 | 说明 | 通过标准 |
|------|------|---------|
| 功能回归测试 | 全量 SQL Corpus 通过 | ≥98% |
| TPC-H SF=0.1 | 22/22 查询通过 | p99 < 50ms |
| UPDATE/DELETE 性能 | 验证优化效果 | QPS ≥ 目标 50% |
| INSERT...SELECT | 功能正确性 | 基础场景通过 |
| CTE 执行 | WITH + 递归 CTE | 基础场景通过 |

### 5.4 Beta 阶段 — 全面测试

**目标**: 完整测试套件，修复所有发现的问题

| 测试 | 说明 | 通过标准 |
|------|------|---------|
| TPC-H SF=1 | 22/22 查询可运行 | 无 OOM，p99 < 2s |
| Sysbench OLTP | Point/UPDATE/INSERT 各场景 | QPS ≥ 目标 80% |
| SERIALIZABLE + Gap Lock | 隔离级别正确性 | 幻读测试通过 |
| INFORMATION_SCHEMA | 兼容性 | 基础查询通过 |
| 窗口函数 | NTILE/LEAD/LAG/NTH_VALUE | 基础场景通过 |
| EXPLAIN ANALYZE | 输出正确性 | 计划与实际执行匹配 |

### 5.5 RC 阶段 — 门禁验证

**目标**: 通过所有门禁，准备发布

| 门禁 | 标准 |
|------|------|
| Coverage | 全局 ≥85%, optimizer ≥70%, planner ≥80%, storage ≥85% |
| TPC-H CI Gate | SF=0.1 全部 + SF=1 全部可运行无 OOM |
| Sysbench CI Gate | Point ≥16,000 QPS, UPDATE ≥8,000 QPS, INSERT ≥8,000 QPS |
| Chaos Engineering | 5 场景全部通过，成功率 ≥99% |
| 崩溃恢复 | kill -9 后数据完整性验证通过 |
| 性能回归检测 | 无 regression (>baseline 90%) |
| MySQL Wire Protocol | 真实 MySQL 容器兼容性测试通过 |

### 5.6 GA 阶段 — 生产验证

| 验证项 | 说明 |
|--------|------|
| 生产环境部署验证 | 72h 稳定运行 |
| 备份恢复演练 | PITR 恢复验证 |
| 监控告警验证 | Prometheus + Grafana 正确采集 |
| 文档完整性 | 所有用户文档齐全 |

---

## 六、新增 CI Gate

### 6.1 Development 阶段 — 新增 CI 任务

在开发阶段建立以下 CI 能力：

```
[NEW] tpch-gate       — TPC-H SF=0.1 + SF=1, p99 阈值
[NEW] sysbench-gate   — QPS 阈值
[NEW] benchmark-trend  — 当前 vs baseline, 回归检测
[NEW] mysql-protocol-test — 真实 MySQL 容器兼容性
```

### 6.2 Coverage SLO

| 模块 | 开发阶段目标 | Alpha 阶段 SLO |
|------|-------------|----------------|
| optimizer | ≥70% | <65% 告警 |
| planner | ≥80% | <75% 告警 |
| executor | ≥80% | <75% 告警 |
| storage | ≥85% | <80% 告警 |
| network | ≥75% | <70% 告警 |

---

## 七、文档计划

### 7.1 Development 阶段 — 文档编写

| 文档 | 说明 |
|------|------|
| `docs/OPERATIONS_MANUAL.md` | 运维手册：备份恢复、监控配置、慢查询分析 |
| `docs/PERFORMANCE_TUNING.md` | 性能调优指南：B+Tree、buffer pool、WAL 调优 |
| `docs/ARCHITECTURE_ADR.md` | 架构决策记录 (正式 ADR 格式) |

### 7.2 RC 阶段 — 文档完善

| 文档 | 说明 |
|------|------|
| `docs/v3.0.0/ROADMAP.md` | v3.0.0 开发总结 |
| `docs/v3.0.0/BENCHMARK_REPORT.md` | v3.0.0 基准测试报告 |
| `docs/v3.0.0/RELEASE_NOTES.md` | 发布说明 |

---

## 八、版本阶段时间线

```
Development (预计 2026-05-15 ~ 2026-08-31)
  ├─ F-01~F-04 (CBO + 缓存 + 连接池 + Group Commit)
  ├─ F-05~F-08 (INSERT...SELECT + 窗口函数 + CTE + 存储过程)
  ├─ F-09~F-12 (SERIALIZABLE + INFO_SCHEMA + EXPLAIN + 慢查询)
  ├─ F-13~F-15 (SSL/TLS + 在线 DDL + XA)
  ├─ 所有性能优化任务
  ├─ T-01~T-04 (测试编写)
  └─ CI 基础设施完善

Alpha (预计 2026-09-01 ~ 2026-09-15)
  ├─ 功能验证测试
  ├─ 问题修复 (仅 bug fix)
  └─ TPC-H SF=0.1 验证

Beta (预计 2026-09-15 ~ 2026-09-30)
  ├─ 全量测试
  ├─ 问题修复 (仅 bug fix)
  ├─ TPC-H SF=1 验证
  └─ Sysbench OLTP 验证

RC (预计 2026-10-01 ~ 2026-10-15)
  ├─ 所有 CI Gate 验证
  ├─ 问题修复 (仅 bug fix)
  └─ 文档完善

GA (2026-10-15)
  └─ 生产发布
```

---

## 九、GA 量化验收标准

> **Development 阶段完成所有功能开发。ABRG 阶段只做测试验证和错误修复。**

### 功能验收 (Development 阶段完成)

```
[ ] INSERT...SELECT 实现
[ ] 窗口函数 (NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE)
[ ] CTE 执行 (WITH + WITH RECURSIVE)
[ ] SERIALIZABLE 隔离级别 + Gap Locking
[ ] INFORMATION_SCHEMA (TABLES/COLUMNS/STATISTICS)
[ ] SSL/TLS 支持
[ ] EXPLAIN ANALYZE
```

### 性能验收 (Development 阶段完成)

```
[ ] Point SELECT ≥20,000 QPS
[ ] UPDATE ≥10,000 QPS
[ ] DELETE ≥5,000 QPS
[ ] TPC-H SF=1 p99 < 2s (所有 22 查询)
[ ] TPC-H SF=1 无 OOM
```

### 测试验收 (ABRG 阶段验证)

```
[ ] 覆盖率 ≥85% (全局)
[ ] optimizer ≥70%, planner ≥80%, storage ≥85%
[ ] TPC-H CI Gate 通过 (SF=0.1 + SF=1)
[ ] Sysbench CI Gate 通过 (所有指标达标)
[ ] 真实 kill -9 崩溃恢复测试通过
[ ] Chaos Engineering Gate 通过 (5 场景)
[ ] Coverage 趋势系统运行
[ ] Benchmark 回归检测运行
[ ] MySQL wire protocol 兼容性测试通过
```

### 文档验收 (RC 阶段完成)

```
[ ] OPERATIONS_MANUAL.md 完成
[ ] PERFORMANCE_TUNING.md 完成
[ ] ADR 记录 ≥5 条
[ ] v3.0.0 BENCHMARK_REPORT 完成
[ ] v3.0.0 RELEASE_NOTES 完成
```

---

## 十、ABRG 阶段约束规则

```
Rule AB-1: No New Features
IF current_phase IN {Alpha, Beta, RC}
THEN reject ANY pull request that introduces:
  - new SQL feature
  - new optimizer rule
  - new storage engine capability
  - new protocol feature
EXCEPT: bug fixes directly related to shipped features

Rule AB-2: Performance Optimization Only in Development
Performance improvements belong in Development phase.
Alpha/Beta/RC may only: verify performance, report regression.
ABRG stages do NOT implement new performance optimizations.

Rule AB-3: Breaking Changes Only in Development
API changes, schema changes, or protocol changes
must be completed before Alpha branch cut.

Rule AB-4: Test Coverage Gate in Development
All new code must have tests BEFORE merging to develop.
ABRG stages cannot be used to "add tests later".
```

---

*本文档为 v3.0.0 规划文档，将随开发进展更新。所有功能和性能工作在 Development 阶段完成，ABRG 阶段严格只做测试和修复。*
