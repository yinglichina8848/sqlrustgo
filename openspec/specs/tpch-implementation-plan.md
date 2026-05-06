# TPC-H 真实查询测试 — 实现任务看板

> 本文件是给实施 Agent 的任务清单。请在开始任务前阅读规范文档：
> `openspec/specs/tpch-real-query-testing.spec.md`

---

## 实现顺序

```
Phase 1 ──────→ Phase 2 ─────────→ Phase 3 ─────────→ Phase 4
数据导入支持      COUNT(DISTINCT)    SUBSTRING 实现       22 查询测试
                ↑ 阻塞 Q16         ↑ 阻塞 Q22        ↑ 阻塞待 Phase 2/3
```

---

## Phase 1: 数据层 (数据导入)

### Task 1.1: 创建标准 DDL

**文件**: 新建 `crates/bench/tests/tpch_schema.sql` 或嵌入 `tpch_test.rs`

**内容**: TPC-H 8 表完整 DDL（见 spec 第 1.1 节）

**验证**: 跑 `cargo test --package sqlrustgo-bench --test tpch_test -- tpch_sf01_data_validation`

### Task 1.2: 实现 .tbl 数据导入

**文件**: `crates/bench/tests/tpch_test.rs`

**修改**: 
- 在文件末尾添加 `TpchFixture` struct
- 添加 `setup_tpch()` 函数，读取 `.tbl` 文件并 INSERT
- 参考 `crates/bench-cli/src/tpch_bench.rs` 的 CSV 导入逻辑
- 数据路径从环境变量 `TPCH_DATA_DIR` 读取，默认 `../data/tpch-sf01`

**验证**: `TpchFixture` 能正确导入全部 8 表数据，COUNT(*) 行数匹配

---

## Phase 2: COUNT(DISTINCT) 实现

### Task 2.1: Parser 添加 CountDistinct

**文件**: `crates/parser/src/parser.rs`

**修改**:
```rust
pub enum AggregateFunction {
    Count,
    CountDistinct,  // <-- 新增
    Sum,
    // ...
}
```

在解析 `COUNT(DISTINCT col)` 时映射到 `CountDistinct`。

**注意**: 当前 parser `aggregate.rs` 或 parser.rs 中的 `parse_count_distinct` 可能已有部分逻辑，检查再改。

### Task 2.2: Executor 实现 CountDistinct

**文件**: `crates/executor/src/stored_proc.rs` 或 `crates/executor/src/local_executor.rs`

**修改**: 在聚合计算分支添加：
```rust
AggregateFunction::CountDistinct => {
    let mut seen = HashSet::new();
    for row in rows { /* 收集 distinct 值 */ }
    Value::Integer(seen.len() as i64)
}
```

### Task 2.3: 测试 COUNT(DISTINCT)

**文件**: `crates/executor/tests/test_aggregate.rs`

**取消注释** `test_aggregate_count_distinct` 并补充用例：
- `COUNT(DISTINCT col)` 基本功能
- `COUNT(DISTINCT NULL)` = 0
- `COUNT(DISTINCT col) GROUP BY x`
- 对比 SQLite 结果（如果有 sqlite_diff 测试）

---

## Phase 3: SUBSTRING 实现

### Task 3.1: Parser 确认 SUBSTRING 解析

**文件**: `crates/parser/src/parser.rs`

**检查**: `SUBSTRING(c_phone FROM 1 FOR 2)` 是否被正确解析。默认被解析为 `FunctionCall`。需要决定方案：
- **方案 A**: 添加 `Expression::Substring` 新变体
- **方案 B**: 在 executor 中处理 `FunctionCall` 的 `"SUBSTRING"` 名称

推荐方案 B（改动最小）。

### Task 3.2: Executor 实现 SUBSTRING 评估

**文件**: `crates/executor/src/stored_proc.rs`

在 `eval_expression()` 的 `Expression::FunctionCall` 分支（line 1352）或新增 `Substring` 分支：

```rust
// 在 FunctionCall 处理中添加:
"substring" => {
    // args: [expr, from, len?]
    let val = eval(&args[0]);
    let start = eval(&args[1]).as_i64()? - 1;  // 1-based → 0-based
    let s = val.as_str();
    let result: String = match args.len() {
        2 => s.chars().skip(start).collect(),
        _ => s.chars().skip(start).take(eval(&args[2]).as_i64()? as usize).collect(),
    };
    Value::Text(result)
}
```

### Task 3.3: 测试 SUBSTRING

**文件**: `crates/bench/tests/tpch_test.rs` 或独立测试

```rust
#[test] fn test_substring_basic() { ... }   // → "He"
#[test] fn test_substring_no_len() { ... }  // → "llo"
#[test] fn test_substring_column() { ... }  // SUBSTRING(c_phone FROM 1 FOR 2)
```

---

## Phase 4: 22 查询测试

### Task 4.1: 更新已有测试（按差异大小排序）

从差异小的到差异大的：

1. **Q3, Q4, Q6, Q10** — 几乎与真实一致，只需验证真实数据下通过
2. **Q1** — 扩展 SELECT 列
3. **Q20** — 添加 JOIN（supplier + nation）
4. **Q13** — 使用 LEFT JOIN
5. **Q18** — 扩展 GROUP BY 列
6. **Q22** — 使用 SUBSTRING + 完整查询（依赖 Phase 3）
7. **Q14** — 使用 CASE WHEN
8. **Q2** — 扩展为 5 表 JOIN
9. **Q5** — 扩展为 6 表 JOIN
10. **Q21** — 扩展为 4 表 JOIN

### Task 4.2: 新增缺失查询测试

按复杂度从简到繁：

1. **Q15** — 2 表 JOIN + GROUP BY（最简单）
2. **Q12** — 2 表 JOIN + IN
3. **Q19** — 2 表 JOIN + IN list + BETWEEN
4. **Q11** — 3 表 JOIN + partsupp
5. **Q17** — 2 表 JOIN + 除法
6. **Q7** — 6 表 JOIN + 别名
9. **Q9** — 6 表 JOIN + LIKE + EXTRACT
7. **Q16** — COUNT(DISTINCT) + NOT LIKE（依赖 Phase 2）
8. **Q8** — 8 表 JOIN + EXTRACT（最复杂）

---

## 测试命令

```bash
# 运行全部 TPC-H 测试
cargo test --package sqlrustgo-bench --test tpch_test

# 运行单个测试
cargo test --package sqlrustgo-bench --test tpch_test tpch_q1

# 运行 COUNT(DISTINCT) 测试
cargo test --package sqlrustgo-executor --test test_aggregate test_count_distinct

# 验证编译
cargo check --package sqlrustgo-bench
cargo check --package sqlrustgo-executor
cargo check --package sqlrustgo-parser
```

---

## 完成后验证清单

所有阶段完成后运行：

```bash
# 1. 编译检查
cargo check --all-features

# 2. 全部 TPC-H 测试
cargo test --package sqlrustgo-bench --test tpch_test

# 3. 聚合测试（含 COUNT DISTINCT）
cargo test --package sqlrustgo-executor --test test_aggregate

# 4. Clippy
cargo clippy --all-features -- -D warnings

# 5. 格式
cargo fmt --check --all
```

---

## 回滚方案

如果 Phase 1 数据导入遇阻，回退方案：
- 使用 `setup_engine()` 方式（合成数据），仅扩展 Schema 和查询文本
- 数据导入作为独立任务推迟到 Phase 1 修复后
