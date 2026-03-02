---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十一讲：CI/CD与OpenClaw自动化

## AI增强的软件工程

---

# 课程大纲

1. **CI/CD概述**（25分钟）
2. **GitHub Actions实践**（25分钟）
3. **OpenClaw多AI协作**（25分钟）
4. **实践练习**（15分钟）

---

# Part 1: CI/CD概述

---

## 1.1 What：什么是CI/CD

### CI（持续集成）

- 频繁集成代码到主分支
- 自动化构建和测试
- 快速发现集成问题

### CD（持续部署/交付）

- 自动化部署流程
- 快速交付价值
- 减少手动操作

---

## 1.2 Why：为什么需要CI/CD

### 提高质量

- 自动化测试保证质量
- 快速发现问题
- 减少人为错误

### 提高效率

- 自动化重复工作
- 加快交付速度
- 减少手动操作

### 降低风险

- 小批量频繁发布
- 快速回滚能力
- 减少发布风险

---

## 1.3 How：CI/CD流程

```
代码提交 → 自动构建 → 自动测试 → 代码审查 → 自动部署
    ↑                                              │
    └──────────────────────────────────────────────┘
```

### CI流程

1. 代码提交
2. 自动构建
3. 自动测试
4. 代码审查

### CD流程

1. 自动打包
2. 自动部署
3. 自动监控

---

# Part 2: GitHub Actions实践

---

## 2.1 GitHub Actions配置

### 工作流文件

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Build
      run: cargo build --verbose
      
    - name: Run tests
      run: cargo test --verbose
```

---

## 2.2 质量门禁配置

### Clippy检查

```yaml
- name: Run Clippy
  uses: actions-rs/clippy-check@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
    args: --all-features
```

### 格式化检查

```yaml
- name: Check formatting
  run: cargo fmt -- --check
```

### 测试覆盖率

```yaml
- name: Generate coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml
    
- name: Upload coverage
  uses: codecov/codecov-action@v3
```

---

## 2.3 自动化发布

### 发布工作流

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Build release
      run: cargo build --release
      
    - name: Create release
      uses: softprops/action-gh-release@v1
      with:
        files: target/release/sqlrustgo
```

---

# Part 3: OpenClaw多AI协作

---

## 3.1 What：什么是OpenClaw

### 定义

多AI协作的自动化工作流平台

### 核心功能

- **AI角色分配**：不同AI承担不同角色
- **自动化工作流**：AI自动执行任务
- **协作机制**：多个AI协同工作

---

## 3.2 Why：为什么需要多AI协作

### 提高效率

- 并行处理任务
- 专业分工
- 24/7工作

### 提高质量

- 多AI审核
- 交叉验证
- 减少遗漏

### 降低成本

- 减少人工干预
- 自动化重复工作

---

## 3.3 How：AI角色分配

### AI角色定义

| 角色 | 职责 | 工具 |
|------|------|------|
| 分析师 | 需求分析、设计评审 | Claude |
| 架构师 | 架构设计、技术选型 | GPT-4 |
| 开发者 | 代码编写、单元测试 | Claude Code |
| 测试员 | 测试用例、集成测试 | Claude |
| 文档员 | 文档编写、翻译 | GPT-4 |
| 审查员 | 代码审查、质量检查 | Claude |

---

## 3.3 How：AI角色分配（续）

### 协作流程

```
分析师 → 架构师 → 开发者 → 测试员 → 审查员
   │                            │
   └────────── 文档员 ──────────┘
```

### 任务分配示例

```yaml
tasks:
  - name: 实现JOIN功能
    analyst:
      - 分析JOIN需求
      - 设计JOIN接口
    architect:
      - 设计JOIN架构
      - 选择实现方案
    developer:
      - 编写JOIN代码
      - 编写单元测试
    tester:
      - 编写集成测试
      - 执行性能测试
    reviewer:
      - 代码审查
      - 质量检查
```

---

# Part 4: 实践练习

---

## 4.1 配置GitHub Actions

### 任务

1. 创建`.github/workflows/ci.yml`文件
2. 配置构建、测试、Clippy检查
3. 提交代码，触发CI

### 验证

- 查看Actions页面
- 确认所有检查通过

---

## 4.2 设计AI协作流程

### 任务

1. 定义AI角色
2. 设计协作流程
3. 分配任务

### 示例

```yaml
workflow:
  name: 实现新功能
  roles:
    - analyst: Claude
    - developer: Claude Code
    - reviewer: Claude
  steps:
    - analyst: 分析需求
    - developer: 编写代码
    - reviewer: 审查代码
```

---

# 核心知识点总结

---

## 1. CI/CD

- **What**：持续集成、持续部署
- **Why**：提高质量、提高效率、降低风险
- **How**：自动化构建、测试、部署

## 2. GitHub Actions

- **工作流配置**
- **质量门禁**
- **自动化发布**

## 3. OpenClaw多AI协作

- **AI角色分配**
- **协作流程**
- **任务分配**

---

# 课后作业

---

## 任务

1. 配置GitHub Actions CI流程
2. 设计一个AI协作流程
3. 执行一次完整的CI/CD流程

## 预习

- 性能优化与重构
- 代码质量提升

---

<!-- _class: lead -->

# 谢谢！

## 下节课：性能优化与重构
