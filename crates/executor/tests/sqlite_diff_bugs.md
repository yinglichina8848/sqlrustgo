# SQLite Differential Bug Tracker
# 分类标准：
#   - correctness: 已支持功能内有语义/逻辑错误（必须修）
#   - capability: 能力缺失，feature 未实现（排入 roadmap）

## 🟥 correctness（当前 sprint 必须修）

| SQL | SQLite | SQLRustGo | 根因 | 状态 |
|-----|--------|-----------|------|------|
| — | — | — | — | 已清空 |

所有 Layer 1 correctness bugs 已修复。

## 🟨 capability（排入 roadmap，分期做）

| Feature | 复杂度 | 建议优先级 | 状态 |
|---------|--------|-----------|------|
| DISTINCT | 低 | P1 | 待实现 |
| SELECT 1（标量） | 低 | P1 | 待实现 |
| UNION ALL | 中 | P2 | 待实现 |
| GROUP BY | 高 | P3 | 待实现 |
| JOIN | 极高 | P4 | 待实现 |

---

## 修复记录

### v2.9.0 — Projection + ORDER BY 修复 (develop/v2.9.0)

**Root cause**: `execute_select` 从未应用 ORDER BY 和投影。

**Fix**:
1. 增加 Step 4: ORDER BY — `compare_order_by()` + `compare_values_for_order()`
2. 增加 Step 5: PROJECTION — 遍历 `select.columns` 用 `evaluate_expression` 投影
3. LIMIT/OFFSET 改为 Step 6，作用于 `final_rows`

**Tests**: test_order_by_int ✅, test_order_by_text ✅, test_where_projection ✅
