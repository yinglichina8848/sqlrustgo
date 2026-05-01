# AI Factory Control Plane — Brainstorming 分析

> **分析问题**: SQLRustGo AI 协作系统的控制面缺失问题
> **依据**: ChatGPT 架构审计建议 (2026-05-01)
> ** brainstorming 方法**: 5Why + 系统动力学因果链

---

## 问题 0：根本隐喻

```
当前系统 = "无交警的复杂路口"
- 车辆 (Agent/Worker) 自由穿行
- 没有红绿灯 (Gate 校验)
- 没有摄像头 (可观测性)
- 没有执法 (Hermes 自己开车+自己指挥)
```

**核心矛盾**：
> 多 Agent 协作系统必须有 Control Plane，但我们的 Control Plane = Hermes，而 Hermes 同时是 Executor。

---

## 问题 1：Branch Protection Bypass — 根因分析

### 表象
`force_merge=true` 和 "DELETE protection → merge → recreate" 成为常态。

### 5Why 追问

| 层级 | 追问 | 答案 |
|------|------|------|
| Why 1 | 为什么需要 bypass？ | CI 不通过或保护规则冲突 |
| Why 2 | 为什么 CI 不通过？ | 可能是 flaky test 或资源不足 |
| Why 3 | 为什么不修 CI 而用 bypass？ | bypass 更快（短视） |
| Why 4 | 为什么允许 bypass 存在？ | 没有强制 Gate 校验机制 |
| Why 5 | 为什么没有强制 Gate？ | **Control Plane 缺失 — 没人负责设计** |

### 因果链

```
CI 失败 → 手动 bypass → Gate 形同虚设
                      → 质量退化
                      → 技术债累积
                      → 系统不可信
```

### 选项评估

| 方案 | 可行性 | 效果 | 风险 |
|------|--------|------|------|
| A. 删除所有分支保护 | ⚡ 极高 | 零 | 灾难 |
| B. 严格 CI 才能 merge | ⚡ 高 | 中 | 频繁阻断开发 |
| C. 引入 pre-merge API hook | ✅ 最佳 | 高 | 需开发 hook |
| D. Hermes 强制执行人工 review | ⚡ 低 | 低 | Hermes 自己违规 |

**结论：方案 C — pre-merge Gate hook**

---

## 问题 2：Hermes 既是裁判又是运动员 — 根因分析

### 表象
Hermes 可以：审核 PR、merge PR、修改代码、执行 CI。

### 5Why 追问

| 层级 | 追问 | 答案 |
|------|------|------|
| Why 1 | 为什么 Hermes 权力这么大？ | 从设计之初就是 "全栈 Agent" |
| Why 2 | 为什么要设计成全栈？ | 快速启动（早期合理） |
| Why 3 | 为什么现在成问题了？ | 协作规模扩大，需要分权 |
| Why 4 | 为什么没有分离权力？ | 没有角色模型设计 |
| Why 5 | 为什么没有角色模型？ | **系统没有架构师角色** |

### 权力矩阵分析

| 操作 | Hermes (当前) | Hermes (应然) | System Gate |
|------|-------------|--------------|-------------|
| 提交代码 | ✅ | ✅ (via PR) | ❌ |
| 创建 PR | ✅ | ✅ | ❌ |
| 审核 PR | ✅ | ✅ | ❌ |
| Merge PR | ✅ | ❌ | ✅ |
| 强制 Bypass | ✅ | ❌ | ❌ |
| 创建分支保护 | ✅ | ❌ | ❌ |

### 选项评估

| 方案 | 描述 | 效果 | 风险 |
|------|------|------|------|
| A. 完全移除 Hermes merge 权限 | Hermes 只有审核权 | 最佳 | 开发速度降 |
| B. Hermes merge 需要 user 审批 | 双重签名 | 中 | 流程变复杂 |
| C. 分层权限 (reviewer/merger) | 引入 GitHub-style | 中高 | 实现成本 |

**结论：方案 A + Gitea Branch Protection 完全接管 merge 权限**

---

## 问题 3：Nomad 调度 ≠ CI 调度 — 根因分析

### 表象
- Gitea Actions 触发 CI
- Nomad 执行编译任务
- 两者共享 Runner 资源但无统一调度

### 系统动力学视角

```
                    Gitea Actions
                         │
                    [事件触发]
                         │
                    ┌────▼────┐
                    │ Runner  │ ← 资源瓶颈
                    └────┬────┘
                         │
            ┌────────────┼────────────┐
            │            │            │
       [短期任务]    [长期编译]    [测试]
           │            │            │
      快速返回       长时间占用    资源争抢
            │            │            │
            └────────────▼────────────┘
                    [无优先级]
                    [无队列]
                    [无 backpressure]
```

### 根因

> **CI 是 "拉模式 (pull)"，Nomad 是 "推模式 (push)"，你在中间没有队列层。**

### 选项评估

| 方案 | 描述 | 效果 | 实现成本 |
|------|------|------|---------|
| A. 移除 Gitea Actions，全部 Nomad | Nomad 唯一调度器 | 最佳 | 高 |
| B. Actions 作为 trigger，Nomad 执行 | 队列化 | 高 | 中 |
| C. 限制并发数，手动调度 | 简单治标 | 低 | 低 |

**结论：方案 B — Gitea Actions 保留为事件源，Nomad 作为唯一执行调度器，通过 Redis/文件队列解耦。**

---

## 问题 4：多 Agent 并发写仓库 — 根因分析

### 表象
Mac Mini 和 TRAE AI 同时修改同一目录导致冲突。

### 根因
> 不是"目录相同"的问题，而是"没有锁机制"的问题。

### Git 语义 vs 多 Writer

```
Hermes (Mac Mini) ──→ git push ──→ origin/main
TRAE AI ───────────→ git push ──→ origin/main
        │
        └─→ 同时修改同一行代码 → merge conflict
                       ↓
               无协调机制
```

### 解决选项

| 方案 | 描述 | 优点 | 缺点 |
|------|------|------|------|
| A. 分目录 | MacMini: /repo-a, TRAE: /repo-b | 隔离彻底 | 合并复杂 |
| B. 分 branch | 不同功能走不同 branch | Git 原生 | 需协调 |
| C. PR 锁 | 文件级锁 | 精确控制 | 实现复杂 |
| D. 工作目录锁 | .hermes/locks/ 机制 | 可落地 | 需要约定 |

**结论：方案 D — 基于工作目录的协调锁**

---

## 问题 5：GBrain 不是机器可执行知识 — 根因分析

### 表象
Pattern 和 ADR 是文档，但 CI 可能违反 Pattern。

### 知识层级模型

```
Level 1: 文档 (GBrain Pattern)      ← 人类可读
Level 2: 规则 (Rule Engine)          ← 机器可执行
Level 3: 策略 (Policy Enforcement)   ← 自动执行 + 报告
```

### 从 Pattern 到执行

```
ChatGPT 建议: "禁止 actions/cache"
     ↓
手动记录到 GBrain Pattern
     ↓
但 CI yaml 仍然可能加回去
     ↓
需要自动校验
```

### 规则引擎设计

```yaml
# .hermes/rules/forbid_actions_cache.yaml
id: forbid_actions_cache
version: 1.0
description: Gitea 1.22.1 不兼容 actions/cache
match:
  file: ".gitea/workflows/*.yml"
  contains_any: ["actions/cache", "actions/upload-artifact"]
severity: error
action: reject_merge
message: "actions/cache 在 Gitea 1.22.1 下会导致 CI 卡死"
evidence: "PR #88 验证"
```

**结论：GBrain → Rule Engine 升级，hermes rule-check <pr> 自动执行**

---

## 系统性结论：Control Plane 架构

```
┌────────────────────────────────────────────────────────────────────┐
│                       CONTROL PLANE (缺失!)                         │
│                                                                   │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐       │
│  │ Auth     │   │ Gate     │   │ Queue    │   │ Rule     │       │
│  │ Manager  │   │ Engine   │   │ Manager  │   │ Engine   │       │
│  │          │   │          │   │          │   │          │       │
│  │ - 身份   │   │ - R-Gate │   │ - 优先级  │   │ - Pattern│       │
│  │ - 权限   │   │ - B-Gate │   │ - 锁     │   │ → YAML  │       │
│  │ - 分权   │   │ - 强制   │   │ - 冲突   │   │ - 自动   │       │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘       │
│        │              │              │              │              │
│        └──────────────┴──────────────┴──────────────┘              │
│                             │                                      │
│                    ┌────────▼────────┐                            │
│                    │ Hermes Agent    │                            │
│                    │ (只读/审核/编排) │                            │
│                    └─────────────────┘                            │
└────────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
    ┌─────▼─────┐       ┌─────▼─────┐       ┌─────▼─────┐
    │ Worker A  │       │ Worker B  │       │ Worker C  │
    │ (Z6G4)   │       │ (ai@250) │       │ (CLI)    │
    └───────────┘       └───────────┘       └───────────┘
```

---

## 优先级重新排序

根据 brainstorming 结果，调整 ChatGPT 的 P0：

| 优先级 | 任务 | 理由 |
|--------|------|------|
| **P0-0** | 备份当前系统 | 先止血 |
| **P0-1** | 移除 Hermes merge 权限 | 最危险权限扩散 |
| **P0-2** | 实现 B-Gate pre-merge hook | Gate 失效是根本问题 |
| **P0-3** | 建立 PR 锁机制 | 解决并发写入 |
| **P1-1** | Nomad Job Queue 化 | 资源争抢 |
| **P1-2** | GBrain → Rule Engine | 知识可执行化 |
| **P2-1** | Hermes 降权为审核者 | 角色分离 |
| **P2-2** | 可观测性面板 | 状态可视化 |

---

## 下一步

本 brainstorming 分析产生的 Implementation Plan 写入：
`docs/plans/2026-05-01-ai-factory-implementation-plan.md`
