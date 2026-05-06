# SQLRustGo v3.0.0 开发计划审计与补充报告

> **审计日期**: 2026-05-05
> **审计范围**: 两份 v3.0.0 开发计划 + v2.9.0 源码实现 + v2.9.0 弱项分析
> **状态**: 正式审计报告

---

## 一、审计对象

| 文档 | 日期 | 定位 | 文件名 |
|------|------|------|--------|
| **计划 A** | 2026-03-28 | MySQL 5.6 功能补齐 | `docs/releases/v3.0.0/DEVELOPMENT_PLAN.md` |
| **计划 B** | 2026-05-05 | 生产级性能交付 | `docs/releases/v2.9.0/V3_0_0_DEVELOPMENT_PLAN.md` |
| **弱项分析** | 2026-05-05 | v2.9.0 代码级差距评估 | `docs/releases/v2.9.0/WEAKNESS_ANALYSIS.md` |

---

## 二、两份计划的核心矛盾

| 维度 | 计划 A (2026-03) | 计划 B (2026-05) | 矛盾 |
|------|-----------------|-----------------|------|
| 版本定位 | MySQL 5.6 兼容 | 生产级性能 | 功能优先 vs 性能优先 |
| QPS 目标 | 5,000 | 20,000 | 4x 差距 |
| CBO 优化器 | 未提及 | F-01 (P0, 4周) | 计划 A 完全忽略性能基础设施 |
| 设计债务 | 未提及 | 识别但未安排修复 | 两份计划均无 Debt Sprint |
| 前置版本 | v2.2 GA | v2.9.0 RC | 基于不同成熟度的代码基线 |
| 时间线 | 2个月 | 5个月 | 2.5x 差距 |

**结论**: 计划 A 基于过时的 v2.2 基线编写，忽略 v2.9.0 弱项分析的全部发现。计划 B 更务实，但存在 10 项结构性缺失。

---

## 三、计划 B 的 10 项结构性缺失

### 缺失 1：性能优先级的错位

当前 v2.9.0 Point Select ~2,000 QPS，与 MySQL 5.7 差距 **112 倍**。这是 v3.0.0 最致命问题。

计划 B 把 CBO（F-01）、查询缓存（F-02）、连接池（F-03）、Group Commit（F-04）分散在 15 个 F 任务中，没有建立"性能口袋"（Performance Pocket）——即一组互相依赖、必须同时交付才能生效的性能优化集合。

**证据**: 任何一项缺失都会使其他优化收益被抵消。例如：实现 CBO 但无连接池 → 连接 overhead 吃掉优化收益；实现连接池但无查询缓存 → 重复查询仍然慢。

**建议**: 合并 F-01~F-04 为 Performance Pocket v1，要求同一里程碑完成。

### 缺失 2：设计债务偿还计划为空

计划 B 识别了弱项中的设计债务但未安排修复任务：

| 设计债务 | 源码证据 | 影响 | 计划 B 状态 |
|----------|---------|------|------------|
| CBO 规则全空 | `crates/optimizer/src/rules.rs:67,95,122` — 3 个 `// TODO: Implement` stub | 所有查询全表扫描 O(n) | F-01 覆盖，但未提先修债务 |
| CTE Parser-Planner 断连 | Parser 有 `WithClause`，Planner `create_physical_plan_internal` 无 CTE 分支 | CTE 语法通过但无法执行 | F-07 覆盖，但未提前置修复 |
| 触发器"假实现" | `crates/storage/src/file_storage.rs:1483` — `"Triggers not supported in FileStorage"` | 68KB executor 代码，存储层返回不支持 | **完全未提及** |
| SSI 脆弱性 | `crates/transaction/src/ssi.rs:287` — TLA+ cycle 注释暗示复杂/脆弱 | 高并发下可能崩溃或假阳性 | F-09 覆盖，但未提修复现有 SSI |
| 无聚簇索引 | B+Tree 无 clustered index 实现 | 每次索引查询多一次随机 I/O | **完全未提及** |

**建议**: Development 阶段之前增加 2 周 **Debt Sprint**，专门按源码证据修复这些已知债务。

### 缺失 3：CI/CD 基础设施建设的遗漏

计划 B "新增 CI Gate"节提到了 4 个新 Gate，但遗漏：

| 遗漏项 | 当前状态 | 风险 |
|--------|---------|------|
| TPC-H/Sysbench 性能回归检测 | `check_perf_baseline.sh` 是 stub（仅检查文件存在，然后 SKIP） | 性能退化无感知 |
| 覆盖趋势存储与告警 | 每次 run 覆盖前次结果，无历史追踪 | 覆盖率下降无告警 |
| MySQL Wire Protocol 兼容性 | 仅测 embedded crate，从未对真实 MySQL 容器测试 | 协议兼容性无保证 |
| Chaos Engineering 迁移至 Gitea CI | 混沌工程仅在 GitHub Actions，Gitea CI 无 | 本地 CI 无韧性验证 |

### 缺失 4：功能依赖图缺失

计划 B 的 15 个 F 任务只有优先级（P0/P1/P2），无依赖标注：

```
F-01 CBO ────────→ F-02 查询缓存（缓存失效依赖 predicate 分析）
F-01 CBO ────────→ F-05 INSERT...SELECT（需 Predicate Pushdown）
F-01 CBO ────────→ F-11 EXPLAIN ANALYZE（需 CBO 输出代价估算）
F-03 连接池 ──────→ F-13 SSL/TLS（TLS 握手开销需要连接池抵消）
F-07 CTE 执行 ────→ 先修 Planner CTE 断连（依赖未列入计划）
```

**建议**: 关键路径 = CBO → Query Cache → Connection Pool → Group Commit。

### 缺失 5：时间线不可行

计划 B 工时总和（按自身估算）：

- F-01~F-15 功能：4+2+2+2+2+3+3+4+3+2+2+2+3+4+2 = **40 周**
- 性能优化任务：4+1+1+2+2+2+1+1+4+3 = **21 周**
- 合计：**61 周**

即使 4 个 Agent 满负荷并行（假设无阻塞依赖）：61/4 ≈ 15 周 ≈ 3.75 个月。

但 Development 阶段分配了 3.5 个月（05-15 ~ 08-31），**恰好等于理论最短时间，无任何 buffer**。未计算测试编写、CI 建设、文档编写时间。

**建议**: P2 功能延后至 v3.1.0，v3.0.0 聚焦核心性能 + P0 功能。

### 缺失 6：回退策略缺失

未定义性能目标未达成时的处理方式。CBO 实现后如果 QPS 只到 8,000 而非 20,000，是延期发布还是降级目标？

**建议**: 引入三级验收阈值：
- **及格线**（RC 准入）：Point Select ≥10,000 QPS
- **目标线**（GA 准入）：Point Select ≥20,000 QPS
- **卓越线**（后续版本）：Point Select ≥50,000 QPS

### 缺失 7：ABRG 规则与现实的矛盾

计划 B 规定 ABRG 阶段"不做新功能、不做性能优化"。但 CBO 这类核心基础设施，Alpha 测试必然发现需要调整策略——“调整策略”是否算"新性能优化"？规则需细化：

**建议**:
- Alpha: 允许 CBO 参数调优（不改算法，只调阈值）
- Beta: 允许修正性能回归（证明是 bug 而非设计变更）
- RC: 严格冻结，只允许 crash/data-loss 修复

### 缺失 8：升级迁移路径缺失

未提及 v2.9.0 → v3.0.0 的用户升级细节：
- 配置文件兼容性
- SQL 行为变更
- 存储格式兼容
- 破坏性变更清单

### 缺失 9：教学定位的延续性

VERSION_ROADMAP.md 战略定位是"教学数据库产品（Teaching DBMS）"，计划 B 完全围绕生产性能展开，未提及教学模式。

**建议**: 新增 F-16: 教学模式增强 —— 确保 CBO、窗口函数、CTE 在教学模式下可被禁用/可见化。

### 缺失 10：与 ARCHITECTURE_EVOLUTION.md 的衔接断裂

ARCHITECTURE_EVOLUTION.md 明确 3.0 核心目标是：
- 完整生命周期管理
- 稳定 API
- 版本控制体系
- 基本可扩展能力
- **删除 20% 不必要接口**（第 8 节）

计划 B 未体现以上任何架构目标。

**建议**: 增加 Phase 4（Architecture Hardening）——模块边界审计、API 版本化、升级兼容性验证。

---

## 四、综合补充方案：五阶段结构

```
Phase 0: Debt Sprint（2周，偿还设计债务）
  D-01: CBO 规则实现（Predicate Pushdown, Projection Pruning, Constant Folding）
  D-02: Planner CTE 断连修复（WithClause → LogicalPlan::With）
  D-03: 触发器 storage 层实现（替换 "not supported"）
  D-04: SSI 脆弱性加固（TLA+ cycle 检测优化 + 并发压力测试）
  D-05: 聚簇索引 ADR（先出架构决策，延迟实现）

Phase 1: Performance Pocket v1（4周，性能核心）
  PP-01: CBO 完善（Index Selection + Join Reordering）
  PP-02: 连接池（Connection Pool, max_connections 可配置）
  PP-03: 查询缓存 DML 自动失效
  PP-04: Group Commit（WAL 批量 fsync, 可配置阈值）
  PP-05: 批量 Insert 优化

Phase 2: SQL Completeness（3周，SQL 补齐）
  F-01: INSERT...SELECT
  F-02: 窗口函数补全（NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE）
  F-03: CTE 执行（WITH + WITH RECURSIVE）
  F-04: SERIALIZABLE 隔离级别 + Gap Locking

Phase 3: Infrastructure（2周，基础设施）
  I-01: INFORMATION_SCHEMA（TABLES/COLUMNS/STATISTICS）
  I-02: EXPLAIN ANALYZE（完整执行计划输出）
  I-03: SSL/TLS 支持
  I-04: 慢查询日志
  I-05: CI Gate 完善（TPC-H/Sysbench/Coverage Trend/MySQL Protocol 兼容测试）

Phase 4: Architecture Hardening（1周，架构加固）
  A-01: 模块边界审计（画依赖图，删除不必要公开接口 ≥10%）
  A-02: API 版本化（v3 API 标记，deprecation 流程）
  A-03: 升级兼容性验证（v2.9.0 → v3.0.0 迁移路径）
  A-04: 教学模式保持（SQLRUSTGO_TEACHING_MODE=1 增强）
```

---

## 五、建议的 GA 验收标准修订

在计划 B 基础上增加：

### 架构验收
- 画出 v3.0.0 模块依赖图，确认无环依赖
- 删除 ≥10% 的不必要公开接口（目标：≥30 个 pub 项转为 pub(crate)）
- API 版本化标记完成（所有对外 API 标注 `#[deprecated]` 或 `#[since = "3.0.0"]`）

### 设计债务验收
- CBO 3 个 TODO stub 清零（PredicatePushdown, ProjectionPruning, ConstantFolding 完成实现）
- Planner 支持 `LogicalPlan::With`（CTE 断连修复）
- FileStorage 对核心触发场景不再返回 "not supported"
- SSI 并发压力测试通过（100 并发, 10 万次操作, 无 cycle 检测假阳性）

### 性能分级验收
| 级别 | Point Select QPS | TPC-H SF=1 | 阶段 |
|------|-----------------|-----------|------|
| 及格线 | ≥10,000 | 22/22 可运行 | RC 准入 |
| 目标线 | ≥20,000 | 22/22 p99<2s | GA 准入 |
| 卓越线 | ≥50,000 | - | v3.1.0 |

### CI/CD 验收
- TPC-H CI Gate 实际运行（非 stub, 回归检测有效）
- Sysbench CI Gate 实际运行（QPS 对比 baseline，回归告警）
- Coverage 趋势存储 ≥30 天历史（可回溯任意 commit 覆盖率）
- MySQL 容器兼容性测试实际运行（mysql:5.7 镜像握手测试通过）

---

## 六、风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| CBO 实现后 QPS 未达 10K 及格线 | 中 | 高 | 预留 2 周调优 buffer；分层验收（RC 仅需 10K） |
| SSI 修复引入新 bug | 中 | 高 | 先写并发压力测试（TDD），后修代码 |
| 4 Agent 并行导致合并冲突 | 高 | 中 | Phase 内按模块分配 Agent；Phase 间串行（Phase 0→1→2→3→4） |
| TPC-H SF=1 OOM 持续 | 中 | 中 | Phase 1 完成后先做内存 profiling 定位泄漏源 |
| 时间不足（3.5月 vs 预估 4-5月） | 高 | 高 | P2 功能延后至 v3.1.0；严格按依赖图排序并行 |
| 教学模式功能退化 | 低 | 中 | Phase 4 A-04 专项检查；每个 F 任务需验证教学模式行为 |

---

## 七、与 ARCHITECTURE_EVOLUTION.md 的衔接

```
ARCHITECTURE_EVOLUTION.md 对 3.0 的核心判断:

✅ 模块依赖图可以画出来 → Phase 4 A-01
✅ 无环依赖 → Phase 4 A-01 审计
✅ 新 feature 不会大面积改动 → Phase 0 Debt Sprint 确保边界稳定

3.0 最大坑：
⚠️ 模块边界假稳定 → Phase 4 审计 + 删除不必要公开接口
⚠️ API 过早冻结 → Phase 4 A-02 仅标记版本，不冻结
⚠️ 扩展点设计错误 → Phase 4 A-02 明确扩展点位置

3.0 成功判断标准:
✅ 是否可以替换 storage 而不改 executor？→ Phase 4 A-01 验证
✅ 是否可以替换 planner 而不改 API？→ Phase 4 A-02 验证
✅ AI 是否可拔掉而系统仍可运行？→ Phase 0 D-04 + Phase 4 A-01 验证
```

---

## 八、建议的 Issue 拆分

v3.0.0 应拆分为两个版本：

| 版本 | 定位 | 核心内容 | 预计 |
|------|------|---------|------|
| **v3.0.0** | 性能核心 + 基础设施 | Phase 0-4（不含 P2） | 2026-Q3 |
| **v3.1.0** | 高级功能补齐 | 触发器、分区表、全文索引、Auto Tuning、GTID | 2026-Q4 |

原计划 A 的"MySQL 5.6 兼容"目标分散到 v3.0.0（基础设施）和 v3.1.0（高级功能）中。

---

*审计完成日期: 2026-05-05*
*审计人: SQLRustGo Agent*
*基于: v2.9.0 源码分析 + 弱项分析 + 两份开发计划对比*
