# 治理自我进化与健康检查机制

> **版本**: 1.0
> **日期**: 2026-05-08
> **目的**: 建立治理规则的自我检查、自我反馈、自动进化机制，防止治理文档僵化
> **适用范围**: 所有 governance 文档

---

## 一、核心原则

### 1.1 治理三权分立

```
┌─────────────────────────────────────────────┐
│              治理规则 (Rule)                  │
│  gate_spec / RELEASE_POLICY / LIFECYCLE     │
└─────────────────────────────────────────────┘
                      ↑
              审查 (Audit)         执行 (Enforce)
                      ↑                       │
┌─────────────────────────────────────────────┐
│           治理执行 (Execution)               │
│  gate scripts / CI/CD / Issue tracking      │
└─────────────────────────────────────────────┘
```

### 1.2 自我进化循环

```
每次发布周期结束
       ↓
治理健康检查（本文档 §三）
       ↓
识别规范与实际执行差距（§四 差距分析）
       ↓
生成优化建议（§五 改进机制）
       ↓
更新 governance 文档
       ↓
下一周期验证效果
```

---

## 二、治理健康检查 — 每个版本发布后必须执行

### 2.1 检查频率

| 检查点 | 时机 | 执行人 |
|--------|------|--------|
| 门禁后自检 | 每个门禁检查（Alpha/Beta/RC/GA）FAIL 时 | Agent |
| 版本发布前 | 进入下一阶段前 | Human Architect |
| 版本发布后 | Tag vX.Y.Z 创建后 48h 内 | Agent |
| 年度复审 | 每年 1 月 | Human Architect |

### 2.2 检查清单

每次门禁 FAIL 时，必须同时执行以下检查：

#### 检查 A: 规则是否完整

```
[ ] 失败的检查项是否有对应的 Issue？
[ ] Issue 是否绑定了 milestone？
[ ] Issue 是否包含：来源门禁、检查命令、失败输出？
[ ] 是否在 gate_lifecycle_tracking.md §7 登记？
```

#### 检查 B: 追踪是否闭环

```
[ ] Issue 关闭前是否验证了 closedByPullRequestsReferences 非空？
[ ] 修复的 PR 是否已合并到目标分支？
[ ] 相关测试是否在本地或 CI 通过？
[ ] 门禁重新检查是否 PASS？
```

#### 检查 C: 版本延续是否执行

```
[ ] 无法在当前版本修复的问题是否已延续到下版本？
[ ] 是否在 DEVELOPMENT_PLAN.md §6 建立了映射？
[ ] 下版本的 Issue 是否引用了原 Issue？
```

#### 检查 D: 豁免是否登记

```
[ ] 无法修复且可豁免的问题是否在 GATE_EXEMPTIONS.md 登记？
[ ] 豁免是否经过 Tech Lead 审批？
[ ] 豁免是否有明确的复审日期？
```

---

## 三、门禁失败标准处置流程

### 3.1 四种处置结果

```
门禁检查 FAIL
       │
       ├── A. 能立即修复（< 1 人天）
       │       → 立即修复 → 验证 → PR 合并
       │
       ├── B. 不能立即修复但可在本版本完成
       │       → 创建 Issue → 标记 milestone → 排期修复
       │
       ├── C. 无法在当前版本完成
       │       → 创建 Issue → 标记 milestone
       │       → 登记到下版本 DEVELOPMENT_PLAN.md §6
       │       → 在 gate_lifecycle_tracking.md §7 登记
       │
       └── D. 不可抗力（外部依赖/基础设施）
               → 评估豁免可能性
               → 在 GATE_EXEMPTIONS.md 登记
               → 设置复审日期
```

### 3.2 Issue 创建模板

每个门禁失败必须创建 Issue，模板：

```markdown
## Issue 标题
[{门禁项}] {简短描述}

## Issue 内容
### 门禁来源
- 门禁脚本: check_{phase}_v300.sh
- 检查项: {G1/B2/R5 等}
- 命令: `{实际执行的命令}`

### 失败输出
```
{粘贴完整的失败输出}
```

### 根因分析
{简要分析失败原因}

### 影响范围
- 阻塞: {当前阶段门禁}
- 影响: {对其他门禁项的连带影响}

### 验收条件
- [ ] {具体可验证的条件 1}
- [ ] {具体可验证的条件 2}

### 版本延续
- [ ] 已延续到 v{next}.0 DEVELOPMENT_PLAN.md §6
- 原 Issue: #{原始 Issue 号}
```

### 3.3 Issue 关闭验证清单

**禁止在无 PR 证据时关闭 Issue**。

```
Issue 关闭前检查
====================

Issue #: ___________

[ ] 1. 执行 gh issue view {id} --json closedByPullRequestsReferences
    结果: ___________ (必须非空才能继续)

[ ] 2. 执行 gh pr view {pr_number} --json state,mergedAt
    state: ___________
    mergedAt: ___________

[ ] 3. 确认代码已合并到目标分支
    分支: ___________
    Commit: ___________

[ ] 4. 确认测试通过
    命令: ___________
    结果: ___________

[ ] 5. 确认门禁重新检查 PASS
    命令: ___________

结论: ___________ (可以关闭 / 禁止关闭 - 原因: ___________)
```

---

## 四、版本间遗留问题延续机制

### 4.1 延续触发条件

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 判定 |
|------|------|
| 修复需要 3 人天以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖 | CBO 需要先完成索引选择等 |

### 4.2 延续标准格式

在 `DEVELOPMENT_PLAN.md` §6 中建立映射：

```markdown
## {NextVersion} 延续任务（来自 {CurrentVersion} 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | {NextVersion} 目标 | 验收条件 |
|----------|----------|------------|---------------------|----------|
| #{issue} | {任务} | {当前状态} | {目标} | {验收条件} |
```

### 4.3 追踪验证

每个版本发布前，验证上版本遗留问题：

```
版本发布前检查
==================

[ ] 上版本所有 OPEN Issue 均有延续记录
[ ] 延续 Issue 已在 gate_lifecycle_tracking.md §7 登记
[ ] GATE_EXEMPTIONS.md 中豁免项均有复审日期
[ ] 所有 Issue 关闭前均验证了 closedByPullRequestsReferences
```

---

## 五、治理规范与实际执行差距分析

### 5.1 差距识别

每次版本发布后，分析以下差距：

| 差距类型 | 识别方法 | 处置 |
|----------|----------|------|
| **规范缺失** | 脚本有检查项但规范未定义 | 补充规范，或移除脚本检查项 |
| **规范过时** | 规范定义与实际流程不符 | 更新规范或修正流程 |
| **执行缺失** | 规范有要求但未实现 | 实现或申请豁免 |
| **阈值偏差** | 规范阈值与实际测试结果不符 | 调整阈值或改进实现 |

### 5.2 自反馈记录

在 `gate_lifecycle_tracking.md` 中，每次版本发布后更新：

```markdown
## v{X}.{Y}.{Z} 周期回顾

### 规范 vs 执行差距
| 差距 | 类型 | 处置 |
|------|------|------|

### Issue 追踪统计
| 指标 | 数值 |
|------|------|
| 新建 Issue | N |
| 已关闭 | M |
| 延续到下版本 | K |
| 豁免登记 | J |

### 治理改进建议
1. {建议 1}
2. {建议 2}
```

---

## 六、门禁脚本与规范的双向同步

### 6.1 原则

```
规范定义 ←→ 脚本实现
    ↑              ↓
    ←── 差距 ────←
```

每当发生以下情况，必须同步更新：

| 触发事件 | 同步要求 |
|----------|----------|
| 规范新增检查项 | 脚本必须在下一版本实现 |
| 脚本新增检查项 | 必须同步到规范 SSOT |
| 规范修改阈值 | 脚本必须同步更新 |
| 脚本发现更好的实现 | 必须回填到规范并说明理由 |

### 6.2 同步检查

```bash
# 检查规范中定义的检查项是否都在脚本中实现
for gate in $(grep "^|| [A-Z][0-9]" gate_spec_v300.md | awk '{print $2}'); do
    if ! grep -q "$gate" scripts/gate/check_*v300.sh; then
        echo "MISSING: $gate not in any gate script"
    fi
done

# 检查脚本中的检查项是否都在规范中定义
for script_check in $(grep "check_output\|TOTAL=\$((TOTAL" scripts/gate/check_*v300.sh | grep -oE '[A-Z][0-9]+'); do
    if ! grep -q "$script_check" gate_spec_v300.md; then
        echo "EXTRA: $script_check in script but not in spec"
    fi
done
```

---

## 七、治理文档自进化流程

### 7.1 进化触发条件

治理文档在以下情况必须更新：

| 触发条件 | 更新内容 | 责任人 |
|----------|----------|--------|
| 新增门禁检查项 | gate_spec_v300.md + check_*.sh | Agent |
| 新增文档要求 | VERSION_DOCS_SPEC.md + DOCUMENT_COMPLETENESS_CHECK.md | Agent |
| 发现治理漏洞 | 本文档 + 相关治理文档 | Human Architect |
| 版本发布后 | gate_lifecycle_tracking.md §周期回顾 | Agent |
| 豁免被滥用 | GATE_EXEMPTIONS.md 豁免政策收紧 | Tech Lead |

### 7.2 进化审查清单

每次治理文档更新后，必须验证：

```
[ ] 新增检查项是否在 gate_spec_v300.md 定义？（SSOT 原则）
[ ] 新增规范是否在 check_*.sh 中实现？
[ ] 变更是否影响了已有 Issue 的追踪状态？
[ ] 是否需要更新 GATE_EXEMPTIONS.md？
[ ] 是否需要更新 DEVELOPMENT_PLAN.md §6 延续任务？
[ ] 是否在本次版本 governance_audit 中记录？
```

### 7.3 文档版本管理

治理文档使用语义化版本：

```
governance/{doc}_v{Major}.{Minor}.md
```

| 变更类型 | 版本规则 | 示例 |
|----------|----------|------|
| 补充检查项 | Minor +1 | v1.0 → v1.1 |
| 修改阈值/定义 | Major +1 | v1.1 → v2.0 |
| 结构性重构 | Major +1 | v2.0 → v3.0 |

---

## 八、闭环追踪系统设计

### 8.1 追踪数据模型

```
Issue (Gitea)
  ├── milestone: vX.Y.Z-{phase}
  ├── labels: [kind/feature, priority/p0, source/gate-failure]
  ├── linked_to: PR (closedByPullRequestsReferences)
  ├── tracked_in: gate_lifecycle_tracking.md §7
  └── carried_forward_to: v{X+1}.Y.Z DEVELOPMENT_PLAN.md §6
        │
        └── merged: true/false
              │
              └── closed: true/false (with PR evidence)
```

### 8.2 自动追踪脚本

在 `scripts/gate/` 中建立追踪脚本：

```bash
#!/usr/bin/env bash
# gate_lifecycle_check.sh — 检查 Issue 追踪闭环
set -euo pipefail

echo "=== 门禁追踪健康检查 ==="
echo ""

# 检查 1: 所有 OPEN Issue 是否都有 milestone
OPEN_ISSUES=$(gh issue list --state open --json number,title,milestone --jq '.[] | "\(.number) \(.title) \(.milestone.title // "NONE")"')
echo "OPEN Issue milestone 覆盖率:"
echo "$OPEN_ISSUES" | while read num title ms; do
    if [ "$ms" = "NONE" ]; then
        echo "  ⚠️  #$num 缺少 milestone: $title"
    else
        echo "  ✅ #$num milestone=$ms"
    fi
done

# 检查 2: 所有未关闭 Issue 是否都有 gate_lifecycle_tracking 记录
echo ""
echo "Issue 追踪记录检查:"
# ...

# 检查 3: GATE_EXEMPTIONS 豁免是否有过期项
echo ""
echo "豁免复审日期检查:"
# ...
```

### 8.3 Issue 自动分类

创建 Issue 时，Agent 必须根据失败类型打标签：

| 标签 | 含义 | 触发条件 |
|------|------|----------|
| `source/gate-beta` | Beta 门禁失败 | check_beta_v300.sh FAIL |
| `source/gate-rc` | RC 门禁失败 | check_rc_v300.sh FAIL |
| `source/gate-ga` | GA 门禁失败 | check_ga_v300.sh FAIL |
| `type/performance` | 性能问题 | QPS 不达标 |
| `type/coverage` | 覆盖率问题 | 覆盖率低于阈值 |
| `type/sql-compat` | SQL 兼容问题 | Corpus 测试失败 |
| `carried/v3.1.0` | 延续到下版本 | 当前版本无法完成 |

---

## 九、实际工作总结（v3.0.0 经验固化）

### 9.1 v3.0.0 实际执行记录

| 阶段 | 事件 | 执行结果 |
|------|------|----------|
| Beta 门禁 | check_beta_v300.sh FAIL | ✅ 已创建 Issue #451 (SQL operations 20%) |
| Beta 门禁 | B-S1~B-S5 未检查 | ❌ 脚本未包含，已补充 |
| Beta 门禁 | FAIL 无详细输出 | ❌ 无法追踪，已增强脚本输出 |
| GA 审查 | 发现 9 个 GA-GAP | ✅ 已创建 GA_GATE_AUDIT.md |
| GA 审查 | 12 个根文档缺失 | ✅ 已登记到 GOVERNANCE_AUDIT.md |
| 版本延续 | Issue #451 延续 | ✅ 已登记到 DEVELOPMENT_PLAN.md §6 |

### 9.2 规则进化项（基于 v3.0.0 经验）

从 v3.0.0 实际工作中抽取的规则：

**规则 1**: 门禁脚本必须输出 FAIL 原因数组

```
旧规则: check() 只报告 PASS/FAIL
新规则: 每个 FAIL 必须追加到 FAIL_REASONS 数组，脚本末尾输出
```

**规则 2**: B-S 稳定性测试必须在 Beta/GA 门禁中检查

```
旧规则: gate_spec.md 未定义 B-S1~B-S5
新规则: gate_spec_v300.md G12 强制要求 B-S1~B-S5
```

**规则 3**: 每个 Issue 必须关联 milestone

```
旧规则: 无强制要求
新规则: 所有 source/gate-* Issue 必须绑定 milestone
```

**规则 4**: 规范与脚本不一致时，脚本优先更新规范

```
旧规则: 可能规范和脚本都不更新
新规则: 发现不一致时，优先更新规范（SSOT），脚本同步
```

---

## 十、相关文档

| 文档 | 作用 |
|------|------|
| `gate_spec_v300.md` | 门禁定义 SSOT |
| `gate_lifecycle_tracking.md` | 门禁生命周期追踪 |
| `GATE_EXEMPTIONS.md` | 门禁豁免记录 |
| `ISSUE_CLOSING_VERIFICATION.md` | Issue 关闭验证流程 |
| `VERSION_DOCS_SPEC.md` | 版本文档规范 |
| `DOCUMENT_COMPLETENESS_CHECK.md` | 文档完整性检查 |

---

*本文档由 hermes agent 创建，基于 v3.0.0 实际工作经验总结。*
*最后更新: 2026-05-08*
