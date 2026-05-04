# Brainstorming: 分级 CI/CD 测试系统 (Tiered CI/CD)

## 1. 问题陈述

**现状问题:**

当前 CI 每次 push/PR 都运行**全量测试套件**（~40min+），无分层。导致：
- PR review 反馈慢（等 40min 才知结果）
- 开发迭代节奏被打断
- 资源浪费（改 parser 不需要跑 storage stress test）
- 无变更感知能力，无法实现增量测试

**目标:** 实现 L1/L2/L3 三层分级 Gate，变更驱动执行。

---

## 2. 现有 CI/CD 资产清单

### 2.1 Workflows (`.github/workflows/`)

| 文件 | 触发分支 | 功能 | 问题 |
|------|----------|------|------|
| `ci-pr.yml` | PR: develop/v2.8.0, v2.7.0, main | fmt + clippy + build + unit + integration + security + sqlite-diff + verify | 无分层，每次全量 |
| `ci.yml` | push: develop/v2.8.0 | Anti-Cheat 全量测试 + proof | 无分层 |
| `coverage.yml` | push/pr: develop/v2.6.0, main | 全量 cargo llvm-cov | 目标分支错误 (v2.6.0) |
| `coverage-parallel.yml` | push/pr: develop/v2.7.0, main | 6 个并行 job 按 crate 分组 | 目标分支错误 (v2.7.0) |
| `formal-smoke-pr.yml` | PR: develop/v2.9.0 | Formal verification smoke | ✅ 分支正确 |
| `bench-pr.yml` | PR/push: develop/** | TPC-H SF=0.1 + OLTP + custom | 属于 L3 |
| `benchmark.yml` | schedule: develop/** | 完整 TPC-H SF=1 + OLTP | 属于 L3 |
| `sql92-compliance.yml` | ? | SQL-92 合规测试 | 属于 L2 |
| `regression.yml` | ? | 回归测试 | 属于 L2 |
| `chaos-test-weekly.yml` | schedule | Chaos 测试 | 属于 L3 |

**关键发现:** `coverage.yml` 监听 `develop/v2.6.0`，`coverage-parallel.yml` 监听 `develop/v2.7.0`，**都不是 `develop/v2.9.0`**。

### 2.2 测试脚本 (`scripts/`)

| 脚本 | 用途 | 是否在 CI 中 |
|------|------|-------------|
| `run_sql_corpus.sh` | SQL 语料库测试 (MySQL 兼容性) | ❌ 未集成 |
| `run_tpch_test.sh` | TPC-H 性能测试 | ❌ 未集成 |
| `test/run_integration.sh` | 集成测试 | ❌ 未集成 |
| `test/run-regression.sh` | 回归测试 | ❌ 未集成 |
| `verification_engine.py` | Baseline verification proof | ✅ ci-pr.yml 集成 |
| `gate/check_coverage.sh` | 覆盖率门控 | ❌ 未集成 |

### 2.3 问题根因

1. **无变更检测** — ci-pr.yml 无法感知"改了什么 crate"
2. **无分层 Gate** — 所有 job 都是全量执行
3. **分支配置错误** — coverage workflows 指向旧分支
4. **测试资产游离** — sql_corpus、benchmarks 存在但未进 CI
5. **无 test_manifest** — 没有测试清单描述每个测试的目的和层级

---

## 3. 分级 CI/CD 设计方案

### 3.1 三层 Gate 模型

```
┌──────────────────────────────────────────────────────┐
│                    CI Trigger                          │
│         (push / PR / schedule / manual)               │
└──────────────────┬─────────────────────────────────────┘
                   ▼
┌──────────────────────────────────────────────────────┐
│  L1: QUICK GATE (< 5 min)                            │
│  目标: 快速反馈，变更感知                             │
│  组成:                                                │
│    • fmt + clippy (必须通过)                         │
│    • changed-crates build (只编译变更 crate)         │
│    • changed-crates unit tests (只测变更 crate)       │
│  触发: 所有 PR + push                                 │
│  门控: 必须 PASS，否则 Block PR                       │
└──────────────────┬─────────────────────────────────────┘
                   │ L1 PASS
                   ▼
┌──────────────────────────────────────────────────────┐
│  L2: INTEGRATION GATE (10-20 min)                    │
│  目标: 验证 crate 间接口和集成                        │
│  组成:                                                │
│    • workspace build (全量编译)                      │
│    • integration tests (所有 --test '*')             │
│    • sql_corpus (SQL 语料库, ~500 条 SQL)            │
│    • sqlite_diff (与 SQLite 差异对比)                 │
│    • regression suite (回归测试)                     │
│  触发: PR merge 前 + push to develop/*              │
│  门控: 必须 PASS                                      │
└──────────────────┬─────────────────────────────────────┘
                   │ L2 PASS
                   ▼
┌──────────────────────────────────────────────────────┐
│  L3: EXTENDED GATE (30-60 min)                       │
│  目标: 完整验证 + 性能基准                             │
│  组成:                                                │
│    • formal smoke (形式化验证)                        │
│    • TPC-H (SF=0.1)                                  │
│    • coverage report (llvm-cov 全量)                │
│    • chaos/stress (可选, scheduled)                   │
│  触发: merge to main / release tag / weekly schedule │
│  门控: 必须 PASS                                      │
└──────────────────────────────────────────────────────┘
```

### 3.2 变更检测实现 (L1 核心)

```bash
# 获取变更的 crate 列表
changed_crates() {
  local base="${1:-origin/develop/v2.9.0}"
  git fetch origin "$base"
  git diff --name-only "origin/develop/v2.9.0...HEAD" \
    | grep '^crates/' \
    | cut -d/ -f2 \
    | sort -u
}

# 变更 parser/planner → 只测试这两个 crate
# 变更 executor → 只测试 executor + 直接依赖
```

### 3.3 关键设计决策

| 决策点 | 选项 A | 选项 B | 推荐 |
|--------|--------|--------|------|
| L1 变更检测 | git diff on crates/ | 读取 PR diff | **选项 A** (通用，push 也适用) |
| L1 测试范围 | 只测变更 crate | 测变更 + 直接依赖 | **选项 B** (更安全) |
| L2/L3 触发 | PR merge 后 | push to develop/* | **两者都要** |
| 分层实现 | 单一大 workflow + if conditions | 多 workflow + workflow_call | **选项 B** (可独立管理) |

### 3.4 迁移路径

**Phase 1 (本次):**
- 创建 `ci-l1-quick.yml` — L1 Gate (PR + push 触发)
- 创建 `ci-l2-integration.yml` — L2 Gate (PR merge 后触发)
- 创建 `ci-l3-extended.yml` — L3 Gate (main / release / schedule)
- 修复 `coverage.yml` 和 `coverage-parallel.yml` 的分支目标
- 废弃旧的 `ci-pr.yml` 和 `ci.yml`

**Phase 2:**
- 创建 `test_manifest.yaml` — 测试资产清单
- 创建 `changed_crates_detector.sh` — 变更 crate 检测脚本
- 集成 `run_sql_corpus.sh` 到 L2

**Phase 3:**
- 集成 TPC-H 到 L3
- 集成 chaos test 到 L3
- 添加 coverage threshold per crate

---

## 4. 控制循环

| 层级 | 传感器 (Sensor) | 控制器 (Controller) | 执行器 (Actuator) |
|------|----------------|---------------------|-------------------|
| L1 | git diff changed crates | if-else job 条件 | cargo test -p <crate> |
| L2 | integration test results | pass/fail gate | cargo test --test '*' |
| L3 | coverage %, perf delta | release blocker | cargo llvm-cov, bench-cli |

---

## 5. Open Questions

- [ ] L1 是否需要覆盖"变更 crate 的下游消费者"测试？（更安全但更慢）
- [ ] L2 的 sql_corpus 失败阈值如何设定？（全部通过还是允许个别 known issues？）
- [ ] 是否需要按 crate 设定不同的 coverage threshold？
- [ ] Gitea Actions 是否支持 `workflow_call` 复用 job 定义？

---

## 6. Next Steps

1. ✅ 整理现有 CI 资产清单 (done)
2. 制定分级 CI/CD 设计方案 (done)
3. 发布本文档到 Gitea Wiki
4. 创建 Gitea Issue，附上 Wiki 链接
5. External AI (ChatGPT/DeepSeek) 分析
6. → Implementation plan
