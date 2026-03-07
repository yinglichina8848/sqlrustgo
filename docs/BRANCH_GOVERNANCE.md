# SQLRustGo 分支治理规范

## 1. 概述

本文档定义了 SQLRustGo 项目的分支治理规范，包括分支保护策略、命名规范、Git Flow 流程和版本推进模型。这些规范旨在确保项目的长期稳定性、可维护性和可扩展性，特别是在多 Agent 协作的环境下。

## 2. 分支保护策略

### 2.1 核心目标

对以下分支实现严格保护：
- ❌ 禁止直接 push
- ❌ 禁止直接 commit
- ✅ 必须通过 Pull Request
- ✅ 至少 1 个 reviewer（不能是自己）
- ✅ CI 通过后才能合并
- ✅ 禁止强制 push
- ✅ 禁止删除分支

### 2.2 GitHub 设置方法

1. **进入仓库设置**：
   - 打开 GitHub 仓库
   - 点击 Settings
   - 左侧选择 Branches
   - 点击 Add branch protection rule

2. **创建保护规则**：

   | 分支模式 | 保护选项 |
   |---------|----------|
   | `main` | ✅ Require PR<br>✅ Require review (1)<br>✅ Require status check<br>✅ Disable force push<br>✅ Disable delete |
   | `alpha` | ✅ Require PR<br>✅ Require review (1)<br>✅ Require status check<br>✅ Disable force push<br>✅ Disable delete |
   | `beta` | ✅ Require PR<br>✅ Require review (1)<br>✅ Require status check<br>✅ Disable force push<br>✅ Disable delete |
   | `rc` | ✅ Require PR<br>✅ Require review (1)<br>✅ Require status check<br>✅ Disable force push<br>✅ Disable delete |
   | `release/*` | ✅ Require PR<br>✅ Require review (1)<br>✅ Require status check<br>✅ Disable force push<br>✅ Disable delete |
   | `feature/*` | ❌ 不强制 review |
   | `docs/*` | ❌ 不强制 review |
   | `hotfix/*` | ❌ 不强制 review |

3. **推荐开启的选项**：
   - ✅ Require a pull request before merging
   - ✅ Require approvals (1 个)
   - ✅ Dismiss stale pull request approvals
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Restrict who can push to matching branches (仅 admin 或 Release Manager)
   - ✅ Do not allow bypassing the above settings
   - ✅ Disable force pushes
   - ✅ Disable deletions

## 3. 分支命名规范

### 3.1 命名原则

分支名应满足：
- ✅ 可分类（通过前缀识别类型）
- ✅ 可批量管理（便于写 branch protection pattern）
- ✅ 可自动化（便于 CI 识别）
- ✅ 可阅读（人一眼知道用途）
- ❌ 不混用风格

### 3.2 标准命名结构

统一格式：
```
<type>/<scope>-<description>
```

### 3.3 推荐分支类型

| 类型 | 作用 | 示例 | 是否保护 |
|------|------|------|----------|
| `main` | 最终发布 | `main` | ✅ |
| `alpha` | 内部开发集成 | `alpha` | ✅ |
| `beta` | 测试稳定分支 | `beta` | ✅ |
| `rc` | 候选版本 | `rc/v1.0.0-1` | ✅ |
| `release/*` | 发布版本 | `release/v1.0.0` | ✅ |
| `feature/*` | 功能开发 | `feature/vector-engine` | ❌ |
| `bugfix/*` | 普通修复 | `bugfix/join-null-check` | ❌ |
| `hotfix/*` | 紧急修复 | `hotfix/memory-leak` | ❌ |
| `docs/*` | 文档 | `docs/version-planning` | ❌ |
| `experiment/*` | 实验 | `experiment/new-optimizer` | ❌ |

### 3.4 命名示例

- **功能分支**：`feature/query-optimizer`、`feature/vector-filter`
- **修复分支**：`bugfix/error-handling`、`hotfix/security-vulnerability`
- **文档分支**：`docs/api-reference`、`docs/installation-guide`
- **版本分支**：`release/v1.0.0`、`rc/v1.0.0-2`

### 3.5 禁止的命名

- ❌ 无意义的名称：`test`、`new-branch`、`temp`、`123`
- ❌ 混用风格：`v1.0`、`release-1.0`、`v1.0-rc`
- ❌ 个人名称：`liying-test`、`developer1-feature`

## 4. 分支重命名流程

### 4.1 本地重命名

```bash
git branch -m old-name new-name
```

### 4.2 远程重命名流程

1. **推送新分支**：
   ```bash
git push origin new-name
   ```

2. **删除旧分支**：
   ```bash
git push origin --delete old-name
   ```

### 4.3 重命名注意事项

⚠️ 重命名前需确认：
- 没有 open PR 指向该分支
- 没有 CI 配置绑定该分支
- 没有 branch protection rule 绑定该分支
- 没有其他人基于它开发

### 4.4 重命名流程

1. **通知团队**：告知所有团队成员
2. **确认状态**：检查 PR、CI、保护规则等
3. **执行重命名**：本地和远程
4. **更新配置**：更新相关的 CI 和保护规则
5. **验证**：确保所有流程正常运行

## 5. 分层稳定性推进模型 (Layered Stability Promotion Model)

### 5.1 核心理念

本模型专为系统级工程（AI + 数据库 + OS 能力）、长周期架构、多阶段稳定性控制和未来可能的多 Agent 协作设计。

### 5.2 主干分支结构

🔷 **核心稳定分支（长期存在）**

| 分支 | 稳定级别 | 作用 |
|------|----------|------|
| `main` | ⭐⭐⭐⭐⭐ | 最终发布版本 |
| `rc` | ⭐⭐⭐⭐ | Release Candidate |
| `beta` | ⭐⭐⭐ | 集成测试 |
| `alpha` | ⭐⭐ | 内部开发集成 |

### 5.3 功能开发分支

所有功能必须从 `beta` 创建分支，格式：

```
feature/<module>
bugfix/<issue>
hotfix/<issue>
experiment/<idea>
docs/<topic>
```

示例：
- `feature/vector-engine`
- `feature/cbo-planner`
- `bugfix/join-null-check`

### 5.4 版本推进流程

#### 5.4.1 日常开发阶段

```
beta
   ↓
feature/*
   ↓ PR + Review
beta
```

**规则**：
- 禁止直接 push beta
- 必须通过 PR
- 至少 1 个 review
- CI 必须通过

#### 5.4.2 冻结版本（进入 RC）

当 beta 达到稳定程度：

```
beta → rc/v1.0.0-1 → rc
```

**操作**：
```bash
git checkout beta
git checkout -b rc/v1.0.0-1
git push origin rc/v1.0.0-1
# 然后合并到 rc 分支
```

**此时**：
- beta 继续开发 v1.1
- rc 只允许 bugfix

#### 5.4.3 RC 修复阶段

只允许 `bugfix/*` 和 `hotfix/*` 合并到 `rc`，禁止新 feature。

#### 5.4.4 正式发布

当 rc 稳定：

```
rc → release/v1.0.0 → main
```

**操作流程**：
```bash
# 从 rc 创建 release 分支
git checkout rc
git checkout -b release/v1.0.0
git push origin release/v1.0.0
# PR 到 main
# 打 tag
git tag v1.0.0
git push origin v1.0.0
```

### 5.5 Hotfix 流程

如果线上 main 出问题：

```
main
   ↓
hotfix/*
   ↓ PR
main
   ↓ 回流
rc
   ↓ 回流
beta
```

**规则**：
- 必须从 main 创建 hotfix 分支
- 修复后 PR 到 main
- 必须回流到 rc 和 beta，否则会产生分叉灾难

### 5.6 完整流程图

```
feature/* ─┐
bugfix/*  ─┼──→ beta ───→ rc ───→ release/v1.0.0 ───→ main
experiment ┘

main hotfix
   ↓
hotfix/* → main
             ↓
            rc
             ↓
            beta
```

## 6. 版本推进模型

### 6.1 版本生命周期

基于分层稳定性推进模型的版本生命周期：

```
feature/* → beta → rc → release/* → main
```

### 6.2 版本号策略

采用 **MAJOR.MINOR.PATCH** 版本号格式：

- **发布版本**：`vX.Y.Z`
  - X：主版本号（重大架构变更）
  - Y：次版本号（新功能）
  - Z：补丁版本号（bug 修复）

- **候选版本**：`rc/vX.Y.Z-N`
  - N：候选版本序号

- **补丁版本**：`vX.Y.Z+1`（hotfix）

### 6.3 版本推进标准

| 阶段 | 稳定级别 | 进入标准 | 退出标准 |
|------|----------|----------|----------|
| **Alpha** | ⭐⭐ | 功能开发开始 | 内部测试通过，可集成到 beta |
| **Beta** | ⭐⭐⭐ | Alpha 集成完成 | 稳定性测试通过，无严重 Bug |
| **RC** | ⭐⭐⭐⭐ | Beta 达到稳定 | 无阻断性 Bug，所有测试通过 |
| **Release** | ⭐⭐⭐⭐⭐ | RC 测试通过 | 准备发布，文档完整 |
| **Main** | ⭐⭐⭐⭐⭐ | Release 完成 | 正式发布，可部署生产 |

### 6.4 版本并行策略

- **v1.0 系列**：在 `rc` 分支维护
- **v1.1 系列**：在 `beta` 分支开发
- **v2.0 系列**：在 `alpha` 分支探索

### 6.5 版本回溯策略

- **Hotfix**：从 `main` 创建，修复后回流到所有相关分支
- **Bugfix**：从对应版本分支创建，修复后同步到后续版本分支

## 7. 适配 AI / 多 Agent 协作

### 7.1 核心原则

- **无差别对待**：无论是人类开发者还是 AI Agent 创建的 PR，规则相同
- **强制审查**：任何 PR 都需要至少 1 个 reviewer，禁止 self-approve
- **权限平等**：AI Agent 应使用专用账号，与人类开发者享有同等权限

### 7.2 实践建议

1. **创建专用 Agent 账号**：
   - `sqlrustgo-bot`
   - `reviewer-bot`

2. **配置 CODEOWNERS**：
   - 明确各模块的负责人
   - 确保关键模块有专人审查

3. **自动化工具**：
   - GitHub Actions 自动检查 PR 合规性
   - 禁止 PR 作者给自己 approve 的 bot
   - 自动标记需要审查的 PR

4. **协作流程**：
   - Agent 可以创建 PR，但不能合并
   - 人类开发者负责最终审查和合并
   - 建立 PR 模板，确保信息完整

### 7.3 未来扩展

- **多 Agent 分工**：不同 Agent 负责不同模块
- **自动测试**：Agent 自动运行测试并报告结果
- **智能审查**：Agent 辅助代码审查，提出建议

## 8. 为什么采用此模型

### 8.1 优势

- ✅ **功能和发布分离**：开发和发布互不干扰
- ✅ **稳定性分层**：明确的稳定级别，便于管理
- ✅ **热修复可回流**：确保所有分支都能获得修复
- ✅ **支持长期版本并行**：多版本同时维护
- ✅ **支持多人/多 Agent**：适合团队协作
- ✅ **支持自动化 CI/CD**：易于集成自动化工具

### 8.2 与传统 Git Flow 的区别

- **更简洁**：减少了分支数量，主干分支清晰
- **更语义化**：`beta` 比 `develop` 更直观
- **更灵活**：RC 分支独立，便于版本控制
- **更适合系统级项目**：分层稳定性更符合系统级工程需求

### 8.3 适用场景

- **系统级工程**：AI + 数据库 + OS 能力
- **长周期架构**：需要长期维护的项目
- **多阶段稳定性控制**：对稳定性有严格要求
- **多 Agent 协作**：未来可能的 AI 协作场景

## 9. 自动化与 CI 集成

### 9.1 CI 触发条件

- **feature/***：每次 push 触发构建和测试
- **alpha**：每次 push 触发完整测试
- **beta**：每次 push 触发完整测试和集成测试
- **rc/***：每次 push 触发完整测试、集成测试和性能测试
- **release/***：每次 push 触发完整测试套件
- **main**：每次 push 触发完整测试套件和发布流程

### 7.2 自动化工具

- **PR 检查**：自动检查 PR 标题、描述、提交信息
- **分支保护**：通过 GitHub API 自动配置分支保护规则
- **版本管理**：自动生成版本号和发布说明
- **代码质量**：自动运行代码风格检查和静态分析

## 10. 团队协作规范

### 10.1 PR 流程

1. **创建 PR**：从功能分支创建 PR 到目标分支
2. **填写描述**：详细描述变更内容、目的和测试结果
3. **添加 reviewer**：至少添加 1 个 reviewer
4. **等待审核**： reviewer 审核代码
5. **处理反馈**：根据审核意见进行修改
6. **合并 PR**：审核通过后合并

### 8.2 代码审查标准

- **代码质量**：符合项目编码规范
- **功能正确性**：实现预期功能，通过测试
- **性能影响**：无明显性能劣化
- **安全性**：无安全漏洞
- **可维护性**：代码清晰，有适当的注释

### 8.3 团队沟通

- **分支创建**：重要分支创建时通知团队
- **PR 提交通知**：PR 创建后通知相关人员
- **审核反馈**：及时回复审核意见
- **发布通知**：版本发布后通知团队

## 11. 实施计划

### 11.1 阶段一：基础设置（1-2 周）

1. **更新分支保护规则**：创建基于 Pattern 的保护规则
2. **优化现有分支**：评估和重命名现有分支
3. **制定 CODEOWNERS**：明确各目录的负责人
4. **更新文档**：完善分支治理文档

### 9.2 阶段二：流程优化（2-4 周）

1. **实现 CI 集成**：配置分支触发条件
2. **自动化工具**：实现 PR 检查和分支保护自动化
3. **团队培训**：培训团队成员遵循规范
4. **流程验证**：验证流程的可行性和有效性

### 9.3 阶段三：持续改进（长期）

1. **定期回顾**：每月回顾分支策略和流程
2. **优化调整**：根据项目演进调整规范
3. **工具升级**：引入新的自动化工具
4. **经验总结**：积累和分享最佳实践

## 12. 附录

### 12.1 术语表

- **Alpha**：内部开发集成阶段
- **Beta**：测试稳定阶段
- **RC**：候选版本阶段
- **Release**：发布版本阶段
- **Main**：最终发布分支
- **Feature**：功能开发分支
- **Bugfix**：普通修复分支
- **Hotfix**：紧急修复分支
- **PR**：Pull Request，代码合并请求
- **CI**：持续集成，自动构建和测试
- **Branch Protection**：分支保护规则，防止意外操作

### 10.2 参考资源

- [GitHub Branch Protection Rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches)
- [Git Flow](https://nvie.com/posts/a-successful-git-branching-model/)
- [Conventional Commits](https://www.conventionalcommits.org/)

### 10.3 变更记录

| 日期 | 版本 | 变更内容 | 作者 |
|------|------|----------|------|
| 2026-02-20 | v1.0 | 初始版本 | SQLRustGo 团队 |

## 13. 结语

分支治理是大型项目成功的关键因素之一，特别是在多 Agent 协作的环境下。通过建立清晰的分支策略、命名规范和工作流程，可以提高开发效率，减少错误，确保项目的长期稳定演进。

本规范将随着项目的发展不断完善，团队成员应共同遵守和维护这些规范，确保项目的顺利进行。