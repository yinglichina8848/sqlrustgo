# SQLRustGo v1.2.0 Draft 阶段目录重构方案

> **版本**: 1.0
> **创建日期**: 2026-03-06
> **创建人**: yinglichina8848
> **阶段**: Draft
> **状态**: 待评审

---

## 一、当前目录结构分析

### 1.1 现状

```
sqlrustgo/
├── src/                    # 单一 src 目录
│   ├── auth/              # 认证 (1 文件)
│   ├── catalog/           # 目录 (1 文件)
│   ├── error/             # 错误 (6 文件)
│   ├── executor/          # 执行器 (4 文件)
│   ├── lexer/             # 词法 (3 文件)
│   ├── monitoring/        # 监控 (2 文件)
│   ├── network/           # 网络 (1 文件)
│   ├── optimizer/         # 优化器 (6 文件)
│   ├── parser/            # 解析器 (1 文件)
│   ├── planner/           # 规划器 (8 文件)
│   ├── query/             # 查询 (1 文件)
│   ├── storage/           # 存储 (7 文件 + bplus_tree/)
│   ├── transaction/       # 事务 (3 文件)
│   └── types/             # 类型 (3 文件)
├── tests/                  # 测试 (5 文件)
├── benches/               # 基准 (8 文件)
├── docs/                  # 文档 (大量)
├── scripts/               # 脚本 (7 文件)
└── data/                  # 数据 (5 文件)
```

### 1.2 问题分析

| 问题 | 影响 | 优先级 |
|------|------|--------|
| 单一 src 目录 | 模块耦合，编译慢 | 🔴 高 |
| 模块文件不完整 | 功能缺失 | 🔴 高 |
| docs 目录混乱 | 文档难以维护 | 🟡 中 |
| 缺少 crates 结构 | 无法独立编译 | 🔴 高 |
| 测试结构简单 | 测试覆盖不足 | 🟡 中 |

---

## 二、目标目录结构设计

### 2.1 顶层结构

```
sqlrustgo/
│
├── Cargo.toml              # Workspace 配置
├── Cargo.lock
├── VERSION
├── README.md
├── CHANGELOG.md
├── LICENSE
│
├── crates/                 # 核心 crate 模块 (新增)
│   ├── parser/
│   ├── planner/
│   ├── optimizer/
│   ├── executor/
│   ├── storage/
│   ├── catalog/
│   ├── transaction/
│   ├── distributed/
│   ├── common/
│   └── server/
│
├── bin/                    # 可执行文件 (新增)
│   └── sqlrustgo-cli/
│
├── tests/                  # 集成测试
│   ├── integration/
│   └── e2e/
│
├── benches/                # 性能基准
│
├── docs/                   # 文档 (重组)
│   ├── architecture/
│   ├── governance/
│   ├── releases/
│   └── api/
│
├── scripts/                # 脚本工具
│
├── examples/               # 示例代码 (新增)
│
└── .github/                # GitHub 配置
```

### 2.2 Crates 结构详细设计

#### parser crate (≈15 文件)

```
crates/parser/
├── Cargo.toml
└── src/
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

**迁移映射**:
- `src/lexer/*` → `crates/parser/src/lexer.rs`, `token.rs`
- `src/parser/*` → `crates/parser/src/parser.rs`

#### planner crate (≈20 文件)

```
crates/planner/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── binder.rs          # SQL 绑定器
    ├── planner.rs         # 查询规划器
    │
    ├── logical_plan/      # 逻辑计划
    │   ├── mod.rs
    │   ├── scan.rs
    │   ├── projection.rs
    │   ├── filter.rs
    │   ├── join.rs
    │   ├── aggregate.rs
    │   ├── limit.rs
    │   ├── sort.rs
    │   └── values.rs
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

**迁移映射**:
- `src/planner/*` → `crates/planner/src/`

#### optimizer crate (≈30 文件)

```
crates/optimizer/
├── Cargo.toml
└── src/
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

**迁移映射**:
- `src/optimizer/*` → `crates/optimizer/src/`
- `src/planner/cost.rs` → `crates/optimizer/src/cascades/cost.rs`

#### executor crate (≈25 文件)

```
crates/executor/
├── Cargo.toml
└── src/
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

**迁移映射**:
- `src/executor/*` → `crates/executor/src/`

#### storage crate (≈30 文件)

```
crates/storage/
├── Cargo.toml
└── src/
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

**迁移映射**:
- `src/storage/*` → `crates/storage/src/`

#### transaction crate (≈15 文件)

```
crates/transaction/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── txn.rs             # 事务
    ├── transaction_manager.rs
    ├── lock_manager.rs    # 锁管理
    ├── lock.rs            # 锁定义
    ├── mvcc.rs            # MVCC
    ├── timestamp.rs       # 时间戳
    ├── isolation_level.rs # 隔离级别
    │
    └── wal/               # Write-Ahead Log
        ├── mod.rs
        ├── wal.rs
        └── log_entry.rs
```

**迁移映射**:
- `src/transaction/*` → `crates/transaction/src/`

#### catalog crate (≈10 文件)

```
crates/catalog/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── catalog.rs         # Catalog trait
    ├── schema.rs          # Schema 定义
    ├── table_catalog.rs   # 表目录
    ├── column_catalog.rs  # 列目录
    ├── index_catalog.rs   # 索引目录
    └── function_catalog.rs
```

**迁移映射**:
- `src/catalog/*` → `crates/catalog/src/`

#### distributed crate (≈25 文件) - 新增

```
crates/distributed/
├── Cargo.toml
└── src/
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

**说明**: 为 v2.0 分布式版本预留

#### common crate (≈20 文件)

```
crates/common/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs           # 错误处理
    ├── result.rs          # Result 类型
    │
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

**迁移映射**:
- `src/types/*` → `crates/common/src/types/`
- `src/error/*` → `crates/common/src/error.rs`

#### server crate (≈15 文件)

```
crates/server/
├── Cargo.toml
└── src/
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

**迁移映射**:
- `src/auth/*` → `crates/server/src/auth/`
- `src/network/*` → `crates/server/src/network/`
- `src/monitoring/*` → `crates/server/src/monitoring/`

---

## 三、Workspace Cargo.toml 配置

```toml
[workspace]
members = [
    "crates/parser",
    "crates/planner",
    "crates/optimizer",
    "crates/executor",
    "crates/storage",
    "crates/catalog",
    "crates/transaction",
    "crates/distributed",
    "crates/common",
    "crates/server",
    "bin/sqlrustgo-cli",
]

[workspace.package]
version = "1.2.0-draft"
edition = "2021"
license = "MIT"
authors = ["yinglichina8848"]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
env_logger = "0.11"

# Internal crates
parser = { path = "crates/parser" }
planner = { path = "crates/planner" }
optimizer = { path = "crates/optimizer" }
executor = { path = "crates/executor" }
storage = { path = "crates/storage" }
catalog = { path = "crates/catalog" }
transaction = { path = "crates/transaction" }
distributed = { path = "crates/distributed" }
common = { path = "crates/common" }
server = { path = "crates/server" }
```

---

## 四、重构步骤

### 4.1 Phase 1: 创建 Workspace 结构 (Week 1)

```bash
# 1. 创建 crates 目录
mkdir -p crates/{parser,planner,optimizer,executor,storage,catalog,transaction,distributed,common,server}

# 2. 创建 bin 目录
mkdir -p bin/sqlrustgo-cli

# 3. 创建各 crate 的 Cargo.toml

# 4. 创建根 Cargo.toml (workspace)
```

### 4.2 Phase 2: 迁移模块 (Week 1-2)

按依赖顺序迁移:

1. **common** (无依赖)
2. **parser** (依赖 common)
3. **catalog** (依赖 common)
4. **storage** (依赖 common, catalog)
5. **transaction** (依赖 common, storage)
6. **planner** (依赖 common, parser, catalog)
7. **optimizer** (依赖 common, planner)
8. **executor** (依赖 common, planner, storage)
9. **server** (依赖所有)
10. **distributed** (依赖所有)

### 4.3 Phase 3: 更新导入 (Week 2)

```rust
// 旧导入
use crate::lexer::Lexer;

// 新导入
use parser::lexer::Lexer;
```

### 4.4 Phase 4: 验证 (Week 2)

```bash
# 编译检查
cargo build --workspace

# 测试检查
cargo test --workspace

# Clippy 检查
cargo clippy --workspace -- -D warnings
```

---

## 五、GitHub 分支管理方案

### 5.1 分支结构

```
main
 │
 ├── develop                    # 下一版本开发
 │
 ├── develop-1.2.0              # 当前版本开发 (Draft)
 │    │
 │    ├── refactor/directory-*  # 目录重构分支
 │    └── fix/v1.2.0-*          # Bug 修复分支
 │
 ├── release/1.1                # v1.1 维护分支
 │
 └── feature/*                  # 功能分支
```

### 5.2 重构分支策略

```
develop-1.2.0 (Draft)
    │
    ├── refactor/directory-phase1    # Phase 1: 创建结构
    │   └── PR → develop-1.2.0
    │
    ├── refactor/directory-phase2    # Phase 2: 迁移模块
    │   └── PR → develop-1.2.0
    │
    ├── refactor/directory-phase3    # Phase 3: 更新导入
    │   └── PR → develop-1.2.0
    │
    └── refactor/directory-phase4    # Phase 4: 验证
        └── PR → develop-1.2.0
```

### 5.3 分支命名规范

| 类型 | 命名规则 | 示例 |
|------|----------|------|
| 重构 | `refactor/directory-*` | `refactor/directory-phase1` |
| 功能 | `feature/v1.2.0-*` | `feature/v1.2.0-cascades` |
| 修复 | `fix/v1.2.0-*` | `fix/v1.2.0-page-bug` |
| 文档 | `docs/v1.2.0-*` | `docs/v1.2.0-api` |

### 5.4 PR 目标分支

| 分支类型 | 目标分支 |
|----------|----------|
| `refactor/*` | `develop-1.2.0` |
| `feature/*` | `develop-1.2.0` |
| `fix/*` | `develop-1.2.0` |
| `docs/*` | `develop-1.2.0` |

---

## 六、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 编译失败 | 高 | 中 | 分阶段迁移，每阶段验证 |
| 导入路径错误 | 中 | 高 | 使用 sed 批量替换 |
| 测试失败 | 高 | 中 | 先迁移测试，后迁移代码 |
| 依赖循环 | 高 | 低 | 仔细规划依赖顺序 |

---

## 七、验收标准

### 7.1 目录结构验收

- [ ] 所有 crate 独立编译
- [ ] Workspace 编译通过
- [ ] 文件数量达到 200+

### 7.2 功能验收

- [ ] 所有测试通过
- [ ] Clippy 零警告
- [ ] 功能无回归

### 7.3 文档验收

- [ ] README 更新
- [ ] ARCHITECTURE 更新
- [ ] 各 crate 有文档

---

## 八、时间计划

| 阶段 | 任务 | 时间 | 负责人 |
|------|------|------|--------|
| Phase 1 | 创建结构 | 2 天 | openheart |
| Phase 2 | 迁移模块 | 5 天 | openheart, heartopen |
| Phase 3 | 更新导入 | 3 天 | heartopen |
| Phase 4 | 验证修复 | 2 天 | maintainer |
| **总计** | | **12 天** | |

---

## 九、相关文档

- [DIRECTORY_STRUCTURE.md](./docs/architecture/DIRECTORY_STRUCTURE.md) - 目标目录规范
- [RELEASE_GOVERNANCE.md](./RELEASE_GOVERNANCE.md) - 版本治理模型
- [BRANCH_GOVERNANCE.md](./BRANCH_GOVERNANCE.md) - 分支治理规范

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本，设计目录重构方案 |

---

*本文档由 yinglichina8848 创建*
