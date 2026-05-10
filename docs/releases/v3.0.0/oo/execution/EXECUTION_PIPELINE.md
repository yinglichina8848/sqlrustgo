# 查询执行流水线

> 从 SQL 输入到结果输出的完整执行链路分析

## 1. 执行流水线总览

### 1.1 架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SQL 查询执行流水线                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   SQL Input                                                              │
│       │                                                                  │
│       ▼                                                                  │
│   ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐    │
│   │ Parser │───▶│Planner │───▶│Optimizer│───▶│Executor│───▶│Storage│    │
│   └────────┘    └────────┘    └────────┘    └────────┘    └────────┘    │
│       │            │             │             │             │            │
│       ▼            ▼             ▼             ▼             ▼            │
│   AST           Logical     Physical       Iterator      Page I/O        │
│               Plan         Plan          Model                         │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐    │
│   │                    Transaction Layer                             │    │
│   │   BEGIN → Operations → COMMIT/ROLLBACK + WAL                  │    │
│   └──────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 关键指标

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| 解析延迟 | < 1ms | ✅ |
| 规划延迟 | < 10ms | ✅ |
| 优化延迟 | < 50ms | ✅ |
| 执行延迟 | Query dependent | ⚠️ |
| 内存峰值 | < 8GB | ✅ |

## 2. 各阶段详解

### 2.1 Parser 阶段

```
Input: "SELECT id, name FROM users WHERE age > 18"
           │
           ▼
┌─────────────────────────────────────────────┐
│                 Lexer                       │
│  Input → Token[SELECT, ID, COMMA, ID, ...]  │
└─────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│                 Parser                       │
│  Token[] → AST (Recursive Descent Parser)   │
│                                              │
│  SelectStmt {                                │
│    projection: [Column(id), Column(name)],   │
│    from: Table(users),                       │
│    where: BinaryExpr(Gt, Column(age), 18)  │
│  }                                          │
└─────────────────────────────────────────────┘
```

### 2.2 Planner 阶段

```
AST → Logical Plan
           │
           ▼
┌─────────────────────────────────────────────┐
│              Binder                          │
│  - 解析表名、列名                           │
│  - 绑定类型信息                             │
│  - 验证 schema                             │
└─────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│              Resolver                       │
│  - 函数解析                                 │
│  - 类型推导                                 │
│  - 权限检查                                 │
└─────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│           Logical Plan                      │
│                                              │
│  Projection(id, name)                        │
│       │                                      │
│  Filter(age > 18)                           │
│       │                                      │
│  TableScan(users)                           │
└─────────────────────────────────────────────┘
```

### 2.3 Optimizer 阶段 (CBO)

```
Logical Plan → Physical Plan (Cost-Based Optimization)
                    │
                    ▼
┌─────────────────────────────────────────────┐
│           Statistics Collector               │
│  - TableStats (row_count, page_count)       │
│  - ColumnStats (ndv, min, max, histogram)  │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│              Cost Estimator                 │
│                                              │
│  Cost = CPU + IO + Memory                   │
│  Cost = rows * cpu_cost_per_row             │
│        + pages * read_latency              │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│           Plan Enumerator                    │
│                                              │
│  - Enumerate join orders (DP)                │
│  - Choose access paths (index vs scan)      │
│  - Select physical operators                 │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│           Physical Plan                      │
│                                              │
│  HashJoin                                    │
│   /        \                                │
│ IndexScan  TableScan                        │
│ (age>18)   (users)                         │
└─────────────────────────────────────────────┘
```

### 2.4 Executor 阶段 (Volcano Model)

```
Physical Plan → Iterator Model (Pull-based)
                    │
                    ▼
┌─────────────────────────────────────────────┐
│           Executor Core                      │
│                                              │
│  impl Executor for HashJoinExecutor {         │
│      fn execute(&mut self) -> Result<()> {  │
│          let left = self.left.execute()?;    │
│          let right = self.right.execute()?; │
│          self.build_phase(left)?;           │
│          self.probe_phase(right)            │
│      }                                      │
│  }                                          │
└─────────────────────────────────────────────┘
```

## 3. Volcano Iterator Model

### 3.1 算子接口

```rust
pub trait Executor {
    fn execute(&mut self) -> Result<Box<dyn RecordBatch>>;
    fn schema(&self) -> &Schema;
}

pub trait ExecutorExt: Executor {
    fn execute_stream(&mut self) -> Result<Stream>;
    fn execute_batch(&mut self) -> Result<Vec<RecordBatch>>;
}
```

### 3.2 算子树

```
                   ┌─────────────┐
                   │   Limit    │  (输出 100 行)
                   │   (100)    │
                   └──────┬─────┘
                          │
                   ┌──────▼─────┐
                   │   Sort     │  (ORDER BY age DESC)
                   │  (age DESC)│
                   └──────┬─────┘
                          │
                   ┌──────▼─────┐
                   │  HashJoin  │  (t1.id = t2.id)
                   │            │
                   └────┬──────┘
                    /        \
            ┌──────▼───┐  ┌───▼───────┐
            │IndexScan │  │TableScan  │
            │ (t1,age>│  │   (t2)    │
            │  18)    │  │           │
            └─────────┘  └───────────┘
```

### 3.3 执行流程

```
1. Limit.next()
       │
       ▼
2. Sort.next()
       │
       ▼
3. HashJoin.next()
       │
       ├──▶ IndexScan.next() ──▶ Storage.read_page()
       │                              │
       │◀─── RecordBatch ◀─────────────┘
       │
       └──▶ TableScan.next() ──▶ Storage.read_page()
                                      │
                              ◀─── RecordBatch
```

## 4. Storage 层交互

### 4.1 页面读取链路

```
Executor requests page
           │
           ▼
┌─────────────────────────────────────────────┐
│           BufferPool                         │
│                                              │
│  if page in pool:                            │
│      return cached_page                     │
│  else:                                       │
│      load from disk                         │
│      if pool_full:                          │
│          evict_lru_page                     │
│      insert into pool                       │
│      return page                            │
└─────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│           FileStorage                        │
│                                              │
│  read_page(page_id) → bytes                 │
│  write_page(page_id, bytes)                  │
└─────────────────────────────────────────────┘
```

### 4.2 WAL 写入链路

```
Transaction commits
           │
           ▼
┌─────────────────────────────────────────────┐
│              WAL Manager                     │
│                                              │
│  1. LogRecord {                              │
│        tx_id: 123,                          │
│        op: UPDATE,                          │
│        page_id: 5,                          │
│        before: bytes,                       │
│        after: bytes                         │
│     }                                       │
│                                              │
│  2. Append to WAL file                      │
│  3. Force write to disk (fsync)            │
│  4. Mark transaction committed               │
└─────────────────────────────────────────────┘
```

## 5. 完整 SELECT 执行链路时序图

```
User: "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id WHERE t1.age > 18 ORDER BY t1.name LIMIT 100"

Time    Component     Action
─────────────────────────────────────────────────────────
T0      Client       发送 SQL 请求
T1      Network      接收请求到缓冲区
T2      Parser       Lexer: 生成 Token 流
T3      Parser       Parser: 生成 AST
T4      Planner      Binder: 绑定表和列
T5      Planner      Resolver: 解析类型
T6      Planner      生成 Logical Plan
T7      Optimizer    收集统计信息
T8      Optimizer    计算代价
T9      Optimizer    枚举 Join 次序
T10     Optimizer    选择物理算子
T11     Optimizer    生成 Physical Plan
T12     Executor     创建算子树
T13     Executor     Limit.open()
T14     Executor     Sort.open()
T15     Executor     HashJoin.open()
T16     Executor     IndexScan.open() - 打开索引
T17     Executor     TableScan.open() - 打开表
T18     Storage      BufferPool: 检查页面缓存
T19     Storage      BufferPool: 缓存未命中, 读取磁盘
T20     Executor     IndexScan.next() - 获取匹配行
T21     Executor     TableScan.next() - 获取驱动表行
T22     Executor     HashJoin: 执行 Hash Join
T23     Executor     Sort: 排序
T24     Executor     Limit: 取前 100 行
T25     Client       返回结果集
```

## 6. 性能瓶颈分析

### 6.1 常见瓶颈点

| 阶段 | 瓶颈 | 解决方案 |
|------|------|----------|
| Parser | 解析大 SQL | 预编译 + 缓存 |
| Optimizer | 动态规划复杂度 | 启发式剪枝 |
| Executor | Hash Join 内存 | 增量 Hash Join |
| Storage | 随机 IO | 批量预读 |
| Network | 结果传输 | 压缩传输 |

### 6.2 优化策略

```
┌─────────────────────────────────────────────┐
│              优化策略                         │
├─────────────────────────────────────────────┤
│ 1. 计划缓存 (Plan Cache)                    │
│    - 相同 SQL 模板复用执行计划                │
│    - LRU 缓存淘汰                           │
│                                              │
│ 2. 增量执行 (Incremental Execution)         │
│    - 子查询结果缓存                          │
│    - 物化视图                               │
│                                              │
│ 3. 批量处理 (Batch Processing)             │
│    - 减少函数调用开销                        │
│    - SIMD 向量化                            │
│                                              │
│ 4. 预读 (Prefetching)                      │
│    - 顺序页面预读                           │
│    - 索引跨度预读                           │
└─────────────────────────────────────────────┘
```

## 7. 测试覆盖要点

### 7.1 链路测试

```rust
#[test]
fn test_select_full_pipeline() {
    // 端到端测试
    let sql = "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id WHERE t1.age > 18";
    let result = execute_sql(sql).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_optimizer_selects_correct_plan() {
    // 优化器测试
    let sql = "SELECT * FROM t WHERE a = 1 AND b > 10";
    let plan = optimizer.optimize(logical_plan).unwrap();
    // 验证选择了索引扫描
    assert!(matches!(plan, PhysicalPlan::IndexScan { .. }));
}
```

### 7.2 边界测试

```rust
#[test]
fn test_empty_result_set() {
    // 空结果集
    let sql = "SELECT * FROM t WHERE 1 = 0";
    let result = execute_sql(sql).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_all_columns_null() {
    // 全 NULL 列
    let sql = "SELECT NULL as a, NULL as b";
    let result = execute_sql(sql).unwrap();
    assert!(result.schema().field("a").is_nullable());
}
```
