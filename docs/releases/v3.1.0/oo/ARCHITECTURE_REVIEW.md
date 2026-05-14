# v3.1.0 架构评审与优化建议

> **版本**: v3.1.0 GA | **日期**: 2026-05-15
> **基于**: GitNexus 代码分析 (67,755 符号, 102,165 关系, 300 执行流) + OO 文档审查 + 性能基准数据

---

## 1. 架构总体评估

### 1.1 评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 模块化 | 9/10 | 47个workspace crate，职责清晰 |
| 可扩展性 | 8/10 | StorageEngine trait 抽象良好 |
| 性能 | 6/10 | MySQL协议瓶颈严重 |
| 正确性 | 7/10 | FULL OUTER JOIN/三表JOIN有gap |
| 安全性 | 8/10 | AES-256, RBAC, 审计链 |
| 可维护性 | 7/10 | 部分模块代码量大（stored_proc 3451行） |
| **综合** | **7.5/10** | |

### 1.2 架构优势

```
✅ 分层架构清晰: Parser → Planner → Optimizer → Executor → Storage
✅ StorageEngine trait 抽象: Memory/File/Columnar 可插拔
✅ MVCC 事务模型: 快照隔离 + SSI 可串行化
✅ 多引擎支持: 行存/列存/向量/图/GIS
✅ Rust 内存安全: 编译时保证
✅ 模块化 crate: 47个独立包，依赖关系清晰
```

### 1.3 架构问题

```
⚠️ MySQL 协议层与执行层紧耦合: mysql-server 直接调用 MemoryExecutionEngine
⚠️ 存储过程/触发器以字符串存储和执行: 每次需要重新解析
⚠️ 约束验证使用全表扫描: validate_primary_key/foreign_key O(k)
⚠️ JOIN 规划器与执行器有 gap: 规划支持但执行器未实现
⚠️ 缺少查询缓存: 相同SQL重复执行无缓存
⚠️ Buffer Pool 单实例: 高并发下 LRU 淘汰效率低
```

---

## 2. 关键架构问题分析

### 2.1 MySQL 协议层紧耦合

**问题**: `mysql-server` crate 直接依赖 `MemoryExecutionEngine`，无法使用 `FileStorage`

```
当前:
mysql-server → MemoryExecutionEngine → MemoryStorage
                                        (无持久化)

期望:
mysql-server → ExecutionEngine trait → MemoryStorage (测试)
                                    → FileStorage (生产)
```

**影响**: 通过 MySQL 协议连接时无法使用持久化存储

**修复建议**:
1. 在 mysql-server 中使用 `Arc<RwLock<dyn StorageEngine>>` 替代具体类型
2. 通过配置选择 MemoryStorage 或 FileStorage
3. 预计工作量: 中 (1周)

### 2.2 存储过程/触发器解释执行

**问题**: 存储过程和触发器体以字符串存储，每次执行都需要 `parse()` + `expand_variables()` + `execute_sql()`

```
当前 (解释执行):
每次 CALL → parse(body_string) → expand_vars → execute_sql
  开销: O(n) 解析 + O(n*m) 变量展开 + O(k) 执行

期望 (预编译):
CREATE PROCEDURE → parse(body) → compile_to_bytecode
每次 CALL → bind_params → execute_bytecode
  开销: O(1) 绑定 + O(k) 执行
```

**影响**: 高频调用的存储过程性能差

**修复建议**:
1. 在 CREATE PROCEDURE 时预编译为 `Vec<CompiledStatement>`
2. 运行时只做参数绑定，跳过解析
3. 预计工作量: 大 (3-4周)

### 2.3 约束验证全表扫描

**问题**: `validate_primary_key()`, `validate_foreign_keys()`, `validate_unique_constraints()` 都使用全表扫描

```
当前:
validate_primary_key(table, row)
  → storage.scan(table)  // O(k) 全表扫描
  → check each row       // O(k) 逐行比较

期望:
validate_primary_key(table, row)
  → btree.search(key)    // O(log k) B+Tree 查找
  → check if exists
```

**影响**: 数据量大时 INSERT/UPDATE 性能急剧下降

**修复建议**:
1. 使用 B+Tree 索引进行主键/唯一约束验证: O(log k)
2. 使用外键索引进行引用完整性验证: O(log k)
3. 预计工作量: 中 (1-2周)

### 2.4 JOIN 规划器与执行器 Gap

**问题**: JoinPlanner 支持 FULL OUTER JOIN 和三表 JOIN，但 LocalExecutor 未实现

```
JoinPlanner 输出:
  JoinPlan {
    base_table: "orders",
    joins: [
      JoinStep { right_table: "customers", on: ... },  // ✅ 已实现
      JoinStep { right_table: "products", on: ... },    // ❌ 未实现
    ]
  }

LocalExecutor:
  execute_hash_join() → 只处理两个输入源
  execute_full_outer_join() → 不存在
```

**影响**: 复杂 JOIN 查询结果不正确

**修复建议**:
1. 实现 `execute_full_outer_join()`: LEFT JOIN + 反连接
2. 实现多表 JOIN: 逐步执行 JoinPlan（左深树）
3. 预计工作量: 中 (1-2周)

### 2.5 Sort/Limit 算子未实现

**问题**: `execute_sort()` 和 `execute_limit()` 直接返回子节点结果，无实际排序/限制

```rust
// 当前实现 (local_executor.rs)
fn execute_sort(&mut self, plan: &PhysicalPlan) -> Result<Vec<Record>> {
    self.execute(plan.child()) // ⚠️ 直接返回，未排序!
}

fn execute_limit(&mut self, plan: &PhysicalPlan) -> Result<Vec<Record>> {
    self.execute(plan.child()) // ⚠️ 直接返回，未限制!
}
```

**影响**: ORDER BY 和 LIMIT 语句不生效，查询结果不正确

**修复建议**:
1. `execute_sort()`: 实现 Vec::sort_by() 或外部归并排序
2. `execute_limit()`: 截断结果集到 limit 行
3. 预计工作量: 小 (3-5天)

### 2.6 Buffer Pool LRU 更新 O(N)

**问题**: BufferPool.get() 中 `lru.retain(|&id| id != page_id)` 是 O(N) 操作

```rust
// 当前实现 (buffer_pool.rs)
fn get(&self, page_id: u32) -> Option<Arc<Page>> {
    let pages = self.pages.lock().unwrap();
    if pages.contains_key(&page_id) {
        let mut lru = self.lru.lock().unwrap();
        lru.retain(|&id| id != page_id);  // O(N) ⚠️
        lru.push_front(page_id);
    }
}
```

**影响**: 每次页面访问都遍历整个 LRU 队列，高并发下性能极差

**修复建议**:
1. 改用 HashMap<u32, NodePtr> + 双向链表实现 O(1) LRU
2. 预计工作量: 中 (1周)

### 2.7 B+Tree 分裂不向上传播

**问题**: B+Tree insert_at() 分裂叶节点后不处理父节点，树高度不增长

```
当前: insert_at() → 分裂叶节点 → 返回 (不更新父节点)
结果: 大量数据后树退化为宽叶节点，搜索退化为线性扫描

期望: insert_at() → 分裂叶节点 → 递归向上传播 → 更新父节点/创建新根
```

**影响**: 大量数据后 B+Tree 性能退化

**修复建议**:
1. 实现 handle_split() 递归向上传播
2. 预计工作量: 中 (1-2周)

---

## 3. 性能优化建议

### 3.1 MySQL 协议优化 (P0)

| 阶段 | 优化项 | 预期提升 | 工作量 |
|------|--------|---------|--------|
| Phase 1 | 移除强制 flush，批量发送 | 50-100% | 小 |
| Phase 2 | 缓存 uppercase 表名 | 10-20% | 小 |
| Phase 3 | 栈分配替代堆分配 | 20-30% | 中 |

**目标**: Sysbench point_select ≥ 30,000 QPS (当前 1,688 QPS)

### 3.2 存储引擎优化 (P1)

| 优化项 | 当前 | 目标 | 方法 |
|--------|------|------|------|
| Buffer Pool | 单实例 LRU | 多实例分区 LRU | 参考InnoDB设计 |
| B+Tree 页合并 | 不支持 | 支持页合并 | 删除后检查填充因子 |
| 列式存储 | 基础 | 向量化+压缩 | Parquet格式 |
| 查询缓存 | 无 | 基于SQL哈希 | LRU缓存+失效策略 |

### 3.3 查询优化器优化 (P2)

| 优化项 | 当前 | 目标 | 方法 |
|--------|------|------|------|
| JOIN 顺序 | 贪心 | DP+贪心 | 小表数DP，大表贪心 |
| 索引选择 | 基础 | 成本感知 | 考虑随机IO成本 |
| 子查询 | 重新执行 | 物化缓存 | CTE结果缓存 |
| 统计信息 | 手动 | 自动收集 | ANALYZE TABLE |

---

## 4. 架构演进路线图

```
v3.1.1 (Hotfix, 2周)
├── FULL OUTER JOIN 执行器
├── 三表 JOIN 执行器
├── MySQL 协议 Phase 1 优化 (移除强制 flush)
├── 约束验证使用 B+Tree 索引
└── 版本号统一

v3.2.0 (Feature, 4-6周)
├── MySQL 协议 Phase 2+3 优化
├── 递归 CTE
├── JSON 函数
├── 存储过程预编译
├── Buffer Pool 多实例
├── MERGE INTO 执行器
└── 查询缓存

v3.3.0 (Advanced, 6-8周)
├── GROUPING SETS / ROLLUP / CUBE
├── LATERAL JOIN
├── Group Replication
├── 在线 DDL
├── FTS 布尔模式 + 相关性排序
├── GIS 空间函数
└── NL2SQL LLM 集成

v4.0.0 (Enterprise, 12周+)
├── 分布式查询 (跨节点 JOIN)
├── 自适应索引 (Adaptive Index)
├── 列存引擎 v2 (Parquet)
├── 自动统计信息收集
├── 查询自动调优
└── 多租户隔离
```

---

## 5. 技术债务优先级

| 优先级 | 债务 | 影响 | 修复版本 | 工作量 |
|--------|------|------|---------|--------|
| P0 | MySQL 协议 109x 性能 | 生产不可用 | v3.1.1 | 1周 |
| P0 | Sort/Limit 算子未实现 | ORDER BY/LIMIT 不生效 | v3.1.1 | 3-5天 |
| P0 | B+Tree 分裂不向上传播 | 大量数据后树退化 | v3.1.1 | 1-2周 |
| P0 | CompositeKey 只取首列 | 多列索引不可用 | v3.1.1 | 1周 |
| P1 | JOIN 执行器 gap (FULL/三表) | SQL 兼容性 | v3.1.1 | 1-2周 |
| P1 | 约束验证全表扫描 | INSERT 性能 | v3.1.1 | 1-2周 |
| P1 | Buffer Pool LRU O(N) | 高并发性能 | v3.2.0 | 1周 |
| P1 | SSI 未实现 | 无法 SERIALIZABLE | v3.2.0 | 2-3周 |
| P1 | CBO 无统计信息 | 代价估算不准 | v3.2.0 | 2周 |
| P1 | 存储过程解释执行 | SP 性能 | v3.2.0 | 3-4周 |
| P1 | MySQL 协议紧耦合 | 架构灵活性 | v3.2.0 | 1周 |
| P2 | MVCC 版本链无 GC | 内存增长 | v3.2.0 | 1周 |
| P2 | FTS 模糊搜索 O(T) | 搜索性能 | v3.3.0 | 2周 |
| P2 | R-Tree 二次分裂 O(n²) | GIS 插入性能 | v3.3.0 | 1周 |
| P2 | 范围锁线性扫描 | 锁获取性能 | v3.3.0 | 1周 |
| P2 | WAL PITR 全量扫描 | 恢复性能 | v3.3.0 | 1周 |

---

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v1.0 | v3.1.0 架构评审与优化建议 |
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充 Sort/Limit/B+Tree/BufferPool 问题，扩展技术债务 |
