# E-08 QPS 基准测试结果报告

## 测试日期
2026-05-05

## 测试环境
- 平台: macOS (Darwin 25.4.0)
- 存储: MemoryStorage
- 数据规模: 1,000 users, 5,000 orders, 100 products

## QPS 基准测试结果

| 操作 | 目标 QPS | 实际 QPS | 达成率 | 状态 |
|------|----------|----------|--------|------|
| Aggregation | - | **195,921** | - | 极佳 |
| ORDER BY | - | **53,539** | - | 良好 |
| **JOIN** | ≥10,000 | **12,617** | 126% | ✅ 达标 |
| **INSERT** | ≥10,000 | **11,545** | 115% | ✅ 达标 |
| Concurrent SELECT | - | **7,620** | - | 良好 |
| Simple SELECT | ≥10,000 | **9,559** | 96% | ⚠️ 接近 |
| Concurrent mixed | - | **3,429** | - | 中等 |
| Complex WHERE | - | **924** | - | 差 |
| **UPDATE** | ≥10,000 | **950** | 9.5% | ❌ 严重不达标 |
| **DELETE** | ≥10,000 | **206** | 2% | ❌ 严重不达标 |

## JOIN QPS 优化效果

Hash Join 优化后：
- **JOIN QPS: 12,617** (目标 ≥10,000) ✅
- 优化前: O(n×m) 嵌套循环
- 优化后: O(n+m) HashMap 查找

## UPDATE/DELETE 问题确认

UPDATE/DELETE 带 WHERE 子句时严重不达标：

| 问题 | 说明 |
|------|------|
| **UPDATE QPS** | 950 (目标 ≥10,000) |
| **DELETE QPS** | 206 (目标 ≥10,000) |
| **根因** | scan-filter-delete-all-reinsert 反模式 |
| **ISSUE** | E-09 (待实施) |

### 详细分析

当前实现的问题：

1. **三倍表扫描**: 找匹配行 → 找保留行 → 实际操作
2. **删除所有行 + 重建**: 效率极低
3. **多次锁获取**: UPDATE 7次, DELETE 6次
4. **未使用 StorageEngine API**: `update/delete` 方法存在但 WHERE 场景未使用

## 下一步

1. **E-09**: 优化 UPDATE/DELETE QPS (Issue #296)
2. 解决 Complex WHERE QPS (924) 问题
3. 提升 Simple SELECT QPS 到 ≥10,000

## 相关文档

- Issue #298: E-08 Step 2 Hash Join 优化
- Issue #296: E-09 UPDATE/DELETE QPS 优化
- PR #299: Hash Join 优化代码
