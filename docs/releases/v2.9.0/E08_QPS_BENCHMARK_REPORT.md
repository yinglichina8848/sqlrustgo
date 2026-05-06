# E-08/E-09 QPS 基准测试结果报告

## 测试日期
- 初始 (E-08): 2026-05-05
- 更新 (E-09 后): 2026-05-06

## 测试环境
- 平台: macOS (Darwin 25.4.0)
- 存储: MemoryStorage
- 数据规模: 1,000 users, 5,000 orders, 100 products
- Rust: 1.85+

## QPS 基准测试结果（E-09 优化后）

| 操作 | 目标 QPS | 优化前 QPS | **优化后 QPS** | 达成率 | 状态 |
|------|----------|-----------|-------------|--------|------|
| Aggregation | - | 195,921 | **1,643,824** | - | 极佳 |
| ORDER BY | - | 53,539 | **81,988** | - | 良好 |
| **DELETE** | ≥10,000 | 206 | **63,568** | 636% | ✅ 超标 6.4x |
| **JOIN** | ≥10,000 | 12,617 | **57,388** | 574% | ✅ 超标 5.7x |
| **UPDATE** | ≥10,000 | 950 | **43,224** | 432% | ✅ 超标 4.3x |
| **INSERT** | ≥10,000 | 11,545 | **33,377** | 334% | ✅ 超标 3.3x |
| **Simple SELECT** | ≥10,000 | 9,559 | **24,516** | 245% | ✅ 达标 |
| Concurrent SELECT (8t) | - | 7,620 | **11,995** | - | 良好 |
| Complex WHERE | - | 924 | **1,226** | - | 中等 |

## E-09 优化效果

### DELETE/UPDATE 关键突破

E-09 通过 PR #313/#317/#322 实现了**基于索引的原位操作路径**：

**DELETE 优化路径**：
```
WHERE id = ? (id 有主键索引):
  ① try_extract_index_lookup → find_by_index（索引查找）
  ② 验证行匹配
  ③ delete_by_indices（原位删除，无全表扫描）
  结果: 206 → 63,568 QPS (+30,758%)
```

**UPDATE 优化路径**：
```
WHERE id = ? (id 有主键索引):
  ① try_extract_index_lookup → find_by_index（索引查找）
  ② get_table_records_mut → 直接修改 Vec 中的行（原位更新）
  ③ 无 delete-all-reinsert
  结果: 950 → 43,224 QPS (+4,450%)
```

### 全局性能提升

表达式缓存优化（#313, E-09 方案4）对 SELECT/INSERT/JOIN 也产生了额外加速：
- JOIN: 12,617 → 57,388 (+355%)
- INSERT: 11,545 → 33,377 (+189%)
- Simple SELECT: 9,559 → 24,516 (+156%)

### 已知局限

- 非索引 WHERE 条件（如 `WHERE age > 30 AND name LIKE '%x%'`）仍走 fallback 全扫描路径
- FileStorage 的 delete_by_indices 仅支持 MemoryStorage
- Complex WHERE (LIKE) 因无倒排索引仍较慢（1,226 QPS）

## R9 性能基线

已建立 R9 性能回归检测：
- 基线文件: `perf_baselines/v2.9.0/baseline.json`
- 检测脚本: `scripts/gate/check_regression.sh`
- 阈值: ≤5% PASS, 5-20% WARN, >20% FAIL

## 相关文档

- Issue #296: E-09 UPDATE/DELETE QPS 优化 ✅ 已完成
- Issue #298: E-08 Step 2 Hash Join 优化 ✅ 已完成
- PR #313: 表达式缓存优化
- PR #317/#322: 双重扫描消除
- R9 性能基线指南: `docs/releases/v2.9.0/R9_PERFORMANCE_BASELINE_GUIDE.md`
