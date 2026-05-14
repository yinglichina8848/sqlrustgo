---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十讲（扩展版）：JOIN实现与Harness优化实战

## 从组合爆炸到生产级Hash Join —— 完整的工程演进

---

# 本讲结构（可拆为4节课）

| 课节 | 内容 | 课时 |
|------|------|------|
| **10-A** | JOIN的组合爆炸问题与工程挑战 | 45分钟 |
| **10-B** | SQLRustGo的JOIN实现：从解析到执行 | 45分钟 |
| **10-C** | Harness检测：性能回归与TPC-H验证 | 45分钟 |
| **10-D** | 优化迭代与Harness自改进闭环 | 45分钟 |

---

# 10-A: JOIN的组合爆炸问题与工程挑战

---

## A.1 先理解：什么是JOIN

### 一个最简单的例子

```
有两张表：

 orders 表（订单）              customers 表（客户）
 ┌──────┬──────────┬───────┐   ┌──────┬──────┐
 │ id   │ customer │ amount│   │ id   │ name │
 ├──────┼──────────┼───────┤   ├──────┼──────┤
 │ 1    │ 101      │ 500   │   │ 101  │ 张三  │
 │ 2    │ 102      │ 300   │   │ 102  │ 李四  │
 │ 3    │ 101      │ 200   │   │ 103  │ 王五  │
 └──────┴──────────┴───────┘   └──────┴──────┘

问题：列出所有订单，并显示客户姓名

SQL：
  SELECT orders.id, customers.name, orders.amount
  FROM orders
  JOIN customers ON orders.customer = customers.id

结果：
  ┌──────┬──────┬────────┐
  │ id   │ name │ amount │
  ├──────┼──────┼────────┤
  │ 1    │ 张三  │ 500    │
  │ 2    │ 李四  │ 300    │
  │ 3    │ 张三  │ 200    │
  └──────┴──────┴────────┘
```

### JOIN 的本质

```
JOIN = 把两张表按某个条件"连起来"

SQLRustGo 支持的 JOIN 类型：
  INNER JOIN    → 只保留两表都有匹配的行
  LEFT JOIN     → 保留左表所有行（右表没有匹配的填 NULL）
  RIGHT JOIN    → 保留右表所有行
  FULL JOIN     → 两表所有行都保留
  CROSS JOIN    → 笛卡尔积（每行 × 每行）
  LEFT SEMI     → 左表中"在右表有匹配"的行
  LEFT ANTI     → 左表中"在右表没有匹配"的行
```

---

## A.2 核心问题：组合爆炸

### 两张表的 JOIN —— 简单

```
orders (100万行) JOIN customers (1000行) ON customer = id

直观想法（Nested Loop）：
  for order in orders:          ← 100万次外层循环
    for customer in customers:  ← 每次1000次内层循环
      if match: output

总比较次数：100万 × 1000 = 10亿次！

即使每次比较只需要 1微秒，也需要 1000秒 ≈ 17分钟
这显然不可接受。
```

### 三张表的 JOIN —— 更复杂

```
orders JOIN customers JOIN products

不仅是"算法效率"的问题，
还涉及"先 JOIN 谁、后 JOIN 谁"的选择：

选项1：(orders JOIN customers) JOIN products
选项2：orders JOIN (customers JOIN products)
选项3：(orders JOIN products) JOIN customers

三种顺序的计算量可能相差数十倍！
```

### 组合爆炸的数学本质

```
假设有 N 张表需要 JOIN

可能的 JOIN 顺序数量：
  N=2  →  2 种
  N=3  →  12 种
  N=4  →  120 种
  N=5  →  1,680 种
  N=6  →  30,240 种
  N=10 →  1.7 × 10^10 种（170亿！）

这是经典的"NP-hard"问题
所有数据库都会遇到，都需要"启发式"近似解
```

---

## A.3 三方面的工程挑战

### 挑战1：算法选择（怎么JOIN？）

```
三种经典 JOIN 算法，各有优劣：

1. Nested Loop Join（嵌套循环）
   原理：双层循环，逐行比较
   优点：实现简单，对小表友好
   缺点：O(m×n)，大表场景炸裂
   适合：一张表很小（<100行）

2. Hash Join（哈希连接）
   原理：建哈希表 → 探测匹配
   优点：O(m+n)，大表场景高效
   缺点：需要额外内存建哈希表
   适合：等值连接（ON a.x = b.y）

3. Sort-Merge Join（排序归并连接）
   原理：先排序 → 再归并
   优点：适合范围连接（ON a.x > b.y）
   缺点：排序本身就有成本
   适合：数据已经有序，或需要范围比较
```

### 挑战2：顺序选择（先JOIN谁？）

```
三张表 JOIN：orders (100万), customers (1000), products (10万)

方案A：(orders ⋈ customers) ⋈ products
  第一步：100万 × 1000 = 10亿次比较 → 产生 100万行中间结果
  第二步：100万 × 10万 = 1000亿次比较
  代价：巨大！

方案B：orders ⋈ (customers ⋈ products)
  第一步：1000 × 10万 = 1亿次比较 → 产生 50行中间结果
  第二步：100万 × 50 = 5000万次比较
  代价：远小于方案A

结论：JOIN顺序的选择对性能有数量级的影响。
```

### 挑战3：资源约束

```
即使算法和顺序都选对了，还有物理约束：

  • 内存限制：Hash Join 需要建哈希表，如果表太大装不进内存？
  • I/O限制：表在磁盘上，每次读盘都是延迟
  • 并发限制：100个查询同时做 JOIN，内存够分吗？
  • 数据倾斜：某个哈希桶特别大，拖慢整体

这些不是"算法问题"，是"工程问题"
——需要在具体系统约束下做最优选择
```

---

## A.4 本章（10-A）小结

```
你已经理解了 JOIN 为什么是数据库最难的问题之一：

  1. 组合爆炸：N 张表的 JOIN 顺序有 (2N)!/(N+1)! 种可能
  2. 算法选择：Nested Loop / Hash Join / Sort-Merge 各有适用场景
  3. 顺序选择：中间结果的大小对最终性能有数量级影响
  4. 资源约束：内存、I/O、并发都是真实限制

下一讲（10-B）：SQLRustGo 如何解决这些问题？
  → JOIN 解析 → 逻辑计划 → 物理计划 → 实际执行
```

---

# 10-B: SQLRustGo的JOIN实现架构

---

## B.1 整体数据流

### 一条 JOIN SQL 在 SQLRustGo 中的旅程

```
SELECT o.id, c.name, o.amount
FROM orders o JOIN customers c ON o.customer_id = c.id
WHERE o.amount > 100

                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│  Step 1: Parser（解析层）                                 │
│  SQL文本 → AST（抽象语法树）                              │
│  识别：JOIN关键字、ON条件、WHERE条件                      │
│  产物：Statement::Select(SelectStatement{...})           │
├─────────────────────────────────────────────────────────┤
│  Step 2: Planner（逻辑计划层）                            │
│  AST → LogicalPlan                                       │
│  产物：LogicalPlan::Join{left, right, join_type, condition}│
│  关键决策：JoinType（INNER/LEFT/RIGHT...）               │
├─────────────────────────────────────────────────────────┤
│  Step 3: Join Planner（JOIN专属规划器）                   │
│  LogicalPlan → JoinPlan                                  │
│  关键决策：predicate拆分 + JOIN顺序                      │
│  产物：JoinPlan{base_table, joins, filters}              │
├─────────────────────────────────────────────────────────┤
│  Step 4: Optimizer（优化层）                              │
│  JoinPlan → PhysicalPlan                                 │
│  关键决策：用 Hash Join 还是 Nested Loop？               │
│  产物：HashJoinExec{left, right, join_type, condition}   │
├─────────────────────────────────────────────────────────┤
│  Step 5: Executor（执行层）                               │
│  PhysicalPlan → 实际执行                                  │
│  核心逻辑：建哈希表 → 探测匹配 → 输出结果                  │
│  产物：查询结果（行集）                                   │
└─────────────────────────────────────────────────────────┘
```

---

## B.2 Step 3 详解：Predicate拆分（把WHERE拆成两部分）

### 核心思想：不是所有WHERE条件都能用来做JOIN

```
原始SQL：
  SELECT * FROM orders o JOIN customers c
  ON o.customer_id = c.id
  WHERE o.amount > 100 AND c.status = 'active'

问题：WHERE条件中，哪些可以"内化"到JOIN内部去执行？

答案：
  o.customer_id = c.id  → "跨表等值条件" → 可以驱动Hash Join
  o.amount > 100        → "单表过滤条件" → 在JOIN之后再过滤
  c.status = 'active'   → "单表过滤条件" → 在JOIN之后再过滤
```

### SQLRustGo的Predicate拆分逻辑

```
代码位置：crates/executor/src/join_planner.rs

核心函数：split_predicates(conditions: &[String])
               → (Vec<JoinPredicate>, Vec<FilterPredicate>)

拆分规则：
  1. 识别"跨表等值比较"（如 t1.id = t2.id）
     → 归类为 JoinPredicate（用于驱动 Hash Join）
  
  2. 识别"单表条件"或"非等值比较"（如 t1.x > 10）
     → 归类为 FilterPredicate（在 JOIN 之后作为过滤器）
  
  3. 识别"同表比较"（如 t1.id = t1.parent_id）
     → 归类为 FilterPredicate（不能跨表使用）
```

### 拆分示例

```
输入条件：
  ["o.customer_id = c.id", "o.amount > 100", "c.status = 'active'"]

拆分结果：
  Join Predicates：  [o.customer_id = c.id]     ← 1个，用于Hash Join
  Filter Predicates： [o.amount > 100, c.status = 'active']  ← 2个，JOIN后过滤

为什么这样拆分？
  • 把"最有用的条件"（等值连接）用于加速JOIN本身
  • 其余条件在JOIN结果产生后，再执行简单过滤
  • "缩小中间结果" = 加速后续步骤
```

---

## B.3 Step 3 详解：JOIN顺序（贪心算法）

### 问题重述

```
三张表 JOIN：t1, t2, t3
已知连接条件：t1.id = t2.id, t2.id = t3.id

可能的JOIN顺序：
  (t1 ⋈ t2) ⋈ t3  vs  t1 ⋈ (t2 ⋈ t3)  vs  (t1 ⋈ t3) ⋈ t2

但(t1 ⋈ t3)不可行！因为没有直接的连接条件。

SQLRustGo 的贪心策略：
  从第一张表开始 → 每次找一个"能连上"的表加入 → 直到全部加入
```

### 贪心算法的执行过程

```
代码位置：crates/executor/src/join_planer.rs
核心函数：build_join_plan(tables, join_preds, filters) → JoinPlan

输入：
  tables = ["orders", "customers", "products"]
  join_preds = [
    {left: "orders", right: "customers", on: "orders.cust_id = customers.id"},
    {left: "customers", right: "products", on: "customers.prod_id = products.id"}
  ]

执行过程：
  Step 1: 选 orders 作为 base_table（FROM的第一个表）
          已访问 = {orders}
  
  Step 2: 找一条"连接已访问表到未访问表"的predicate
          orders.cust_id = customers.id → 连接 orders → customers
          已访问 = {orders, customers}
          JOIN链：orders ⋈ customers
  
  Step 3: 找下一条
          customers.prod_id = products.id → 连接 customers → products
          已访问 = {orders, customers, products}
          JOIN链：(orders ⋈ customers) ⋈ products
  
  Step 4: 全部表都在已访问集合中 → 结束
  
结果：
  JoinPlan {
    base_table: "orders",
    joins: [
      {right_table: "customers", on: orders.cust_id = customers.id},
      {right_table: "products", on: customers.prod_id = products.id}
    ],
    filters: [...]
  }
```

### 贪心算法的局限

```
贪心 = 每步做"当前看起来最好"的选择

优点：
  ✅ 速度快（O(N²)，不需要遍历所有排列）
  ✅ 实现简单（~100行代码）
  ✅ 大多数情况能得到"不错"的结果

缺点：
  ❌ 不保证最优（可能漏掉更好的JOIN顺序）
  ❌ 对FROM子句中表的顺序敏感
  ❌ 不考虑表的大小（实际cost信息）

改进方向（v4.0目标）：
  → 基于统计信息的动态规划（DP）选最优顺序
  → 利用CBO（Cost-Based Optimization）做成本估算
```

---

## B.4 Step 4：物理算法选择（Hash Join 的实际执行）

### HashJoinExec 的结构

```
代码位置：crates/planner/src/physical_plan.rs

pub struct HashJoinExec {
    left: Box<dyn PhysicalPlan>,    // 左子树（Build侧——建哈希表）
    right: Box<dyn PhysicalPlan>,   // 右子树（Probe侧——探测匹配）
    join_type: JoinType,            // INNER/LEFT/RIGHT/FULL...
    condition: Option<Expr>,        // JOIN条件（如 o.cust_id = c.id）
    schema: Schema,                 // 输出结果的Schema
}
```

### Hash Join 的执行原理

```
Hash Join 分两个阶段：

阶段1：Build（建哈希表）
  ─────────────────────
  读左子树（通常选较小的表）的全部数据
  以 JOIN 条件中的左表列为 key，建立 HashMap
  
  例如：SELECT ... FROM orders JOIN customers ON orders.cust_id = customers.id
  
  Build 侧选 customers（较小的表）：
    HashMap: cust_id → customer_row
    {101 → {name: "张三", ...}, 102 → {name: "李四", ...}}

阶段2：Probe（探测匹配）
  ─────────────────────
  读右子树（通常选较大的表）的每一行
  以 JOIN 条件中的右表列为 key，查找 HashMap
  
  例如：Probe 侧读 orders：
    order.cust_id = 101 → 查 HashMap → 找到 张三 → 输出一行
    order.cust_id = 101 → 查 HashMap → 找到 张三 → 再输出一行
    order.cust_id = 102 → 查 HashMap → 找到 李四 → 输出一行

复杂度：O(m + n)，其中 m 是 Build 表行数，n 是 Probe 表行数
对比 Nested Loop 的 O(m × n)，Hash Join 在大表场景有数量级优势
```

### 为什么 Hash Join 需要选"小表"做 Build 侧

```
如果选"大表"做 Build（建哈希表）：
  • 内存占用：100万行的 HashMap
  • 可能超出可用内存 → 需要"磁盘溢出" → 性能灾难

如果选"小表"做 Build：
  • 内存占用：1000行的 HashMap
  • 轻松放在内存中 → 探测阶段每个 lookup 都是 O(1)
```

---

## B.5 Step 5：成本模型（Cost Model）

### SQLRustGo 的 SimpleCostModel

```
代码位置：crates/optimizer/src/cost.rs

核心：对每种 JOIN 算法计算"代价"

pub fn join_cost(&self, left_rows: u64, right_rows: u64, join_method: &str) -> f64 {
    match join_method {
        "nested_loop" => (left_rows * right_rows) as f64 * self.cpu_cost_per_row,
        // Nested Loop: O(m×n) —— 乘积关系，大表时爆炸

        "hash_join" => {
            let build_cost = left_rows as f64 * self.cpu_cost_per_row;    // 建表
            let probe_cost = right_rows as f64 * self.cpu_cost_per_row;   // 探测
            build_cost + probe_cost
            // Hash Join: O(m+n) —— 加法关系，大表友好
        }
        
        "sort_merge" => {
            let sort_cost = (left_rows + right_rows) as f64 * self.cpu_cost_per_row;
            let merge_cost = (left_rows + right_rows) as f64 * self.cpu_cost_per_row;
            sort_cost + merge_cost
            // Sort-Merge: O((m+n)log(m+n)) —— 排序本身有成本
        }
    }
}
```

### 成本模型的价值：自动选择最优算法

```
场景1：小表 JOIN 小表（orders(100行) JOIN customers(50行)）
  
  nested_loop: 100 × 50 × 1.0 = 5,000
  hash_join:   100 × 1.0 + 50 × 1.0 = 150
  sort_merge:  2 × (100+50) × 1.0 = 300
  
  → Hash Join 最优，但差距不大

场景2：大表 JOIN 大表（orders(100万) JOIN customers(10万)）

  nested_loop: 1,000,000 × 100,000 × 1.0 = 10^11 = 1000亿
  hash_join:   1,000,000 × 1.0 + 100,000 × 1.0 = 1,100,000
  sort_merge:  2 × (1,100,000) × 1.0 = 2,200,000

  → Nested Loop 比 Hash Join 慢 90,909 倍！
  → 成本模型让优化器自动避开 Nested Loop
```

---

## B.6 本章（10-B）小结

```
你已经理解了 SQLRustGo 的 JOIN 实现全栈：

  解析层：识别 JOIN 语法、提取表名和条件
    ↓
  Predicate拆分：把 WHERE 拆成 Join条件 + Filter条件
    ↓
  JOIN顺序（贪心算法）：决定先 JOIN 谁、后 JOIN 谁
    ↓
  算法选择（成本模型）：Hash Join vs Nested Loop vs Sort-Merge
    ↓
  物理执行：Build哈希表 → Probe匹配 → 输出结果

下一讲（10-C）：有了实现，怎么验证它是"对的"和"快的"？
  → Harness 登场：TPC-H 基准 + 回归检测
```

---

# 10-C: Harness检测 — 性能回归与TPC-H验证

---

## C.1 问题：有了实现，怎么证明它"够好"？

### 回顾三层模型

```
提示词层：开发者说"帮我实现 JOIN 功能"
上下文层：AI（或开发者）写了 HashJoinExec + JoinPlanner
Harness层：怎么验证这个实现是正确的、高性能的？
```

### 你以为写完了，但其实才刚刚开始

```
开发者：JOIN 实现写完了，测试也过了，提交 PR！

Gate 系统的反应：
  BP1：✅ 编译通过、格式正确、零 Clippy 警告
  BP2：🟡 cargo test 通过了...
  BP2：🔴 TPC-H Q5（5表JOIN）超时！30秒没跑完！

发生了什么？
  → 简单 JOIN 测试通过了（2表 × 100行）
  → 但 TPC-H Q5 是 5 表 JOIN × 几十万行
  → 实际数据量下的性能完全不行
```

---

## C.2 TPC-H：JOIN 的"终极考验"

### TPC-H 中 JOIN 密集的查询

```
TPC-H 22 个查询中，大多数都涉及 JOIN：

  2表 JOIN：Q6, Q14, Q15, Q17, Q19, Q20
  3表 JOIN：Q1, Q4, Q12, Q13, Q16, Q18, Q22
  4表 JOIN：Q3, Q10, Q11
  5表 JOIN：Q5, Q7, Q8, Q9
  6表 JOIN：Q2, Q21

最复杂的是 Q2 和 Q21 —— 6 张表！
  如果 JOIN 顺序选择不好，
  中间结果爆炸 → 查询永远跑不完
```

### TPC-H 像"裁判"一样工作

```
PR 提交 → Gate 触发 TPC-H 检查

1. 生成测试数据（SF=1，约1GB，涉及数百万行）
2. 执行 22 个标准查询
3. 对比结果正确性
4. 测量每个查询的执行时间
5. 检查是否超时（阈值：单个查询 ≤ 30秒）

如果 Q5（5表JOIN）超时 → Gate Fail → PR Blocked
开发者必须回头分析："为什么我的 JOIN 实现在大数据量下变慢了？"
```

### 一个真实的场景

```
场景：开发者实现了一个新的 JOIN 优化
  → 2 表 JOIN 测试通过（100行 × 100行）
  → 3 表 JOIN 测试通过
  → 提交 PR

Gate 反馈：
  ✅ Q1（单表聚合）→ 0.68秒 ← 正常
  ✅ Q3（3表JOIN）→ 2.1秒 ← 正常
  ❌ Q5（5表JOIN）→ 31.2秒 ← 超时！
  ❌ Q9（5表JOIN）→ 28.5秒 ← 接近超时

根因定位（GitNexus + 开发者分析）：
  → 贪心 JOIN 顺序对 5 表场景选择了"糟糕的顺序"
  → 因为贪心算法只看了连接条件，没有考虑表的大小

修复：
  → 在贪心选择时，优先选"估计行数少"的表作为中间结果
  → 重新提交 → Q5从31.2秒降到8.3秒 → Gate通过
```

---

## C.3 回归检测：防止"优化变劣化"

### 什么是 JOIN 的性能回归

```
场景：你"优化"了 Hash Join 的哈希函数
  → 本意：减少哈希碰撞，提高查找速度
  → 实际：新哈希函数计算更复杂，对小数量的key反而变慢

结果：小表 JOIN 变慢了 30%（但大表 JOIN 变快了 5%）

如果没有回归检测：
  → 你觉得"优化成功"
  → 上线后，用户发现常用的小查询变慢了
  → 投诉 → 回滚

有回归检测（check_regression.sh）：
  → 自动对比优化前后的所有 JOIN 查询性能
  → 发现小表 JOIN 退化 30%
  → Gate Fail → 开发者需要重新评估哈希函数的选择
```

### 回归检测的量化标准

```
check_regression.sh 对每个基准查询的检查：

  simple_select：400K QPS  →  新版本 398K  →  Δ = -0.5% ✅
  join_2tables： 57,854 QPS → 新版本 40,500 →  Δ = -30%  ❌
  join_5tables： 12,000 QPS  →  新版本 13,200 →  Δ = +10% ✅

标准：任何查询的性能退化 > 5%，Gate 就拦截

这里的关键哲学：
  不是追求"平均性能提升"
  而是防止"任何一个查询变差"
  
  因为对用户来说，他最常用的那个查询
  如果变慢了，他不会在乎"其他查询平均变快了"
```

---

## C.4 提示词-上下文-Harness 在 JOIN 优化中的具体映射

### 完整的三层协作实例

```
场景：开发者想让 JOIN 支持更多表

提示词层（需求）：
  "让 JOIN 支持 10 张以上的表关联查询"

上下文层（AI 和开发者的信息）：
  ✅ 现有 JOIN 实现（join_planner.rs——贪心算法）
  ✅ 成本模型（cost.rs——三种算法代价估算）
  ❌ 不知道贪心算法在 7+ 表场景的错误率（上下文盲区）
  ❌ 不知道 10 表 JOIN 的中间结果爆炸会有多大

Harness 层（Gate 的反馈）：
  1. TPC-H Q2（6表）→ 18秒 → 接近超时
  2. TPC-H Q21（6表）→ 22秒 → 接近超时
  3. 手工构造 10 表 JOIN → 超时！
  
  结论：贪心算法在 >5 表场景下性能不可接受
  → 必须引入更好的 JOIN 顺序算法

Harness 的价值：
  不是"AI 告诉我怎么做"，而是"Gate 告诉我现在做的还不够好"
  反馈回路：实现 → Gate验证 → 发现问题 → 改进实现
```

---

## C.5 本章（10-C）小结

```
你已经理解了 Harness 如何验证 JOIN 实现的质量：

  TPC-H 基准：
    → 22个标准查询，从2表到6表 JOIN
    → 覆盖简单到复杂的所有 JOIN 场景
    → 任何查询超时 → Gate Fail

  回归检测：
    → 自动对比优化前后的性能
    → 不是看"整体有没有变快"
    → 而是看"有没有谁变慢了"（Δ > 5%）

  Harness 的角色：
    → 不是替代人的判断，而是给人提供"数据化的事实"
    → "你的优化在 Q5 上慢了 30%" → 这是事实，不是意见
    → 驱动开发者去定位根因、改进设计

下一讲（10-D）：发现问题之后怎么办？
  → 定位根因 → 修复 → 验证 → 沉淀知识 → 优化流程
  → 这就是 Harness 的自改进闭环
```

---

# 10-D: 优化迭代与Harness自改进闭环

---

## D.1 从"发现问题"到"解决问题"的完整流程

### 一个虚构但真实可信的案例

```
背景：
  开发者为 SQLRustGo 实现了基础 Hash Join
  测试通过，提交 PR

Gate 反馈：
  ✅ 编译/格式/Clippy → 通过
  ✅ 单元测试 → 通过
  ✅ 覆盖率 → 78%（达标）
  🔴 TPC-H Q5（5表JOIN，SF=1）→ 超时（31.2秒 > 30秒阈值）

PR 被 Block → 开发者开始调查...
```

---

## D.2 Step 1：定位根因（GitNexus + 工具链）

### 用 GitNexus 分析影响范围

```
开发者问 GitNexus：
  "Q5 查询涉及哪些函数？"

GitNexus 返回 Q5 的执行流（execution flow）：
  Parser → Planner → JoinPlanner::build_join_plan() → HashJoinExec::execute()
       → SeqScan(orders) → SeqScan(customer) → SeqScan(supplier)
       → SeqScan(nation) → SeqScan(region)

关键发现：
  Q5 涉及 5 张表 + 多层 JOIN
  执行时间分布（通过 EXPLAIN ANALYZE）：
    Build Hash Table (customer):  0.3秒
    Build Hash Table (supplier):  0.5秒
    Build Hash Table (nation):    0.1秒
    Build Hash Table (region):    0.1秒
    Probe + 中间结果合并：         30.2秒 ← 瓶颈！
```

### 深入分析瓶颈

```
为什么 Probe 阶段花了 30 秒？

原因1：JOIN 顺序不佳
  当前顺序：(orders ⋈ customer) ⋈ supplier ⋈ nation ⋈ region
  第一步产生 150 万行中间结果（几乎等于 orders 全表）
  后续每一步都在这个巨大的中间结果上做 Hash Join

原因2：贪心算法的盲区
  join_planner.rs 的 build_join_plan() 只看连接条件，
  不看表的大小。导致"把最大的中间结果传给下一步"

原因3：内存不够
  150 万行的中间结果需要在每次 Probe 中反复读取
  缓存命中率低 → 频繁磁盘 I/O → 慢
```

---

## D.3 Step 2：设计优化方案

### 方案A：优化 JOIN 顺序（启发式改进）

```
改进贪心算法：在选择下一个表时，加入"表大小"约束

原算法：
  "随便找一个能连上的表就加入"
  
改进后：
  "在能连上的表中，优先选行数最少的表"
  
效果：
  新 JOIN 顺序：region ⋈ nation ⋈ supplier ⋈ customer ⋈ orders
  → 最先 JOIN 小表（region 5行 + nation 25行 = 极小的中间结果）
  → 逐步扩大，最后 JOIN orders（最大的表）
  
  Q5 耗时：31.2秒 → 8.3秒 ✅
```

### 方案B：提前过滤（Predicate Pushdown）

```
改进 predicate 处理：把过滤条件"推"到 JOIN 之前执行

原逻辑：
  JOIN 所有表 → 再过滤 WHERE 条件
  
改进后：
  先对各表执行过滤 → 再 JOIN 过滤后的精简数据
  
例如：
  WHERE region.name = 'ASIA'
  → 原来：JOIN 5张表全量 → 最后过滤 region.name
  → 改进：先过滤 region（5行→1行）→ 再 JOIN 其他4张表
  
效果（叠加方案A后）：
  Q5 耗时：8.3秒 → 3.1秒 ✅✅
```

### 方案C：成本模型驱动选择（CBO增强）

```
不只是改 JOIN 顺序，而是让优化器"算"出哪个方案最优：

在 cost.rs 的 join_cost() 基础上，增加"中间结果大小估算"：

  fn estimate_intermediate_size(&self, left_rows: u64, right_rows: u64, 
                                  join_selectivity: f64) -> u64 {
      // Hash Join 的 selectivity 通常接近 1/N（每个左行匹配约1个右行）
      (left_rows.max(right_rows) as f64 * join_selectivity) as u64
  }

有了中间结果估算，优化器可以：
  比较不同 JOIN 顺序的"总代价" → 选最优的
```

---

## D.4 Step 3：验证修复（Gate 重新检查）

### 重新提交 PR

```
修复后的 PR 提交 → Gate 重新触发：

  BP1: ✅ 编译/格式/Clippy
  BP2: ✅ cargo test (93% pass)
  BP2: ✅ 覆盖率 79% (> 75%)
  BP2: ✅ TPC-H SF=1 全部通过
       Q1: 0.68秒  ✅
       Q3: 2.1秒   ✅
       Q5: 3.1秒   ✅ (之前31.2秒 → 优化10倍！)
       Q9: 5.2秒   ✅ (之前28.5秒 → 优化5倍！)
  BP2: ✅ 回归检测：无性能退化

PR 合并 → Issue 关闭
```

### 关键：不是"修完就完了"

```
修复完成后的额外步骤（Harness 的精髓）：

1. 更新基准数据
   → 把新的 QPS/延迟数据写入 current.json
   → 下次检测以新数据为基准

2. 记录知识（GBrain SSOT）
   → 创建 Episode："Q5 JOIN顺序优化——性能提升10倍"
   → 创建 Pattern："优先JOIN小表的贪心策略"
   → 下次遇到类似问题，Agent 可以直接查到

3. 更新 Gate Spec（如果必要）
   → 考虑："5表以上JOIN是否应该作为 BP3 的专项检查？"
```

---

## D.5 Step 4：流程本身的自改进

### 从这次优化中学到了什么

```
这次优化暴露了 Gate 系统的一个盲区：

  原来的 BP2 检查：TPC-H 22个查询全部通过
  但 Q5 在 30 秒阈值边缘（31.2秒，差 1.2 秒）
  如果下一个 PR 让 Q5 从 31.2 秒变成 34 秒呢？
  → 也都是"超时"，Gate 检测不到"恶化"

流程改进：
  不只是检查"超时不超时"
  更要检查"有没有比上次更慢"
  
  新规则：每个 TPC-H 查询的执行时间如果比基准慢 > 20%
         即使仍在 30 秒阈值内，也触发警告

这就是 CMM-5 的"自改进"：
  不只是"Gate 检查代码"
  而是"从 Gate 的行为中学习，优化 Gate 本身"
```

---

## D.6 自优化闭环全景图

### 完整的 PDCA 循环

```
┌─────────────────────────────────────────────────────────────┐
│                  JOIN 优化自改进闭环                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Plan（计划）                                                │
│  ────────────                                               │
│  "我们需要实现 JOIN 支持，目标：TPC-H Q5 < 30秒"             │
│       │                                                     │
│       ▼                                                     │
│  Do（执行）                                                  │
│  ────────────                                               │
│  实现 HashJoinExec + JoinPlanner + 贪心算法                  │
│  AI（或人）写代码 → 提交 PR                                  │
│       │                                                     │
│       ▼                                                     │
│  Check（检查）                                               │
│  ────────────                                               │
│  TPC-H Gate: Q5 超时（31.2 > 30秒）→ PR Blocked             │
│  回归检测: 发现瓶颈在 Probe 阶段                              │
│  GitNexus: 定位到 JoinPlanner 的贪心算法                      │
│       │                                                     │
│       ▼                                                     │
│  Act（改进）                                                 │
│  ────────────                                               │
│  1. 优化贪心算法（加入表大小约束）→ Q5 降到 8.3秒            │
│  2. Predicate Pushdown → Q5 降到 3.1秒                      │
│  3. 更新基准数据 → current.json                              │
│  4. 记录知识 → GBrain Episode                                │
│  5. 优化 Gate 规则 → 增加"退化 20%" 警告                     │
│       │                                                     │
│       └──→ 回到 Plan（下一个优化目标）                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## D.7 与三层模型的最终映射

### 整个案例中三层的角色

```
┌─────────────────────────────────────────────────────────────┐
│          提示词-上下文-Harness 在 JOIN 优化中的体现           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  提示词层（Prompt）                                          │
│  ─────────────────                                          │
│  开发者说："实现 JOIN 功能"                                  │
│  AI 说："好的，这是代码"                                     │
│  问题：需求没说清楚"5表JOIN时不能超时"                       │
│                                                             │
│  上下文层（Context）                                         │
│  ─────────────────                                          │
│  AI 看到：当前函数实现（HashJoinExec）                       │
│  AI 看不到：                                             │
│    • TPC-H Q5 是5表JOIN（不知道实际负载有多复杂）            │
│    • 贪心算法在 3 表以下 OK，5 表以上容易选错顺序            │
│    • 中间结果 150 万行对内存的压力                           │
│                                                             │
│  Harness 层（约束）                                          │
│  ─────────────────                                          │
│  TPC-H Gate："你的实现在 Q5 上超时了"                        │
│  回归检测："你的优化让某个查询慢了 10%"                       │
│  GitNexus："你的改动影响了这些执行流"                        │
│  GBrain："上次类似的优化教训在这里"                          │
│                                                             │
│  核心：Harness 弥补了上下文盲区                              │
│        AI 不知道的问题，Gate 帮它发现                         │
│        AI 不理解的性能权衡，量化指标帮它判断                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## D.8 本章（10-D）小结与全讲总结

### 10-D 小结

```
你已经理解了 Harness 自改进闭环的完整运作：

  发现问题 → 定位根因（GitNexus + EXPLAIN ANALYZE）
      ↓
  设计方案 → 多种优化策略（JOIN顺序、Predicate Pushdown、CBO）
      ↓
  验证修复 → Gate 重新检查，TPC-H 全量验证
      ↓
  沉淀知识 → 基准更新 + GBrain Episode + Gate Spec 优化
      ↓
  流程进化 → Gate 自己变得更智能

这就是 CMM-5（优化级）的实操版本——
不只是"有流程"，而是"流程在自我进化"
```

### 全讲总结：为什么 JOIN 是最好的 Harness 教学案例

```
1. 复杂性可见：
   JOIN 的组合爆炸让"为什么需要 Harness"变得直观
   因为 AI 不可能在有限上下文中预判所有 JOIN 顺序的性能

2. 量化可操作：
   TPC-H 提供 2表→6表的标准化测试场景
   回归检测提供精确的 Δ% 反馈
   不依赖"感觉"，完全数据驱动

3. 优化可迭代：
   贪心算法 → 表大小约束 → Predicate Pushdown → CBO
   每一步都有明确的指标改善

4. 闭环可教学：
   从"实现功能"到"优化性能"到"沉淀知识"到"改进流程"
   完整展示了 Harness 工程的所有核心思想
```

---

# 课后作业

---

## 综合练习

1. **手算代价**：假设表A有1000行，表B有100行，表C有10000行。
   连接条件：A.id = B.id, B.id = C.id。
   计算三种 JOIN 顺序的总代价（用 SimpleCostModel）。
   哪种最优？为什么？

2. **设计 Gate 检查**：
   为你的项目中的"多表关联查询"设计一个 BP2 级别的 Gate 检查。
   - 阈值设多少？为什么？
   - 超时了怎么办？

3. **分析自改进闭环**：
   回顾你最近做的一次性能优化。画出你的"Plan-Do-Check-Act"循环图。
   问你：你当时有 Check（检查）这一步吗？如果没有，加入它会有什么不同？

---

<!-- _class: lead -->

# 谢谢！

## 下节课：CI/CD与Nomad自动化
