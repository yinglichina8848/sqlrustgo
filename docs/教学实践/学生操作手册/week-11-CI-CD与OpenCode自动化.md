# 第11周：CI/CD与OpenCode自动化

> 实验时间：2学时（🕹️ 手动档第4周）
> 实验类型：设计性+验证性

---

## 一、实验目标

- [ ] 理解CI/CD原理
- [ ] 能够配置GitHub Actions
- [ ] 能够配置测试自动化和代码检查
- [ ] 理解多AI协作模式

---

## 二、🕹️ 手动档深入：使用OpenCode配置自动化

**本周重点：使用AI工具配置自动化流程**

| 任务 | AI辅助方式 | 我的角色 |
|------|-----------|---------|
| 配置CI工作流 | 让AI生成YAML模板 | 理解并修改配置 |
| 添加测试检查 | 让AI解释为什么需要 | 决策检查项 |
| 配置覆盖率 | 让AI推荐工具 | 验证配置正确性 |

---

## 三、实验环境

| 项目 | 要求 |
|------|------|
| GitHub账号 | 需要仓库Push权限 |
| 项目代码 | SQLRustGo |

---

## 四、操作步骤

### 步骤1：创建CI工作流文件（30分钟）

#### 1.1 创建工作流目录

```bash
mkdir -p .github/workflows
```

#### 1.2 创建CI配置文件

创建 `.github/workflows/ci.yml`：

```yaml
name: CI

on:
  push:
    branches: [ develop/v2.6.0 ]
  pull_request:
    branches: [ develop/v2.6.0 ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Build
      run: cargo build --verbose --all-features

    - name: Run tests
      run: cargo test --all-features

    - name: Clippy
      run: cargo clippy --all-features -- -D warnings

    - name: Format check
      run: cargo fmt --check --all
```

#### 1.3 使用AI辅助生成配置

```
提示词示例：
"帮我生成一个Rust项目的GitHub Actions CI配置，包含：
1. cargo build
2. cargo test
3. cargo clippy
4. cargo fmt
要求使用最新版本的action"
```

#### ✅ 检查点1：保存CI配置文件

---

### 步骤2：配置覆盖率报告（20分钟）

#### 2.1 修改CI配置添加覆盖率

在 `.github/workflows/ci.yml` 中添加：

```yaml
    - name: Coverage
      uses: actions-rust-lang/cargo-tarpaulin-action@v1
      with:
        args: '--out Xml --packages parser,executor,storage'
```

#### 2.2 配置覆盖率阈值

```yaml
    - name: Check Coverage
      run: |
        COVERAGE=$(cat coverage.xml | grep -o 'line-rate="[0-9.]*"' | cut -d'"' -f2)
        if (( $(echo "$COVERAGE < 0.7" | bc -l) )); then
          echo "Coverage $COVERAGE is below 70%"
          exit 1
        fi
```

#### ✅ 检查点2：记录覆盖率配置

---

### 步骤3：触发CI并验证（20分钟）

#### 3.1 推送代码触发CI

```bash
git add .
git commit -m "ci: add GitHub Actions workflow"
git push origin feature/add-uppercase-function
```

#### 3.2 在GitHub上查看CI状态

1. 访问仓库 Actions 页面
2. 查看构建状态
3. 查看测试结果
4. 查看覆盖率报告

#### 3.3 验证所有检查通过

| 检查项 | 状态 |
|--------|------|
| cargo build | ⬜ |
| cargo test | ⬜ |
| cargo clippy | ⬜ |
| cargo fmt | ⬜ |
| coverage ≥70% | ⬜ |

#### ✅ 检查点3：截图CI运行结果

---

### 步骤4：设计多AI协作流程（20分钟）

#### 4.1 理解AI角色定义

根据PPT中的AI角色定义：

| 角色 | 工具 | 任务 |
|------|------|------|
| Analyst | Claude | 分析需求、编写需求文档 |
| Architect | GPT-4 | 设计架构、选择实现方案 |
| Developer | Claude Code | 编写代码、编写单元测试 |
| Tester | Claude | 编写集成测试、执行性能测试 |
| Reviewer | Claude | 代码审查、质量检查 |

#### 4.2 设计JOIN功能开发的AI协作流程

创建 `docs/design/multi-ai-workflow.md`：

```yaml
workflow:
  name: JOIN功能开发

  phases:
    - phase: 需求分析
      roles:
        analyst:
          tool: Claude
          task: 分析JOIN需求、编写需求文档
      parallel: false

    - phase: 架构设计
      roles:
        architect:
          tool: GPT-4
          task: 设计JOIN架构、选择实现方案
      parallel: false
      depends_on: 需求分析

    - phase: 代码实现
      roles:
        developer:
          tool: Claude Code
          task: 编写JOIN代码、编写单元测试
      parallel: false
      depends_on: 架构设计

    - phase: 测试验证
      roles:
        tester:
          tool: Claude
          task: 编写集成测试、执行性能测试
      parallel: false
      depends_on: 代码实现

    - phase: 代码审查
      roles:
        reviewer:
          tool: Claude
          task: 代码审查、质量检查
      parallel: false
      depends_on: 测试验证
```

#### ✅ 检查点4：保存多AI协作流程设计

---

## 五、实验报告

### 5.1 报告内容

1. **CI工作流配置文件**
2. **CI运行结果截图**
3. **多AI协作流程设计文档**
4. **遇到的问题和解决方法**

### 5.2 提交方式

```bash
git checkout -b experiment/week-11-你的学号
mkdir -p reports/week-11
# 放入报告
git add reports/week-11/
git commit -m "experiment: submit week-11 report"
git push origin experiment/week-11-你的学号
```

---

## 六、思考与反思

### 🎯 本周能力阶段：Harness Engineering（规则约束工程）

**本周重点**：设计自动化规则，让AI在约束范围内工作

### 问题1：优化与改进
- 我设计的CI流程，哪些是"必须检查"的？哪些是"最好检查"的？
- 如何用代码/配置来表达这些规则，而不是依赖人工判断？

### 问题2：AI不可替代的能力
- AI能理解"为什么这个检查是必要的"这个业务逻辑吗？
- 人类如何决定哪些规则要严格？哪些可以灵活？

### 问题3：自我提升
- 我是否开始尝试用"规则"而不是"对话"来约束AI的行为？
- 我的规则足够清晰吗？AI能理解吗？

---

## 七、评分标准

| 检查项 | 分值 |
|--------|------|
| CI工作流配置正确 | 25分 |
| CI能够成功运行 | 25分 |
| 覆盖率配置正确 | 15分 |
| 多AI协作流程设计 | 20分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-04-19*
*🕹️ 手动档第4周 - CI/CD自动化*
