# Alpha → Beta 版本迁移流程规划

> 版本：v1.0
> 日期：2026-02-18
> 创建者：TRAE (GLM-5.0) - 人类李哥控制

---

## 一、ChatGPT 建议分析

### 1.1 建议内容摘要

ChatGPT 提出了以下建议：

| 建议 | 内容 | 评估 |
|:-----|:-----|:-----|
| 打 Alpha Tag | `git tag v1.0.0-alpha.1` | ✅ 正确，需要执行 |
| 建立标准分支 | main/develop/alpha/beta/release | ⚠️ 部分已存在 |
| Issue + Milestone | 规范 Issue 和里程碑 | ⚠️ 已有 Phase Issue |
| PR 流程 | 功能分支 → alpha → beta | ✅ 正确 |
| 验证清单 | Test Plan 文档 | ✅ 需要补充 |
| 协作文档 | CONTRIBUTING.md 等 | ⚠️ 部分已存在 |

### 1.2 信息缺失分析

ChatGPT 的建议**信息不全面**，未考虑以下已有内容：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ChatGPT 未考虑的已有内容                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 已有分支策略文档                                                        │
│      └── docs/plans/2026-02-16-branch-strategy.md                           │
│                                                                              │
│   2. 已有 baseline 分支                                                      │
│      └── origin/baseline (基线分支)                                         │
│                                                                              │
│   3. 已有 Phase Issue 规划                                                   │
│      ├── #17: Phase 1 - v1.0.0-alpha 收尾                                   │
│      ├── #18: Phase 2 - v1.1.0-beta 功能与流程                              │
│      ├── #19: Phase 3 - baseline 集成与发布闸门                             │
│      └── #20: Phase 4 - 教学演示与复盘                                      │
│                                                                              │
│   4. 已有协作文档                                                            │
│      ├── CONTRIBUTING.md                                                    │
│      └── docs/plans/2026-02-16-pr-workflow.md                               │
│                                                                              │
│   5. 已有标签                                                                │
│      └── v1.0.0 (需要更新)                                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、修正后的版本迁移流程

### 2.1 当前分支状态

```
当前分支结构:
├── main                    # 稳定版本 (保护分支)
├── baseline                # 基线版本 (保护分支) ← ChatGPT 未考虑
├── feature/v1.0.0-alpha    # Alpha 开发分支 (当前工作分支)
├── feature/v1.0.0-beta     # Beta 开发分支
└── feature/*               # 各种功能分支
```

### 2.2 baseline 分支处理方案

**问题**: baseline 分支应该怎么处理？是否和 alpha 同步？

**回答**: **不应该同步**，原因如下：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     baseline 分支定位                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   baseline 是"合并点"，不是"开发分支"                                        │
│                                                                              │
│   正确流程:                                                                  │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                  │
│   │   Alpha     │ ──► │  baseline   │ ──► │    main     │                  │
│   │   开发      │     │   验证点    │     │   发布      │                  │
│   └─────────────┘     └─────────────┘     └─────────────┘                  │
│         │                   │                                                │
│         │                   │                                                │
│         ▼                   ▼                                                │
│   文档补全+错误修复    门禁检查+评审                                         │
│                                                                              │
│   错误做法:                                                                  │
│   ❌ baseline 和 alpha 同步                                                 │
│   ❌ 直接在 baseline 上开发                                                 │
│   ❌ baseline 作为开发分支                                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**baseline 分支的正确使用**:

| 阶段 | 操作 | 说明 |
|:-----|:-----|:-----|
| Alpha 完成 | `git checkout baseline && git merge --no-ff feature/v1.0.0-alpha` | Alpha 合并到 baseline |
| Beta 完成 | `git checkout baseline && git merge --no-ff feature/v1.0.0-beta` | Beta 合并到 baseline |
| 发布 | `git checkout main && git merge --no-ff baseline` | baseline 合并到 main |

---

## 三、完整的 Alpha → Beta 迁移步骤

### 3.1 Step 1: 打 Alpha Tag

```bash
# 当前在 feature/v1.0.0-alpha 分支
git tag -a v1.0.0-alpha.1 -m "Alpha release v1.0.0-alpha.1

Features:
- Basic SQL parsing (SELECT, INSERT, UPDATE, DELETE)
- B+ Tree index
- Volcano execution engine
- Transaction management
- WAL logging
- MySQL protocol basics

Documentation:
- Complete API documentation
- User manual
- Project evolution guide

Quality:
- 284 tests passing
- Coverage: ~76%
- Clippy clean"

git push origin v1.0.0-alpha.1
```

### 3.2 Step 2: 合并 Alpha 到 baseline

```bash
# 切换到 baseline 分支
git checkout baseline

# 合并 alpha（带 --no-ff 保留历史）
git merge --no-ff feature/v1.0.0-alpha

# 推送 baseline
git push origin baseline
```

### 3.3 Step 3: 初始化 Beta 分支

```bash
# 从 baseline 创建/更新 beta 分支
git checkout feature/v1.0.0-beta

# 如果分支已存在，合并 baseline 的更新
git merge --no-ff baseline

# 推送 beta
git push origin feature/v1.0.0-beta
```

### 3.4 Step 4: 创建 Beta Tag

```bash
# Beta 开发完成后
git tag -a v1.0.0-beta.0 -m "Beta release v1.0.0-beta.0

New Features:
- Aggregate functions (COUNT/SUM/AVG/MIN/MAX)
- Improved error handling
- Client/Server separation

Quality:
- Coverage: 85%+
- Benchmark tests
- CI/CD pipeline"

git push origin v1.0.0-beta.0
```

---

## 四、版本阶段定义（修正版）

### 4.1 阶段条件对照表

| 阶段 | 条件 | 当前状态 | 下一步 |
|:-----|:-----|:---------|:-------|
| **Alpha** | 功能完成，无 major bug | ✅ 已达成 | 打 Tag |
| **Alpha→baseline** | 门禁通过，文档完整 | ⏳ 待验证 | 合并 |
| **Beta** | 功能稳定，测试覆盖 ≥80% | ❌ 未达成 | 开发中 |
| **Beta→baseline** | 完整测试+文档齐全 | ❌ 未达成 | - |
| **Release Candidate** | 修复所有 showstopper bugs | ❌ 未达成 | - |
| **GA/Main** | 正式稳定发布 | ❌ 未达成 | - |

### 4.2 门禁检查清单

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Alpha → baseline 门禁检查                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   代码质量                                                                   │
│   ├── [ ] cargo build --release 通过                                        │
│   ├── [ ] cargo test --all-features 通过                                    │
│   ├── [ ] cargo clippy --all-features -- -D warnings 通过                   │
│   └── [ ] cargo fmt --check 通过                                            │
│                                                                              │
│   测试覆盖                                                                   │
│   ├── [ ] 总覆盖率 ≥ 76%                                                    │
│   └── [ ] 关键模块覆盖率 ≥ 70%                                              │
│                                                                              │
│   文档完整性                                                                 │
│   ├── [ ] README.md 更新                                                    │
│   ├── [ ] CHANGELOG.md 更新                                                 │
│   ├── [ ] API 文档完整                                                      │
│   └── [ ] 用户手册完整                                                      │
│                                                                              │
│   流程完整性                                                                 │
│   ├── [ ] 所有 Alpha Issue 关闭                                             │
│   ├── [ ] PR 审查通过                                                       │
│   └── [ ] 门禁结论记录                                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 五、Issue 与 Milestone 规划（修正版）

### 5.1 当前 Issue 状态

| Issue | 标题 | 状态 | 阶段 |
|:------|:-----|:-----|:-----|
| #17 | Phase 1: v1.0.0-alpha 收尾 | OPEN | Alpha |
| #18 | Phase 2: v1.1.0-beta 功能与流程 | OPEN | Beta |
| #19 | Phase 3: baseline 集成与发布闸门 | OPEN | Release |
| #20 | Phase 4: 教学演示与复盘 | OPEN | GA |

### 5.2 建议的 Milestone（需在 GitHub 创建）

由于 `gh milestone` 命令不可用，建议手动在 GitHub 创建：

```
Milestones:
├── v1.0.0-alpha    → 关联 Issue #17
├── v1.0.0-beta     → 关联 Issue #18
├── v1.0.0-rc       → 关联 Issue #19
└── v1.0.0          → 关联 Issue #20
```

### 5.3 建议的 Labels（补充）

当前只有 `enhancement` 标签，建议添加：

| Label | 颜色 | 用途 |
|:------|:-----|:-----|
| `alpha` | `#ff6b6b` | Alpha 阶段任务 |
| `beta` | `#4ecdc4` | Beta 阶段任务 |
| `release` | `#45b7d1` | Release 阶段任务 |
| `bug` | `#e74c3c` | Bug 修复 |
| `documentation` | `#9b59b6` | 文档相关 |
| `testing` | `#f39c12` | 测试相关 |
| `P0` | `#c0392b` | 最高优先级 |
| `P1` | `#e67e22` | 高优先级 |
| `P2` | `#3498db` | 中优先级 |

---

## 六、PR 流程规范（修正版）

### 6.1 功能分支命名规范

```
feature/v{version}-{feature-name}

Examples:
├── feature/v1.1.0-aggregate      # 聚合函数
├── feature/v1.1.0-error-handling # 错误处理
├── feature/v1.1.0-coverage       # 覆盖率提升
└── feature/beta-network-improvement  # Network 改进
```

### 6.2 PR 流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PR 流程图                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 创建功能分支                                                            │
│      git checkout feature/v1.0.0-alpha                                      │
│      git checkout -b feature/v1.1.0-aggregate                               │
│                                                                              │
│   2. 开发 + 测试                                                             │
│      cargo test --all-features                                              │
│      cargo clippy --all-features                                            │
│                                                                              │
│   3. 提交 PR                                                                 │
│      gh pr create --base feature/v1.0.0-alpha \                             │
│                  --head feature/v1.1.0-aggregate \                          │
│                  --title "feat: add aggregate functions"                    │
│                                                                              │
│   4. 代码审查                                                                │
│      ├── AI-CLI 评阅                                                        │
│      ├── CI 检查通过                                                        │
│      └── 至少 1 个 approve                                                  │
│                                                                              │
│   5. 合并                                                                    │
│      git checkout feature/v1.0.0-alpha                                      │
│      git merge --no-ff feature/v1.1.0-aggregate                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 七、验证清单（Test Plan）

### 7.1 Beta 阶段验证清单

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Beta 验证清单                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   功能完整性                                                                 │
│   ├── [ ] 聚合函数: COUNT/SUM/AVG/MIN/MAX                                   │
│   ├── [ ] 错误处理: 无 panic 风险                                           │
│   ├── [ ] 客户端/服务器: 基础连接                                           │
│   └── [ ] 所有 SQL 语句正常执行                                             │
│                                                                              │
│   性能测试                                                                   │
│   ├── [ ] SELECT 1000 行性能基线                                            │
│   ├── [ ] INSERT 100 行性能基线                                             │
│   ├── [ ] B+ Tree 查询性能基线                                              │
│   └── [ ] 并发查询性能基线                                                  │
│                                                                              │
│   容错 & 并发测试                                                            │
│   ├── [ ] 断开连接恢复                                                      │
│   ├── [ ] 超时处理                                                          │
│   ├── [ ] 大数据包处理                                                      │
│   └── [ ] 多客户端并发                                                      │
│                                                                              │
│   文档验证                                                                   │
│   ├── [ ] API 文档完整                                                      │
│   ├── [ ] 用户手册更新                                                      │
│   ├── [ ] CHANGELOG 更新                                                    │
│   └── [ ] 教学交付物完整                                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 八、协作文档补充

### 8.1 已有文档

| 文档 | 状态 | 说明 |
|:-----|:-----|:-----|
| CONTRIBUTING.md | ✅ 已有 | 需要更新 |
| docs/plans/2026-02-16-branch-strategy.md | ✅ 已有 | 分支策略 |
| docs/plans/2026-02-16-pr-workflow.md | ✅ 已有 | PR 流程 |

### 8.2 建议新增文档

| 文档 | 优先级 | 说明 |
|:-----|:-------|:-----|
| BRANCHING.md | P1 | 分支职责说明 |
| RELEASE_PROCESS.md | P1 | 发布流程 |
| CODE_OF_CONDUCT.md | P2 | 行为准则 |
| TestPlan.md | P0 | 验证清单 |

---

## 九、执行计划

### 9.1 立即执行

```bash
# 1. 打 Alpha Tag
git checkout feature/v1.0.0-alpha
git tag -a v1.0.0-alpha.1 -m "Alpha release v1.0.0-alpha.1"
git push origin v1.0.0-alpha.1

# 2. 合并到 baseline
git checkout baseline
git merge --no-ff feature/v1.0.0-alpha
git push origin baseline

# 3. 更新 Beta 分支
git checkout feature/v1.0.0-beta
git merge --no-ff baseline
git push origin feature/v1.0.0-beta
```

### 9.2 后续任务

| 任务 | 负责人 | 时间 |
|:-----|:-------|:-----|
| 创建 Milestone | 人工 | 今天 |
| 添加 Labels | 人工 | 今天 |
| 开始 Beta 开发 | AI-CLI | 明天 |
| 创建 TestPlan.md | TRAE | 今天 |

---

## 十、总结

### 10.1 ChatGPT 建议修正

| 原建议 | 修正 |
|:-------|:-----|
| 创建 alpha/beta 分支 | 已存在，使用现有分支 |
| baseline 和 alpha 同步 | ❌ 错误，baseline 是合并点 |
| 创建 CONTRIBUTING.md | 已存在，需要更新 |
| Issue 不规范 | 已有 Phase Issue 规划 |

### 10.2 关键决策

1. **baseline 分支**: 不与 alpha 同步，作为合并验证点
2. **Alpha Tag**: 立即打 `v1.0.0-alpha.1`
3. **Beta 初始化**: 从 baseline 合并，不是从 alpha
4. **Issue 规划**: 使用现有 Phase Issue，补充 Milestone

---

*本文档由 TRAE (GLM-5.0) 创建，基于 ChatGPT 建议进行补全和修正*
