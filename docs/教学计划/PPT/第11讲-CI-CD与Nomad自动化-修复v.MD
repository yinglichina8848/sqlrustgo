---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十一讲：CI/CD与Nomad自动化

## Harness工程：自托管Runner与Gate联动

---

# 课程大纲

1. **CI/CD架构演进**（25分钟）
2. **Nomad Runner部署原理**（25分钟）
3. **Gitea Actions与Gate联动**（25分钟）
4. **实践练习**（15分钟）

---

# Part 1: CI/CD架构演进

---

## 1.1 CI/CD的三次架构演进

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

## 1.2 为什么需要架构3.0

### AI时代的特殊需求

| 需求 | GitHub SaaS | 架构3.0 |
|------|-------------|---------|
| 网络隔离 | ❌ 不支持 | ✅ 完全内网 |
| 自定义Runner | ⚠️ 有限支持 | ✅ 完全控制 |
| 知识管理集成 | ❌ 无 | ✅ GBrain SSOT |
| 多平台同步 | ❌ 无 | ✅ GitHub/Gitea/GitCode/Gitee |
| 成本控制 | 按分钟计费 | 固定资源成本 |

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

## 1.3 Nomad集群的核心概念

### Nomad是什么

> **Nomad是一个分布式工作负载管理器**，支持在集群中调度和运行容器化或非容器化的工作负载。

### Nomad vs Kubernetes

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

## 2.1 Runner注册机制

### 什么是Runner注册

> **Runner是注册到Gitea Actions的"执行器"，负责实际运行CI Job。**

### 注册流程

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

# Part 4: 实践练习

---

## 4.1 观察Nomad集群状态

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

## 4.2 触发Gate检查

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

## 4.3 设计自己的Gate检查

### 任务

思考：在你的项目中，什么检查是"必须通过才能合并"的？

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

---

# 课后作业

---

## 任务

1. 在本地运行 `nomad node status`，记录集群状态
2. 创建一个测试PR，观察Gate触发流程
3. 设计一个适合你项目的Gate检查项

## 预习

- 性能优化与重构
- TPC-H基准测试与回归检测

---

<!-- _class: lead -->

# 谢谢！

## 下节课：性能优化与回归检测

