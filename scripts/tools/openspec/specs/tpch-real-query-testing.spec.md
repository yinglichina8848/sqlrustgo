---
id: tpch-real-query-testing
title: TPC-H 真实查询测试规范
version: 1.0
status: draft
created: 2026-05-05
author: openclaw
---

# TPC-H 真实查询测试规范

## 概要

将 SQLRustGo 的 TPC-H 查询测试从简化版升级为接近真实 TPC-H 规格的完整测试体系，包含：
1. 扩展表 Schema 为 TPC-H 标准 8 表，导入真实 `.tbl` 数据
2. 实现缺失功能：`COUNT(DISTINCT)`, `SUBSTRING`
3. 22 个接近真实 TPC-H SQL 的查询测试

## 当前状态

| 维度 | 当前值 |
|------|--------|
| 测试查询覆盖 | 13/22（简化版） |
| 测试数据 | 6000 行合成 INSERT |
| 表 Schema | 精简版本（部分列缺失） |
| 缺失表 | `partsupp` |
| COUNT(DISTINCT) | ❌ 已解析但 executor 返回 NULL |
| SUBSTRING | ❌ 已解析但 executor 返回 NULL |
| CASE WHEN | ✅ 已支持 |
| EXTRACT/YEAR | ✅ 已支持 |
| NOT LIKE | ✅ 已支持 |

## 涉及文件

| 文件 | 用途 |
|------|------|
| `crates/bench/tests/tpch_test.rs` | 主测试文件（23 tests, 536 行） |
| `crates/bench-cli/src/commands/tpch.rs` | 真实 TPC-H 22 查询定义 |
| `crates/bench-cli/src/tpch_bench.rs` | .tbl 数据导入 + DDL 解析逻辑 |
| `crates/executor/src/stored_proc.rs` | 表达式评估（所有 Expression 变体） |
| `crates/executor/tests/test_aggregate.rs` | 聚合测试（COUNT(DISTINCT) 被注释） |
| `crates/parser/src/parser.rs` | SQL 解析（AggregateFunction, Expression） |
| `~/sqlrustgo-data/tpch-sf01/` | TPC-H SF=0.1 真实 .tbl 数据 |

---

## 1. 数据层规范

### 1.1 TPC-H 标准 Schema

创建以下 8 表 SQL DDL，所有列名和类型遵循 TPC-H 规范：

```sql
CREATE TABLE region (
    r_regionkey INTEGER, r_name TEXT, r_comment TEXT
);

CREATE TABLE nation (
    n_nationkey INTEGER, n_name TEXT, n_regionkey INTEGER, n_comment TEXT
);

CREATE TABLE supplier (
    s_suppkey INTEGER, s_name TEXT, s_address TEXT, s_nationkey INTEGER,
    s_phone TEXT, s_acctbal REAL, s_comment TEXT
);

CREATE TABLE customer (
    c_custkey INTEGER, c_name TEXT, c_address TEXT, c_nationkey INTEGER,
    c_phone TEXT, c_acctbal REAL, c_mktsegment TEXT, c_comment TEXT
);

CREATE TABLE part (
    p_partkey INTEGER, p_name TEXT, p_mfgr TEXT, p_brand TEXT, p_type TEXT,
    p_size INTEGER, p_container TEXT, p_retailprice REAL, p_comment TEXT
);

CREATE TABLE partsupp (
    ps_partkey INTEGER, ps_suppkey INTEGER, ps_availqty INTEGER,
    ps_supplycost REAL, ps_comment TEXT
);

CREATE TABLE orders (
    o_orderkey INTEGER, o_custkey INTEGER, o_orderstatus TEXT,
    o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT, o_clerk TEXT,
    o_shippriority INTEGER, o_comment TEXT
);

CREATE TABLE lineitem (
    l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER,
    l_linenumber INTEGER, l_quantity REAL, l_extendedprice REAL,
    l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT,
    l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT,
    l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT
);
```

### 1.2 数据导入方式

使用 bench-cli 现有 `.tbl` 导入能力，数据存于 `~/sqlrustgo-data/tpch-sf01/`。

**数据文件**（管道分隔 .tbl）：
- `region.tbl` (5 行)
- `nation.tbl` (25 行)
- `supplier.tbl`
- `customer.tbl`
- `part.tbl`
- `partsupp.tbl`
- `orders.tbl`
- `lineitem.tbl` (SF=0.1: ~600,000 行)

### 1.3 测试 Fixture

```rust
/// TPC-H 测试环境
struct TpchFixture {
    engine: ExecutionEngine,
}

impl TpchFixture {
    /// 创建并导入 SF=0.1 数据
    fn new(data_dir: &str) -> Self {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let mut engine = ExecutionEngine::new(storage);
        
        // 1. 建表（执行 DDL）
        Self::create_tables(&mut engine, data_dir);
        // 2. 导入 .tbl 数据
        Self::import_data(&mut engine, data_dir);
        
        TpchFixture { engine }
    }
    
    fn query(&mut self, sql: &str) -> Result<Vec<Record>, SqlError> {
        self.engine.execute(sql)
    }
}
```

数据路径通过环境变量 `TPCH_DATA_DIR` 配置，默认路径兼容 CI 环境。

---

## 2. 功能层规范 — COUNT(DISTINCT)

### 2.1 要求

- 解析 `COUNT(DISTINCT col)` SQL 语法 ✅ 已支持
- 在 executor 中计算不重复值的数量
- NULL 值不计入
- 兼容 GROUP BY

### 2.2 实现

在 `crates/parser/src/parser.rs` 中：

```rust
pub enum AggregateFunction {
    Count,
    CountDistinct,  // 新增
    Sum,
    Avg,
    Min,
    Max,
}
```

在 executor 聚合器中使用 `HashSet<Value>` 跟踪已见值，最终返回 `set.len()`。

### 2.3 测试用例

```rust
#[test]
fn test_count_distinct_basic() { /* COUNT(DISTINCT l_returnflag) → 2 */ }
#[test]
fn test_count_distinct_with_null() { /* NULL 不计入 */ }
#[test]
fn test_count_distinct_group_by() { /* GROUP BY + COUNT(DISTINCT) */ }
```

---

## 3. 功能层规范 — SUBSTRING

### 3.1 要求

支持语法：`SUBSTRING(str FROM start FOR length)`

- `start`: 1-based 索引
- `length`: 可选，不指定则到字符串末尾
- 非字符串输入返回 NULL
- 越界处理：返回空字符串或有效子串

### 3.2 实现

解析器将 `SUBSTRING` 解析为 `Expression::Substring` 变体（或作为 `FunctionCall` 处理）。

executor 中添加：

```rust
Expression::Substring(expr, from, len) => {
    let val = eval(expr, row)?;
    let start = eval(from, row)?.as_i64()?;
    let length = len.as_ref().map(|l| eval(l, row)?.as_i64()).transpose()?;
    match val {
        Value::Text(s) => {
            let s = s.chars().skip((start - 1) as usize);
            let s: String = match length {
                Some(n) => s.take(n as usize).collect(),
                None => s.collect(),
            };
            Value::Text(s)
        }
        _ => Value::Null,
    }
}
```

### 3.3 测试用例

```rust
#[test] fn test_substring_basic()    // SUBSTRING('Hello' FROM 1 FOR 2) → "He"
#[test] fn test_substring_no_len()   // SUBSTRING('Hello' FROM 3) → "llo"
#[test] fn test_substring_null()     // NULL 输入 → NULL
#[test] fn test_substring_oob()      // start 越界 → ""
```

---

## 4. 测试层规范 — 22 查询测试

### 4.1 测试方法

所有测试使用 `TpchFixture` + 真实数据，验证：

1. **可执行**: `engine.execute(sql).is_ok()`
2. **有结果**: `rows.len() > 0`
3. **结构正确**: 列数/列名符合预期
4. **数量合理**: COUNT 值在合理范围内

### 4.2 查询对照表

#### Q1 — Pricing Summary Report
```sql
SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag
```
**验证**: 返回 2 行 (R, N)，SUM 值 > 0

#### Q2 — Minimum Cost Supplier
```sql
SELECT s_acctbal, s_name, n_name, p_partkey
FROM part, supplier, partsupp, nation, region
WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15
  AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey
  AND r_name = 'EUROPE'
ORDER BY s_acctbal DESC LIMIT 10
```
**验证**: ≤10 行
**需更新**: 当前测试只有 3 表（无 partsupp, region）

#### Q3 — Shipping Priority
```sql
SELECT o_orderkey, SUM(l_extendedprice)
FROM orders JOIN lineitem ON o_orderkey = l_orderkey
WHERE o_orderdate < '1995-03-15'
GROUP BY o_orderkey
```
**验证**: 已有 ✅

#### Q4 — Order Priority Check
```sql
SELECT o_orderpriority, COUNT(*)
FROM orders
WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01'
GROUP BY o_orderpriority
```
**验证**: 已有 ✅

#### Q5 — Local Supplier Volume
```sql
SELECT n_name, SUM(l_extendedprice)
FROM customer, orders, lineitem, supplier, nation, region
WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey
  AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey
  AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey
  AND r_name = 'ASIA'
GROUP BY n_name
```
**需更新**: 当前测试 3 表，需扩展为 6 表 + region filter

#### Q6 — Discounted Revenue
```sql
SELECT SUM(l_extendedprice) FROM lineitem
WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'
```
**验证**: 已有 ✅

#### Q7 — Shipping Profitability **🆕**
```sql
SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation,
       SUM(l_extendedprice)
FROM supplier, lineitem, orders, customer, nation n1, nation n2
WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey
  AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey
  AND c_nationkey = n2.n_nationkey
GROUP BY n1.n_name, n2.n_name
```
**需验证**: 表别名支持 ✅，6 表 JOIN ✅

#### Q8 — National Market Share **🆕**
```sql
SELECT extract(year FROM o_orderdate) AS o_year,
       SUM(l_extendedprice * (1 - l_discount)) AS volume
FROM part, supplier, lineitem, orders, customer, nation n1, nation n2, region
WHERE p_partkey = l_partkey AND s_suppkey = l_suppkey
  AND l_orderkey = o_orderkey AND o_custkey = c_custkey
  AND c_nationkey = n1.n_nationkey AND n1.n_regionkey = r_regionkey
  AND r_name = 'AMERICA' AND s_nationkey = n2.n_nationkey
  AND o_orderdate >= '1995-01-01' AND o_orderdate <= '1996-12-31'
  AND p_type = 'ECONOMY ANODIZED STEEL'
GROUP BY o_year
```
**关键特性**: EXTRACT(YEAR) ✅

#### Q9 — Product Type Profit **🆕**
```sql
SELECT n_name AS nation, extract(year FROM o_orderdate) AS o_year,
       SUM(l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity) AS amount
FROM part, supplier, lineitem, partsupp, orders, nation
WHERE s_suppkey = l_suppkey AND ps_suppkey = l_suppkey
  AND ps_partkey = l_partkey AND p_partkey = l_partkey
  AND o_orderkey = l_orderkey AND s_nationkey = n_nationkey
  AND p_name LIKE '%green%'
GROUP BY n_name, o_year
```
**关键特性**: LIKE ✅，EXTRACT ✅，多表 ✅

#### Q10 — Returned Item
```sql
SELECT c_custkey, SUM(l_extendedprice)
FROM customer JOIN orders ON c_custkey = o_custkey
JOIN lineitem ON o_orderkey = l_orderkey
WHERE o_orderdate >= '1993-10-01'
GROUP BY c_custkey
```
**验证**: 已有 ✅

#### Q11 — Important Stock **🆕**
```sql
SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value
FROM partsupp, supplier, nation
WHERE ps_suppkey = s_suppkey AND s_nationkey = n_nationkey
  AND n_name = 'GERMANY'
GROUP BY ps_partkey
```
**关键特性**: partsupp 表 ✅

#### Q12 — Shipping Mode **🆕**
```sql
SELECT l_shipmode, COUNT(*)
FROM orders, lineitem
WHERE l_orderkey = o_orderkey
  AND l_shipmode IN ('MAIL', 'SHIP')
  AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate
  AND o_orderdate >= '1993-01-01' AND o_orderdate < '1994-01-01'
GROUP BY l_shipmode
```
**关键特性**: IN ✅，日期比较 ✅

#### Q13 — Customer Orders
```sql
SELECT c_custkey, COUNT(*)
FROM customer LEFT JOIN orders ON c_custkey = o_custkey
GROUP BY c_custkey
```
**需更新**: LEFT JOIN（当前使用 JOIN 替代），添加真实数据后测试

#### Q14 — Promotion Effect
```sql
SELECT SUM(CASE WHEN p_type LIKE 'PROMO%'
           THEN l_extendedprice * (1 - l_discount) ELSE 0 END) AS promo_revenue
FROM lineitem, part
WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01'
  AND l_shipdate < '1995-10-01'
```
**需更新**: 使用 CASE WHEN（已支持）+ date range

#### Q15 — Create View **🆕**
```sql
SELECT s_suppkey, s_name, s_address, s_phone,
       SUM(l_extendedprice * (1 - l_discount)) AS total_revenue
FROM supplier, lineitem
WHERE l_suppkey = s_suppkey
  AND l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01'
GROUP BY s_suppkey, s_name, s_address, s_phone
```
**注意**: TPC-H 标准使用 CREATE VIEW，此处用简化版（直接 SELECT）

#### Q16 — Parts/Supplier **🆕**
```sql
SELECT p_brand, p_type, p_size,
       COUNT(DISTINCT ps_suppkey) AS supplier_cnt
FROM partsupp, part
WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45'
  AND p_type NOT LIKE 'MEDIUM POLISHED%'
  AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9)
GROUP BY p_brand, p_type, p_size
```
**关键特性**: COUNT(DISTINCT) ⚠️ 需要实现，NOT LIKE ✅，IN ✅

#### Q17 — Small-Quantity-Order **🆕**
```sql
SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly
FROM lineitem, part
WHERE p_partkey = l_partkey AND p_brand = 'Brand#23'
  AND p_container = 'MED BOX'
```
**注意**: 标准版使用相关子查询，此处用简化 JOIN 版本

#### Q18 — Large Volume Customer
```sql
SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice,
       SUM(l_quantity)
FROM customer, orders, lineitem
WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey
GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice
```
**需更新**: 完善 GROUP BY 列（当前测试 GROUP BY 列数不足）

#### Q19 — Discounted Revenue **🆕**
```sql
SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue
FROM lineitem, part
WHERE p_partkey = l_partkey AND p_brand = 'Brand#12'
  AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG')
  AND l_quantity >= 1 AND l_quantity <= 11
  AND p_size BETWEEN 1 AND 5
```
**关键特性**: IN list ✅，BETWEEN ✅

#### Q20 — Potential Part Promotion
```sql
SELECT s_name, s_address
FROM supplier, nation
WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'
```
**需更新**: 使用 JOIN（当前无 JOIN 版本）

#### Q21 — Suppliers Who Kept Orders Waiting
```sql
SELECT s_name, COUNT(*) AS numwait
FROM supplier, lineitem, orders, nation
WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey
  AND o_orderstatus = 'F' AND s_nationkey = n_nationkey
  AND n_name = 'SAUDI ARABIA'
GROUP BY s_name
```
**需更新**: 扩展为 4 表（当前 2 表）

#### Q22 — Global Sales Opportunity
```sql
SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode,
       COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal
FROM customer
WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17')
  AND c_acctbal > 0.00
GROUP BY cntrycode
```
**关键特性**: SUBSTRING ⚠️ 需要实现，IN ✅

### 4.3 验证标准

所有测试必须满足：

```
✅ engine.execute(sql) 返回 Ok
✅ result.rows 非空
✅ 列数匹配 SELECT 列表
✅ 数值结果 > 0（聚合查询）
✅ 结果行数在合理范围（1-1000）
```

---

## 5. 验收标准

### 5.1 功能验收

| 标准 | 验证方式 | 优先级 |
|------|---------|--------|
| COUNT(DISTINCT) 正确计算 | COUNT(DISTINCT l_returnflag) = 2 | P0 |
| SUBSTRING 正确切片 | SUBSTRING('Hello' FROM 1 FOR 2) = "He" | P0 |
| LEFT JOIN 支持 | customer LEFT JOIN orders 可执行 | P1 |
| 表别名支持 | nation n1, nation n2 可执行 | P0 |
| EXTRACT(YEAR) 支持 | EXTRACT(YEAR FROM o_orderdate) 返回整数 | P0 |

### 5.2 测试验收

```
cargo test --package sqlrustgo-bench --test tpch_test
```

- 全部 22+ 测试通过
- 无测试超时（单个测试 < 60s）

### 5.3 数据验收

- 8 表全部创建，列名对齐 TPC-H 标准
- SF=0.1 数据完整导入
- COUNT(*) 行数验证：

| 表 | SF=0.1 行数 |
|----|-------------|
| region | 5 |
| nation | 25 |
| supplier | 1,000 |
| customer | 15,000 |
| part | 20,000 |
| partsupp | 80,000 |
| orders | 150,000 |
| lineitem | ~600,000 |

---

## 6. 执行优先级与依赖

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
(数据)     (COUNT      (SUBSTRING   (22 测试)
            DISTINCT)   实现)

依赖关系图:
  Phase 4[Q1-Q15, Q17-Q21] ─── 依赖 Phase 1
  Phase 4[Q16] ──────────────── 依赖 Phase 1 + Phase 2
  Phase 4[Q22] ──────────────── 依赖 Phase 1 + Phase 3
  Phase 1 ───────────────────── 数据导入
  Phase 2 ───────────────────── COUNT(DISTINCT) 实现
  Phase 3 ───────────────────── SUBSTRING 实现
```

### 推荐执行顺序

1. **Phase 1** → 数据层（基础）
2. **Phase 2** → COUNT(DISTINCT)（Q16 阻塞依赖）
3. **Phase 3** → SUBSTRING（Q22 阻塞依赖）
4. **Phase 4** → 22 测试（先写不阻塞的，再写依赖 Phase 2/3 的）

---

## 附录：当前测试 vs 真实查询差异矩阵

| # | 当前测试 SQL | 真实 TPC-H SQL | 与真实的差距 |
|---|-------------|----------------|-------------|
| Q1 | 已接近 | 同左 | 小（需扩展列数） |
| Q2 | 3 表, 无 partsupp/region | 5 表 + ORDER BY + LIMIT | **大** |
| Q3 | 接近 | 同左 | 小 |
| Q4 | 接近 | 同左 | 小 |
| Q5 | 3 表, 无 supplier/nation/region | 6 表 | **大** |
| Q6 | 接近 | 同左 | 小 |
| Q7 | 🆕 无 | 6 表 + alias | — |
| Q8 | 🆕 无 | 8 表 + EXTRACT | — |
| Q9 | 🆕 无 | 6 表 + EXTRACT + LIKE | — |
| Q10 | 接近 | 同左 | 小 |
| Q11 | 🆕 无 | 3 表 + partsupp | — |
| Q12 | 🆕 无 | 2 表 + IN | — |
| Q13 | 简化（无 LEFT JOIN） | LEFT JOIN | **中** |
| Q14 | 简化（无 CASE WHEN） | CASE WHEN + date | **大** |
| Q15 | 🆕 无 | 2 表 + SUM | — |
| Q16 | 🆕 无 | COUNT(DISTINCT) + NOT LIKE | **阻塞** |
| Q17 | 🆕 无 | 2 表 + JOIN | — |
| Q18 | GROUP BY 列不足 | 5 列 GROUP BY | **中** |
| Q19 | 🆕 无 | IN + BETWEEN | — |
| Q20 | 无 JOIN | JOIN + WHERE | **中** |
| Q21 | 2 表 | 4 表 + GROUP BY | **大** |
| Q22 | 无 SUBSTRING | SUBSTRING + GROUP BY | **大** + **阻塞** |
