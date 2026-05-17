---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十一讲：CI/CD与Nomad自动化

## Harness工程：自托管Runner与Agent编排

---

# 课程大纲

1. **CI/CD架构演进**（25分钟）
2. **Nomad Runner部署原理**（25分钟）
3. **Gitea Actions与Gate联动**（25分钟）
4. **Agent编排：CI/CD的新视角**（25分钟）
5. **实践练习**（10分钟）

---

# Part 1: CI/CD架构演进

---

## 1.1 What：CI/CD是什么

### 一个简单类比

```
没有 CI/CD 的团队（手动）：
  写代码 → 手动编译 → 手动测试 → 手动部署 → 半夜加班修 Bug

有 CI/CD 的团队（自动）：
  写代码 → push → 自动编译 → 自动测试 → 自动部署 → 该下班下班

CI（持续集成）：每次 push 自动编译 + 测试
CD（持续部署）：通过测试后自动部署
```

### CI（持续集成）
- 频繁集成代码到主分支
- 自动化构建和测试
- 快速发现集成问题

### CD（持续部署/交付）
- 自动化部署流程
- 快速交付价值
- 减少手动操作

---

## 1.2 三次架构演进

### 架构1.0 → 2.0 → 3.0

```
架构1.0（共享 Runner）：
  用 GitHub 的公共机器 → 排队等 → 缺乏控制
  问题：慢、有排队、没有 root 权限

架构2.0（自托管 Runner）：
  自己买一台机器注册为 Runner → 资源独占 → 更快
  问题：仍依赖 GitHub SaaS，网络隔离场景不能用

架构3.0（Nomad + Gitea，SQLRustGo 的方案）：
  完全自托管 → Nomad 调度 → 容器隔离 → 双节点高可用
  优势：完全自主、网络隔离、成本可控

  ┌──────────┐     ┌──────────┐
  │ HP Z6G4  │     │ MacMini  │     ← 两台机器组成 Nomad 集群
  │ 40核主机  │◄───►│ 备用节点  │
  └────┬─────┘     └────┬─────┘
       │                │
       └──────┬─────────┘
              │
        Nomad Runner 容器
        (cargo build/test/gate)
```

### 架构1.0：共享Runner（共享资源）

```
                    ┌─────────────┐
                    │  GitHub     │
                    │  Actions    │
                    └──────┬──────┘
                           │ (托管)
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         [Runner A]   [Runner B]   [Runner C]
         (共享租户)    (共享租户)    (共享租户)

优点：零运维
缺点：排队等待、资源争抢、无root权限、构建时间长
```

### 架构2.0：自托管Runner（独占资源）

```
                    ┌─────────────┐
                    │  GitHub     │
                    │  Actions    │
                    └──────┬──────┘
                           │ (自托管)
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         [Runner X]   [Runner Y]   [Runner Z]
         (独占主机)    (独占主机)    (独占主机)

优点：资源独占、构建更快、可定制环境
缺点：仍依赖GitHub SaaS、网络隔离场景不适用
```

### 架构3.0：Nomad + Gitea自托管（完全自主）

```
                    ┌─────────────┐
                    │   Gitea     │
                    │ (自托管)    │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ Nomad    │ │ Nomad    │ │ 存储层    │
        │ Node 1   │ │ Node 2   │ │ (GBrain) │
        │ (HP Z6G4)│ │(250Mac)  │ │  (SSOT)  │
        └──────────┘ └──────────┘ └──────────┘

优点：完全自主、网络隔离、水平扩展、SSOT知识管理
```

---

## 1.3 为什么需要架构3.0

### AI时代的特殊需求

| 需求 | GitHub SaaS | 架构3.0 |
|------|-------------|---------|
| 网络隔离 | ❌ 不支持 | ✅ 完全内网 |
| 自定义Runner | ⚠️ 有限支持 | ✅ 完全控制 |
| 知识管理集成 | ❌ 无 | ✅ GBrain SSOT |
| 多平台同步 | ❌ 无 | ✅ GitHub/Gitea/GitCode/Gitee |
| 成本控制 | 按分钟计费 | 固定资源成本 |

### AI 时代运维的"自主可控"需求

```
GitHub SaaS（架构2.0）的限制：
  ✗ 内网隔离场景 → 代码不能出公司网络
  ✗ 长时间运行测试 → 按分钟计费，TPC-H 一跑就是半小时
  ✗ 定制化环境 → 需要特定硬件（如 ARM 测试）

架构3.0 的解决方案：
  ✅ 完全内网 → 代码不出公司
  ✅ 固定成本 → 设备是自己的
  ✅ 任意定制 → 想装什么装什么
  ✅ Agent 友好 → AI Agent 可以通过 API 触发任务
```

### SQLRustGo的实际架构

```
GitHub (镜像)
    ↑
    │  push sync
    │
Gitea (主仓库) ← Admin Token认证
    │
    ├──→ Gitea Actions (触发器)
    │         │
    │         └──→ Nomad API → Nomad Runner (容器)
    │                              │
    │                              └──→ cargo build/test/gate
    │
    ├──→ Wiki (Git同步)
    │
    └──→ GBrain SSOT (192.168.0.250:8081)
              │
              └──→ Wiki页面 → SSOT知识 → 跨节点查询
```

---

## 1.4 Nomad集群的核心概念

### Nomad是什么（一句话版）

> **Nomad 是一个"任务调度器"——你把任务交给它，它自动分配到空闲的机器上执行。**

### Nomad vs Kubernetes（初学者友好对比）

```
Nomad：
  • 单个二进制文件，装好就能用
  • 学习曲线：1-2 天
  • 适合：小团队、混合负载、教学场景

Kubernetes：
  • 多个组件，需要专业运维
  • 学习曲线：1-2 月
  • 适合：大规模集群、云原生、专业运维团队
```

### Nomad vs Kubernetes详细对比

| 维度 | Nomad | Kubernetes |
|------|-------|-----------|
| 复杂度 | 低（单二进制） | 高（多组件） |
| 学习曲线 | 平缓 | 陡峭 |
| 资源类型 | 容器/VM/裸金属 | 主要是容器 |
| 调度能力 | 强 | 强 |
| 适用场景 | 单宿主/混合负载 | 大规模云原生 |

### Nomad在SQLRustGo中的角色

```
┌────────────────────────────────────────────────────────────┐
│                     Nomad 集群                             │
│  ┌─────────────┐         ┌─────────────┐                │
│  │  Node 1     │         │  Node 2     │                │
│  │  HP Z6G4    │◄──────►│  250MacMini  │                │
│  │  40核心     │  网络   │  (待机)     │                │
│  └──────┬──────┘         └──────┬──────┘                │
│         │                         │                        │
│         │  ┌─────────────┐       │                        │
│         └──│ nomad-runner│◄──────┘  注册到集群            │
│            │  (容器)      │                                │
│            └─────────────┘                                │
│                  │                                        │
│                  └──→ 执行 CI/CD Job                       │
└────────────────────────────────────────────────────────────┘
```

---

# Part 2: Nomad Runner部署原理

---

## 2.1 Runner的注册和执行流程

### 什么是Runner注册

> **Runner是注册到Gitea Actions的"执行器"，负责实际运行CI Job。**

### 注册流程

```
1. Runner 启动 → 注册到 Nomad Server
2. Nomad Server 收到 Gitea webhook → 创建 Job
3. Nomad 调度 Job 到空闲 Node
4. Runner 容器执行 Job（cargo build/test/gate）
5. 结果上报 → PR 评论通知
6. 失败 → 自动重试（可配置次数）

关键特性：
  • 容器隔离：每个 Job 在独立容器运行，互不影响
  • 自动恢复：容器挂了 Nomad 自动重启
  • 资源限制：每个测试最多用 8GB 内存
```

### 详细注册流程

```
1. Runner启动
      │
      ▼
2. 连接Nomad Server (192.168.0.252:4646)
      │
      ▼
3. 获取任务分配
      │
      ▼
4. 执行 cargo build/test/gate
      │
      ▼
5. 上报结果到Gitea
```

### 为什么用Nomad管理Runner

| 问题 | 直接Docker | Nomad管理 |
|------|-----------|-----------|
| 资源分配 | 手动指定 | 自动调度 |
| 故障恢复 | 手动重启 | 自动重启 |
| 扩展性 | 手动扩容 | 水平扩展 |
| 监控 | 无 | 内置 |

---

## 2.2 Runner的高可用设计

### 双节点冗余

```
Gitea Server
      │
      ├──→ Nomad Node 1 (HP Z6G4) ──→ nomad-runner [UP]
      │
      └──→ Nomad Node 2 (250MacMini) ──→ nomad-runner [STANDBY]

任一节点故障，Runner自动切换到另一节点
```

### 健康检查机制

```
Runner状态检查：
    │
    ├──→ HTTP ping (nomad-runner健康端点)
    │
    ├──→ 进程存活 (ps aux | grep nomad-runner)
    │
    └──→ 任务队列 (是否有pending job)

健康检查失败 → 自动重启Runner → 日志记录
```

---

## 2.3 CI Job的调度流程

### 完整调度链路

```
开发者 push 代码
        │
        ▼
Gitea 检测到 push
        │
        ▼
触发 .gitea/workflows/ci.yml
        │
        ▼
调用 Nomad API 提交Job
        │
        ▼
Nomad Master 调度Job到 Node
        │
        ▼
Runner 执行 Job (cargo build/test)
        │
        ├──→ 通过 → 上报结果 → PR评论通知
        │
        └──→ 失败 → 上报结果 → PR blocked
```

### 为什么需要Nomad作为中间层

```
直接模式（不稳定）：
  Gitea Actions → 直接在主机上运行命令
        │
        └──→ 资源争抢、难以隔离、难以扩展

Nomad模式（稳定）：
  Gitea Actions → Nomad调度 → Runner容器 → 资源隔离
        │
        └──→ 容器隔离、自动调度、故障恢复
```

---

# Part 3: Gitea Actions与Gate联动

---

## 3.1 Gitea Actions配置结构

### Workflow文件位置

```
仓库根目录/
  └── .gitea/
        └── workflows/
              ├── ci.yml        ← 主CI workflow
              ├── gate.yml      ← Gate检查workflow
              └── beta.yml      ← Beta发布workflow
```

### Workflow触发条件

```yaml
# ci.yml 核心配置
on:
  push:
    branches:
      - develop/v3.0.0      # 主开发分支
      - beta/v3.0.0         # Beta分支
  pull_request:
    branches:
      - develop/v3.0.0
```

### 三层Trigger设计

```
develop/v3.0.0 push
    │
    ├──→ ci.yml    → BP1静态检查（编译/格式/Clippy）
    │                    ↓
    │                结果上报Gitea
    │
beta/v3.0.0 push
    │
    ├──→ gate.yml  → BP2集成检查（覆盖率/回归）
    │                    ↓
    ├──→ beta.yml  → BP3 Beta检查（KPI/功能闭环）
```

---

## 3.2 Gate检查的执行流程

### BP1/BP2/BP3 检查层级

```
PR创建/代码Push
        │
        ▼
┌───────────────────────────────────────────┐
│  BP1 静态检查 (无人值守，~2分钟)            │
│  • cargo build --all-features             │
│  • cargo fmt --check                     │
│  • cargo clippy --all-features           │
│  • 文档链接检查                           │
└───────────────────┬───────────────────────┘
                    │ Pass
                    ▼
┌───────────────────────────────────────────┐
│  BP2 集成检查 (无人值守，~10分钟)           │
│  • cargo test --all-features (≥90% pass) │
│  • cargo llvm-cov (≥75% coverage)        │
│  • check_tpch.sh (TPC-H SF=1)            │
│  • check_regression.sh (性能基线)         │
└───────────────────┬───────────────────────┘
                    │ Pass
                    ▼
┌───────────────────────────────────────────┐
│  BP3 Beta检查 (人工触发或定时，~30分钟)     │
│  • 31 Issues 闭环追踪                      │
│  • 5类 KPI 指标验证                       │
│  • BetaPhase2 功能完整性                   │
│  • E-09 Floor 豁免评估                    │
└───────────────────┬───────────────────────┘
                    │ Pass
                    ▼
              PR Merge Ready
```

### 检查结果的上报

```
检查执行
    │
    ├──→ 结果写入 Nomad Job日志
    │
    ├──→ 状态更新到 Gitea PR comment
    │      └──→ @mention 相关开发者
    │
    └──→ 关键指标同步到 GBrain SSOT
           └──→ 跨节点可查询
```

---

## 3.3 Admin Token的认证机制

### 为什么需要Admin Token

> **普通Token只能操作自己的仓库，Admin Token可以管理整个Gitea实例。**

### Token权限分级

```
普通Token (read/write repo)
    │
    ├──→ 读取代码 ✅
    ├──→ 提交PR ✅
    └──→ 管理Runner ❌

Admin Token (instance admin)
    │
    ├──→ 读取代码 ✅
    ├──→ 提交PR ✅
    ├──→ 管理Runner ✅  ← 关键差异
    ├──→ 创建/删除仓库 ✅
    └──→ 修改全局设置 ✅
```

### Admin Token的安全管理

```
存储位置： ~/.ssh/openclaw-gitea-write.PAT

使用原则：
    • 不写入代码仓库
    • 不在日志中输出
    • 定期轮换
    • 最小权限原则（仅Gitea API操作）
```

---

## 3.4 Nomad Runner的调度优势

### 为什么比直接shell更可靠

| 维度 | 直接SSH执行 | Nomad Runner |
|------|-----------|--------------|
| 环境一致性 | 依赖主机环境 | 容器镜像一致 |
| 资源限制 | 无隔离 | CPU/内存限制 |
| 故障恢复 | 手动 | 自动重启 |
| 并行执行 | 串行 | 并行调度 |
| 日志持久化 | 临时 | 持久化存储 |

### 容器化的隔离效果

```
Runner容器内：
    │
    ├──→ 文件系统隔离 (overlayfs)
    │
    ├──→ 网络隔离 (bridge)
    │
    ├──→ 资源限制 (cgroups)
    │      └──→ 8GB memory limit per test
    │
    └──→ 进程隔离 (namespace)

不影响宿主机和其他容器
```

---

# Part 4: Agent编排 — CI/CD的新视角

---

## 4.1 传统 CI vs Agent 增强 CI

### 传统 CI Pipeline（线性）

```
编译 → 测试 → 覆盖率 → 安全检查
  │      │       │         │
  └──────┴───────┴─────────┘
        串行执行，一步等一步
```

### Agent 增强 CI（并行 + 智能）

```
PR 创建
    │
    ├──→ explore Agent  "找到所有受影响的测试"    ┐
    ├──→ librarian Agent "查外部库有没有 Breaking Change" ├ 并行
    ├──→ oracle Agent  "评估这个改动的架构风险"  ┘
    │
    └──→ 收集结果 → 合成报告 → 通知开发者
```

---

## 4.2 Agent 在 CI/CD 中的典型任务

```
Agent 类型         CI/CD 中的角色
────────────────  ─────────────────────────────────
explore           搜索代码库，找到受影响的测试文件
librarian         查外部文档，验证 API 用法正确性
oracle            评估架构决策的风险等级
quick             执行简单的编译/格式检查
deep              运行长时间的集成测试（如 72 小时浸泡测试）
```

### 提示词-上下文-Harness 在 CI/CD 中的映射

```
提示词层：开发者写的 PR 描述 + Issue 需求
上下文层：GitNexus 影响分析 + GBrain 知识查询
Harness 层：BP1/BP2/BP3 Gate + Nomad 执行

CI/CD 就是 Harness 层的"自动化执行引擎"
```

---

## 4.3 Multi-Agent协作与业界实践

### 业界主流平台（2025）

| 平台 | 提供方 | 特点 |
|------|--------|------|
| GitHub Agent HQ | GitHub/Microsoft | 统一编排Copilot、Claude、GPT、Gemini |
| Azure AI Foundry | Microsoft | Lead Agent + Sub-Agent架构 |
| Claude Code + MCP | Anthropic | 终端Agent + 上下文共享协议 |
| OpenAI Codex | OpenAI | 异步编码Agent |

### 核心概念

- **MCP (Model Context Protocol)**：AI工具间上下文共享的标准协议
- **Branch-per-Agent**：每个Agent在独立分支工作，避免冲突
- **Plan Mode**：Agent先制定执行计划，人类确认后再执行
- **Human-in-the-Loop**：关键节点需要人类审批

---

## 4.4 Why：为什么需要Multi-Agent协作

### 提高效率

- 并行处理任务（多Agent同时在不同分支工作）
- 专业分工（每个Agent专注特定角色）
- 24/7工作

### 提高质量

- 多Agent交叉审核
- 交叉验证减少遗漏
- Anthropic研究：多Agent系统比单Agent正确率高90%

### 降低成本

- 减少人工干预
- 自动化重复工作
- 2025年企业Multi-Agent项目占比达72%

---

## 4.5 How：AI角色分配与编排

### AI角色定义

| 角色 | 职责 | 推荐工具 |
|------|------|----------|
| 分析师 | 需求分析、设计评审 | Claude |
| 架构师 | 架构设计、技术选型 | GPT-4 |
| 开发者 | 代码编写、单元测试 | Claude Code |
| 测试员 | 测试用例、集成测试 | Claude |
| 审查员 | 代码审查、质量检查 | Claude |

### 编排器（Orchestrator）

- 统一调度所有Agent
- 管理上下文共享（MCP协议）
- 处理Agent间冲突
- 执行Human-in-the-Loop审批

### 协作流程

```
分析师 → 架构师 → 开发者 → 测试员 → 审查员
    │                            │
    └────────── 文档员 ──────────┘
```

---

# Part 5: 实践练习

---

## 5.1 观察Nomad集群

### 任务

1. SSH到Nomad Server
2. 查看节点列表
3. 查看Runner状态

### 命令

```bash
# 查看节点
ssh admin@192.168.0.252 "nomad node status"

# 查看Runner容器
ssh admin@192.168.0.252 "docker ps | grep nomad-runner"

# 查看Nomad jobs
ssh admin@192.168.0.252 "nomad job status"
```

---

## 5.2 触发Gate检查

### 任务

1. 创建一个测试PR
2. 观察Gate检查的触发
3. 查看检查结果

### 检查触发日志

```bash
# 查看最新job
ssh admin@192.168.0.252 "nomad job history -short <job_id>"

# 查看Runner日志
ssh admin@192.168.0.252 "docker logs nomad-runner --tail 100"
```

---

## 5.3 设计Agent增强CI

### 任务

为你的项目设计一个利用 Agent 的 CI 流程：
- PR 创建时，哪些 Agent 并行执行？
- 每个 Agent 负责什么检查？
- 结果如何汇总？

### Gate设计模板

```
我的项目 Gate 设计：

BP1 静态检查：
  □ 编译通过
  □ 格式化通过
  □ Lint通过

BP2 集成检查：
  □ 单元测试通过率 ≥ __%
  □ 覆盖率 ≥ __%

BP3 功能检查：
  □ 关键业务流程测试通过
  □ 性能基准达标
```

---

# 核心知识点总结

---

## 1. CI/CD架构演进

- **1.0**：共享Runner（零运维，但资源争抢）
- **2.0**：自托管Runner（独占资源，但依赖SaaS）
- **3.0**：Nomad + Gitea自托管（完全自主）

## 2. Nomad Runner

- **注册机制**：连接Nomad Server → 接收Job → 执行 → 上报
- **高可用**：双节点冗余 + 健康检查 + 自动恢复
- **调度优势**：容器隔离 + 资源限制 + 并行执行

## 3. Gate联动

- **BP1/BP2/BP3**：静态 → 集成 → Beta三层检查
- **Admin Token**：管理Gitea全局资源的认证
- **结果上报**：PR评论 + SSOT存储

## 4. Agent编排

- **Agent增强CI**：explore/librarian/oracle并行分析，弥补AI上下文不足
- **Multi-Agent协作**：专业分工+交叉验证，比单Agent更高效

---

# 课后作业

---

## 任务

1. 在本地运行 `nomad node status`，记录集群状态
2. 创建一个测试PR，观察Gate触发流程
3. 设计一个利用 Agent 的 CI 流程
4. 设计一个适合你项目的Gate检查项

## 预习

- 性能优化与重构
- TPC-H基准测试与回归检测

---

<!-- _class: lead -->

# 谢谢！

## 下节课：性能优化与重构
