# SQLite Differential Bug Tracker
# 分类标准：
#   - correctness: 已支持功能内有语义/逻辑错误（必须修）
#   - capability: 能力缺失，feature 未实现（排入 roadmap）

## 🟥 correctness（当前 sprint 必须修）

| SQL | SQLite | SQLRustGo | 根因 | 状态 |
|-----|--------|-----------|------|------|
| WHERE 过滤 | filtered rows | wrong rows | 条件求值错误 | OPEN |

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

_按修复顺序填入_
