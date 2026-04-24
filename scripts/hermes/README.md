# Hermes Gate v0.3

**Contract-driven + Commit-bound + Auto-trigger Audit**

SQLRustGo PR 准入控制系统。Gate 读取 `contract/v2.8.0.json` 动态生成 R1-R7 规则，所有 artifact 必须携带当前 HEAD commit SHA。

## 架构

```
Hermes Gate v0.3
│
├─ Layer 0: Auto-trigger audit (if report stale)
│   └─ python3 scripts/self_audit.py
│
├─ Layer 1: Artifact integrity (commit binding)
│   ├─ verification_report.json.commit == HEAD
│   └─ audit_report.json.commit == HEAD
│
├─ Layer 2: Contract rules (R1-R7 from contract/v2.8.0.json)
│   ├─ R1: Test Immutability        → FAIL
│   ├─ R2: Ignore Injection          → FAIL
│   ├─ R3: Proof Authenticity       → BLOCK
│   ├─ R4: Full Execution            → BLOCK
│   ├─ R5: Baseline Verification    → BLOCK
│   ├─ R6: Test Count Monotonicity  → BLOCK
│   └─ R7: CI Workflow Integrity   → WARN
│
└─ Layer 3: Hygiene rules
    ├─ REQUIRE_ISSUE               → FAIL
    ├─ SQL_SEMANTIC_TEST            → FAIL (CI is ground truth)
    ├─ ROADMAP_PRIORITY             → BLOCK
    └─ TEST_COMPLETENESS            → FAIL (CI is ground truth)
```

**Gate 负责结构检查，CI + proof + audit 负责语义正确性。**

## 文件结构

```
scripts/hermes/
  rules/core.json              # v0.3 规则定义（机器可解析）
  engine/hermes_gate.sh       # 执行引擎
scripts/gate/run_hermes_gate.sh  # CI 入口
contract/v2.8.0.json          # 治理契约（规则来源）
docs/versions/v2.8.0/
  verification_report.json    # CI 证明（需含 commit 字段）
  audit_report.json           # 独立审计（需含 commit 字段）
```

## 关键机制

### Commit 绑定（防 stale artifact attack）

所有 artifact 必须携带 commit SHA：

```json
{
  "commit": "4b93ec2d",
  "baseline_verified": true
}
```

Gate 验证：

```bash
REPORT_COMMIT=$(python3 -c "import json; print(json.load(open('verification_report.json'))['commit'])")
HEAD=$(git rev-parse HEAD)
if [ "$REPORT_COMMIT" != "$HEAD" ]; then
  echo "BLOCK: stale artifact"
  exit 2
fi
```

### Auto-trigger Audit

若 `audit_report.json` 不存在或其 `commit != HEAD`，Gate 自动运行：

```bash
python3 scripts/self_audit.py --output docs/versions/v2.8.0/audit_report.json
```

### Contract-driven Rules

R1-R7 规则从 `contract/v2.8.0.json` 解析，不再硬编码。后续规则变更只需修改 contract 文件。

## 执行

```bash
# 手动执行
./scripts/hermes/engine/hermes_gate.sh \
    "Closes #1234" \
    "P0" \
    "crates/executor/src/lib.rs"

# CI 执行
./scripts/gate/run_hermes_gate.sh
```

## Exit Codes

| Code | 含义 |
|------|------|
| 0 | PASS — 所有规则通过 |
| 1 | FAIL — PR 卫生问题，需修改 |
| 2 | BLOCK — 系统状态不允许合并 |
| 3 | WARN — 警告（不阻塞） |
| 4 | ERROR — 系统错误 |

## CI 集成（GitHub Actions）

```yaml
- name: Hermes Gate
  run: ./scripts/gate/run_hermes_gate.sh
  env:
    CI_PR_BODY: "${{ github.event.pull_request.body }}"
    CI_PR_LABELS: "${{ join(github.event.pull_request.labels.*.name, ',') }}"
    CI_BASE_SHA: "${{ github.event.pull_request.base.sha }}"
    CI_PR_SHA: "${{ github.event.pull_request.head.sha }}"
```

**重要**：Gate 必须在 `cargo test` + `self_audit.py` 之后运行。

## 已知限制

- `SQL_SEMANTIC_TEST` / `TEST_COMPLETENESS` 仅文本检查，语义正确性由 CI 验证
- R1-R7 当前通过 bash 实现，未来可迁移到 Python（解析 contract.json）提升鲁棒性
- Commit 绑定依赖 `git rev-parse HEAD`，CI 环境需保证 git 可用

## 扩展方式

1. **新增 hygiene 规则**：编辑 `core.json` 的 `hygiene_rules`
2. **修改 R1-R7**：编辑 `contract/v2.8.0.json` 的 `rules` 节
3. **升级规则引擎**：将 bash 迁移至 Python，解析 `core.json` 动态执行
