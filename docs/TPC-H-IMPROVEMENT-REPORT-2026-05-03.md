# TPC-H Testing Improvement Report — 2026-05-03

**项目**: SQLRustGo
**分支**: `feature/multi-table-join-planner`
**日期**: 2026-05-03
**执行**: hermes-agent (HP Z440)

---

## 1. 本次改动摘要

### 1.1 JOIN 执行器修复 (核心)

**文件**: `src/execution_engine.rs` — `hash_inner_join_rows`

**问题**: `hash_inner_join_rows` 只处理 LEFT/FULL 的未匹配左表行，RIGHT JOIN 被当作 INNER JOIN 执行，导致 `test_right_join_basic` 失败。

**根因**: 构建 right_hash 时没有记录 `right_row` 的原始索引，导致无法跟踪哪些右表行已被匹配，也就无法在 RIGHT JOIN 时保留未匹配的右表行。

**修复**:

```rust
// Before: right_hash: HashMap<String, Vec<Vec<Value>>>
let mut right_hash: HashMap<String, Vec<(usize, Vec<Value>)>> = HashMap::new();
//                                            ↑ 记录 (行索引, 行数据)
for (ri, right_row) in right_rows.iter().enumerate() {
    // ...
    right_hash.entry(key).or_default().push((ri, right_row.clone()));
}

// 新增: right_matched 跟踪已匹配的右表行
let mut right_matched: HashSet<usize> = HashSet::new();
for (li, left_row) in left_rows.iter().enumerate() {
    // ...
    if let Some(right_match_rows) = right_hash.get(&key) {
        for (ri, right_row) in right_match_rows {
            right_matched.insert(*ri);  // ← 记录
            // ...
        }
    }
}

// RIGHT / FULL: 保留未匹配的右表行
if matches!(join_type, JoinType::Right | JoinType::Full) {
    for (ri, right_row) in right_rows.iter().enumerate() {
        if !right_matched.contains(&ri) {
            let mut combined = vec![Value::Null; left_col_count];
            combined.extend(right_row.clone());
            matched.push(combined);
        }
    }
}
```

**影响测试**:

| 测试文件 | 测试数 | 结果 |
|----------|--------|------|
| `test_join.rs` | 9 | 9/9 ✅ |
| `join_tests.rs` | 11 | 11/11 ✅ |
| **合计** | **20** | **20/20 ✅** |

**测试用例验证** (`test_right_join_basic`):

```sql
-- t1: (1)  /  t2: (1, 2, 3)
SELECT t1.id, t2.id FROM t1 RIGHT JOIN t2 ON t1.id = t2.id
```

| 阶段 | 结果 |
|------|------|
| 修复前 | 1 行 (INNER join 行为) |
| 修复后 | 3 行 ✅ |

### 1.2 CBO → Executor 闭环确认

**验证**: 所有 join 测试均打印 `[EXEC] join order = ["t1", "t2"]`，旧路径 `[TRACE] OLD_EXECUTE_JOIN` 已完全消失。

---

## 2. TPC-H 测试工具现状

### 2.1 相关文件

| 文件 | 用途 |
|------|------|
| `benches/tpch_bench.rs` | bench-cli TPC-H 基准，含 tpch_csv_import 子命令 |
| `benches/tpch_comprehensive.rs` | 综合 TPC-H 基准框架 |
| `scripts/tpch_comparison.py` | 多引擎 (SQLite/PostgreSQL/SQLRustGo) 对比脚本 |
| `scripts/tpch_comparison.sh` | shell 封装对比脚本 |
| `docs/sprint4/TPC-H-BENCHMARK-REPORT.md` | Sprint 4 报告 (当前 9/22 查询 41%) |

### 2.2 当前覆盖状态

根据 `docs/sprint4/TPC-H-BENCHMARK-REPORT.md`:

- **可运行**: Q1, Q3, Q4, Q6, Q10, Q13, Q14, Q19, Q20, Q22 — 共 9/22 (41%)
- **目标**: ≥18/22 (Sprint 4 要求)
- **数据规模**: SF=1 (lineitem 6M 行, orders 1.5M 行)

### 2.3 `tpch_csv_import` 子命令 (bench-cli)

`benches/tpch_bench.rs` 提供数据导入功能：

```bash
# 导入 SF=0.1 数据
cargo run --bin bench-cli -- tpch csv-import \
  --ddl data/tpch-sf01/tpch.sql \
  --data data/tpch-sf01/ \
  --scale 0.1

# 运行 TPC-H 基准
cargo run --bin bench-cli -- tpch bench \
  --queries Q1,Q3,Q6,Q10 \
  --iterations 3 \
  --sf 0.1
```

---

## 3. Sprint 4 TPC-H 目标差距分析

| 指标 | 当前状态 | Sprint 4 目标 | 差距 |
|------|----------|--------------|------|
| 可运行查询数 | 9/22 (41%) | ≥18/22 (82%) | -9 查询 |
| 性能基线 | 部分建立 | 完整 tpch_baseline.json | 未完成 |
| 性能退化检测 | 无 | 可检测 10%+ 退化 | 未实现 |
| SF=0.1 数据 | 有 | — | ✅ |
| SF=1 数据 | 有 | — | ✅ |

### 3.1 缺失的查询 (13 个)

Q2, Q5, Q7, Q8, Q9, Q11, Q12, Q15, Q16, Q17, Q18, Q21, Q22 中部分已可运行

**主要障碍**:
- Q17, Q18: 相关子查询 (correlated subquery) 复杂
- Q15: CREATE VIEW + DROP VIEW 支持
- Q21: 多表 LEFT JOIN + EXISTS
- 聚合 + 排序 + 窗口函数组合

---

## 4. 下一步工作建议

### 4.1 短期 (1-2 周)

1. **JOIN 修复验证**: 确认 3 表以上 JOIN 的 RIGHT/FULL 组合正确
2. **TPC-H 覆盖率提升**: 从 9/22 → 18/22
   - 优先: Q2, Q5, Q7, Q8, Q9, Q11, Q12
   - 中期: Q15, Q16, Q21
   - 困难: Q17, Q18 (相关子查询)

### 4.2 中期

1. **tpch_baseline.json 完整生成**: 22 个查询在 SF=1 上的性能基线
2. **性能退化检测**: 对比 `develop/v2.9.0` 基线，快照结果入库
3. **跨引擎对比**: SQLRustGo vs PostgreSQL vs SQLite，图表化

---

## 5. 测试命令

```bash
# JOIN 测试 (20/20 全通过)
cd ~/sqlrustgo
cargo test --package sqlrustgo-executor --test test_join
cargo test --package sqlrustgo-executor --test join_tests

# TPC-H bench-cli
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6,Q10 --iterations 3 --sf 0.1

# TPC-H 数据导入
cargo run --bin bench-cli -- tpch csv-import \
  --ddl ~/sqlrustgo/data/tpch-sf01/tpch.sql \
  --data ~/sqlrustgo/data/tpch-sf01/ \
  --scale 0.1
```

---

## 6. 提交记录

```
fix(executor): add RIGHT/FULL JOIN support in hash_inner_join_rows
- Track right_matched HashSet to identify unmatched right rows
- Emit NULL-padded left side for unmatched right rows (RIGHT/FULL)
- Fix test_right_join_basic: t1=(1) RIGHT JOIN t2=(1,2,3) now returns 3 rows
- All 20 join tests pass (9 + 11)
```

---

*Report generated: 2026-05-03 by hermes-agent on HP Z440*
