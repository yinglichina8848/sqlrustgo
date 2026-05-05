# v2.9.0 门禁规范 (Gate Specification)

> **版本**: 1.2
> **更新日期**: 2026-05-05
> **维护人**: hermes-z6g4
> **适用版本**: v2.9.0+

> **SSOT 声明**: `gate_spec.md` 是 SQLRustGo 门禁定义的唯一权威来源。`RELEASE_POLICY.md`、`RELEASE_LIFECYCLE.md`、`AI_COLLABORATION.md` 等文档中的门禁描述仅作引用，不得独立定义门禁检查项。

---

## 一、门禁概述

v2.9.0 采用四级门禁模型，确保每个发布阶段的质量：

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

同时，R-Gate 内部包含 R1-R10 十项检查：

| 门禁 | 名称 | 目标 | 覆盖率目标 |
|------|------|------|-----------|
| A-Gate | Alpha Gate | 开发完成 | ≥50% |
| B-Gate | Beta Gate | 功能冻结 | ≥75% |
| R-Gate | RC Gate | 发布候选 | ≥75% |
| G-Gate | GA Gate | 正式发布 | ≥85% |

### R1-R10 内部检查项

| Gate | 名称 | 说明 | 证据格式 |
|------|------|------|----------|
| R1 | Build | `cargo build --release --workspace` | `{command, exit_code, artifact_path}` |
| R2 | Test | `cargo test --all-features` | `{command, passed, failed, exit_code}` |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | `{command, warnings, exit_code}` |
| R4 | Format | `cargo fmt --all -- --check` | `{command, diff_count, exit_code}` |
| R5 | Coverage | Per-module: `cargo llvm-cov -p <pkg> --all-features --lib --json` + `scripts/gate/aggregate_coverage.sh` | ≥75% | `{command, module_pcts, artifact_path}` |
| R6 | Security | `cargo audit -d ~/.cargo/advisory-db --no-fetch` | `{command, vulnerabilities, exit_code}` |
| R7 | Docs | `check_docs_links.sh` + R7b + R7c + R7d | `{command, broken_links, missing_docs, version_mismatches}` |
| R8 | SQL Compat | `cargo test -p sql-corpus` | `{command, passed, total, pct, exit_code}` |
| R9 | Performance | `cargo bench && scripts/gate/check_regression.sh` | `{command, baseline_path, delta_pct, pass}` |
| R10 | Formal Proof | Proof Registry with `tool_output` field | `{command, verified_count, tool_output_summary}` |

---

## 二、A-Gate (Alpha Gate)

### 2.1 入口条件

- 所有计划的 feature 已实现
- 核心功能可运行
- 无 P0 Bug

### 2.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| 编译检查 | `cargo build --workspace` | 无错误 |
| 单元测试 | `cargo test --workspace` | ≥80% 通过 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| 文档检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 |

### 2.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥45% |
| optimizer | ≥40% |
| storage | ≥15% |
| catalog | ≥50% |
| parser | ≥50% |
| **整体** | **≥50%** |

---

## 三、B-Gate (Beta Gate)

### 3.1 入口条件

- A-Gate 已通过
- 功能开发完成，进入冻结期
- 无 P0/P1 Bug

### 3.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| 编译检查 | `cargo build --release --workspace` | 无错误 |
| 全量测试 | `cargo test --all-features` | ≥90% 通过 |
| Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| 覆盖率 | `scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all` | ≥75% |
| 形式化证明 | TLA+/Dafny/Formulog | B3 通过 |
| Proof Registry | - | 18/18 verified |

### 3.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥60% |
| optimizer | ≥50% |
| storage | ≥20% |
| catalog | ≥60% |
| parser | ≥60% |
| **整体** | **≥75%** |

---

## 四、R-Gate (RC Gate)

### 4.1 入口条件

- B-Gate 已通过
- 功能冻结，只允许 Bug Fix
- 所有已知 Bug 已修复或延期

### 4.2 R1-R10 检查清单

| Gate | 检查项 | 命令 | 通过标准 | 证据格式 |
|------|--------|------|----------|----------|
| R1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code, artifact_path}` |
| R2 | Test | `cargo test --all-features` | 100% 通过 | `{command, passed, failed, exit_code}` |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{command, warnings, exit_code}` |
| R4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{command, diff_count, exit_code}` |
| R5 | Coverage | `scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all` | ≥75% | `{command, module_pcts, artifact_path}` |
| R6 | Security | `cargo audit -d ~/.cargo/advisory-db --no-fetch` | 无漏洞 | `{command, vulnerabilities, exit_code}` |
| R7 | Docs | `check_docs_links.sh` + R7b + R7c + R7d | 无死链/缺失/版本不一致 | `{command, broken_links, missing_docs, version_mismatches}` |
| R8 | SQL Compat | `cargo test -p sql-corpus` | ≥80% | `{command, passed, total, pct, exit_code}` |
| R9 | Performance | `cargo bench && scripts/gate/check_regression.sh` | 无性能回归 | `{command, baseline_path, delta_pct, pass}` |
| R10 | Formal Proof | Proof Registry with `tool_output` field | ≥10 proof files | `{command, verified_count, tool_output_summary}` |

### 4.3 R7/R9/R10 详细说明

#### R7 文档门禁扩展（v2.9.0+）

R7 包含四个子检查，全部通过才算 R7 通过：

| 子项 | 检查内容 | 命令/方法 |
|------|----------|-----------|
| R7a | 死链检查 | `bash scripts/gate/check_docs_links.sh` |
| R7b | 必选文档存在性 | 检查 `docs/governance/VERSION_DOCS_SPEC.md` 定义的最小文档集 |
| R7c | 版本号一致性 | 所有文档头部版本号与当前版本一致，无遗留旧版本号 |
| R7d | 文档与代码状态一致性 | 代码中标注的 feature 与文档描述匹配，Issue 引用有效 |

#### R9 性能回归检查（v2.9.0+）

v2.9.0 必须执行性能回归检查，不能豁免。

**检查流程**:
1. 运行 `cargo bench` 获取当前性能数据
2. 对比 `perf_baselines/v2.9.0 baseline.json` 中的历史基准
3. 执行 `scripts/gate/check_regression.sh` 自动判定
4. 允许浮动范围: QPS 退化 ≤5% 视为 PASS；5%-20% 需人工解释；>20% FAIL

**基线文件路径**: `perf_baselines/v2.9.0 baseline.json`
**退化判定**: `scripts/gate/check_regression.sh`
**证据要求**: 必须包含 `baseline_path`, `delta_pct`, `pass` 三个字段

> **注意**: 若 `perf_baselines/v2.9.0 baseline.json` 不存在，须在 R-Gate 通过前建立。可使用首次 `cargo bench` 结果作为临时基准，但须标注 `(PROVISIONAL)`。

#### R10 形式化证明 tool_output 要求（v2.9.0+）

每个 Proof JSON 文件必须包含 `tool_output` 字段，记录验证工具的最后运行日志。现有 18 个 proof files 须在 v2.9.0 GA 前完成追溯补充。

**Proof JSON 格式要求**:
```json
{
  "proof_id": "EXE-001",
  "title": "...",
  "status": "verified",
  "tool": "Coq / Lean / TLA+",
  "tool_output": "<工具运行日志摘要，包含验证结果、运行时间、内存使用>",
  "last_verified": "2026-05-05",
  "file_path": "proofs/executor/EXE-001.json"
}
```

**tool_output 最小内容**:
- 验证工具名称和版本
- 验证结果（PASS/FAIL）
- 运行时间
- 内存峰值（可选）

#### R5 覆盖率检查（v2.9.0+）

R5 使用 per-module llvm-cov + aggregate 聚合脚本。**不能使用 `--workspace`**（会混淆不同 crate 的覆盖率数据）。

**检查流程**:
1. `scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all` — 按 wave 并行运行各模块
2. 每个模块生成 `artifacts/coverage/<module>.json`
3. `scripts/gate/aggregate_coverage.sh` — 聚合所有模块 JSON，输出加权平均

**通过标准**: 整体加权平均 ≥75%（B-Gate）或 ≥85%（G-Gate），各模块最低不低于：
- executor ≥75% (R-Gate), ≥80% (G-Gate)
- optimizer ≥60% (R-Gate), ≥70% (G-Gate)
- storage ≥30% (R-Gate), ≥40% (G-Gate)
- catalog ≥70% (R-Gate), ≥75% (G-Gate)
- parser ≥70% (R-Gate), ≥80% (G-Gate)

**关键约束**: `--workspace` 模式覆盖率数据严重失真（不同 crate 数据混合），必须使用 per-module 模式。

### 4.4 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥75% |
| optimizer | ≥60% |
| storage | ≥30% |
| catalog | ≥70% |
| parser | ≥70% |
| **整体** | **≥75%** |

---

## 五、G-Gate (GA Gate)

### 5.1 入口条件

- R-Gate 已通过
- 所有问题已关闭
- 发布审批已获得

### 5.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| Release 构建 | `cargo build --release --workspace` | 无错误 |
| 全量测试 | `cargo test --all-features` | 100% 通过 |
| Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| 覆盖率 | `scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all` | ≥85% |
| 安全扫描 | `cargo audit -d ~/.cargo/advisory-db --no-fetch` | 无漏洞 |
| 性能基准 | `cargo bench` | 无性能回归 |

### 5.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥80% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥75% |
| parser | ≥80% |
| **整体** | **≥85%** |

---

## 六、完整门禁检查脚本

### 6.1 A-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.9.0 A-Gate 检查 ==="

echo "[1/4] 编译检查..."
cargo build --workspace
echo "✅ 编译通过"

echo "[2/4] 测试检查..."
cargo test --workspace
echo "✅ 测试通过"

echo "[3/4] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[4/4] 文档链接检查..."
bash scripts/gate/check_docs_links.sh
echo "✅ 文档检查通过"

echo "=== A-Gate 检查完成 ==="
```

### 6.2 B-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.9.0 B-Gate 检查 ==="

echo "[1/6] Release 编译..."
cargo build --release --workspace
echo "✅ Release 编译通过"

echo "[2/6] 全量测试..."
cargo test --all-features
echo "✅ 全量测试通过"

echo "[3/6] Clippy 检查..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy 通过"

echo "[4/6] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[5/6] 覆盖率检查..."
bash scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all
echo "✅ 覆盖率检查完成"

echo "[6/6] 形式化证明..."
bash scripts/gate/check_proof.sh
echo "✅ 形式化证明通过"

echo "=== B-Gate 检查完成 ==="
```

### 6.3 R-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.9.0 R-Gate 检查 ==="

echo "[1/10] Release 编译 (R1)..."
cargo build --release --workspace
echo "✅ R1 Build 通过"

echo "[2/10] 全量测试 (R2)..."
cargo test --all-features
echo "✅ R2 Test 通过"

echo "[3/10] Clippy 检查 (R3)..."
cargo clippy --all-features -- -D warnings
echo "✅ R3 Clippy 通过"

echo "[4/10] 格式化检查 (R4)..."
cargo fmt --all -- --check
echo "✅ R4 Format 通过"

echo "[5/10] 覆盖率检查 (R5)..."
bash scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all
echo "✅ R5 Coverage ≥75%"

echo "[6/10] 安全扫描 (R6)..."
cargo audit -d ~/.cargo/advisory-db --no-fetch
echo "✅ R6 Security 通过"

echo "[7/10] 文档检查 (R7)..."
bash scripts/gate/check_docs_links.sh
echo "✅ R7 Docs 通过"

echo "[8/10] SQL 兼容性 (R8)..."
bash scripts/gate/check_sql_compat.sh
echo "✅ R8 SQL Compat ≥80%"

echo "[9/10] 性能基准 (R9)..."
cargo bench
scripts/gate/check_regression.sh
echo "✅ R9 Performance 通过"

echo "[10/10] 形式化证明 (R10)..."
bash scripts/gate/check_proof.sh
echo "✅ R10 Formal Proof ≥10 files"

echo "=== R-Gate 检查完成 ==="
```

### 6.4 G-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.9.0 G-Gate 检查 ==="

echo "[1/7] Release 编译..."
cargo build --release --workspace
echo "✅ Release 编译通过"

echo "[2/7] 全量测试..."
cargo test --all-features
echo "✅ 全量测试通过"

echo "[3/7] Clippy 检查..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy 通过"

echo "[4/7] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[5/7] 覆盖率检查..."
bash scripts/gate/check_coverage_parallel.sh --parallel 4 --wave all
echo "✅ 覆盖率 ≥85%"

echo "[6/7] 安全扫描..."
cargo audit -d ~/.cargo/advisory-db --no-fetch
echo "✅ 安全扫描通过"

echo "[7/7] 性能基准测试..."
cargo bench
echo "✅ 性能基准通过"

echo "=== G-Gate 检查完成 ==="
```

---

## 七、门禁状态追踪

### 7.1 各分支门禁要求

| 分支 | 门禁 | 覆盖率目标 | 测试要求 |
|------|------|-----------|----------|
| develop/v2.9.0 | A-Gate | ≥50% | ≥80% |
| alpha/v2.9.0 | B-Gate | ≥75% | ≥90% |
| beta/v2.9.0 | R-Gate | ≥75% | 100% |
| rc/v2.9.0 | G-Gate | ≥85% | 100% |

### 7.2 当前状态 (v2.9.0)

| 门禁 | 状态 | 完成日期 | 备注 |
|------|------|----------|------|
| A-Gate | ✅ 完成 | 2026-05-03 | v2.9.0-alpha |
| B-Gate | ✅ 完成 | 2026-05-04 | hermes_gate + run_hermes_gate PASS, 84.18% |
| R-Gate | 🔄 进行中 | 2026-05-05 | R1-R10 检查进行中 |
| G-Gate | ⚪ 未启动 | TBD | 需 R-Gate 完成 |

---

## 八、门禁豁免规则

以下情况可申请门禁豁免：

| 豁免类型 | 条件 | 审批人 |
|----------|------|--------|
| 覆盖率豁免 | 新增代码可证明难以测试 | Tech Lead |
| 性能豁免 | 性能测试环境不稳定 | QA Lead |
| 文档豁免 | 文档更新不影响功能 | Docs Lead |

---

## 九、相关文档

| 文档 | 说明 |
|------|------|
| [release_process.md](./release_process.md) | 发布流程 |
| [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md) | 版本生命周期 |
| [RC_TO_GA_GATE_CHECKLIST.md](./RC_TO_GA_GATE_CHECKLIST.md) | RC→GA 清单 |
| [GATE_CI_CD.md](./GATE_CI_CD.md) | CI/CD 自动化 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.2 | 2026-05-05 | v2.9.0 对齐: tarpaulin→llvm-cov, R-Gate模块阈值(含parser), R7扩展(R7a-d), R9规范(必须做+回归判定), R10 tool_output要求, 证据格式, SSOT声明 |
| 1.1 | 2026-05-05 | v2.9.0 更新：B-Gate≥75%, R-Gate≥75%, R1-R10 定义 |
| 1.0 | 2026-05-01 | 初始版本，定义 A/B/R/G 四级门禁 |

---

*本文档由 hermes-z6g4 维护。SSOT: gate_spec.md 是门禁唯一权威来源。*
