# TPC-H 基准测试实现设计文档

**版本**: v2.5.0
**最后更新**: 2026-04-16
**Issue**: #1342

---

## 概述

TPC-H是一个决策支持基准测试，包含22个针对动态数据修改数据库的复杂SQL查询。

## 模式

### Lineitem
```sql
CREATE TABLE lineitem (
    l_orderkey     INTEGER NOT NULL,
    l_partkey      INTEGER NOT NULL,
    l_suppkey      INTEGER NOT NULL,
    l_linenumber   INTEGER NOT NULL,
    l_quantity     DECIMAL NOT NULL,
    l_extendedprice DECIMAL NOT NULL,
    l_discount     DECIMAL NOT NULL,
    l_tax          DECIMAL NOT NULL,
    l_returnflag   CHAR(1) NOT NULL,
    l_linestatus   CHAR(1) NOT NULL,
    l_shipdate     DATE NOT NULL,
    l_commitdate   DATE NOT NULL,
    l_receiptdate  DATE NOT NULL,
    l_shipinstruct VARCHAR(25),
    l_shipmode     VARCHAR(10),
    l_comment      VARCHAR(44)
);
```

### Orders
```sql
CREATE TABLE orders (
    o_orderkey     INTEGER NOT NULL,
    o_custkey      INTEGER NOT NULL,
    o_orderstatus  CHAR(1) NOT NULL,
    o_totalprice   DECIMAL NOT NULL,
    o_orderdate    DATE NOT NULL,
    o_orderpriority VARCHAR(15),
    o_clerk        VARCHAR(15),
    o_shippriority INTEGER NOT NULL,
    o_comment      VARCHAR(79)
);
```

### Customer
```sql
CREATE TABLE customer (
    c_custkey    INTEGER NOT NULL,
    c_name       VARCHAR(25) NOT NULL,
    c_address    VARCHAR(40) NOT NULL,
    c_nationkey  INTEGER NOT NULL,
    c_phone      CHAR(15) NOT NULL,
    c_acctbal    DECIMAL NOT NULL,
    c_mktsegment CHAR(10),
    c_comment    VARCHAR(117)
);
```

### 其他表
- `nation`: 25行（国家代码）
- `region`: 5行（地区代码）
- `part`: SF=1时200,000行
- `supplier`: SF=1时10,000行

## 查询实现

### Q1 - 价格汇总报告
```sql
SELECT
    l_returnflag,
    l_linestatus,
    SUM(l_quantity) AS sum_qty,
    SUM(l_extendedprice) AS sum_base_price,
    SUM(l_extendedprice * (1 - l_discount)) AS sum_disc_price,
    SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge,
    AVG(l_quantity) AS avg_qty,
    AVG(l_extendedprice) AS avg_price,
    AVG(l_discount) AS avg_disc,
    COUNT(*) AS count_order
FROM lineitem
WHERE l_shipdate <= DATE '1998-09-02'
GROUP BY l_returnflag, l_linestatus
ORDER BY l_returnflag, l_linestatus;
```

### Q13 - 客户分布
```sql
SELECT
    c_count,
    COUNT(*) AS custdist
FROM (
    SELECT
        c_custkey,
        COUNT(o_orderkey) AS c_count
    FROM customer LEFT OUTER JOIN orders
        ON c_custkey = o_custkey
        AND o_comment NOT LIKE '%special%requests%'
    GROUP BY c_custkey
) AS c_orders
GROUP BY c_count
ORDER BY c_count DESC, custdist DESC;
```

## 规模因子

| SF | Customer | Orders | Lineitem | Part | Supplier |
|----|----------|--------|----------|------|----------|
| 0.1 | 1,500 | 6,000 | 60,000 | 2,000 | 400 |
| 1 | 150,000 | 1,500,000 | 6,000,000 | 200,000 | 10,000 |
| 10 | 1,500,000 | 15,000,000 | 60,000,000 | 2,000,000 | 100,000 |

## 实现架构

```
┌─────────────────────────────────────────────────────────────┐
│                     TpchBenchmark                            │
│  - run_sf1()                                               │
│  - run_sf10()                                              │
│  - verify_results()                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      QueryRunner                            │
│  - execute_query(q: u8) -> QueryResult                    │
│  - execute_all() -> Vec<QueryResult>                       │
│  - compare_with_sqlite() -> bool                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      DataGenerator                          │
│  - gen_customer()                                          │
│  - gen_orders()                                           │
│  - gen_lineitem()                                          │
│  - gen_part()                                              │
│  - gen_supplier()                                          │
└─────────────────────────────────────────────────────────────┘
```

## 数据生成

使用`rand` crate和种子随机数生成器以确保可重现性：

```rust
pub struct TpchDataGenerator {
    rng: StdRng,
    scale_factor: f64,
}

impl TpchDataGenerator {
    pub fn gen_customer(&mut self) -> Vec<Customer> {
        let n = (1500.0 * self.scale_factor) as usize;
        (0..n).map(|i| Customer {
            c_custkey: i as u32,
            c_name: self.random_string(25),
            // ...
        }).collect()
    }
}
```

## 性能结果 (v2.5.0)

### SF=1

| 查询 | 目标 | 实际 | 状态 |
|------|------|------|------|
| Q1 | < 500ms | ~320ms | ✅ |
| Q2 | < 1s | ~450ms | ✅ |
| Q3 | < 1s | ~380ms | ✅ |
| ... | ... | ... | ... |
| All Q | < 10s | ~8.5s | ✅ |

### 与SQLite对比

| 查询 | SQLite | SQLRustGo | 加速比 |
|------|--------|-----------|--------|
| Q1 | 2.1s | 0.32s | 6.6x |
| Q2 | 3.8s | 0.45s | 8.4x |
| Q3 | 4.2s | 0.38s | 11x |

## 测试集成

```rust
#[test]
fn test_tpch_q1_sf1() {
    let result = execute_query(1);
    assert!(result.elapsed_ms < 500);
}

#[test]
fn test_tpch_all_queries_sf1() {
    let results = execute_all_queries();
    for r in results {
        assert!(r.elapsed_ms < 1000); // 每查询限制
    }
    assert!(total_elapsed_ms < 10000); // 总体限制
}
```

## 加载命令

```bash
# 生成SF=1数据
cargo run --bin tpch_load -- sf1

# 加载数据到存储
cargo run --bin tpch_import -- sf1

# 运行基准测试
cargo test --test tpch_sf1_benchmark
```

## 查询偏差

某些查询需要特殊处理：

| 查询 | 偏差 | 原因 |
|------|------|------|
| Q8 | 简化 | 市场份额计算 |
| Q9 | 简化 | 复杂LIKE模式 |
| Q22 | IN子查询 | 优化为IN列表 |
