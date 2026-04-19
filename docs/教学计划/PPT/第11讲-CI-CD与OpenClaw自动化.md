---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十一讲：CI/CD与多AI Agent强制约束

## AI增强的软件工程

---

# 课程大纲

1. **CI/CD：多AI协作的"法律"**（25分钟）
2. **GitHub Actions实践**（20分钟）
3. **OpenClaw多AI协作**（20分钟）
4. **实践练习**（15分钟）

---

# Part 1: CI/CD：多AI协作的"法律"

---

## 1.1 为什么多AI需要CI/CD？

### 没有CI/CD的多AI协作

```
┌─────────────────────────────────────────────────────────────────────┐
│                    没有CI/CD = 失控                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  AI #1 提交代码：                                                   │
│  ├── 编译错误？ → 不知道                                           │
│  ├── 测试失败？ → 不知道                                           │
│  ├── 风格混乱？ → 不知道                                           │
│  └── 合并后能工作？ → 不知道                                      │
│                                                                      │
│  AI #2, #3, #4... 同样不知道                                      │
│                                                                      │
│  结果：代码库逐渐腐烂，最终无法维护                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 有CI/CD的多AI协作

```
┌─────────────────────────────────────────────────────────────────────┐
│                    有CI/CD = 秩序                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  AI #1 提交代码：                                                   │
│  ├── 编译错误？ → ❌ CI阻止合并                                    │
│  ├── 测试失败？ → ❌ CI阻止合并                                    │
│  ├── 风格混乱？ → ❌ CI阻止合并                                    │
│  └── 质量不达标？ → ❌ CI阻止合并                                  │
│                                                                      │
│  只有所有检查通过，才能合并                                         │
│                                                                      │
│  结果：代码库始终保持健康                                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 1.2 CI/CD的核心作用

### 作为"法律"强制执行

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CI/CD = 自动法律                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  人类制定规则（法律）                                                │
│       ↓                                                             │
│  CI/CD 自动执行（警察）                                            │
│       ↓                                                             │
│  违规代码被阻止（法庭）                                             │
│       ↓                                                             │
│  开发者/AI 修改（改造）                                            │
│                                                                      │
│  关键点：                                                           │
│  • 规则一旦设定，不可绕过                                         │
│  • 所有人都必须遵守（人类和AI同等）                                │
│  • 违规必有后果                                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### CI/CD检查清单

| 检查项 | 作用 | 违规后果 |
|--------|------|---------|
| cargo build | 编译通过 | ❌ 阻止合并 |
| cargo test | 功能正确 | ❌ 阻止合并 |
| cargo clippy | 无警告 | ❌ 阻止合并 |
| cargo fmt | 格式统一 | ❌ 阻止合并 |
| coverage | 测试覆盖 | ⚠️ 警告 |
| cargo audit | 依赖安全 | ❌ 阻止合并 |

---

## 1.3 多AI时代的CI/CD

### 多AI协作的CI/CD流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                    多AI协作的CI/CD流程                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  AI #1 提交PR ─┐                                                    │
│                 │                                                    │
│  AI #2 提交PR ─┼──►  CI统一检查  ──►  人类审查  ──►  合并        │
│                 │         │                │                         │
│  AI #3 提交PR ─┘      ▼                 ▼                         │
│                   检查项：                                         │
│                   • 编译 ✓                                         │
│                   • 测试 ✓                                         │
│                   • 格式 ✓                                         │
│                   • 风格 ✓                                         │
│                   • 冲突检测 ✓                                     │
│                                                                      │
│  关键点：                                                           │
│  • 每个AI的代码都必须通过CI才能合并                                │
│  • 合并后所有AI的代码一起测试                                      │
│  • 任何AI的问题都会被立即发现                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part 2: GitHub Actions实践

---

## 2.1 GitHub Actions配置

### 基本工作流

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop/v2.6.0 ]
  pull_request:
    branches: [ main, develop/v2.6.0 ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Rust
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

---

## 2.2 多AI协作的CI配置

### 针对多AI的CI策略

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
    branches: [ develop/v2.6.0 ]

# 确保所有AI的代码都不冲突
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  # 1. 基础检查（所有PR必须通过）
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
      - name: Format
        run: cargo fmt --check --all
      - name: Security audit
        run: cargo audit

  # 2. 测试（所有PR必须通过）
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --all-features
      - name: Test
        run: cargo test --all-features
      - name: Coverage
        uses: actions-rust-lang/cargo-tarpaulin-action@v1
        with:
          args: '--out Xml --packages parser,executor,storage'

  # 3. 合并检查（只有管理员可以合并）
  merge-check:
    needs: [lint, test]
    runs-on: ubuntu-latest
    steps:
      - name: Check merge eligibility
        run: |
          echo "All checks passed. Ready to merge."
```

---

## 2.3 质量门禁配置

### 必需的质量门禁

```yaml
# 必需门禁（不通过不能合并）
gates_required:
  - cargo build        # 编译
  - cargo test         # 测试
  - cargo clippy       # 代码质量
  - cargo fmt          # 格式

# 建议门禁（不通过警告但不阻止）
gates_suggested:
  - coverage ≥ 70%    # 覆盖率
  - cargo audit       # 安全
  - doc check         # 文档
```

---

# Part 3: OpenClaw多AI协作

---

## 3.1 OpenClaw作为AI编排工具

### OpenClaw的核心功能

| 功能 | 说明 | 多AI协作中的作用 |
|------|------|-----------------|
| 代码生成 | AI编写代码 | 让多个AI同时工作 |
| 会话管理 | 保存对话历史 | 追踪每个AI的工作 |
| 工具调用 | 执行命令 | 运行测试、构建 |
| 多Agent | 多个AI协作 | 分工协调 |

---

## 3.2 使用OpenClaw进行多AI协作

### 场景：同时开发三个模块

```bash
# 1. 任务分解
# 告诉OpenClaw要做什么

任务：同时开发SQLRustGo的三个模块
- AI #1: 负责Parser模块 - 添加JOIN语法支持
- AI #2: 负责Executor模块 - 实现Hash Join
- AI #3: 负责Storage模块 - 优化数据读取

约束：
1. 每个模块独立开发
2. 通过固定接口通信
3. 必须通过所有CI检查
```

### OpenClaw协作配置

```yaml
# openclaw-config.yaml
agents:
  - name: ai-parser
    role: developer
    module: parser
    tasks:
      - add JOIN syntax
      - add JOIN AST nodes
    
  - name: ai-executor
    role: developer
    module: executor
    tasks:
      - implement Hash Join
      - add JoinOperator
    
  - name: ai-storage
    role: developer
    module: storage
    tasks:
      - optimize read path
      - add join data source

coordination:
  - name: sync-point
    trigger: after each task
    action: run CI checks
    
  - name: conflict-check
    trigger: before commit
    action: check file conflicts
```

---

## 3.3 AI Agent的工作流程

### 典型工作流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AI Agent的工作流程                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. 接收任务                                                        │
│     "请实现Hash Join执行器"                                         │
│                                                                      │
│  2. 分析上下文                                                       │
│     ├── 查看现有代码结构                                             │
│     ├── 理解接口约定                                                │
│     └── 了解依赖模块                                                │
│                                                                      │
│  3. 编写代码                                                        │
│     ├── 实现功能代码                                                 │
│     └── 编写测试用例                                                │
│                                                                      │
│  4. 本地验证                                                         │
│     ├── cargo build                                                 │
│     ├── cargo test                                                  │
│     └── cargo clippy                                                │
│                                                                      │
│  5. 提交PR                                                          │
│     ├── 创建分支                                                     │
│     ├── 提交代码                                                    │
│     └── 创建PR + CI自动检查                                         │
│                                                                      │
│  6. 响应审查                                                        │
│     ├── 响应人类审查意见                                            │
│     └── 修改代码（如需要）                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part 4: 实践练习

---

## 4.1 配置GitHub Actions

```bash
# 1. 创建工作流目录
mkdir -p .github/workflows

# 2. 创建CI配置文件
cat > .github/workflows/ci.yml << 'EOF'
name: CI

on:
  push:
    branches: [ develop/v2.6.0 ]
  pull_request:
    branches: [ develop/v2.6.0 ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
EOF
```

---

## 4.2 验证CI运行

```bash
# 推送触发CI
git add .
git commit -m "ci: add GitHub Actions workflow"
git push origin develop/v2.6.0

# 查看GitHub Actions
# https://github.com/your-repo/actions
```

---

# 核心知识点总结

---

## 1. CI/CD的作用

- **What**：自动化构建、测试、部署
- **Why**：强制执行规则、保证代码质量
- **How**：每次提交/合并自动运行检查

## 2. 多AI协作

- **分工**：每个AI负责不同模块
- **隔离**：通过分支隔离工作
- **协调**：通过PR和CI协调

## 3. OpenClaw

- **任务分配**：明确每个AI的工作
- **自动化**：自动运行CI检查
- **协作**：追踪整体进度

---

# 课后作业

---

## 任务

1. 配置一个完整的GitHub Actions CI流程
2. 设计一个多AI协作的任务分配方案
3. 实际运行一次CI，验证配置正确

## 预习

- 性能优化与重构
- 多AI协作的最佳实践

---

<!-- _class: lead -->

# 谢谢！

## 下节课：性能优化与重构
