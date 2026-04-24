# Hermes Gate v0.2

SQLRustGo 可执行规则系统 — PR 规范门禁 + 治理状态验证。

## 架构原则

```
Hermes Gate (PR hygiene, pre-merge)
         ↓ (passes only if hygiene OK)
    CI / cargo test (ground truth)
         ↓
verification_engine.py (proof generation)
         ↓
   self_audit.py (independent audit, detects AV4)
```

**Gate 负责结构检查，Proof + Audit 负责语义正确性。**

Gate 通过 ≠ 系统正确。Gate 失败 = 必须修复。

## 文件结构

```
scripts/
  hermes/
    rules/
      core.json              # 规则定义（v0.2）
    engine/
      hermes_gate.sh         # 规则执行引擎
    README.md
  gate/
    run_hermes_gate.sh       # CI 入口
```

## 规则（core.json）

### Layer 1 — 系统状态（BLOCKING）

| ID | 描述 | 条件 | Action |
|----|------|------|--------|
| `PROOF_VERIFIED` | 系统必须有 VERIFIED baseline | 始终 | BLOCK |
| `AUDIT_TRUSTED` | self-audit 必须返回 TRUSTED | 始终 | BLOCK |

### Layer 2 — PR 卫生（静态检查）

| ID | 描述 | 触发条件 | Action |
|----|------|---------|--------|
| `REQUIRE_ISSUE` | PR 必须关联 ISSUE | 所有 PR | FAIL |
| `SQL_SEMANTIC_TEST` | executor 等修改必须有 NULL + JOIN 测试 | SQL 相关文件变更 | FAIL |
| `ROADMAP_PRIORITY` | P0 未完成时阻塞 P1/P2 PR | PR 含 P1/P2 label | BLOCK |
| `TEST_COMPLETENESS` | 每个 PR 必须有 happy_path + edge_case + regression | 非文档变更 | FAIL |

## 执行

```bash
# 手动执行
./scripts/hermes/engine/hermes_gate.sh \
    "Closes #1234" \
    "P0" \
    "crates/executor/src/lib.rs" \
    docs/versions/v2.8.0/verification_report.json \
    docs/versions/v2.8.0/audit_report.json
```

输出示例（BLOCK）：
```
[PROOF_VERIFIED] BLOCK (verification_report.json not found)
[AUDIT_TRUSTED] BLOCK (audit_report.json not found)
[REQUIRE_ISSUE] PASS
[SQL_SEMANTIC_TEST] PASS
[ROADMAP_PRIORITY] PASS
[TEST_COMPLETENESS] PASS
Final Decision: BLOCK
```

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

## Exit Codes

| Code | 含义 | 说明 |
|------|------|------|
| 0 | PASS | 所有规则通过 |
| 1 | FAIL | PR 卫生问题，需修改 |
| 2 | BLOCK | 系统状态不允许合并 |
| 3 | ERROR | 系统错误（文件缺失等） |

## 扩展方式

1. **新增规则**：在 `core.json` 的 `rules` 数组添加条目
2. **新条件类型**：在 `hermes_gate.sh` 添加 `eval_*` 函数
3. **Proof 路径**：提供 `docs/versions/v2.8.0/verification_report.json`
4. **Audit 路径**：提供 `docs/versions/v2.8.0/audit_report.json`
