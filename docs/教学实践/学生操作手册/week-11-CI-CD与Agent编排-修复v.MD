# 第11周：CI/CD与Agent编排

> 实验时间：2学时
> 实验类型：设计性+验证性

---

## 一、实验目标

- [ ] 理解 CI/CD 在 Harness 中的角色（"自动化执行引擎"）
- [ ] 能够配置 Gitea Actions CI 工作流
- [ ] 理解 Agent 编排在 CI/CD 中的应用（explore/librarian/oracle 并行分析）
- [ ] 能够设计一个多 Agent 协作的 CI 流程

---

## 二、操作步骤

### 步骤1：配置 Gitea Actions CI（25分钟）

```bash
mkdir -p .gitea/workflows
```

创建 `.gitea/workflows/ci.yml`：

```yaml
name: CI
on:
  push:
    branches: [ develop/v3.0.0 ]
  pull_request:
    branches: [ develop/v3.0.0 ]

jobs:
  bp1-static:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo build --all-features
    - run: cargo fmt --check --all
    - run: cargo clippy --all-features -- -D warnings

  bp2-integration:
    needs: bp1-static
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --all-features
    - run: cargo test --test qps_benchmark_test -- --ignored --nocapture
```

#### ✅ 检查点1：保存 CI 配置文件

---

### 步骤2：设计 Agent 编排流程（25分钟）

```
传统 CI：串行执行，一步等一步
Agent CI：并行触发多个分析 Agent

PR 创建
  ├── explore Agent  → "找到 DELETE 相关的所有调用点"
  ├── librarian Agent → "查外部文档，验证 API 用法"
  └── oracle Agent   → "评估这个改动对性能的风险"

收集结果 → 合成报告 → 通知开发者
```

**你的任务**：为 DELETE 优化 PR 设计一个 3-Agent 的 CI 分析流程。

| Agent | 分析任务 | 预期发现 |
|-------|---------|---------|
| explore | | |
| librarian | | |
| oracle | | |

#### ✅ 检查点2：保存 Agent 编排设计

---

### 步骤3：模拟 Gate 联动（10分钟）

```
完整的 Gate 联动流程：
  push 代码 → Gitea webhook → Nomad 调度 Job
  → BP1 静态检查 → BP2 集成检查 → BP3 风险检查
  → 结果上报 PR 评论 → 开发者收到通知
```

#### ✅ 检查点3：画出 Gate 联动流程图

---

## 三、评分标准

| 检查项 | 分值 |
|--------|------|
| CI 配置文件 | 30分 |
| Agent 编排设计 | 40分 |
| Gate 联动流程 | 30分 |

---

*最后更新: 2026-05-09*
