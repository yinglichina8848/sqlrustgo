# Brainstorming: SQLRustGo v2.9.0 测试体系重构

> **目标：** 建立统一、分层、可观测的工业级测试体系
> **日期：** 2026-06-03
> **状态基线：** v2.9.0 develop 分支，3630+ 测试用例，覆盖率不可用

---

## 1. 问题陈述

**现状问题：**

| 症状 | 根因推测 |
|------|----------|
| 覆盖率不可用（tarpaulin 全量跑崩溃） | tarpaulin 内存爆炸，SF=0.01 的 TPC-H 也会 OOM |
| 无统一观测系统 | 没有 `cargo-llvm-cov` + merge 机制 |
| CI 不分层 | 每次 push 跑全量，30min+，反馈慢 |
| SQL corpus 存在但未集成 CI | `scripts/run_sql_corpus.sh` 存在但未进 CI |
| test_manifest.yaml 缺失 | 无测试资产清单 |

**影响：**
- 开发者无法知道模块级覆盖率（只知道"跑了没"）
- CI 反馈周期过长（L1 应该是 2min，实际 30min+）
- 覆盖率变化无追踪（PR 改了什么模块，覆盖率升/降多少？不知道）

---

## 2. 5Why 深挖

### 问题：覆盖率系统不可用

**Why 1:** tarpaulin 全量跑会 OOM
- **Why 2:** tarpaulin 需要加载所有 profraw 到内存
  - **Why 3:** 没有 merge 机制，所有数据同时在内存
    - **Why 4:** 架构上假设"单进程完成所有测试"
      - **Why 5:** 没有考虑 instrumented server + 多测试进程的场景

### 问题：CI 无分层

**Why 1:** 所有测试一起跑
- **Why 2:** 没有变更检测（changed crate 识别）
  - **Why 3:** 没有 test_manifest.yaml 记录哪些 crate 有哪些测试
    - **Why 4:** 测试资产没有标准化清单

### 问题：SQL corpus 未进 CI

**Why 1:** scripts/run_sql_corpus.sh 存在但没被 CI 调用
- **Why 2:** CI workflow 没集成 corpus 步骤
  - **Why 3:** 覆盖率采集方案未定（server 模式未验证）

---

## 3. 因果链

```
根因：tarpaulin 架构不适配多进程/instrumented server
    ↓
中间效应：覆盖率数据无法生成
    ↓
最终症状：开发者不知道模块级覆盖率和质量状态
```

```
根因：CI 无分层设计 + 无 test_manifest
    ↓
中间效应：每次 push 跑全量 30min+
    ↓
最终症状：开发者不愿意开 PR（反馈太慢）
```

---

## 4. 控制环合成

| 组件 | 实现 |
|------|------|
| **Sensors（测量）** | `cargo llvm-cov --no-report` + profraw merge → coverage.json |
| **Controllers（决策）** | CI 脚本根据 changed crate 决定跑哪个层级（L1/L2/L3） |
| **Actuators（执行）** | `scripts/ci/test_all.sh` 统一执行入口 |
| **Dashboard（展示）** | `scripts/ci/coverage_dashboard.py` 生成模块级 HTML |

---

## 5. 提议方案

### 5.1 核心架构

```
cargo llvm-cov
      ↓
instrumented server (sqlrustgo-mysql-server)
      ↓
所有测试（unit + corpus + tpch + sysbench）
      ↓
profraw files
      ↓
llvm-profdata merge
      ↓
HTML + JSON + lcov.info + Dashboard
```

### 5.2 工具选型（关键决策）

```diff
- tarpaulin（全量 OOM）
+ cargo-llvm-cov 0.8.x（已安装）+ 统一进程 + merge
```

**理由：** cargo-llvm-cov 支持 `--no-report`（只收集 profraw）+ 事后 merge，内存可控。

### 5.3 测试分层

| 层级 | 触发 | 内容 | 目标时间 |
|------|------|------|----------|
| L1 Quick | push/PR | 变更 crate 的 unit test | <5 min |
| L2 Full | PR → develop | unit + integration + SQL corpus | <30 min |
| L3 Nightly | 定时 | 全量 + coverage + sysbench + TPC-H | 无限制 |

### 5.4 测试注册表

`test_manifest.yaml` 记录每个 crate 的测试数量：

```yaml
version: "2.9.0"
crates:
  parser:        { unit: 72,  integration: 0,  coverage: null }
  executor:      { unit: 141, integration: 0,  coverage: null }
  storage:       { unit: 80,  integration: 0,  coverage: null }
  distributed:   { unit: 75,  integration: 685, coverage: null }
  mysql-server:  { unit: 30,  integration: 48, coverage: null }
totals:
  unit: 680
  integration: 1649
  all: 3630
```

### 5.5 覆盖率输出

```
target/coverage/
├── html/               # 模块级 HTML 报告
├── coverage.json       # JSON 格式（CI 消费）
├── lcov.info          # lcov 格式（CI 系统集成）
└── dashboard.html     # 可视化 dashboard
```

---

## 6. 实施计划

### Phase 1（1天）

| 任务 | 内容 |
|------|------|
| P1.1 | 引入 cargo-llvm-cov，建立 `scripts/ci/test_all.sh` 统一执行 |
| P1.2 | 验证 instrumented server + profraw merge 流程 |
| P1.3 | 输出 coverage.json |
| P1.4 | 验证 L1 Quick Gate（变更 crate unit test） |

### Phase 2（3天）

| 任务 | 内容 |
|------|------|
| P2.1 | 生成 `test_manifest.yaml`（自动统计脚本） |
| P2.2 | 实现 `scripts/ci/coverage_dashboard.py`（模块级 HTML） |
| P2.3 | SQL corpus 自动化集成（进 L2 Gate） |
| P2.4 | L1/L2 CI 分层 workflow |

### Phase 3（1周）

| 任务 | 内容 |
|------|------|
| P3.1 | sysbench 接入 coverage（instrumented server） |
| P3.2 | TPC-H 初版（SF=0.01, Q1/Q3/Q6） |
| P3.3 | coverage gate（模块级阈值，如 mvcc < 50% → BLOCK） |

### Phase 4（2周）

| 任务 | 内容 |
|------|------|
| P4.1 | 全量 TPC-H（22 queries） |
| P4.2 | 混沌工程/稳定性测试 |
| P4.3 | 最终 dashboard + 告警机制 |

---

## 7. 禁止事项

- ❌ tarpaulin 全量跑
- ❌ 每次 CI 跑 sysbench
- ❌ 每次 CI 跑 TPC-H
- ❌ 多套测试系统并存

---

## 8. 开放问题

- [ ] Q1: sysbench 在 container 内运行还是 Nomad job？
- [ ] Q2: TPC-H data generator 放在哪个 crate？
- [ ] Q3: coverage threshold（模块级）如何设定？
- [ ] Q4: L3 Nightly 的 trigger 条件（cron 还是手动）？

---

## 9. 下一步

1. 发布本 Wiki 到 Gitea Wiki
2. 创建 Issue，链接到 Wiki
3. 外部 AI（DeepSeek）分析并给出建议
4. 形成执行计划（Phase 1 → Phase 4）
