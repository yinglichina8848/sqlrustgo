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

1. **CI/CD的架构演进**（20分钟）
2. **Nomad Runner部署原理**（20分钟）
3. **Agent编排：CI/CD的新视角**（20分钟）
4. **Gate联动与自动化**（20分钟）
5. **实践练习**（10分钟）

---

# Part 1: CI/CD的架构演进

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

---

## 1.3 为什么需要架构3.0

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

---

# Part 2: Nomad Runner 部署原理

---

## 2.1 Nomad 是什么（一句话版）

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

---

## 2.2 Runner 的注册和执行流程

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

---

## 2.3 Gitea Actions 与 Gate 联动

### 三层触发机制

```
push 到 develop/v3.0.0
    │
    ├──→ 触发 ci.yml     → BP1 静态检查（编译+格式+Clippy）
    │                       ~2 分钟，快速反馈
    │
PR 创建
    │
    ├──→ 触发 gate.yml   → BP2 集成检查（测试+覆盖率+TPC-H+回归）
    │                       ~10 分钟，功能验证
    │
合并前
    │
    └──→ 触发 beta.yml   → BP3 风险检查（Crash+MVCC+死锁+审计）
                           ~30 分钟，生产可信度验证
```

---

# Part 3: Agent 编排 — CI/CD 的新视角

---

## 3.1 传统 CI vs Agent 增强 CI

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

## 3.2 Agent 在 CI/CD 中的典型任务

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

# Part 4: 实践练习

---

## 4.1 观察 Nomad 集群

```bash
# 查看节点
ssh admin@192.168.0.252 "nomad node status"

# 查看 Runner 容器
ssh admin@192.168.0.252 "docker ps | grep nomad-runner"
```

---

## 4.2 设计 Agent 增强 CI

### 任务

为你的项目设计一个利用 Agent 的 CI 流程：
- PR 创建时，哪些 Agent 并行执行？
- 每个 Agent 负责什么检查？
- 结果如何汇总？

---

# 核心知识点总结

---

- **CI/CD 演进**：共享 Runner → 自托管 Runner → Nomad 容器化
- **Nomad**：轻量级任务调度器，双节点高可用
- **Gate 联动**：BP1（~2分钟）→ BP2（~10分钟）→ BP3（~30分钟）
- **Agent 编排**：explore/librarian/oracle 并行分析，弥补 AI 上下文不足

---

# 课后作业

1. 观察 Nomad 集群状态
2. 创建一个测试 PR，观察 Gate 触发流程
3. 设计一个利用 Agent 的 CI 流程

---

<!-- _class: lead -->

# 谢谢！

## 下节课：性能优化与回归检测
