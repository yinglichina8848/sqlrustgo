# 递归 CTE 执行链路

> WITH RECURSIVE 递归查询执行原理 - 迭代计算、固定点语义、层次遍历

## 1. CTE 概述

```
┌─────────────────────────────────────────────────────────────┐
│                    CTE 分类                                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   CTE (Common Table Expression)        │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │                                                       │   │
│  │  非递归 CTE (Non-Recursive)                         │   │
│  │  ────────────────────────────                       │   │
│  │  WITH regional_sales AS (                           │   │
│  │      SELECT region, SUM(amount) as total            │   │
│  │      FROM orders GROUP BY region                     │   │
│  │  )                                                  │   │
│  │  SELECT * FROM regional_sales;                      │   │
│  │                                                       │   │
│  │  - 类似视图，只计算一次                              │   │
│  │  - 简单物化                                        │   │
│  │                                                       │   │
│  │  递归 CTE (Recursive)                               │   │
│  │  ────────────────────                               │   │
│  │  WITH RECURSIVE employee_chain AS (                  │   │
│  │      SELECT id, name, manager_id, 1 as depth        │   │
│  │      FROM employees WHERE manager_id IS NULL         │   │
│  │      UNION ALL                                       │   │
│  │      SELECT e.id, e.name, e.manager_id, ec.depth+1 │   │
│  │      FROM employees e                               │   │
│  │      JOIN employee_chain ec ON e.manager_id = ec.id  │   │
│  │  )                                                  │   │
│  │  SELECT * FROM employee_chain;                       │   │
│  │                                                       │   │
│  │  - 自引用，迭代计算                                 │   │
│  │  - 知道没有新行为止                                 │   │
│  │  - 用于层次/图遍历                                 │   │
│  │                                                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 2. 递归 CTE 架构

### 2.1 递归 CTE 执行模型

```
┌─────────────────────────────────────────────────────────────┐
│                 递归 CTE 执行模型                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  递归 CTE 本质上是一个"迭代计算"过程:                       │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                                                       │ │
│  │   初始结果集 (Base)                                  │ │
│  │   ─────────────────                                   │ │
│  │   anchor_query 结果                                   │ │
│  │   例如: 根节点 (manager_id IS NULL)                  │ │
│  │                                                       │ │
│  │            ↓                                          │ │
│  │   ┌─────────────────────────────────────────────┐ │ │
│  │   │           迭代计算 (Recursive)                 │ │ │
│  │   │   ────────────────────────────               │ │ │
│  │   │   1. 用当前结果集作为输入                    │ │ │
│  │   │   2. 执行 recursive_query                    │ │ │
│  │   │   3. 产生新的结果集                          │ │ │
│  │   │   4. UNION ALL 合并到总结果                  │ │ │
│  │   └─────────────────────────────┬───────────────┘ │ │
│  │                                 │                   │ │
│  │   ┌────────────────────────────▼───────────────┐ │ │
│  │   │           固定点检测 (Fixed Point)            │ │ │
│  │   │   ────────────────────────────               │ │ │
│  │   │   新结果集为空？                            │ │ │
│  │   │   是 → 终止，返回总结果                      │ │ │
│  │   │   否 → 回到"迭代计算"                       │ │ │
│  │   └─────────────────────────────────────────────┘ │ │
│  │                                                       │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 递归 CTE 语义

```
┌─────────────────────────────────────────────────────────────┐
│                 递归 CTE 语义                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  WITH RECURSIVE R AS (                                    │
│      anchor_query                                        │
│      UNION ALL                                            │
│      recursive_query referencing R                        │
│  )                                                        │
│  SELECT ... FROM R                                        │
│                                                              │
│  等价于:                                                   │
│                                                              │
│  R₀ = anchor_query()                                      │
│  R₁ = anchor_query() ∪ recursive_query(R₀)                │
│  R₂ = R₁ ∪ recursive_query(R₁)                           │
│  ...                                                       │
│  Rₙ = Rₙ₋₁ ∪ recursive_query(Rₙ₋₁)                     │
│                                                              │
│  固定点: R* = Rₙ 当 recursive_query(Rₙ) ⊆ Rₙ            │
│                                                              │
│  最终结果: R* (如果 Rₙ = Rₙ₋₁, 则 R* = Rₙ)            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. 递归 CTE 执行流程

### 3.1 状态机

```
                ┌─────────────────┐
                │   INIT         │
                │  清空临时结果集 │
                │  max_iter=1000 │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  EXEC ANCHOR   │
                │  计算初始结果   │
                │  放入 work_set │
                └────────┬────────┘
                         │
                ┌────────▼────────┐
                │  ITERATION     │
                │  iteration++   │
                └────────┬────────┘
                         │
              ┌──────────┴──────────┐
              │iteration > max_iter?│
              └──────────┬──────────┘
               Yes               No
               │                 │
     ┌─────────▼────────┐       │
     │  MAX ITERATIONS  │       │
     │   EXCEEDED ERROR │       │
     └──────────────────┘       │
                                │
                   ┌────────────▼────────────┐
                   │    EXEC RECURSIVE       │
                   │  recursive_query(       │
                   │    work_set             │
                   │  )                      │
                   └────────────┬────────────┘
                                │
                   ┌────────────▼────────────┐
                   │    NEW_ROWS EMPTY?     │
                   └────────────┬────────────┘
                     Yes                No
                     │                  │
           ┌─────────▼────────┐        │
           │   FIXED POINT    │        │
           │   RETURN RESULT  │        │
           └──────────────────┘        │
                                     │
                           ┌─────────▼─────────┐
                           │  APPEND NEW_ROWS  │
                           │  TO RESULT        │
                           │  SET work_set =   │
                           │    NEW_ROWS       │
                           └─────────┬─────────┘
                                     │
                                     │
                     ┌───────────────┴───────────────┐
                     │     GO TO ITERATION          │
                     └──────────────────────────────┘
```

### 3.2 执行时序图

```
WITH RECURSIVE org_chart AS (
    -- Anchor: 找到 CEO (manager_id IS NULL)
    SELECT id, name, manager_id, 1 as depth
    FROM employees
    WHERE manager_id IS NULL

    UNION ALL

    -- Recursive: 找到直接下属
    SELECT e.id, e.name, e.manager_id, oc.depth + 1
    FROM employees e
    JOIN org_chart oc ON e.manager_id = oc.id
)
SELECT * FROM org_chart;

执行流程:

ITERATION 0 (Anchor)
───────────────────────────────────────────────────────────────
work_set = { (CEO, depth=1) }
result = { (CEO, depth=1) }

ITERATION 1
───────────────────────────────────────────────────────────────
recursive_query(work_set=CEO):
    SELECT e.id, e.name, e.manager_id, 1+1
    FROM employees e
    JOIN (CEO) ON e.manager_id = CEO.id
    → { (VP1, depth=2), (VP2, depth=2) }

new_rows = { (VP1, depth=2), (VP2, depth=2) }
result = { (CEO), (VP1), (VP2) }
work_set = new_rows

ITERATION 2
───────────────────────────────────────────────────────────────
recursive_query(work_set=VP1, VP2):
    SELECT e.id, e.name, e.manager_id, 2+1
    FROM employees e
    JOIN (VP1, VP2) ON e.manager_id IN (VP1.id, VP2.id)
    → { (MGR1, depth=3), (MGR2, depth=3) }

new_rows = { (MGR1, depth=3), (MGR2, depth=3) }
result = { (CEO), (VP1), (VP2), (MGR1), (MGR2) }
work_set = new_rows

ITERATION 3
───────────────────────────────────────────────────────────────
recursive_query(work_set=MGR1, MGR2):
    SELECT ... WHERE manager_id IN (MGR1, MGR2)
    → { (EMP1, depth=4), (EMP2, depth=4), ... }

new_rows = { (EMP1, depth=4), (EMP2, depth=4), ... }
result = ... (继续追加)
work_set = new_rows

ITERATION N (最后一层)
───────────────────────────────────────────────────────────────
recursive_query(work_set=last_employees):
    SELECT ... WHERE manager_id IN (last_employees)
    → {}  // 没有新员工了

new_rows = {}  // 空!
FIXED POINT REACHED → 终止
```

## 4. 递归查询类型

### 4.1 层次遍历 (Hierarchy Traversal)

```sql
-- 组织架构遍历: 从根到叶
WITH RECURSIVE org_chain AS (
    SELECT id, name, manager_id, name as path
    FROM employees
    WHERE manager_id IS NULL  -- CEO

    UNION ALL

    SELECT e.id, e.name, e.manager_id, oc.path || ' > ' || e.name
    FROM employees e
    JOIN org_chain oc ON e.manager_id = oc.id
)
SELECT * FROM org_chain;

-- 结果:
-- id | name  | manager_id | path
-- ----+-------+------------+---------------------
-- 1  | CEO   | NULL       | CEO
-- 2  | VP1   | 1          | CEO > VP1
-- 3  | VP2   | 1          | CEO > VP2
-- 4  | MGR1  | 2          | CEO > VP1 > MGR1
-- ...
```

### 4.2 路径查找 (Path Finding)

```sql
-- 查找从 A 到 B 的所有路径
WITH RECURSIVE path_finder AS (
    -- Anchor: 起点
    SELECT city_id, city_name, ARRAY[city_id] as route
    FROM cities
    WHERE city_name = 'Beijing'

    UNION ALL

    -- Recursive: 扩展路径
    SELECT c.city_id, c.city_name, pf.route || c.city_id
    FROM cities c
    JOIN path_finder pf ON c.from_city_id = pf.city_id
    WHERE NOT c.city_id = ANY(pf.route)  -- 防止循环
      AND array_length(pf.route, 1) < 10  -- 最大路径长度
)
SELECT route FROM path_finder
WHERE city_name = 'Shanghai';

-- 结果: 所有从 Beijing 到 Shanghai 的路径
-- route
-- --------
-- {1, 5, 10}  -- Beijing -> Zhengzhou -> Shanghai
-- {1, 3, 10}  -- Beijing -> Jinan -> Shanghai
-- ...
```

### 4.3 递归聚合 (Recursive Aggregation)

```sql
-- 计算每个部门的累计销售额
WITH RECURSIVE dept_sales AS (
    -- Anchor: 叶子部门
    SELECT dept_id, SUM(amount) as total
    FROM sales
    GROUP BY dept_id

    UNION ALL

    -- Recursive: 累加子部门销售额
    SELECT d.parent_dept_id, ds.total
    FROM dept_sales ds
    JOIN departments d ON ds.dept_id = d.dept_id
    WHERE d.parent_dept_id IS NOT NULL
)
SELECT dept_id, SUM(total) as cumulative_sales
FROM dept_sales
GROUP BY dept_id;
```

### 4.4 图遍历 (Graph Traversal)

```sql
-- 社交网络: 查找"6度以内"的朋友
WITH RECURSIVE friends_n_degrees AS (
    -- Anchor: 直接好友 (1度)
    SELECT user_id, friend_id, 1 as degree
    FROM friendships
    WHERE user_id = 1

    UNION ALL

    -- Recursive: 朋友的朋友 (2-6度)
    SELECT f.friend_id, f.friend_of_friend, fn.degree + 1
    FROM friendships f
    JOIN friends_n_degrees fn ON f.user_id = fn.friend_id
    WHERE fn.degree < 6
      AND f.friend_id != 1  -- 排除自己
)
SELECT DISTINCT friend_id, MIN(degree) as min_degree
FROM friends_n_degrees
GROUP BY friend_id
ORDER BY min_degree;
```

## 5. 循环检测

### 5.1 循环检测机制

```
┌─────────────────────────────────────────────────────────────┐
│                    循环检测                                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  递归 CTE 可能产生循环:                                      │
│                                                              │
│  员工关系: A→B, B→A (互相汇报)                            │
│  ───────────────────────────────────────                   │
│  A 是 B 的 manager, B 是 A 的 manager                     │
│  简单的递归会无限循环:                                     │
│                                                              │
│  A → B → A → B → A → B → ...                             │
│                                                              │
│  解决方案:                                                  │
│  ─────────                                                  │
│  1. 路径跟踪 (Path Tracking)                               │
│     - 保存访问过的节点路径                                  │
│     - 拒绝已在路径中的节点                                 │
│                                                              │
│  2. 层级限制 (Depth Limit)                                 │
│     - 设置最大递归深度                                      │
│     - 防止无限递归                                        │
│                                                              │
│  3. 延迟合并 (Lazy Evaluation)                             │
│     - 使用生成器/迭代器                                    │
│     - 按需计算                                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 路径跟踪实现

```rust
pub struct RecursiveCTEExecutor {
    max_iterations: usize,
    max_depth: usize,
    track_path: bool,
}

impl RecursiveCTEExecutor {
    pub fn execute(&self, cte: &CTE) -> Result<DataSet> {
        let mut result = DataSet::new();
        let mut work_set = DataSet::new();
        let mut visited_paths: HashSet<Vec<Value>> = HashSet::new();

        // 执行 anchor query
        let anchor_result = self.execute_query(&cte.anchor_query)?;
        for row in anchor_result {
            let path = self.extract_key_path(&row, &cte.recursive_on);
            visited_paths.insert(path.clone());
            work_set.push(row.with_depth(1));
            result.push(row.with_depth(1));
        }

        // 迭代执行递归查询
        let mut iteration = 0;
        while iteration < self.max_iterations {
            iteration += 1;

            let new_rows = self.execute_recursive(&cte.recursive_query, &work_set)?;

            if new_rows.is_empty() {
                break;  // 固定点: 没有新行
            }

            // 添加新行 (带循环检测)
            let mut new_work_set = DataSet::new();
            for row in new_rows {
                let path = self.extract_key_path(&row, &cte.recursive_on);

                if visited_paths.contains(&path) {
                    continue;  // 跳过重复路径
                }

                if path.len() > self.max_depth {
                    continue;  // 超过最大深度
                }

                visited_paths.insert(path);
                let depth = row.depth();
                new_work_set.push(row.clone());
                result.push(row.with_depth(depth));
            }

            if new_work_set.is_empty() {
                break;  // 没有新工作
            }

            work_set = new_work_set;
        }

        Ok(result)
    }

    fn extract_key_path(&self, row: &Row, on_columns: &[ColumnRef]) -> Vec<Value> {
        on_columns.iter().map(|col| row[col].clone()).collect()
    }
}
```

## 6. 执行策略

### 6.1 迭代策略 vs 递归策略

```
┌─────────────────────────────────────────────────────────────┐
│                 执行策略对比                                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  迭代策略 (Iterative)                                      │
│  ─────────────────────                                     │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                                                     │ │
│  │   维护 work_set，每次迭代用整个 work_set            │ │
│  │                                                     │ │
│  │   R₀ = anchor()                                    │ │
│  │   R₁ = R₀ ∪ recursive(R₀)                         │ │
│  │   R₂ = R₁ ∪ recursive(R₁)                         │ │
│  │   ...                                               │ │
│  │                                                     │ │
│  │   优点: 简单实现                                    │ │
│  │   缺点: 每次迭代可能重复处理大量数据               │ │
│  │                                                     │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                              │
│  递归策略 (Recursive / Push-Based)                         │
│  ──────────────────────────────                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                                                     │ │
│  │   使用生成器/迭代器，按需产生新行                   │ │
│  │   新行立即被下游消费                               │ │
│  │                                                     │ │
│  │   优点: 延迟计算，内存效率高                        │ │
│  │   缺点: 实现复杂，需要协程支持                      │ │
│  │                                                     │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 迭代执行实现

```rust
pub struct IterativeRecursiveExecutor {
    max_iterations: usize,
}

impl RecursiveExecutor for IterativeRecursiveExecutor {
    fn execute(&self, cte: &RecursiveCTE) -> Result<DataSet> {
        let mut anchor_result = self.execute_query(&cte.anchor)?;
        let mut result = anchor_result.clone();
        let mut work_set = anchor_result;

        for iteration in 0..self.max_iterations {
            // 执行递归查询，使用当前 work_set
            let recursive_result = self.execute_with_binding(
                &cte.recursive,
                &work_set,
            )?;

            if recursive_result.is_empty() {
                break;  // 固定点
            }

            // 追加到结果
            result.append(recursive_result.clone());
            work_set = recursive_result;
        }

        if iteration == self.max_iterations {
            return Err(Error::MaxIterationsExceeded(self.max_iterations));
        }

        Ok(result)
    }
}
```

## 7. 代价估算

### 7.1 递归 CTE 代价模型

```
┌─────────────────────────────────────────────────────────────┐
│                递归 CTE 代价估算                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  TotalCost = AnchorCost + Σ(IterationCost_i) + FinalCost   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐ │
│  │  AnchorCost:                                        │ │
│  │    执行一次，通常较小                               │ │
│  │                                                     │ │
│  │  IterationCost_i:                                   │ │
│  │    第 i 次迭代的代价                                │ │
│  │    = |work_set_{i-1}| × avg_row_cost               │ │
│  │                                                     │ │
│  │  FinalCost:                                        │ │
│  │    结果合并、去重 (如果有 UNION 而非 UNION ALL)    │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                              │
│  关键因素:                                                  │
│  ─────────                                                  │
│  1. 递归深度 (Iterations)                                 │
│  2. 每层结果数量 (fan-out)                                │
│  3. 连接代价 (join cost)                                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 代价估算公式

```rust
pub fn estimate_recursive_cte_cost(
    cte: &RecursiveCTE,
    stats: &StatsCache,
) -> Cost {
    // Anchor 代价
    let anchor_cost = estimate_query_cost(&cte.anchor, stats);

    // 估算递归深度和每层大小
    let (estimated_depth, rows_per_level) = estimate_recursion_params(cte, stats);

    // 迭代代价
    let mut iteration_cost = Cost::zero();
    let mut work_set_size = anchor_cost.output_rows;

    for depth in 1..=estimated_depth {
        // 每次迭代的 work_set 大小
        let current_work_set = work_set_size;

        // 递归查询代价
        let recursive_cost = estimate_recursive_query_cost(
            &cte.recursive,
            current_work_set,
            stats,
        );

        iteration_cost = iteration_cost.add(&recursive_cost);

        // 更新 work_set 大小 (假设每层递减)
        work_set_size = (work_set_size as f64 * rows_per_level) as u64;
        work_set_size = work_set_size.min(MAX_WORK_SET_SIZE);
    }

    anchor_cost.add(&iteration_cost)
}
```

## 8. 与标准 SQL 的差异

### 8.1 SQL 标准 vs MySQL 实现

| 特性 | SQL 标准 | MySQL | SQLRustGo |
|------|----------|-------|-----------|
| WITH RECURSIVE | ✅ | ✅ | ✅ |
| UNION ALL | ✅ | ✅ | ✅ |
| UNION (自动去重) | ✅ | ✅ | ❌ 当前只支持 UNION ALL |
| MAXRECURSION | ✅ | ✅ | ✅ (max_iterations) |
| 路径跟踪 | ✅ (LATERAL) | ❌ | 部分支持 |
| 循环检测 | ✅ | ❌ | ✅ |

### 8.2 已知限制

```sql
-- 当前 SQLRustGo 不支持的去重
WITH RECURSIVE seq AS (
    SELECT 1 as n
    UNION  -- 自动去重，不会有重复
    SELECT 1
)
SELECT * FROM seq;
-- 结果: 1 (不是 1, 1)

-- 需要去重时，用户需要自己处理
WITH RECURSIVE seq AS (
    SELECT 1 as n
    UNION ALL
    SELECT 1
)
SELECT DISTINCT * FROM seq;
```

## 9. 关键测试用例

### 9.1 基础递归测试

```rust
#[test]
fn test_recursive_cte_hierarchy() {
    // 创建测试数据
    execute("CREATE TABLE org(id INT, name TEXT, manager_id INT)");
    execute("INSERT INTO org VALUES (1, 'CEO', NULL), (2, 'VP1', 1), (3, 'VP2', 1), (4, 'MGR1', 2)");

    // 递归 CTE
    let result = query("
        WITH RECURSIVE chain AS (
            SELECT id, name, manager_id, 1 as depth
            FROM org WHERE manager_id IS NULL
            UNION ALL
            SELECT o.id, o.name, o.manager_id, c.depth + 1
            FROM org o
            JOIN chain c ON o.manager_id = c.id
        )
        SELECT * FROM chain ORDER BY depth, name
    ");

    assert_eq!(result.len(), 4);
    assert_eq!(result[0].name, "CEO");
    assert_eq!(result[1].depth, 2);
}
```

### 9.2 路径查找测试

```sql
WITH RECURSIVE path AS (
    SELECT city_id, city_name, 1 as steps, ARRAY[city_id] as route
    FROM cities WHERE city_name = 'A'

    UNION ALL

    SELECT c.city_id, c.city_name, p.steps + 1, p.route || c.city_id
    FROM cities c
    JOIN path p ON c.from_city_id = p.city_id
    WHERE NOT c.city_id = ANY(p.route)  -- 循环检测
      AND p.steps < 10
)
SELECT * FROM path WHERE city_name = 'D';
```

### 9.3 循环检测测试

```rust
#[test]
fn test_recursive_cycle_detection() {
    // 创建有循环的测试数据
    execute("CREATE TABLE edges(src INT, dst INT)");
    execute("INSERT INTO edges VALUES (1, 2), (2, 1)");  // 1→2→1→2 循环

    // 应该检测到循环并停止
    let result = query("
        WITH RECURSIVE reach AS (
            SELECT src, dst, ARRAY[src] as path
            FROM edges
            WHERE src = 1

            UNION ALL

            SELECT e.src, e.dst, r.path || e.src
            FROM edges e
            JOIN reach r ON e.src = r.dst
            WHERE NOT e.src = ANY(r.path)  -- 循环检测
        )
        SELECT * FROM reach
    ");

    // 应该只返回有限结果，不会无限循环
    assert!(result.len() < 100);
}
```

### 9.4 深度限制测试

```rust
#[test]
fn test_max_depth_limit() {
    // 创建深层结构
    execute("CREATE TABLE chain(id INT, next_id INT)");
    for i in 1..=1000 {
        execute(&format!("INSERT INTO chain VALUES ({}, {})", i, i + 1));
    }

    // 设置最大深度为 100
    let result = query_with_max_depth("
        WITH RECURSIVE seq AS (
            SELECT id, 1 as depth FROM chain WHERE id = 1
            UNION ALL
            SELECT c.id, s.depth + 1
            FROM chain c
            JOIN seq s ON c.id = s.next_id
        )
        SELECT * FROM seq
    ", 100);

    // 应该只返回 100 行
    assert_eq!(result.len(), 100);
    assert_eq!(result.last().depth, 100);
}
```

### 9.5 性能测试

```rust
#[test]
fn test_recursive_performance_large_depth() {
    // 创建深层结构 (10万层)
    create_deep_chain(100000);

    let start = Instant::now();
    let result = query("
        WITH RECURSIVE seq AS (
            SELECT id, 1 as depth FROM chain WHERE id = 1
            UNION ALL
            SELECT c.id, s.depth + 1
            FROM chain c
            JOIN seq s ON c.id = s.next_id
        )
        SELECT COUNT(*) FROM seq
    ");
    let elapsed = start.elapsed();

    // 应该能在合理时间内完成
    assert!(elapsed < Duration::from_secs(5));
    assert_eq!(result[0].count, 100000);
}
```

## 10. 覆盖率差距

| 场景 | 当前覆盖 | 目标覆盖 | 差距 |
|------|---------|---------|------|
| 基础层次遍历 | 90% | 95% | 5% |
| 路径查找 | 75% | 95% | 20% |
| 循环检测 | 80% | 95% | 15% |
| 深度限制 | 85% | 95% | 10% |
| UNION vs UNION ALL | 70% | 90% | 20% |
| 代价估算 | 60% | 85% | 25% |
| 大规模递归 (>1000层) | 65% | 90% | 25% |
| 性能优化 | 55% | 85% | 30% |

## 11. 相关文件

| 文件 | 说明 |
|------|------|
| `executor/src/cteExecutor.rs` | CTE 执行器 |
| `planner/src/ctePlanner.rs` | CTE 查询规划 |
| `parser/src/cte.rs` | CTE 语法解析 |
| `types/src/row.rs` | 行数据类型 |

---

*文档版本: v3.0.0*
*最后更新: 2026-05-11*