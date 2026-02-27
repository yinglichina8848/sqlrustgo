# 企业级 GitHub 权限模型实施计划

## 文档信息
- **版本**: v1.0
- **创建日期**: 2026-02-21
- **状态**: 执行计划
- **适用范围**: SQLRustGo 项目

## 目标

根据企业级 GitHub 权限模型配置清单，完成以下实施步骤，达到真正的不可变发布状态：

1. **开启 Tag 保护**
2. **锁死 main 分支**
3. **CI 增加 tag 校验**
4. **测试模拟发布**

## 实施步骤

### 1. 开启 Tag 保护

**操作位置**: GitHub → Settings → Rules → Tag protection

**配置要求**:
- **规则模式**: `v*`
- **禁止删除**: ✅ 启用
- **禁止重建**: ✅ 启用
- **Include administrators**: ✅ 启用
- **限制创建**: 仅 CI 可创建 Tag（如支持）

**验证步骤**:
- 尝试删除现有 Tag: `git tag -d v1.0.0`
- 尝试推送删除: `git push origin --delete v1.0.0`
- 验证操作被拒绝

### 2. 锁死 main 分支

**操作位置**: GitHub → Settings → Branch protection rules → main

**配置要求**:
- ✅ Require pull request
- ✅ Require approvals（至少 1）
- ✅ Require status checks
- ✅ Require conversation resolution
- ✅ Require signed commits
- ✅ Require linear history
- ❌ Allow force push（必须关闭）
- ❌ Allow deletions（必须关闭）
- ✅ Include administrators（关键）
- ✅ Restrict who can push（只允许 CI Bot）

**验证步骤**:
- 尝试直接 push: `git push origin main`
- 尝试 force push: `git push origin main --force`
- 验证操作被拒绝

### 3. 配置 release/* 分支保护

**操作位置**: GitHub → Settings → Branch protection rules → Add rule

**配置要求**:
- **规则模式**: `release/*`
- ✅ Require pull request
- ✅ Require approvals（至少 1）
- ✅ Require status checks
- ✅ Require conversation resolution
- ✅ Require linear history
- ❌ Allow force push
- ❌ Allow deletions
- ✅ Include administrators

**验证步骤**:
- 尝试直接 push 到 release 分支
- 验证操作被拒绝

### 4. CI 增加 tag 校验

**操作位置**: `.github/workflows/release-validation.yml`

**配置要求**:
- 当创建 Tag 时触发
- 校验版本号一致性
- 校验 CHANGELOG 包含版本
- 校验发布证据存在
- 校验签名（如启用）

**验证步骤**:
- 创建测试 Tag
- 验证 CI 执行校验
- 验证失败场景

### 5. 测试模拟发布

**操作位置**: 本地测试

**测试步骤**:
1. 在 develop 分支创建测试变更
2. 创建 PR 到 rc 分支
3. 验证 CI 执行
4. 创建 PR 到 release 分支
5. 验证 CI 执行
6. 测试 Tag 创建流程
7. 验证 Release 创建流程
8. 验证 main 分支更新流程

**验证要点**:
- 所有步骤必须通过 PR
- 所有步骤必须通过 CI
- 无直接 push 操作
- 无绕过保护的操作

## 权限模型配置清单

### Repository 级别权限

| 角色 | 权限 | 说明 |
|------|------|------|
| Owner | 管理仓库设置 | 不参与日常合并 |
| Maintainer | Merge PR | 不能绕过保护 |
| Developer | 提交 PR | 无 push 权限 |
| CI Bot | 写入 Release | 仅 workflow 使用 |

### 分支保护矩阵

| 分支类型 | PR 要求 | 审批要求 | CI 要求 | 签名要求 | 线性历史 | Force Push | 删除 |
|---------|---------|---------|---------|---------|---------|-----------|------|
| main | ✅ | ✅ (1+) | ✅ | ✅ | ✅ | ❌ | ❌ |
| release/* | ✅ | ✅ (1+) | ✅ | ✅ | ✅ | ❌ | ❌ |
| rc/* | ✅ | ⚠ (可选) | ✅ | ⚠ (可选) | ⚠ (可选) | ❌ | ❌ |
| develop | ✅ | ⚠ (可选) | ✅ | ❌ | ❌ | ❌ | ❌ |

### CI 权限最小化

| Workflow | 权限 | 说明 |
|---------|------|------|
| 测试 | read | 只读权限 |
| 构建 | read | 只读权限 |
| 发布 | write | 仅发布操作 |
| Tag 创建 | write | 仅 Tag 操作 |

## 不可变发布架构图

```
                ┌────────────┐
                │  develop   │
                └─────┬──────┘
                      │ PR
                      ▼
                ┌────────────┐
                │  rc/vX.X   │
                └─────┬──────┘
                CI完整门禁
                      │ PR
                      ▼
                ┌────────────┐
                │ release/vX │
                └─────┬──────┘
                Freeze + 保护
                      │
               CI 自动创建 Tag
                      ▼
                ┌────────────┐
                │   vX.X.X   │  ← 不可变
                └─────┬──────┘
                      │
                自动创建 Release
                      │
                      ▼
                ┌────────────┐
                │   main     │
                └────────────┘
```

## 不可变发布的 6 个条件

1. ✅ Tag 不能删除
2. ✅ Tag 不能重建
3. ✅ main 不能直接 push
4. ✅ release 不能 force push
5. ✅ 发布必须 CI 触发
6. ✅ 构建产物带 Hash

## 成熟度评估

### 当前状态
- **结构模型**: ✅ 完成
- **release 冻结**: ✅ 完成
- **main 对齐**: ✅ 完成
- **快照封存**: ✅ 完成
- **Tag 保护**: ❌ 待实施
- **CI 强校验**: ❌ 待实施

### 目标状态
- **Tag 保护**: ✅ 完成
- **CI 强校验**: ✅ 完成
- **权限模型**: ✅ 完成
- **不可变发布**: ✅ 完成

### 成熟度提升
- **当前**: 75%
- **目标**: 100%

## 实施时间表

| 步骤 | 时间 | 负责人 | 状态 |
|------|------|--------|------|
| 开启 Tag 保护 | 2026-02-21 | @yinglichina8848 | ⏳ |
| 锁死 main 分支 | 2026-02-21 | @yinglichina8848 | ⏳ |
| 配置 release/* 保护 | 2026-02-21 | @yinglichina8848 | ⏳ |
| CI 增加 tag 校验 | 2026-02-22 | @yinglichina8848 | ⏳ |
| 测试模拟发布 | 2026-02-22 | @yinglichina8848 | ⏳ |
| 验证不可变状态 | 2026-02-23 | @yinglichina8848 | ⏳ |

## 风险评估

### 潜在风险
1. **配置错误**: 可能导致合法操作被拒绝
2. **权限过严**: 可能影响紧急修复流程
3. **CI 失败**: 可能阻止正常发布
4. **学习曲线**: 团队需要适应新流程

### 缓解措施
1. **详细测试**: 在非生产环境测试配置
2. **紧急流程**: 建立紧急修复流程
3. **CI 监控**: 实时监控 CI 状态
4. **培训文档**: 提供详细的流程文档

## 成功标准

### 技术标准
- ✅ 所有保护规则正确配置
- ✅ 所有验证测试通过
- ✅ 发布流程自动化
- ✅ 不可变状态验证通过

### 流程标准
- ✅ 所有人必须走 PR
- ✅ CI 是唯一放行通道
- ✅ 无直接 push 操作
- ✅ 无绕过保护的操作

### 审计标准
- ✅ 完整的操作审计日志
- ✅ 不可篡改的发布证据
- ✅ 可验证的构建产物
- ✅ 清晰的版本历史

## 后续行动

### 完成实施后
1. 更新发布政策文档
2. 完善发布流程文档
3. 培训团队成员
4. 开始 v1.1 开发

### 长期改进
1. 引入签名 Tag
2. 实施 SBOM
3. 供应链安全扫描
4. 自动化发布流程

---

**实施负责人**: @yinglichina8848
**审核负责人**: @sonaheartopen
**实施日期**: 2026-02-21
**目标完成日期**: 2026-02-23