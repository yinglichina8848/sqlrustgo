# v3.0.0 门禁生命周期追踪规范

> **版本**: 1.0
> **日期**: 2026-05-08
> **目的**: 建立门禁失败项的闭环追踪机制，确保所有差距有 Issue、有修复、有验证
> **适用范围**: Alpha / Beta / RC / GA 各阶段门禁检查

---

## 一、核心原则

### 1.1 门禁失败 = 必须有 Issue

任意门禁检查项 FAIL，必须同时满足以下条件之一：

| 条件 | 处理方式 |
|------|----------|
| 能立即修复 | 立即修复，验证后合并 |
| 不能立即修复 | **必须**创建 Issue 追踪，标记为当前版本的未决项 |
| 需要较长开发周期 | 将任务延续到 **下个版本**的 `DEVELOPMENT_PLAN.md`，并建立 Issue 映射 |

### 1.2 禁止的模式

```
❌ 门禁 FAIL → 跳过 → 合并代码 → Issue 未创建 → 问题丢失
❌ Issue 已创建 → 未关联到门禁失败项 → 后续无人追踪
❌ 下版本开发 → 不知道上版本有未完成任务 → 重复遗漏
```

### 1.3 正确的模式

```
✅ 门禁检查 FAIL → 创建 Issue + 标记 milestone → 修复 → 验证通过 → 关闭 Issue
✅ 门禁检查 FAIL → 当前版本无法修复 → 延续到 v3.1.0 → 在 v3.1.0 DEVELOMENT_PLAN.md 中引用原 Issue
```

---

## 二、门禁检查执行流程

### 2.1 本地预检（推荐）

```bash
# 在 PR 分支上运行
bash scripts/gate/check_beta_v300.sh 2>&1 | tee /tmp/gate-output.txt
```

### 2.2 CI 自动检查

Gitea Actions 在 `develop/v3.0.0` 分支的每次 push 触发对应门禁脚本。

---

## 三、门禁失败处理流程

### 3.1 失败项提取

门禁脚本失败后，从输出中提取：

```
Gate 检查失败报告
====================
[beta-v3.0.0] B2: FAIL (87% = 131/150 tests passed)
[beta-v3.0.0] B-S1: FAIL (concurrency_stress_test: 2/41 failed)
[beta-v3.0.0] B9: FAIL (SQL Corpus 20.0% < 85%)

未通过项:
- B2: 测试通过率 87% < 90%
  → 影响: v3.0.0 无法进入 RC 阶段
- B-S1: concurrency_stress_test 死锁检测 2 个用例失败
  → 影响: 事务状态机可靠性存疑
- B9: SQL Corpus 20.0% < 85%
  → 影响: operations 测试类别语法不支持
  → 涉及用例: BACKUP, SAVEPOINT, SET TRANSACTION ISOLATION LEVEL, LIMIT/OFFSET, TRUNCATE, REPLACE, SHOW, EXPLAIN ANALYZE, ...
```

### 3.2 Issue 创建标准

| 门禁项 | Issue 标题模板 | 必须包含内容 |
|--------|---------------|-------------|
| B2 测试通过率 | `[B2] 全量测试通过率 {X%}，低于 {Y%} 要求` | 失败测试清单、最后通过率记录 |
| B5 覆盖率 | `[B5] {模块} 覆盖率 {X%}，低于 {Y%} 要求` | 差距分析、预期增加覆盖的模块 |
| B8/B9 TPC-H | `[B-SF] TPC-H SF={N} {N}/22 通过` | 失败查询号、超时/OOM/结果错误分类 |
| B-S1~B-S5 稳定性 | `[B-S{N}] {测试名} {M}/{K} 通过` | 失败用例名称、复现步骤 |
| SQL Corpus | `[B9] SQL Corpus {X%}，{Y} 个语法不支持` | 不支持语法分类清单 |
| R8/R11 性能 | `[R{N}] {指标} {X}，低于 {Y} 要求` | 性能数据、退化比例 |

### 3.3 Issue 与 Milestone 绑定

- Issue 的 `milestone` 必须设为**当前版本 milestone**（如 v3.0.0-beta）
- Issue 内容必须包含：`来源门禁脚本`、`检查命令`、`失败输出摘要`

### 3.4 延续到下版本的判定

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 说明 |
|------|------|
| 修复需要 3 人周以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖完成 | 如 CBO 需要先完成索引选择 |

**延续流程：**

```
当前版本 Issue #451 (SQL operations 20%)
         ↓
在 v3.1.0 DEVELOPMENT_PLAN.md 中建立映射
┌──────────────────────────────────────┐
│ v3.1.0 P1 任务（延续自 v3.0.0）      │
├──────────────────────────────────────┤
│ #451 → SQL Operations 语法支持        │
│ 目标: 通过率从 20% 提升至 80%          │
│ 涉及: BACKUP, SAVEPOINT, LIMIT, ...  │
│ 优先级: P1                           │
└──────────────────────────────────────┘
```

---

## 四、Issue 关闭验证（强制）

### 4.1 Issue 关闭前置条件

**禁止在没有 PR 证据的情况下关闭 Issue。**

Issue #451 的正确关闭流程：

```bash
# Step 1: 检查是否有 PR 关闭该 Issue
gh issue view 451 --json closedByPullRequestsReferences
# 结果非空 → 可以关闭
# 结果为空 → 禁止手动关闭

# Step 2: 验证 PR 已合并
gh pr view <pr_number> --json state,mergedAt
# state = MERGED 且 mergedAt 有值 → 继续
# 否则 → 禁止关闭

# Step 3: 验证测试通过
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_operations
# 必须 11+ 个用例通过（原 11 个基础上增加）

# Step 4: 确认覆盖率未下降
cargo llvm-cov --all-features | grep total
```

### 4.2 Gitea Issue 状态机

```
┌─────────┐         ┌─────────┐         ┌─────────┐
│  OPEN   │ ──────▶│ CLOSED  │ ──────▶│ REOPENED│
└─────────┘         └─────────┘         └─────────┘
    │                     ▲                    │
    │                     │                    │
    └─────────────────────┴────────────────────┘
              无 PR 证据禁止手动关闭
```

---

## 五、门禁脚本增强要求

### 5.1 必需的错误分类

每个门禁脚本（Beta/RC/GA）必须输出以下格式：

```bash
# 对于失败的检查项，必须输出：
echo "[gate-v3.0.0] {ID}: FAIL"
echo "  → 失败原因: {具体原因}"
echo "  → 建议操作: {修复建议或 Issue 创建指引}"
echo "  → 影响范围: {对其他门禁项的连带影响}"
```

### 5.2 输出格式规范

```bash
# check_beta_v300.sh 输出规范
[beta-v3.0.0] B1: PASS
[beta-v3.0.0] B2: FAIL (87% < 90%, 19 tests failed in 3 suites)
  → 失败 suites: sqlrustgo-executor, sqlrustgo-optimizer, sqlrustgo-storage
  → 建议: 创建 Issue 追踪，优先处理 executor 失败项
  → 影响: B-Gate 无法通过，需修复后重新检查
[beta-v3.0.0] B3: PASS
[beta-v3.0.0] B9: FAIL (20.0% < 85%, 44/55 operations tests failed)
  → 失败类别: BACKUP, SAVEPOINT, SET TRANSACTION ISOLATION LEVEL, LIMIT/OFFSET, TRUNCATE, REPLACE, SHOW, EXPLAIN ANALYZE, TEMPORARY TABLE, ALTER TABLE INPLACE, BATCH INSERT
  → 建议: 创建 Issue 追踪，开发语法支持
  → 影响: SQL 兼容性严重不足，无法进入 RC

=== Beta Gate Results: PASS=12 / 14, BLOCKERS=2 ===
=== 未通过项 ===
  - B2: 全量测试通过率 87% < 90%
  - B9: SQL Corpus 20.0% < 85%
=== 建议行动 ===
  1. 为每个 BLOCKER 创建 Gitea Issue（milestone: v3.0.0-beta）
  2. 修复后重新运行 check_beta_v300.sh
  3. 如当前版本无法修复，将任务延续到 v3.1.0 DEVELOPMENT_PLAN.md
```

---

## 六、版本间任务延续机制

### 6.1 延续清单

每个版本的 `DEVELOPMENT_PLAN.md` 必须包含：

```markdown
## 从上版本延续的任务

| 原 Issue | 任务描述 | 原版本状态 | v3.1.0 目标 | 验收条件 |
|----------|----------|------------|-------------|----------|
| #451 | SQL Operations 语法支持 | 20% (11/55) | ≥80% (44/55) | test_sql_corpus_operations 通过率 ≥80% |
| #379 | 事务状态机压力测试 | 未开始 | B-S2 PASS | crash_recovery_test 全部通过 |
```

### 6.2 版本里程碑映射

```
v3.0.0 milestone         v3.1.0 milestone
(v3.0.0-alpha/beta/rc)   (v3.1.0-alpha/beta/rc)
     │                         │
     │ #451 延续              │
     └─────────────────────────┼──▶ Issue #451 (open, milestone=v3.1.0-beta)
                              │
                              ▼
                         验证 test_sql_corpus_operations ≥ 80%
                              │
                              ▼
                         关闭 Issue #451
```

---

## 七、当前版本差距追踪

> 基于 check_beta_v300.sh 和 ALPHA_GAPS.md 分析

### 7.1 Beta Gate 失败项（v3.0.0）

| Issue | 门禁项 | 失败描述 | 状态 | 延续版本 |
|-------|--------|----------|------|----------|
| #451 | B9 (SQL Corpus) | test_sql_corpus_operations 20% (11/55)，44 个语法不支持 | OPEN | v3.1.0 |

### 7.2 Alpha 阶段未完成任务（ALPHA_GAPS.md）

| 任务 | 优先级 | 状态 | 延续版本 |
|------|--------|------|----------|
| CBO 代价模型集成 | P0 | 未完成 | v3.1.0 |
| TPC-H 内存治理防 OOM | P0 | 未完成 | v3.1.0 |
| 事务状态机压力测试 (#379) | P0 | 未完成 | v3.1.0 |
| Optimizer 测试扩展 (#380) | P1 | 未完成 | v3.1.0 |
| Planner 测试扩展 (#381) | P1 | 未完成 | v3.1.0 |
| TPC-H SF=1 CI Gate (#382) | P1 | 已完成（PR #391） | — |
| 连接池/缓存/Group Commit 正确性 | P1 | 部分完成 | v3.1.0 |

### 7.3 下版本必需完成项（v3.1.0）

```markdown
## v3.1.0 必须完成的延续任务

### P0（阻塞 Beta Gate）
1. SQL Operations 语法支持（#451 延续）：BACKUP, SAVEPOINT, SET TRANSACTION ISOLATION LEVEL, LIMIT/OFFSET, TRUNCATE, REPLACE, SHOW, EXPLAIN ANALYZE, TEMPORARY TABLE, ALTER TABLE INPLACE, BATCH INSERT
   - 目标: test_sql_corpus_operations ≥ 80%
   - 依赖: parser 扩展
2. CBO 代价模型集成（#392）：SimpleCostModel + 索引选择
   - 目标: EXPLAIN 选择索引扫描而非全表
3. TPC-H SF=1 无 OOM（#382 延续）
   - 目标: 22/22 通过，p99 < 5s
4. 事务状态机压力测试（#379）
   - 目标: crash_recovery_test 全部通过

### P1（阻塞 RC Gate）
5. Optimizer 测试扩展（#380）：覆盖率 ≥ 75%
6. Planner 测试扩展（#381）：覆盖率 ≥ 80%
7. 连接池/缓存/Group Commit 正确性验证
```

---

## 八、检查清单

### 8.1 门禁检查前

```
[ ] 确认在正确分支运行（develop/v3.0.0）
[ ] 确认 milestone 已创建（v3.0.0-beta）
[ ] 确认有写入权限创建 Issue
```

### 8.2 门禁检查后（FAIL 时）

```
[ ] 提取所有 FAIL 项的 ID、命令、输出
[ ] 为每个 FAIL 项创建 Issue（milestone 绑定到当前版本）
[ ] Issue 包含：来源门禁、检查命令、失败输出、验收条件
[ ] 将无法在当前版本修复的任务添加到下版本 DEVELOPMENT_PLAN.md
[ ] 通知负责人
```

### 8.3 Issue 关闭前

```
[ ] 验证有 PR 关闭该 Issue（closedByPullRequestsReferences 非空）
[ ] 验证 PR 已合并（state=MERGED）
[ ] 验证相关测试在本地或 CI 通过
[ ] 验证门禁重新检查 PASS
[ ] 更新 DEVELOPMENT_PLAN.md 状态为 DONE
```

---

## 九、相关文档

| 文档 | 作用 |
|------|------|
| `gate_spec_v300.md` | 门禁定义 SSOT |
| `check_beta_v300.sh` | Beta 门禁检查脚本 |
| `check_rc_v300.sh` | RC 门禁检查脚本 |
| `check_ga_v300.sh` | GA 门禁检查脚本 |
| `ISSUE_CLOSING_VERIFICATION.md` | Issue 关闭验证流程 |
| `ALPHA_GAPS.md` | Alpha 阶段未完成任务 |
| `BETA_GATE_MASTER_CONTROL.md` | Beta Gate 任务总控 |
| `DEVELOPMENT_PLAN.md` | 开发计划（包含未完成任务延续） |

---

## 十、版本周期回顾机制

> 每次版本发布后必须执行，记录在本文档 §十

### 10.1 回顾触发条件

| 时机 | 触发条件 | 执行人 |
|------|----------|--------|
| 版本发布后 | Tag vX.Y.Z 创建后 48h 内 | Agent |
| 年度复审 | 每年 1 月 | Human Architect |
| 重大豁免后 | EX-XXX 豁免被批准时 | Agent |

### 10.2 回顾内容

每次版本发布后，在本文档 §十添加：

```markdown
## v{X}.{Y}.{Z} 周期回顾 — {日期}

### 门禁执行统计
| 门禁 | 检查次数 | PASS | FAIL | 阻塞次数 |
|------|----------|------|------|----------|
| Alpha | N | M | K | J |
| Beta | N | M | K | J |
| RC | N | M | K | J |
| GA | N | M | K | J |

### Issue 追踪统计
| 指标 | 数值 |
|------|------|
| 新建 Issue | N |
| 已关闭（含 PR 证据）| M |
| 延续到下版本 | K |
| 豁免登记 | J |

### 规范 vs 执行差距
| 差距类型 | 数量 | 示例 |
|----------|------|------|
| 规范缺失 | N | {描述} |
| 规范过时 | N | {描述} |
| 执行缺失 | N | {描述} |
| 阈值偏差 | N | {描述} |

### 治理改进建议
1. {建议 1}
2. {建议 2}

### 遗留问题清单
| Issue | 类型 | 状态 | 延续到 |
|-------|------|------|--------|
| #N | GA-GAP | OPEN | v{X+1}.Y |
```

### 10.3 自我改进验证

版本发布后 30 天内，验证上版本的改进建议是否落实：

```
[ ] 上版本改进建议 #1 是否落实？
[ ] 上版本改进建议 #2 是否落实？
[ ] 未落实的建议是否说明了原因？
```

---

## 十一、追踪脚本使用

### 11.1 gate_lifecycle_check.sh

追踪健康检查脚本，每周或每次门禁 FAIL 后执行：

```bash
bash scripts/gate/gate_lifecycle_check.sh
```

**检查项**:
1. OPEN Issue milestone 覆盖率
2. source/gate-* Issue 追踪状态
3. Issue 有 PR 但未关闭的情况
4. GATE_EXEMPTIONS 豁免过期检查
5. 延续任务映射完整性
6. gate_lifecycle_tracking.md §7 登记完整性
7. 门禁规范与脚本一致性

### 11.2 自动执行时机

| 时机 | 执行 |
|------|------|
| 每次门禁 FAIL 后 | 手动或 CI 自动 |
| 每周一 | CI cronjob 自动 |
| 版本发布前 | 手动 |

---

## 十二、v3.0.0 周期回顾

> 初始版本，基线建立

### v3.0.0 门禁执行统计

| 门禁 | 检查次数 | PASS | FAIL | 阻塞 |
|------|----------|------|------|------|
| Alpha | 1 | 0 | 1 | 1 (B9 SQL Corpus) |
| Beta | 1 | 0 | 1 | 1 (B9 SQL Corpus) |
| RC | 0 | — | — | — |
| GA | 0 | — | — | — |

### v3.0.0 Issue 追踪统计

| 指标 | 数值 |
|------|------|
| 新建 Issue | 1 (#451) |
| 已关闭 | 0 |
| 延续到 v3.1.0 | 1 (#451) |
| 豁免登记 | 0 |

### v3.0.0 治理改进（基于本次工作）

| 改进项 | 来源 | 状态 |
|--------|------|------|
| gate_lifecycle_tracking.md 建立 | 本次工作 | ✅ 已完成 |
| check_beta_v300.sh 增强 FAIL 输出 | 本次工作 | ✅ 已完成 |
| B-S1~B-S5 稳定性测试加入 Beta Gate | 本次工作 | ✅ 已完成 |
| GA_GATE_AUDIT.md 建立 | 本次工作 | ✅ 已完成 |
| GOVERNANCE_AUDIT.md 建立 | 本次工作 | ✅ 已完成 |
| gate_lifecycle_check.sh 脚本创建 | 本次工作 | ✅ 已完成 |
| governance_self_improvement.md 建立 | 本次工作 | ✅ 已完成 |

### v3.0.0 遗留问题

| Issue | 类型 | 状态 | 延续到 |
|-------|------|------|--------|
| #451 | B9 SQL Corpus 20% | OPEN | v3.1.0 |
| GA-GAP-01~09 | GA 门禁差距 | OPEN | v3.1.0 |
| DOC-GAP-01~11 | 文档治理差距 | OPEN | v3.1.0 |

---

*本文档由 hermes agent 创建，用于建立门禁生命周期闭环追踪机制。*
*最后更新: 2026-05-08*

