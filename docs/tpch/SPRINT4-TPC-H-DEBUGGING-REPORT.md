# TPC-H SF0.1 Q1-Q22 调试报告

**日期**: 2026-05-03
**分支**: `feat/tpch-csv-import-v2` → `develop/v2.9.0`
**对比基准**: PostgreSQL 15 (Docker `tpch-postgres`, 用户 `tpch`)

---

## 1. 测试结果总览

| Q | PG rows | SQLRustGo | 状态 | 初步判定 |
|---|---------|-----------|------|---------|
| Q1  | 3       | 3         | ✅   | 正确 |
| Q2  | 10      | 20000     | ❌   | LIMIT 未生效 |
| Q3  | 10      | HANG      | ❌   | 执行引擎死锁 |
| Q4  | 5       | 5         | ✅   | 正确 |
| Q5  | 5       | 1         | ❌   | JOIN 结果缩减 |
| Q6  | 1       | 1         | ✅   | 正确 |
| Q7  | 625     | 0         | ❌   | 多表 JOIN 无结果 |
| Q8  | 2       | 0         | ❌   | 多表 JOIN 无结果 |
| Q9  | 175     | 0         | ❌   | 多表 JOIN 无结果 |
| Q10 | 20      | HANG      | ❌   | 执行引擎死锁 |
| Q11 | 10      | 0         | ❌   | 多表 JOIN 无结果 |
| Q12 | 2       | 1         | ❌   | JOIN 条件丢失 |
| Q13 | 10      | 15000     | ❌   | LIMIT 未生效 |
| Q14 | 1       | 0         | ❌   | CASE WHEN 无结果 |
| Q15 | 10000   | 0         | ❌   | 日期范围过滤失效 |
| Q16 | 18314   | 1         | ❌   | GROUP BY 列解析错误 |
| Q17 | 1       | 0         | ❌   | 无 JOIN 结果 |
| Q18 | 10      | 1         | ❌   | LIMIT 未生效 + JOIN 问题 |
| Q19 | 1       | 0         | ❌   | 多表 JOIN 无结果 |
| Q20 | 412     | 1000      | ❌   | nation JOIN 条件丢失 |
| Q21 | 10      | 1         | ❌   | HAVING COUNT>1 未生效 |
| Q22 | 7       | 0         | ❌   | SUBSTRING 条件无效 |

**通过**: 3/22 (Q1, Q4, Q6)
**失败**: 19/22

---

## 2. 根因分类分析

### 类别 A：LIMIT/OFFSET 未生效（4条）

**Q2** (20000行 vs 期望10行)
```sql
-- 标准 Q2 有 ORDER BY s_acctbal DESC LIMIT 10
SELECT ... ORDER BY s_acctbal DESC LIMIT 10
-- SQLRustGo 返回了全部 20000 行（所有满足条件的 part+supplier 组合）
```

**Q13** (15000行 vs 期望10行)
```sql
-- 标准 Q13: GROUP BY 后 LIMIT 10
SELECT c_custkey, COUNT(*) ... GROUP BY c_custkey ORDER BY c_custkey LIMIT 10
-- SQLRustGo 返回了全部 15000 行
```

**Q18** (1行 vs 期望10行)
```sql
-- 标准 Q18: ORDER BY o_totalprice DESC, o_orderdate DESC LIMIT 10
-- SQLRustGo 返回 1 行
```

**Q20** (1000行 vs 期望412行)
```sql
-- 标准 Q20: WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'
SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'
-- SQLRustGo 返回了全部 1000 行 supplier，说明 nation JOIN 条件未生效
```

**根因**: `LIMIT n` 子句可能被忽略或执行引擎未正确应用。

### 类别 B：多表 JOIN 返回 0 行（9条）

**Q5, Q7, Q8, Q9, Q11, Q14, Q17, Q19, Q22**

这些查询都使用了隐式 JOIN（逗号分隔 FROM 子句）或多表 JOIN。

**典型模式**:
```sql
-- Q5: 6表 JOIN
SELECT n_name, SUM(l_extendedprice)
FROM customer, orders, lineitem, supplier, nation, region
WHERE c_custkey = o_custkey
  AND l_orderkey = o_orderkey
  AND l_suppkey = s_suppkey
  AND c_nationkey = s_nationkey
  AND s_nationkey = n_nationkey
  AND n_regionkey = r_regionkey
  AND r_name = 'ASIA'
GROUP BY n_name
```

**可能根因**:
1. 隐式 JOIN 语法（逗号）的解析和执行与 PostgreSQL 不一致
2. 字符串比较可能是 case-sensitive，但数据是小写（如 `'ASIA'` vs `'asia'`）
3. `l_suppkey = s_suppkey` 的类型不匹配
4. nation/region 表的 key 类型与外键类型不一致

**数据验证**（从 PG）:
```
nation: n_nationkey 范围 0-24, r_regionkey 0-4
region: r_name = 'ASIA', 'AMERICA', 'EUROPE', 'AFRICA', 'MIDDLE EAST'
supplier: s_nationkey 对应 nation.n_nationkey
```

### 类别 C：执行引擎死锁（2条）

**Q3** 和 **Q10**

```sql
-- Q3: orders × lineitem 的 JOIN + GROUP BY
SELECT o_orderkey, SUM(l_extendedprice)
FROM orders JOIN lineitem ON o_orderkey = l_orderkey
WHERE o_orderdate < '1995-03-15'
GROUP BY o_orderkey
-- 600K lineitem × 150K orders = 巨大笛卡尔积，无索引情况下 OOM 或死循环

-- Q10: customer × orders × lineitem 三表 JOIN
SELECT c_custkey, SUM(l_extendedprice) ...
FROM customer
JOIN orders ON c_custkey = o_custkey
JOIN lineitem ON o_orderkey = l_orderkey
WHERE o_orderdate >= '1993-10-01'
GROUP BY c_custkey
```

**根因**: JOIN 执行策略是 Nested Loop 时，大表无索引会导致指数级复杂度。

### 类别 D：GROUP BY 语义错误（1条）

**Q16** (1行 vs 18314行)
```sql
SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) AS supplier_cnt
FROM partsupp, part
WHERE p_partkey = ps_partkey
  AND p_brand <> 'Brand#45'
  AND p_type NOT LIKE 'MEDIUM POLISHED%'
  AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9)
GROUP BY p_brand, p_type, p_size
ORDER BY p_brand, p_type, p_size
```

SQLRustGo 返回 1 行，PG 返回 18314 行。可能是 GROUP BY 列解析只取了第一列（p_brand），或 `p_type NOT LIKE` 条件异常。

### 类别 E：其他语义错误（3条）

**Q12** (1行 vs 2行): `l_shipmode IN ('MAIL', 'SHIP')` 和日期范围可能只匹配到一种 shipmode。

**Q14** (0行 vs 1行): `CASE WHEN p_type LIKE 'PROMO%'` 可能 LIKE 语义未实现。

**Q21** (1行 vs 10行): `HAVING COUNT(*) > 1` 未生效，只返回了所有分组。

**Q22** (0行 vs 7行): `SUBSTRING(c_phone FROM 1 FOR 2)` 语法或字符串函数问题。

---

## 3. 下一步调试计划

### P0 - 立即修复
1. **LIMIT 未生效**: 检查执行引擎对 LIMIT 的处理（可能在 Aggregate 或 Project 阶段被丢弃）
2. **GROUP BY 语义**: 检查 parser/executor 对多列 GROUP BY 的处理
3. **HAVING 子句**: HAVING COUNT(*) 未生效

### P1 - 优先修复
4. **JOIN 0 行问题**: 验证字符串比较 case-sensitivity，检查 nationkey/regionkey 类型一致性
5. **SUBSTRING 语法**: 确认 `SUBSTRING(col FROM 1 FOR 2)` 是否正确解析
6. **CASE WHEN**: 验证 LIKE 在 WHEN 条件中的使用

### P2 - 性能优化
7. **Q3/Q10 死锁**: 实现 Hash Join 或排序合并Join，而非 Nested Loop

---

## 4. 环境信息

```
PostgreSQL: Docker tpch-postgres (tpch:tpch@localhost:55432)
SQLite: 3.37.2
SQLRustGo: feat/tpch-csv-import-v2 (c01ecbf8f)
数据: SF0.1 (customer=15000, lineitem=600572, orders=150000, partsupp=80000, part=20000, supplier=1000, nation=25, region=5)
总行数: 866,602
```

---

## 5. 参考：PostgreSQL 标准答案（行数）

```
Q1:  3    | Q2:  10  | Q3:  10  | Q4:  5   | Q5:  5
Q6:  1    | Q7:  625 | Q8:  2   | Q9:  175 | Q10: 20
Q11: 10   | Q12: 2   | Q13: 10  | Q14: 1   | Q15: 10000
Q16: 18314| Q17: 1   | Q18: 10  | Q19: 1   | Q20: 412
Q21: 10   | Q22: 7
```
