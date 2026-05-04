# Implementation Plan: 分级 CI/CD 测试系统

> **目标：** 建立 L1/L2/L3 三层分级 Gate + Nomad 分布式执行
> **日期：** 2026-05-04
> **基于：** 2026-06-03 brainstorming + DeepSeek Nomad 架构 + CI 现状审计
> **状态基线：** v2.9.0 develop 分支，3630+ 测试，3 个错误分支的 coverage workflow

---

## 1. 现状总结

### 1.1 现有 Workflows（问题一览）

| 文件 | 触发分支 | 问题 |
|------|----------|------|
| `ci-pr.yml` | PR: v2.8.0/v2.7.0/main | ❌ 无分层，每次全量 ~40min |
| `ci.yml` | push: v2.8.0 | ❌ 无分层，目标分支已废弃 |
| `coverage.yml` | push/pr: **v2.6.0** | ❌ **分支错误** |
| `coverage-parallel.yml` | push/pr: **v2.7.0** | ❌ **分支错误** |
| `formal-smoke-pr.yml` | PR: v2.9.0 | ✅ 分支正确，可作为 L3 组件 |
| `bench-pr.yml` | PR/push: develop/** | ⚠️ 属于 L3，未分层 |
| `sql92-compliance.yml` | ? | ❌ 未检查 |
| `regression.yml` | ? | ❌ 未检查 |

### 1.2 现有测试脚本（未集成 CI）

| 脚本 | 用途 | CI 状态 |
|------|------|---------|
| `scripts/run_sql_corpus.sh` | SQL 语料库 (~500 条) | ❌ 未集成 |
| `scripts/run_tpch_test.sh` | TPC-H 性能 | ❌ 未集成 |
| `scripts/test/run_integration.sh` | 集成测试 | ❌ 未集成 |
| `scripts/test/run-regression.sh` | 回归测试 | ❌ 未集成 |
| `scripts/gate/check_coverage.sh` | 覆盖率门控 | ❌ 未集成 |
| `scripts/verification_engine.py` | Baseline proof | ✅ ci-pr.yml 已集成 |

### 1.3 Nomad 集群现状

| 节点 | 角色 | 状态 |
|------|------|------|
| Z6G4 (192.168.0.252) | Server + Client | ✅ Gitea + Nomad 已运行 |
| Z440 | Client only | ⚠️ 需要确认状态 |

---

## 2. 目标架构

```
PR / push (develop/v2.9.0)
        │
        ▼
┌───────────────────┐
│  L1: QUICK GATE  │  < 5 min
│  changed crate    │  Z440 + Z6G4 (Nomad raw_exec)
│  unit tests       │  分片并行
└───────┬───────────┘
        │ PASS
        ▼
┌───────────────────┐
│ L2: INTEGRATION   │  10-20 min
│ full build        │  Z6G4 (Nomad)
│ integration tests │
│ sql_corpus        │
│ sqlite_diff       │
└───────┬───────────┘
        │ PASS
        ▼
┌───────────────────┐
│  L3: EXTENDED     │  30-60 min
│ formal verification│  Z6G4 (Nomad)
│ coverage (split)  │  按 crate 并行 llvm-cov
│ TPC-H SF=0.1      │
└───────────────────┘

Nightly (cron 0 2 * * *)
        │
        ▼
┌───────────────────┐
│  FULL PIPELINE    │  2 hours
│  all of L3 +      │
│  sysbench +       │
│  TPC-H SF=1       │
│  chaos test       │
└───────────────────┘
```

---

## 3. 实施计划（4 Phase）

### Phase 1: 修复 CI 基础设施（1-2 天）

**目标：** 修复错误的分支配置，建立基础执行框架。

| 任务 | 操作 | 文件 |
|------|------|------|
| P1.1 | 修复 coverage.yml 分支目标 → `develop/v2.9.0` | `.github/workflows/coverage.yml` |
| P1.2 | 修复 coverage-parallel.yml 分支目标 → `develop/v2.9.0` | `.github/workflows/coverage-parallel.yml` |
| P1.3 | 更新 ci.yml 触发分支 → `develop/v2.9.0` | `.github/workflows/ci.yml` |
| P1.4 | 更新 ci-pr.yml 触发分支 → `develop/v2.9.0` | `.github/workflows/ci-pr.yml` |

### Phase 2: 建立 L1/L2/L3 分层（2-3 天）

**目标：** 实现变更感知的分级 Gate。

| 任务 | 操作 | 文件 |
|------|------|------|
| P2.1 | 创建变更检测脚本 `scripts/ci/changed_crates.sh` | `scripts/ci/changed_crates.sh` |
| P2.2 | 创建 L1 Quick workflow | `.github/workflows/ci-l1-quick.yml` |
| P2.3 | 创建 L2 Integration workflow | `.github/workflows/ci-l2-integration.yml` |
| P2.4 | 创建 L3 Extended workflow | `.github/workflows/ci-l3-extended.yml` |
| P2.5 | 创建 Nightly workflow（schedule） | `.github/workflows/nightly.yml` |
| P2.6 | 创建 `scripts/ci/run_l1_tests.sh` | `scripts/ci/run_l1_tests.sh` |
| P2.7 | 创建 `scripts/ci/run_l2_tests.sh` | `scripts/ci/run_l2_tests.sh` |

### Phase 3: Nomad 集成（2-3 天）

**目标：** 将 heavy job 卸载到 Nomad 集群。

| 任务 | 操作 | 文件 |
|------|------|------|
| P3.1 | 创建 Nomad job 定义：regression | `nomad/jobs/regression.nomad.hcl` |
| P3.2 | 创建 Nomad job 定义：coverage（按 crate 分片） | `nomad/jobs/coverage.nomad.hcl` |
| P3.3 | 创建 Nomad job 定义：tpch | `nomad/jobs/tpch.nomad.hcl` |
| P3.4 | 确认 Z440 Nomad client 在线 | — |
| P3.5 | Gitea Actions runner 配置 NOMAD_ADDR | `runner.yaml` |
| P3.6 | 创建 `scripts/ci/submit_nomad_job.sh` | `scripts/ci/submit_nomad_job.sh` |

### Phase 4: 测试资产集成（3-5 天）

**目标：** 将游离的测试脚本纳入 CI。

| 任务 | 操作 | 文件 |
|------|------|------|
| P4.1 | 生成 `test_manifest.yaml`（自动统计） | `scripts/ci/generate_manifest.sh` |
| P4.2 | 集成 `run_sql_corpus.sh` 到 L2 | `scripts/ci/run_sql_corpus.sh`（修改） |
| P4.3 | 创建 coverage dashboard | `scripts/ci/coverage_dashboard.py` |
| P4.4 | 集成 coverage 到 L3（按 crate 分片） | `ci-l3-extended.yml` |
| P4.5 | TPC-H SF=0.1 集成到 L3 | `ci-l3-extended.yml` |
| P4.6 | 设置 coverage threshold per crate | `scripts/gate/check_coverage.sh`（修改） |

---

## 4. 关键文件清单

### 4.1 Workflows（新建 + 修改）

```
.github/workflows/
├── coverage.yml              # 修改：分支 → develop/v2.9.0
├── coverage-parallel.yml     # 修改：分支 → develop/v2.9.0
├── ci.yml                    # 修改：分支 → develop/v2.9.0，保留 Anti-Cheat
├── ci-pr.yml                 # 修改：分支 → develop/v2.9.0，合并到 L1/L2
├── ci-l1-quick.yml          # 新建：L1 Gate
├── ci-l2-integration.yml    # 新建：L2 Gate
├── ci-l3-extended.yml       # 新建：L3 Gate
├── nightly.yml              # 新建：Nightly cron
└── formal-smoke-pr.yml      # 保留（已正确）
```

### 4.2 Scripts（新建 + 修改）

```
scripts/ci/
├── changed_crates.sh        # 新建：检测变更 crate
├── run_l1_tests.sh          # 新建：L1 执行器
├── run_l2_tests.sh          # 新建：L2 执行器
├── run_sql_corpus.sh        # 修改：适配 CI 环境
├── coverage_dashboard.py     # 新建：生成 HTML dashboard
├── generate_manifest.sh     # 新建：生成 test_manifest.yaml
└── submit_nomad_job.sh      # 新建：Nomad job 提交

nomad/jobs/
├── regression.nomad.hcl     # 新建
├── coverage.nomad.hcl       # 新建
└── tpch.nomad.hcl           # 新建
```

### 4.3 配置

```
test_manifest.yaml            # 新建：测试资产清单
```

---

## 5. L1/L2/L3 触发规则

| 事件 | L1 | L2 | L3 | Nightly |
|------|----|----|----|---------|
| push to develop/v2.9.0 | ✅ 跑 | ✅ 跑 | ❌ | ❌ |
| PR to develop/v2.9.0 | ✅ 跑 | ✅ 跑 | ❌ | ❌ |
| PR merge to develop/v2.9.0 | ✅ 跑 | ✅ 跑 | ✅ 跑 | ❌ |
| push to main | ❌ | ❌ | ✅ 跑 | ❌ |
| schedule (cron) | ❌ | ❌ | ❌ | ✅ 跑 |
| workflow_dispatch | ❌ | ❌ | ❌ | ✅ 跑 |

---

## 6. Nomad Job 设计

### 6.1 Regression Job（ephemeral_disk 解决磁盘问题）

```hcl
job "sqlrustgo-regression" {
  datacenters = ["dc1"]
  type        = "batch"

  group "test" {
    count = 4  # 4 分片并行

    ephemeral_disk {
      size  = 5000  # 5GB per task，任务结束自动释放
    }

    task "run" {
      driver = "raw_exec"
      config {
        command = "/bin/bash"
        args    = ["-c", "cd /src && cargo nextest run --workspace --profile ci --partition count:4/${NOMAD_ALLOC_INDEX}"]
      }
      resources {
        cpu    = 4000   # 4 核
        memory = 8192   # 8GB
      }
      template {
        data        = <<EOF
GITHUB_REPO={{ env "attr.user.repo"}}
COMMIT_SHA={{ env "attr.job.version"}}
EOF
        destination = "local/env"
        env         = true
      }
    }
  }
}
```

### 6.2 Coverage Job（按 crate 分片 + 流式上传）

```hcl
job "sqlrustgo-coverage" {
  datacenters = ["dc1"]
  type        = "batch"
  priority    = 50

  group "cov-executor" {
    count = 1
    ephemeral_disk { size = 10000 }

    task "llvm-cov" {
      driver = "raw_exec"
      config {
        command = "/bin/bash"
        args    = ["-c", "cargo llvm-cov --package sqlrustgo-executor --no-report -- --test-threads=4 && curl -X PUT -d @coverage.profraw http://storage:9000/cov/executor.profraw"]
      }
      resources { cpu = 8000; memory = 16384 }
    }
  }
  # 类似 group 重复: cov-parser, cov-planner, cov-optimizer, cov-storage
}
```

---

## 7. 禁止事项（延续 06-03 规则）

- ❌ tarpaulin（全量 OOM）
- ❌ 每次 CI 跑 sysbench
- ❌ 每次 CI 跑 TPC-H 全量
- ❌ 多套测试系统并存（废弃旧的 ci-pr.yml / ci.yml 合并到新分层）
- ❌ coverage 跑在 CI 节点（必须跑在 Nomad）

---

## 8. 验证标准

| 阶段 | 验证条件 |
|------|----------|
| Phase 1 | `coverage.yml` 监听 develop/v2.9.0；所有 workflow 无语法错误 |
| Phase 2 | L1 < 5min 反馈；L2 < 20min；L3 < 60min |
| Phase 3 | Nomad job 成功执行 regression；ephemeral_disk 自动清理 |
| Phase 4 | sql_corpus 在 L2 中运行；coverage dashboard 生成 HTML |

---

## 9. Open Questions（待 External AI 解答）

- [ ] Q1: L1 是否需要覆盖"变更 crate 的下游消费者"测试？（更安全但更慢）
- [ ] Q2: L2 的 sql_corpus 失败阈值如何设定？（全部通过还是允许 known issues）
- [ ] Q3: coverage threshold per crate 如何设定？（历史数据基准还是硬编码）
- [ ] Q4: Z440 Nomad client 确认在线状态（需现场验证）

---

## 10. Next Steps（顺序执行）

1. ✅ CI 现状审计（done）
2. ✅ 分级设计方案（done）
3. ✅ Brainstorming doc 已推送 docs/tiered-ci-cd-brainstorming 分支
4. → External AI 分析（用户操作：分享到 ChatGPT/DeepSeek）
5. → 根据 AI 反馈修订 plan
6. → Phase 1 实施（修复分支配置）
7. → Phase 2 实施（创建分层 workflow）
