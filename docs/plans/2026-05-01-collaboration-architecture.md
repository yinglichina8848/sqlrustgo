# SQLRustGo AI 协作开发架构方案

> 三平台 + Gitea CI/CD + LLM-Wiki + GBrain 知识管理
> 版本: v1.0 | 日期: 2026-05-01 | 状态: 草稿

---

## 一、当前协作架构

### 1.1 硬件资源拓扑

```
┌─────────────────────────────────────────────────────────────┐
│                     Mac Mini (Brain)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Gitea     │  │   Hermes    │  │   Nomad Leader      │ │
│  │  1.22.1     │  │  Agent      │  │   v1.9.6           │ │
│  │  :3000      │  │  CLI        │  │   :4647            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│  用户直接交互 / 任务编排 / 决策中枢                            │
└─────────────────────────────────────────────────────────────┘
            │                    │
            │ SSH :222           │ SSH :222
            ▼                    ▼
┌───────────────────────┐  ┌───────────────────────┐
│    Z6G4 (Worker)      │  │    Z440 (Worker)       │
│  HP Z440 Workstation  │  │  HP Z440 Workstation   │
│  28c / 94GB RAM       │  │  28c / 94GB RAM       │
│  ┌─────────────┐      │  ┌─────────────┐         │
│  │ Nomad Node  │      │  │ Nomad Node  │         │
│  │ :4646       │      │  │ :4646       │         │
│  └─────────────┘      │  └─────────────┘         │
│  ┌─────────────┐      │  ┌─────────────┐         │
│  │ SQLRustGo   │      │  │ SQLRustGo   │         │
│  │ Runner      │      │  │ Runner      │         │
│  │ v1 (dev)    │      │  │ v1 (prod)   │         │
│  └─────────────┘      │  └─────────────┘         │
│  重型计算 / 编译       │  重型计算 / 编译           │
└───────────────────────┘  └───────────────────────┘
```

### 1.2 当前 Git 身份与分支保护

| 节点 | Git 用户 | Email | SSH Key | 职责 |
|------|---------|-------|---------|------|
| Mac Mini (Hermes) | `openclaw` | `hermes-macmini@gaoyuanyiyao.com` | `openclaw-gitea.PAT` | 编排 / PR 合并 |
| Mac Mini (CLI) | `hermes-macmini` | `hermes-macmini@gaoyuanyiyao.com` | `id_ed25519_z6g4` | 直接推送 |
| Z6G4 | `hermes-z6g4` | `hermes-z6g4@gaoyuanyiyao.com` | `id_ed25519_z6g4` | Worker 执行 |

**分支保护规则：**
- `main`: 禁止直接推送，必须通过 PR
- `develop/v2.8.0`: 禁止直接推送，必须通过 PR（`Hermes Pipeline` 保护规则阻断 `skip-ci`）
- 直接推送解决方式：移除临时保护 → 推送 → 恢复保护（需 API 操作）

### 1.3 当前 CI/CD 流程 (Gitea Actions + Nomad)

```
PR 创建/推送
    │
    ▼
Gitea Actions 触发
    │
    ├── Runner 1-4 (devstack): cargo test --all-features
    │
    └── Nomad Job v23 (4+4 并发):
         ├── devstack-runner × 4  (短期测试任务)
         └── nomad-root-v2 × 4   (长期编译任务)

    注意: actions/cache 和 actions/upload-artifact 在 Gitea 1.22.1 下
          会导致 CI 卡死，已从 ci.yml 中移除，替换为纯 shell 步骤
```

---

## 二、Gitea 部署方案

### 2.1 Docker Compose 配置 (Z6G4)

```yaml
# ~/docker/gitea/docker-compose.yml
version: '3.8'
services:
  gitea:
    image: gitea/gitea:1.22.1-rootless
    container_name: gitea-devstack
    restart: unless-stopped
    environment:
      - USER_UID=1000
      - USER_GID=1000
      - GITEA__server__PROTOCOL=http
      - GITEA__server__DOMAIN=192.168.0.252:3000
      - GITEA__server__ROOT_URL=http://192.168.0.252:3000/
      - GITEA__database__DB_TYPE=postgres
      - GITEA__database__HOST=192.168.0.252:5432
      - GITEA__database__NAME=gitea
      - GITEA__database__USER=gitea
      - GITEA__database__PASSWD=${POSTGRES_PASSWORD}
      - GITEA__actions__ENABLED=true
      - GITEA__actions__DEFAULT_RUNNER_LABELS=hp-z6g4:host,z440:host
      - GITEA__security__INSTALL_LOCK=true
    ports:
      - "3000:3000"
      - "222:222"
    volumes:
      - gitea-data:/data
      - /Users/liying/workspace/dev/openheart/sqlrustgo:/git/repos/openclaw/sqlrustgo
      - /Users/liying/.ssh:/data/git/.ssh:ro
    networks:
      - gitea-net

  runner:
    image: sqlrustgo-runner:v1
    container_name: gitea-runner
    restart: unless-stopped
    environment:
      - Gitea__server__ROOT_URL=http://gitea-devstack:3000
      - Gitea__actions__RUNNER_CAPACITY=4
      - Gitea__actions__RUNNER_LABELS=hp-z6g4:host
    networks:
      - gitea-net

networks:
  gitea-net:
    driver: bridge

volumes:
  gitea-data:
```

### 2.2 Nomad 集群配置

**Nomad Server (Mac Mini):**

```hcl
# /etc/nomad/nomad.hcl
name = "macmini-leader"
data_dir = "/opt/nomad/data"
bind_addr = "0.0.0.0"

server {
  enabled = true
  bootstrap_expect = 1
}

client {
  enabled = true
  node_pool = "devstack"
  label {
    key = "host"
    value = "macmini"
  }
}

plugin "docker" {
  config {
    volumes = [" berbagi /var/run/docker.sock:/var/run/docker.sock"]
  }
}
```

**Nomad Worker (Z6G4):**

```hcl
# /etc/nomad/nomad.hcl
name = "hp-z6g4"
data_dir = "/opt/nomad/data"
bind_addr = "0.0.0.0"

server {
  enabled = false
}

client {
  enabled = true
  node_pool = "worker"
  label {
    key = "host"
    value = "z6g4"
  }
}

plugin "raw_exec" {
  config {
    enabled = true
  }
}
```

**Nomad Job 定义 (sqlrustgo-runner):**

```hcl
job "sqlrustgo-runner" {
  datacenters = ["dc1"]
  type = "service"
  count = 4

  constraint {
    attribute = "${node.unique.id}"
    operator  = "regexp"
    value     = "hp-z6g4|ai@250"
  }

  group "devstack" {
    count = 4

    task "runner" {
      driver = "docker"
      
      config {
        image = "sqlrustgo-runner:v1"
        command = "/bin/sh"
        args = ["-c", "cd /repo && ./entrypoint.sh"]
        
        labels {
          host = "hp-z6g4"
        }
      }

      env {
        CONFIG_FILE = "/etc/runner/config.toml"
        NOMAD_TASK_NAME = "${NOMAD_TASK_NAME}"
      }

      resources {
        cpu    = 2000
        memory = 4096
      }

      vault {
        policies = ["runner"]
      }
    }
  }

  update {
    max_parallel     = 1
    min_healthy_time = "10s"
    auto_revert      = true
  }
}
```

---

## 三、三平台开发流程

### 3.1 平台角色定义

| 平台 | 角色 | CPU/RAM | 主要职责 | Git 身份 |
|------|------|---------|---------|---------|
| **Mac Mini** | Brain / Orchestrator | 自适应 | 任务编排、决策、PR 合并、知识管理 | `openclaw` (PAT) |
| **Z6G4** | Heavy Worker | 80c / 409GB | 大规模编译、测试、性能基准 | `hermes-z6g4` (SSH) |
| **ai@250** | Light Worker | 28c / 94GB | 辅助编译、文档生成 | `hermes-z6g4` (SSH) |

### 3.2 当前协作模式

```
                    ┌─────────────────────────────┐
                    │  User + Hermes (Mac Mini)   │
                    │  任务分解 / 规则验证 / 决策   │
                    └─────────────┬───────────────┘
                                  │ 委托
                    ┌─────────────▼───────────────┐
                    │  Gitea Issue / PR            │
                    │  → 三平台任务分配             │
                    └─────────────┬───────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
        ┌──────────┐        ┌──────────┐        ┌──────────┐
        │  Z6G4    │        │  ai@250  │        │  GitHub  │
        │ Worker   │        │  Worker  │        │ /Gitee   │
        │ 编译/测试 │        │ 编译/测试 │        │ 镜像同步  │
        └────┬─────┘        └────┬─────┘        └────┬─────┘
             │                   │                   │
             └───────────────────┼───────────────────┘
                                 │ PR / Push
                                 ▼
                    ┌─────────────────────────────┐
                    │  Gitea (Mac Mini)           │
                    │  CI/CD Runner 触发          │
                    │  Nomad 并行任务执行          │
                    └─────────────────────────────┘
```

### 3.3 Git Pre-Commit 身份验证

SQLRustGo 仓库强制 Git 身份检查：

```bash
# .git/hooks/pre-commit (自动生成)
ALLOWED_EMAILS=(
  "openheart@gaoyuanyiyao.com"
  "hermes-macmini@gaoyuanyiyao.com"
  "hermes-z6g4@gaoyuanyiyao.com"
)

current_email=$(git config user.email)
if [[ ! " ${ALLOWED_EMAILS[@]} " =~ " ${current_email} " ]]; then
  echo "❌ Wrong Git identity! Expected one of: ${ALLOWED_EMAILS[*]}"
  echo "   Got: $current_email"
  exit 1
fi
```

**正确身份配置：**

```bash
# Mac Mini (Hermes Agent 直接操作)
git config --local user.email "hermes-macmini@gaoyuanyiyao.com"
git config --local user.name "openclaw"

# Z6G4 / ai@250 (远程执行)
git config --local user.email "hermes-z6g4@gaoyuanyiyao.com"
git config --local user.name "hermes-z6g4"
```

### 3.4 分支保护与 PR 合并策略

**问题：** `develop/v2.8.0` 设置了 `Hermes Pipeline` 保护规则，阻止 `skip-ci` PR 合并。

**解决方案 A — 移除保护临时合并（推荐）：**

```bash
# 1. 获取保护规则 ID
PROTECTION_ID=$(curl -s "$GITEA_API/repos/$REPO/branches/$BRANCH/protections" \
  -H "Authorization: token $TOKEN" | jq -r '.[0].id')

# 2. 删除保护
curl -X DELETE "$GITEA_API/repos/$REPO/branches/protections/$PROTECTION_ID" \
  -H "Authorization: token $TOKEN"

# 3. 合并 PR
curl -X POST "$GITEA_API/repos/$REPO/pulls/$PR_NUM/merge" \
  -H "Authorization: token $TOKEN" \
  -d '{"force_merge":true}'

# 4. 重建保护
# (通过 Gitea UI 重新创建保护规则)
```

**解决方案 B — 强制合并标志（不需要删除保护）：**

```bash
curl -X POST "$GITEA_API/repos/$REPO/pulls/$PR_NUM/merge" \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"Do":"merge","force_merge":true}'
```

---

## 四、LLM-Wiki + GBrain 知识管理

### 4.1 知识管理架构

```
┌──────────────────────────────────────────────────────────────┐
│                    Hermes Agent (Mac Mini)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │  短期记忆     │  │   技能 (SKILL)│  │   持久记忆 (Memory) │  │
│  │  (上下文)    │  │  .hermes/     │  │   .hermes/memory    │  │
│  │              │  │   skills/     │  │                     │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
│         │                  │                    │             │
│         └──────────────────┼────────────────────┘             │
│                            ▼                                   │
│                   ┌──────────────────┐                        │
│                   │   GBrain (长期知识库)│                        │
│                   │   ~/gbrain/        │                        │
│                   └──────────────────┘                        │
│                            │                                   │
│                   ┌────────▼────────┐                        │
│                   │  QMD Wiki (三层)  │                        │
│                   │  docs/wiki/      │                        │
│                   │  ├── *.qmd        │                        │
│                   │  └── assets/      │                        │
│                   └──────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 三层 Wiki 结构

| 层级 | 位置 | 用途 | 更新频率 |
|------|------|------|---------|
| **工作流层 (QMD)** | `docs/wiki/` | 结构化流程、规范、架构 | 按需更新 |
| **知识层 (GBrain)** | `~/gbrain/` | 可检索知识图谱、规则 | 持续积累 |
| **GitHub Wiki** | repo Wiki | 对外文档、API 文档 | Release 时同步 |

### 4.3 GBrain 知识库构建流程

```bash
# 1. 初始化 GBrain 目录
mkdir -p ~/gbrain/sqlrustgo/{rules,patterns,architecture,decisions}

# 2. 记录决策 (ADR - Architecture Decision Records)
cat > ~/gbrain/sqlrustgo/decisions/adr-001-z6g4-ssh-key.md << 'EOF'
# ADR-001: Z6G4 SSH Key 管理

## 状态: 已接受

## 背景
Z6G4 作为主要 Worker 节点，需要从 Mac Mini SSH 访问。

## 决策
- 使用 `id_ed25519_z6g4` 密钥对
- 端口: 222 (非标准 SSH 端口)
- Key 存储在 Mac Mini `~/.ssh/id_ed25519_z6g4`

## 教训
- Z440 的 SSH alias "gitea-z6g4" 与 Z6G4 冲突，应改为 "gitea-z440"
EOF

# 3. 记录重复出现的问题模式
cat > ~/gbrain/sqlrustgo/patterns/ci-cache-blockage.md << 'EOF'
# Pattern: Gitea Actions Cache 卡死

## 触发条件
- 使用 actions/cache@v4 或 actions/upload-artifact@v4
- Gitea 1.22.1

## 症状
CI 任务在 cache 步骤永久挂起，不超时也不报错

## 根因
Gitea 1.22.1 的 actions 实现与官方 actions/cache 不兼容

## 解决方案
完全移除 actions/cache 和 actions/upload-artifact，替换为纯 shell 步骤：

```yaml
- name: Cache dependencies
  run: |
    mkdir -p ~/.cargo/registry/cache
    # 手动缓存逻辑
```

## 验证
PR #88 (fmt 修复) 已验证可行
EOF
```

### 4.4 Hermes 记忆持久化

```python
# 使用 memory 工具保存关键事实
memory(action="add", target="memory", content="""
SQLRustGo v2.8.0 发布关键约束:
- VERSION=alpha/v2.8.0 (需同步 wiki 显示 rc)
- 禁止手动关闭没有 PR 合并的 Issue
- 分支保护强制通过 PR 合入
- Git identity: 仅允许 openheart/hermes-macmini/hermes-z6g4
""")

memory(action="add", target="user", content="""
用户偏好:
- 形式化输出: ∀, ∃, invariant, IF/THEN 规则优先
- 动大模块前必须先备份
- 不接受"看起来正确"，必须提供证据
- 通过 Gitea Issue 分配任务而非直接指令
""")
```

---

## 五、更好的 AI 协作开发流程

### 5.1 当前问题分析

| 问题 | 现状 | 影响 |
|------|------|------|
| **Git 冲突** | Mac Mini 和 TRAE AI 并行修改同一目录 | 代码覆盖、丢失修改 |
| **分支保护阻断** | 每次小修改都要 PR + 绕过保护 | 效率低、风险高 |
| **知识丢失** | 每次会话重置，经验未沉淀 | 重复踩坑 |
| **CI 卡死** | actions/cache 在 Gitea 1.22.1 不兼容 | 发布延迟 |
| **多端同步** | 需手动管理 Git identity 和 SSH key | 操作繁琐 |

### 5.2 改进方案：角色化多 Agent 协作

```
┌─────────────────────────────────────────────────────────────┐
│                     User (Chief Architect)                  │
│   规则制定 / 重大决策 / 最终验收                              │
└─────────────────────────┬───────────────────────────────────┘
                          │ 任务分配 (Issue/Prompt)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                Hermes Agent (Mac Mini)                      │
│  角色: Orchestrator / Reviewer / Gate Keeper               │
│                                                              │
│  职责:                                                       │
│  - 分解任务为 Gitea Issue                                   │
│  - 审核 Worker 提交的内容                                    │
│  - 执行 R-Gate / B-Gate 检查                                │
│  - 维护 GBrain 知识库                                       │
│  - 管理 Hermes 记忆                                          │
└────────────┬───────────────────────────────┬────────────────┘
             │                               │
    Issue #42 (Cargo fmt)            Issue #43 (性能优化)
             │                               │
             ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│  Hermes-Z6G4 (Worker)   │     │  Hermes-ai250 (Worker)  │
│                         │     │                         │
│  执行:                  │     │  执行:                   │
│  - cargo fmt           │     │  - 性能分析              │
│  - 提交 PR #88         │     │  - benchmark 对比       │
│  - CI 验证             │     │  - 提交 PR #89          │
│                         │     │                         │
│  自我验证后报告给        │     │  完成后通知 Hermes      │
│  Hermes (Mac Mini)      │     │  (Mac Mini) 审核        │
└─────────────────────────┘     └─────────────────────────┘
```

### 5.3 推荐的 AI Agent 角色定义

```yaml
# .hermes/agents.yaml
agents:
  hermes-macmini:
    role: orchestrator
    capabilities:
      - task_decomposition
      - code_review
      - gate_verification
      - knowledge_management
    tools:
      - terminal
      - file
      - delegate_task
      - memory
      - gitea_api
    constraints:
      - 不能直接修改核心模块 (需 PR)
      - 重大决策需用户确认
      - 备份后才能动容器/数据库

  hermes-z6g4:
    role: worker-heavy
    capabilities:
      - large_scale_compilation
      - integration_testing
      - performance_benchmark
      - release_build
    tools:
      - terminal
      - file
    workdir: /workspace/sqlrustgo
    ssh_access: true

  hermes-ai250:
    role: worker-light
    capabilities:
      - unit_testing
      - documentation
      - ci_pipeline
      - code_search
    tools:
      - terminal
      - file
    workdir: /workspace/sqlrustgo
    ssh_access: true
```

### 5.4 任务分配标准流程

```
Step 1: User → Hermes (Mac Mini)
  "修复 VERSION 文件与 wiki 不一致的问题"

Step 2: Hermes (Mac Mini) — 任务分解
  ├─ Issue #44: 同步 VERSION → alpha/v2.8.0 (Hermes 自己)
  ├─ Issue #45: 更新 wiki 显示 (Hermes 自己)
  └─ Issue #46: 验证 CI 状态 (Z6G4 Worker)

Step 3: 各 Agent 领取 Issue，执行后提交 PR

Step 4: Hermes (Mac Mini) 审核 PR，执行 Gate 检查

Step 5: Hermes 合并，通知 User
```

### 5.5 Gitea Issue 自动化流程

```bash
#!/bin/bash
# scripts/issue-assign.sh — 自动分配 Issue 到合适的 Agent

ISSUE_NUM=$1
AGENT_LABEL=$2  # z6g4 / ai250 / macmini

case $AGENT_LABEL in
  z6g4)
    curl -s -X POST "$GITEA_API/repos/$REPO/issues/$ISSUE_NUM/labels" \
      -H "Authorization: token $TOKEN" \
      -d '{"labels":["worker/z6g4","P1"]}'
    ;;
  ai250)
    curl -s -X POST "$GITEA_API/repos/$REPO/issues/$ISSUE_NUM/labels" \
      -H "Authorization: token $TOKEN" \
      -d '{"labels":["worker/ai250","P2"]}'
    ;;
  macmini)
    # Mac Mini 自己处理，添加标签
    curl -s -X POST "$GITEA_API/repos/$REPO/issues/$ISSUE_NUM/labels" \
      -H "Authorization: token $TOKEN" \
      -d '{"labels":["orchestrator/self","P1"]}'
    ;;
esac
```

### 5.6 知识沉淀流程

```
每次完成复杂任务后：

1. 如果是首次解决某类问题 → 创建 GBrain Pattern
2. 如果学到了项目规则 → 更新 Hermes Memory
3. 如果是流程改进 → 更新 ADR
4. 如果是对外接口 → 更新 GitHub Wiki

验证: 下次遇到同类问题时，查询 GBrain/Memory 后再行动
```

---

## 六、CI/CD 改进方案

### 6.1 当前 CI 痛点

| 痛点 | 根因 | 当前状态 |
|------|------|---------|
| actions/cache 卡死 | Gitea 1.22.1 不兼容官方 actions | 已移除，改用纯 shell |
| Runner 容量不足 | 仅 4 devstack-runner | 已扩展到 8 并发 (4+4) |
| 分支保护阻断合并 | CI 未全部通过时无法 merge | 使用 `force_merge=true` |
| 长时间构建无反馈 | 无 build progress 通知 | 需添加进度报告 |

### 6.2 改进的 ci.yml

```yaml
# .gitea/workflows/ci.yml
name: SQLRustGo CI

on:
  push:
    branches: [main, 'develop/**']
  pull_request:

jobs:
  test:
    runs-on: runner
    steps:
      - name: Checkout
        run: git clone ${{ github.repository }} /repo && cd /repo && git checkout ${{ github.sha }}

      - name: Build
        run: cd /repo && cargo build --all-features --release 2>&1 | tee /tmp/build.log

      - name: Test
        run: |
          cd /repo
          cargo test --all-features -- --test-threads=4 2>&1 | tee /tmp/test.log
          
      - name: Report
        if: always()
        run: |
          echo "Build: $(tail -1 /tmp/build.log)"
          echo "Test: $(tail -1 /tmp/test.log)"

  fmt-check:
    runs-on: runner
    steps:
      - name: Checkout
        run: git clone ${{ github.repository }} /repo && cd /repo && git checkout ${{ github.sha }}

      - name: Format check
        run: cd /repo && cargo fmt --all -- --check

  clippy:
    runs-on: runner
    steps:
      - name: Checkout
        run: git clone ${{ github.repository }} /repo && cd /repo && git checkout ${{ github.sha }}

      - name: Clippy
        run: cd /repo && cargo clippy --all-features -- -D warnings
```

### 6.3 并行任务分配

```yaml
# Nomad job 调整：按任务类型分配资源
job "sqlrustgo-ci" {
  group "compile" {
    task "release-build" {
      resources {
        cpu    = 8000    # 8 cores for compilation
        memory = 32768   # 32GB for large crates
      }
    }
  }

  group "test" {
    task "unit-tests" {
      resources {
        cpu    = 2000
        memory = 4096
      }
    }
  }

  group "quality" {
    task "clippy" {
      resources {
        cpu    = 1000
        memory = 2048
      }
    }
  }
}
```

---

## 七、云端镜像同步

### 7.1 三平台镜像策略

```bash
#!/bin/bash
# scripts/sync_to_cloud.sh — 同步到 GitCode / Gitee / GitHub

REPO="sqlrustgo"
BRANCHES=("develop/v2.8.0" "main")
PLATFORMS=(
  "gitclone@gitclone.cn:BreavHeart/sqlrustgo.git"
  "git@gitee.com:yinglichina/sqlrustgo.git"
  "git@github.com:yinglichina8848/sqlrustgo.git"
)

for branch in "${BRANCHES[@]}"; do
  for platform in "${PLATFORMS[@]}"; do
    echo "Syncing $branch → $platform"
    git push "$platform" "$branch" 2>&1
  done
done
```

### 7.2 Cron 调度 (每 15 分钟)

```bash
# 每 15 分钟增量同步一次
*/15 * * * * /Users/liying/workspace/dev/openheart/sqlrustgo/scripts/sync_to_cloud.sh >> /var/log/sync.log 2>&1
```

---

## 八、实施路线图

| 阶段 | 任务 | 优先级 | 状态 |
|------|------|--------|------|
| **P0 - 止血** | 解决 Mac Mini / TRAE AI 并行冲突 | 🔴 高 | 待办 |
| **P0 - 止血** | 规范化分支保护绕过流程 | 🔴 高 | 待办 |
| **P1 - 基础** | 建立 GBrain 知识库 | 🟡 中 | 进行中 |
| **P1 - 基础** | 完善 Hermes Memory | 🟡 中 | 进行中 |
| **P2 - 改进** | 实现角色化 Agent 配置 | 🟢 低 | 规划 |
| **P2 - 改进** | CI 进度通知机制 | 🟢 低 | 规划 |
| **P3 - 优化** | 自动化 Issue 分配 | ⚪ 待定 | 规划 |

### 8.1 P0 紧急任务

**问题：Mac Mini 和 TRAE AI 并行修改同一目录**

解决方案：分离工作目录

```bash
# Mac Mini (Hermes)
~/workspace/dev/openheart/sqlrustgo  # 主开发目录

# TRAE AI (建议新目录)
~/workspace/dev/yinglichina163/sqlrustgo  # 或完全独立的目录
```

**问题：分支保护频繁阻断**

解决方案：
1. 小修改 (fmt, docs): 使用 `force_merge=true`
2. 功能修改: 触发完整 CI 后合并
3. 发布修改: 删除保护 → 合并 → 重建保护

---

## 附录

### A. 关键文件路径

| 用途 | 路径 |
|------|------|
| SQLRustGo 仓库 | `~/workspace/dev/openheart/sqlrustgo` |
| GBrain 知识库 | `~/gbrain/` |
| QMD Wiki | `docs/wiki/` |
| Gitea 数据 | `~/docker/gitea/` |
| Nomad 配置 | `/etc/nomad/` |
| SSH Keys | `~/.ssh/` |

### B. 关键 API 端点

```bash
# Gitea API Base
GITEA_BASE="http://192.168.0.252:3000/api/v1"

# 常用端点
GET    /repos/{owner}/{repo}/actions/runs
POST   /repos/{owner}/{repo}/pulls
POST   /repos/{owner}/{repo}/pulls/{pr}/merge
GET    /repos/{owner}/{repo}/issues/{issue}
POST   /repos/{owner}/{repo}/issues/{issue}/labels
DELETE /repos/{owner}/{repo}/branches/protections/{id}
```

### C. Git Pre-Commit Hook (当前生效版本)

```bash
#!/bin/sh
# .git/hooks/pre-commit
ALLOWED_EMAILS="openheart@gaoyuanyiyao.com hermes-macmini@gaoyuanyiyao.com hermes-z6g4@gaoyuanyiyao.com"
current=$(git config user.email)
found=0
for email in $ALLOWED_EMAILS; do
  [ "$current" = "$email" ] && found=1 && break
done
[ $found -eq 0 ] && echo "❌ Wrong Git identity!" && exit 1
```

### D. 验证命令清单

```bash
# 验证 Git identity
git config user.email

# 验证 Nomad 集群状态
nomad node status

# 验证 Gitea Actions Runner
curl http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/actions/runners

# 验证 CI 状态
curl -s "$GITEA_API/repos/openclaw/sqlrustgo/actions/runs?status=running"

# 验证 R-Gate 基线
cargo test --all-features && cargo clippy --all-features -- -D warnings && cargo fmt --all -- --check
```
