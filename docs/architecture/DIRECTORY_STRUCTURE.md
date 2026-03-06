# SQLRustGo 数据库级目录规范

> **版本**: 1.0
> **制定日期**: 2026-03-06
> **制定人**: yinglichina8848
> **目标规模**: 200-350 Rust 源文件

---

## 一、顶层结构

```
sqlrustgo/
│
├── Cargo.toml              # Workspace 配置
├── Cargo.lock
├── VERSION
├── README.md
├── CHANGELOG.md
│
├── crates/                 # 核心 crate 模块
│
├── bin/                    # 可执行文件
│
├── tests/                  # 集成测试
│
├── benches/                # 性能基准
│
├── docs/                   # 文档
│
├── scripts/                # 脚本工具
│
└── .github/                # GitHub 配置
```

---

## 二、核心 Crate 结构

```
crates/
│
├── parser/         # SQL 解析器 (≈15 文件)
├── planner/        # 查询规划器 (≈20 文件)
├── optimizer/      # 优化器 (≈30 文件)
├── executor/       # 执行器 (≈25 文件)
├── storage/        # 存储引擎 (≈30 文件)
├── catalog/        # 目录管理 (≈10 文件)
├── transaction/    # 事务管理 (≈15 文件)
├── distributed/    # 分布式执行 (≈25 文件)
├── common/         # 公共模块 (≈20 文件)
└── server/         # 服务器 (≈15 文件)
```

---

## 三、Parser 模块 (≈15 文件)

```
parser/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── lexer.rs           # 词法分析器
    ├── token.rs           # Token 定义
    ├── parser.rs          # 语法分析器
    │
    ├── ast/               # 抽象语法树
    │   ├── mod.rs
    │   ├── query.rs       # Query AST
    │   ├── select.rs      # SELECT AST
    │   ├── expr.rs        # Expression AST
    │   ├── table.rs       # Table AST
    │   ├── function.rs    # Function AST
    │   └── ddl.rs         # DDL AST
    │
    └── error.rs           # 错误处理
```

---

## 四、Planner 模块 (≈20 文件)

```
planner/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── binder.rs          # SQL 绑定器
    ├── planner.rs         # 查询规划器
    │
    ├── logical_plan/      # 逻辑计划
    │   ├── mod.rs
    │   ├── scan.rs        # TableScan
    │   ├── projection.rs  # Projection
    │   ├── filter.rs      # Filter
    │   ├── join.rs        # Join
    │   ├── aggregate.rs   # Aggregate
    │   ├── limit.rs       # Limit
    │   ├── sort.rs        # Sort
    │   └── values.rs      # Values
    │
    ├── physical_plan/     # 物理计划
    │   ├── mod.rs
    │   ├── scan.rs
    │   ├── projection.rs
    │   ├── filter.rs
    │   ├── join.rs
    │   └── aggregate.rs
    │
    └── rewrite/           # 查询重写
        ├── mod.rs
        ├── predicate_pushdown.rs
        ├── projection_pruning.rs
        └── join_reorder.rs
```

---

## 五、Optimizer 模块 (≈30 文件)

```
optimizer/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── optimizer.rs       # 优化器入口
    │
    ├── cascades/          # Cascades 优化器
    │   ├── mod.rs
    │   ├── memo.rs        # Memo 表
    │   ├── group.rs       # Group 结构
    │   ├── group_expr.rs  # GroupExpr
    │   ├── rule.rs        # 规则接口
    │   ├── rule_set.rs    # 规则集合
    │   ├── task.rs        # 优化任务
    │   ├── cost.rs        # 成本模型
    │   ├── property.rs    # 物理属性
    │   └── optimizer.rs   # 核心优化循环
    │
    ├── rules/             # 优化规则
    │   ├── mod.rs
    │   ├── filter_pushdown.rs
    │   ├── join_commute.rs
    │   ├── join_associate.rs
    │   ├── index_scan.rs
    │   ├── limit_pushdown.rs
    │   ├── projection_pushdown.rs
    │   └── constant_folding.rs
    │
    └── stats/             # 统计信息
        ├── mod.rs
        ├── table_stats.rs
        ├── column_stats.rs
        └── histogram.rs
```

---

## 六、Executor 模块 (≈25 文件)

```
executor/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── executor.rs        # 执行器入口
    │
    ├── operators/         # 执行算子
    │   ├── mod.rs
    │   ├── operator.rs    # Operator trait
    │   ├── scan.rs        # TableScan
    │   ├── filter.rs      # Filter
    │   ├── projection.rs  # Projection
    │   ├── hash_join.rs   # HashJoin
    │   ├── nested_loop_join.rs
    │   ├── aggregate.rs   # Aggregate
    │   ├── limit.rs       # Limit
    │   ├── sort.rs        # Sort
    │   └── values.rs      # Values
    │
    ├── pipeline/          # 执行流水线
    │   ├── mod.rs
    │   ├── pipeline.rs
    │   ├── pipeline_builder.rs
    │   └── driver.rs
    │
    └── batch/             # 向量化执行
        ├── mod.rs
        ├── record_batch.rs
        ├── column.rs
        └── array.rs
```

---

## 七、Storage 模块 (≈30 文件)

```
storage/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── engine.rs          # StorageEngine trait
    │
    ├── table/             # 表管理
    │   ├── mod.rs
    │   ├── table.rs
    │   ├── row.rs
    │   ├── schema.rs
    │   └── column.rs
    │
    ├── page/              # 页管理
    │   ├── mod.rs
    │   ├── page.rs
    │   ├── page_cache.rs
    │   ├── buffer_pool.rs
    │   └── page_id.rs
    │
    ├── index/             # 索引
    │   ├── mod.rs
    │   ├── index.rs       # Index trait
    │   ├── btree.rs       # B+ Tree
    │   ├── hash_index.rs
    │   └── index_builder.rs
    │
    ├── file/              # 文件存储
    │   ├── mod.rs
    │   ├── file_storage.rs
    │   ├── file_manager.rs
    │   └── block.rs
    │
    └── memory/            # 内存存储
        ├── mod.rs
        └── memory_storage.rs
```

---

## 八、Catalog 模块 (≈10 文件)

```
catalog/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── catalog.rs         # Catalog trait
    ├── schema.rs          # Schema 定义
    ├── table_catalog.rs   # 表目录
    ├── column_catalog.rs  # 列目录
    ├── index_catalog.rs   # 索引目录
    └── function_catalog.rs
```

---

## 九、Transaction 模块 (≈15 文件)

```
transaction/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── txn.rs             # 事务
    ├── transaction_manager.rs
    ├── lock_manager.rs    # 锁管理
    ├── lock.rs            # 锁定义
    ├── mvcc.rs            # MVCC
    ├── timestamp.rs       # 时间戳
    ├── isolation_level.rs # 隔离级别
    └── wal/               # Write-Ahead Log
        ├── mod.rs
        ├── wal.rs
        └── log_entry.rs
```

---

## 十、Distributed 模块 (≈25 文件)

```
distributed/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── scheduler.rs       # 任务调度
    ├── node.rs            # 节点管理
    ├── task.rs            # 任务定义
    │
    ├── exchange/          # 数据交换
    │   ├── mod.rs
    │   ├── exchange.rs
    │   ├── shuffle.rs
    │   └── broadcast.rs
    │
    ├── rpc/               # RPC 通信
    │   ├── mod.rs
    │   ├── client.rs
    │   ├── server.rs
    │   └── protocol.rs
    │
    └── consensus/         # 一致性协议
        ├── mod.rs
        └── raft.rs
```

---

## 十一、Common 模块 (≈20 文件)

```
common/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── error.rs           # 错误处理
    ├── result.rs          # Result 类型
    ├── types/             # 类型系统
    │   ├── mod.rs
    │   ├── value.rs
    │   ├── data_type.rs
    │   └── scalar.rs
    │
    ├── util/              # 工具函数
    │   ├── mod.rs
    │   ├── hash.rs
    │   ├── compare.rs
    │   └── timer.rs
    │
    └── config/            # 配置
        ├── mod.rs
        └── config.rs
```

---

## 十二、Server 模块 (≈15 文件)

```
server/
│
├── Cargo.toml
│
└── src/
    │
    ├── lib.rs
    ├── server.rs          # 服务器入口
    ├── session.rs         # 会话管理
    ├── query_handler.rs   # 查询处理
    ├── protocol.rs        # 协议实现
    │
    ├── auth/              # 认证授权
    │   ├── mod.rs
    │   ├── auth.rs
    │   └── permission.rs
    │
    └── network/           # 网络层
        ├── mod.rs
        ├── connection.rs
        └── listener.rs
```

---

## 十三、测试目录

```
tests/
│
├── integration/           # 集成测试
│   ├── sql_test.rs
│   ├── storage_test.rs
│   ├── transaction_test.rs
│   └── distributed_test.rs
│
└── e2e/                   # 端到端测试
    ├── basic_test.rs
    └── performance_test.rs
```

---

## 十四、文档目录

```
docs/
│
├── architecture/          # 架构文档
│   ├── ARCHITECTURE.md
│   └── DESIGN.md
│
├── governance/            # 治理文档
│   ├── RELEASE_GOVERNANCE.md
│   ├── BRANCH_GOVERNANCE.md
│   └── ARCHITECTURE_RULES.md
│
├── releases/              # 发布文档
│   ├── v1.0.0/
│   ├── v1.1.0/
│   ├── v1.2.0/
│   └── v1.3.0/
│
└── api/                   # API 文档
    └── API.md
```

---

## 十五、文件统计

| 模块 | 文件数 | 说明 |
|------|--------|------|
| parser | 15 | SQL 解析 |
| planner | 20 | 查询规划 |
| optimizer | 30 | Cascades 优化器 |
| executor | 25 | 执行引擎 |
| storage | 30 | 存储引擎 |
| catalog | 10 | 目录管理 |
| transaction | 15 | 事务管理 |
| distributed | 25 | 分布式执行 |
| common | 20 | 公共模块 |
| server | 15 | 服务器 |
| tests | 20 | 测试 |
| docs | 15 | 文档 |
| **总计** | **≈240** | |

---

## 十六、架构原则

### 16.1 模块化原则

1. **每个 crate 独立编译**
2. **清晰的依赖关系**
3. **最小化公开 API**
4. **内部实现隐藏**

### 16.2 解耦原则

```
Parser → Planner → Optimizer → Executor → Storage
         ↓           ↓           ↓
      Catalog    Statistics   Transaction
```

### 16.3 扩展原则

1. **Trait 抽象**: 所有核心组件使用 trait
2. **插件化**: 存储引擎、优化规则可插拔
3. **分布式预留**: 为 2.0 预留接口

---

## 十七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本，定义 200+ 文件目录结构 |

---

*本文档由 yinglichina8848 制定*
