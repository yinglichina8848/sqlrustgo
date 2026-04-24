# Hermes Gate v0.1

SQLRustGo 可执行规则系统。

## 文件结构

```
scripts/
  hermes/
    rules/
      core.json          # 规则定义（JSON，可解析）
    engine/
      hermes_gate.sh     # 规则执行引擎
    README.md
  gate/
    run_hermes_gate.sh   # CI 入口
```

## 规则（core.json）

| ID | 描述 | 触发条件 | Action |
|----|------|---------|--------|
| `REQUIRE_ISSUE` | PR 必须关联 ISSUE | 所有 PR | FAIL |
| `SQL_SEMANTIC_TEST` | 修改 executor/parser/planner/storage 必须有 NULL + JOIN 测试 | SQL 相关文件变更 | FAIL |
| `ROADMAP_PRIORITY` | 存在未完成 P0 时禁止 P1/P2 PR | PR 含 P1/P2 label | BLOCK |
| `TEST_COMPLETENESS` | 每个 PR 必须包含 happy_path + edge_case + regression 测试 | 非文档变更 | FAIL |

## 执行引擎

```bash
# 直接调用
./scripts/hermes/engine/hermes_gate.sh \
  --pr-body "Closes #1234" \
  --pr-labels "P0" \
  --changed-files "crates/executor/src/lib.rs crates/executor/tests/test_null.rs"

# 输出示例
[Hermes Gate v0.1]
[REQUIRE_ISSUE] PASS
[SQL_SEMANTIC_TEST] PASS
[ROADMAP_PRIORITY] PASS
[TEST_COMPLETENESS] PASS
Final Decision: PASS
```

## CI 集成

```bash
# GitHub Actions
- name: Hermes Gate
  run: ./scripts/gate/run_hermes_gate.sh
  env:
    CI_PR_BODY: "${{ github.event.pull_request.body }}"
    CI_PR_LABELS: "${{ join(github.event.pull_request.labels.*.name, ',') }}"
    CI_BASE_SHA: "${{ github.event.pull_request.base.sha }}"
    CI_PR_SHA: "${{ github.event.pull_request.head.sha }}"
```

```bash
# OpenCode / CLI
./scripts/gate/run_hermes_gate.sh
```

## Exit Codes

| Code | 含义 |
|------|------|
| 0 | PASS — 允许合并 |
| 1 | FAIL — 要求修改 |
| 2 | BLOCK — 阻塞（P0 未完成） |
| 3 | ERROR — 系统错误 |

## 扩展方式

1. **新增规则**：在 `core.json` 的 `rules` 数组添加条目
2. **新条件类型**：在 `hermes_gate.sh` 添加 `eval_*` 函数
3. **Roadmap 集成**：提供 `docs/roadmap.json`（含 P0 状态字段）

## Roadmap.json 格式

```json
{
  "issues": [
    {
      "id": 1823,
      "title": "v2.8.0 发布准备",
      "priority": "P0",
      "status": "open"
    }
  ]
}
```
