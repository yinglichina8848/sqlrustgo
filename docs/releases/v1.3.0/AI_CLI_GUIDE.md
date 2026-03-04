# v1.3.0 AI-CLI 协作指南

> **版本**: v1.3.0
> **代号**: Observability Engine
> **制定日期**: 2026-03-05

---

## 一、版本概述

v1.3.0 版本是一个重要的里程碑版本，专注于可观测性系统建设，同时完善插件系统和 CBO。

### 1.1 版本目标

- **版本号**: v1.3.0
- **目标成熟度**: L4 企业级
- **核心目标**: 商用级内核，完整 CBO，可观测性
- **预计时间**: v1.2.0 GA 后 2 个月

### 1.2 开发分支

- **开发分支**: `develop-v1.3.0`
- **发布分支**: `release/v1.3.0-*`

---

## 二、分支管理策略

### 2.1 分支结构

```
main                        → 生产分支
baseline                    → 基线分支
develop-v1.3.0             → v1.3.0 开发分支 (当前)
release/v1.3.0-draft       → Draft 发布分支
release/v1.3.0-alpha       → Alpha 发布分支
release/v1.3.0-beta       → Beta 发布分支
release/v1.3.0-rc         → RC 发布分支
release/v1.3.0 (GA)       → GA 发布分支
fix/v1.3.0-*              → 功能修复分支
```

### 2.2 分支保护

| 分支 | 保护级别 | 审核要求 | 说明 |
|------|----------|----------|------|
| `main` | 🔒 冻结 | 2 人审核 | 生产分支 |
| `baseline` | 🔒 冻结 | 2 人审核 | 基线分支 |
| `develop-v1.3.0` | 🔒 开发 | 1 人审核 | 开发分支 |
| `release/v1.3.0-*` | 🔒 发布 | 1-2 人审核 | 发布分支 |

---

## 三、AI-CLI 工作流程

### 3.1 任务领取

1. 在 Issue 中评论声明领取任务
2. 说明预计完成时间

**示例**:
```
我领取任务: H-002 /health/live 端点实现
预计完成时间: 2 小时
```

### 3.2 分支创建

```bash
# 1. 确保在开发分支
git checkout develop-v1.3.0
git pull origin develop-v1.3.0

# 2. 创建功能分支
git checkout -b fix/v1.3.0-[任务编号]-[简述]

# 示例
git checkout -b fix/v1.3.0-health-live
git checkout -b fix/v1.3.0-metrics-buffer-pool
```

### 3.3 开发规范

#### 编码规范

```bash
# 编译检查
cargo build --release

# 测试检查
cargo test --all

# Clippy 检查
cargo clippy --all-targets -- -D warnings

# 格式检查
cargo fmt --all -- --check

# 文档检查
cargo doc --no-deps
```

#### 提交规范

```bash
# 添加变更
git add -A

# 提交信息格式
git commit -m "feat(v1.3.0): [模块] [描述]

- [变更1]
- [变更2]

Closes #[Issue编号]"
```

**类型前缀**:
- `feat`: 新功能
- `fix`: Bug 修复
- `refactor`: 重构
- `docs`: 文档
- `test`: 测试
- `chore`: 构建/工具

### 3.4 PR 创建

```bash
# 推送分支
git push origin fix/v1.3.0-[分支名]

# 创建 PR
gh pr create \
  --base develop-v1.3.0 \
  --head fix/v1.3.0-[分支名] \
  --title "[类型](v1.3.0): [任务描述]" \
  --body "## Summary

完成 [任务编号] [任务名称]

## Changes

- [变更1]
- [变更2]

## Testing

- [测试结果]

## Checklist

- [x] 编译通过
- [x] 测试通过
- [x] Clippy 零警告
- [x] 格式检查通过

Closes #[Issue编号]"
```

### 3.5 审核要求

| 分支 | 审核要求 |
|------|----------|
| `develop-v1.3.0` | 1 人审核 |
| `release/v1.3.0-draft` | 1 人审核 |
| `release/v1.3.0-alpha` | 1 人审核 |
| `release/v1.3.0-beta` | 1 人审核 |
| `release/v1.3.0-rc` | 2 人审核 |

---

## 四、任务领取指南

### 4.1 优先任务 (P0)

#### 性能监控 (M)

| ID | 任务 | 负责人 | 工时 |
|----|------|--------|------|
| M-001 | Metrics trait 定义 | openheart | 4h |
| M-002 | BufferPoolMetrics 实现 | heartopen | 4h |
| M-003 | ExecutorMetrics 实现 | heartopen | 4h |
| M-004 | NetworkMetrics 实现 | heartopen | 4h |
| M-005 | 指标聚合器 | openheart | 4h |

#### 健康检查 (H)

| ID | 任务 | 负责人 | 工时 |
|----|------|--------|------|
| H-001 | HealthChecker 实现 | heartopen | 4h |
| H-002 | /health/live 端点 | heartopen | 2h |
| H-003 | /health/ready 端点 | heartopen | 2h |
| H-004 | /health 综合端点 | heartopen | 4h |
| H-005 | 组件健康检查器 | heartopen | 4h |

#### 插件系统 (P)

| ID | 任务 | 负责人 | 工时 |
|----|------|--------|------|
| P-001 | Plugin trait 定义 | openheart | 4h |
| P-002 | 插件加载器实现 | openheart | 6h |

### 4.2 领取示例

```
我领取任务: H-002 /health/live 端点实现
预计完成时间: 2 小时
任务描述: 实现存活探针端点，返回 {"status": "alive", "version": "1.3.0"}
```

---

## 五、验收标准

### 5.1 代码质量

- [ ] 编译通过 (`cargo build --release`)
- [ ] 测试通过 (`cargo test --all`)
- [ ] Clippy 零警告 (`cargo clippy --all-targets -- -D warnings`)
- [ ] 格式检查通过 (`cargo fmt --all -- --check`)
- [ ] 文档生成无警告 (`cargo doc --no-deps`)

### 5.2 功能验收

- [ ] 健康检查端点功能正确
- [ ] 指标采集正确
- [ ] Prometheus 格式正确

### 5.3 性能要求

- /health/live 延迟 < 10ms
- /health/ready 延迟 < 100ms
- /metrics 延迟 < 50ms

---

## 六、常见问题

### Q1: 如何开始?

1. 阅读 TASK_MATRIX.md 了解任务详情
2. 选择一个待认领的任务
3. 在 Issue 中声明领取

### Q2: 遇到问题怎么办?

1. 先尝试自己解决
2. 在 Issue 中提问
3. 寻求 maintainer 帮助

### Q3: 如何确保代码质量?

1. 遵循编码规范
2. 编写必要的测试
3. 运行所有检查命令
4. 提交前自检

---

## 七、联系与支持

| 角色 | 职责 |
|------|------|
| **openheart** | 架构设计, 核心功能开发 |
| **heartopen** | 功能实现, 端点开发 |
| **maintainer** | 代码审核, 文档审查 |
| **yinglichina8848** | 版本管理, 发布控制 |

---

## 八、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
- [TASK_MATRIX.md](./TASK_MATRIX.md)

---

*本文档由 AI 助手生成*
