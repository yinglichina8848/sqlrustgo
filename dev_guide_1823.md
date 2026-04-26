# v2.8.0 开发执行指南

> **来源**: 基于系统现状分析 + Issue #1785 + Issue #1823
> **编写时间**: 2026-04-24
> **执行代理**: OpenCode (HERMES Gate v0.3)

---

## 一、系统现状

```
main 分支 (v2.5.0 baseline):
  cargo build:    ✅ 通过（mysql-server warnings 可忽略）
  parser tests:   ✅ 79 passed / 1 ignored
  storage tests:  ✅ 163 passed / 0 failed
  executor tests: ❌ 0 tests（无测试文件）
  mysql-server:   ⚠️  编译通过，但 ExecutionEngine 缺失
  trigger:        ✅ 3/3 PASS（memory layer）

治理系统:
  Hermes Gate v0.3 (bootstrap): ✅ 可运行
  contract/v2.8.0.json:          ✅ 存在
  verification/audit report:      ❌ 需生成
```

---

## 二、执行路线图

### Phase 0 — 前置条件（不可绕过）

| 顺序 | 任务 | 文件变更 | 验收标准 |
|------|------|---------|---------|
| **0.1** | **修复 mysql-server 编译残留问题** | `crates/mysql-server/src/lib.rs` | `cargo build -p sqlrustgo-mysql-server` 无 error |
| **0.2** | **合并 Hermes Gate v0.3 PR** | `scripts/hermes/`, `contract/`, `context/` | Gate CI 通过 |
| **0.3** | **生成 baseline verification report** | `docs/versions/v2.8.0/verification_report.json` | 226 passed, baseline_verified=true |

**注意**: Phase 0 不完成，后续所有 PR 的 Gate 会 BLOCK。

---

### Phase 1 — 核心执行器 + 语义正确性（P0）

| 顺序 | 任务 | 依赖 | 文件变更 | 验收标准 |
|------|------|------|---------|---------|
| **1.1** | **实现 MemoryExecutionEngine** | 0.1 | `crates/executor/src/engine.rs` | TCP server 可启动 |
| **1.2** | **实现 NULL 三值逻辑** | 0.1 | `crates/executor/src/` | NULL = NULL -> UNKNOWN, NULL OR true -> UNKNOWN 等 |
| **1.3** | **实现 JOIN + WHERE 执行** | 1.2 | `crates/executor/src/` | LEFT/RIGHT/INNER JOIN 测试通过 |
| **1.4** | **executor NULL 语义测试** | 1.2 | `crates/executor/tests/null_logic.rs` | 20+ 测试用例覆盖三值逻辑 |
| **1.5** | **executor JOIN 测试** | 1.3 | `crates/executor/tests/join.rs` | 20+ 测试用例覆盖 JOIN |

**关键**: 1.1-1.3 必须按顺序完成，互相依赖。

---

### Phase 2 — TCP Server + OLTP（P0）

| 顺序 | 任务 | 依赖 | 文件变更 | 验收标准 |
|------|------|------|---------|---------|
| **2.1** | **TCP listener + 连接处理** | 1.1 | `crates/mysql-server/src/` | 可接受 TCP 连接 |
| **2.2** | **PreparedStatement 完整实现** | 1.1 | `crates/mysql-server/src/` | COM_STMT_EXECUTE 无 panic |
| **2.3** | **Sysbench smoke test** | 2.1 | `scripts/benchmark/` | `sysbench select` 通过 |
| **2.4** | **mysql-server 集成测试** | 2.2 + 1.5 | `crates/mysql-server/tests/` | 端到端 SQL 测试 |

---

### Phase 3 — Trigger 稳定性 + 集成（P1）

| 顺序 | 任务 | 依赖 | 文件变更 | 验收标准 |
|------|------|------|---------|---------|
| **3.1** | **Trigger server 集成验证** | Phase 2 | `crates/storage/src/` | INSERT 触发器在 server 环境工作 |
| **3.2** | **Trigger regression test** | 3.1 | `crates/storage/tests/` | 3 existing tests pass + 5 new |
| **3.3** | **Trigger 性能基线** | 3.1 | `scripts/benchmark/` | 延迟 < 10ms/trigger |

---

### Phase 4 — 测试全覆盖（P1）

| 顺序 | 任务 | 依赖 | 文件变更 | 验收标准 |
|------|------|------|---------|---------|
| **4.1** | **executor 单元测试** | 1.4+1.5 | `crates/executor/tests/` | happy + edge + regression 三类 |
| **4.2** | **OLTP 集成测试套件** | Phase 2 | `tests/oltp/` | 10+ sysbench scenarios |
| **4.3** | **覆盖率 85% CI** | 4.1 | `.github/workflows/` | `make coverage` 自动化 |

---

### Phase 5 — 文档（P1）

| 顺序 | 任务 | 依赖 | 文件变更 | 验收标准 |
|------|------|------|---------|---------|
| **5.1** | **API 文档 90%+** | Phase 1 | 所有 `pub` 函数 | `cargo doc --no-deps` 无警告 |
| **5.2** | **架构图统一** | Phase 2 | `docs/architecture/` | 数据流 + 模块交互图 |
| **5.3** | **ROADMAP.md 更新** | Phase 1-4 | `ROADMAP.md` | v2.8.0 完成状态 |

---

### Phase 6 — v2.8.0 Release

| 顺序 | 任务 | 依赖 | 验收标准 |
|------|------|------|---------|
| **6.1** | **Baseline verification** | Phase 1 完成 | 226+ tests pass, report generated |
| **6.2** | **Tag v2.8.0-alpha** | 6.1 | git tag v2.8.0-alpha |
| **6.3** | **PR 合并 + 发布** | 6.2 | GitHub release |

---

### Phase 7 — 高级功能（P2，可并行）

| 顺序 | 任务 | 依赖 |
|------|------|------|
| 7.1 | 并行查询执行 | Phase 1 |
| 7.2 | OOM/spill 机制 | Phase 1 |
| 7.3 | 分布式集成测试 | Phase 2 + 7.1 |

---

## 三、关键路径（最长链）

```
0.1 → 1.1 → 1.2 → 1.3 → 1.4+1.5 → 2.1 → 2.2 → 2.3 → 2.4 → 6.1 → 6.2 → 6.3
```

**预计**: 约 15 个独立任务，分 5-6 轮 PR 完成。

---

## 四、PR 规范（HERMES Gate v0.3）

每个 PR 必须通过：

```bash
# 1. 本地验证
cargo test
bash scripts/gate/run_hermes_gate.sh

# 2. Push 后 CI 自动检查
# - Hermes Gate (PR hygiene)
# - cargo test (semantic ground truth)
# - verification_engine.py (proof generation)
# - self_audit.py (independent audit)
```

**PR 格式要求**:
- Title: `{module}: {short description} (#ISSUE)`
- Body: 必须包含 `Closes #N` 和测试说明
- Labels: `P0`/`P1`/`P2`

---

## 五、Issue #1785 当前真实状态

```
Task 1 (触发器):
  MemoryStorage::update:  ✅ 已实现
  3 个触发器测试:         ✅ PASS
  coverage --skip 移除:    ❌ scripts/gate/check_coverage.sh 仍存在

Task 2 (OLTP Server):
  ExecutionEngine:        ❌ 缺失（阻塞）
  TCP server:            ❌ 不可用
  Sysbench 集成:         ❌ 未开始
```

**Issue #1785 Task 2 完全阻塞在 Task 1.1 (MemoryExecutionEngine)。**

---

## 六、禁止事项

1. ❌ 不要跳过 Phase 0 直接做 Phase 1
2. ❌ 不要 ignore 测试（宁可缩小数据集 100x）
3. ❌ 不要修改 `tests/` 文件（R1 规则）
4. ❌ 不要在 Gate BLOCK 时强制合并
5. ❌ 不要在 PR 中引入新的 `#[ignore]`（R2 规则）

---

## 七、HERMES Gate v0.3 降级说明

当前为 **bootstrap 模式**：

```
Layer 0: audit    → WARN (skip)
Layer 1: artifact → WARN (skip)
Layer 2: R1-R7   → PASS (deferred to CI)
Layer 3: hygiene  → WARN (no FAIL)
Final: exit 0
```

完整 Gate 需 Phase 0.3 (生成 verification_report) 后自动激活。
