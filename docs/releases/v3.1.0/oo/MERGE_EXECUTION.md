# MERGE 语句执行链路

> **版本**: v3.1.0  
> **对应 Issue**: #613  
> **状态**: 开发中

---

## 1. 概述

### 1.1 语法

```sql
MERGE INTO target_table [AS alias]
USING source_table_or_query
ON join_condition
WHEN MATCHED THEN
    UPDATE SET col1 = expr1 [, col2 = expr2 ...]
    [WHERE condition]
WHEN NOT MATCHED THEN
    INSERT [(col1, col2, ...)] VALUES (expr1, expr2, ...)
    [WHERE condition]
```

### 1.2 功能描述

MERGE 等价于 INSERT + UPDATE + DELETE 的组合，用于：
- 数据同步（UPSERT）
- 缓慢变化维度（SCD Type 1/2）
- 数据仓库 ELT

---

## 2. 架构图

```
MERGE SQL
   │
   ▼
┌─────────────────┐
│  Parser          │ → MergeStmt AST
└─────────────────┘
   │
   ▼
┌─────────────────┐
│  Planner         │ → MergeLogicalPlan
└─────────────────┘
   │
   ├──► ┌──────────────────────┐
   │    │  Source Scan         │ (source query)
   │    └──────────────────────┘
   │           │
   ▼           ▼
┌─────────────────────┐
│  Hash Join (ON)      │ ← target table build
└─────────────────────┘
   │
   ├──► MATCHED ──► UPDATE / DELETE pipeline
   └──► NOT MATCHED ──► INSERT pipeline
```

---

## 3. 时序图

```
MERGE INTO orders o
USING new_orders n
ON o.order_id = n.order_id
WHEN MATCHED THEN UPDATE SET o.status = n.status
WHEN NOT MATCHED THEN INSERT (order_id, status) VALUES (n.order_id, n.status)

Timeline:
──────────────────────────────────────────────────────────────
t1: Parser → MergeStmt { target: "orders", source: "new_orders", ... }
t2: Planner → SourceScan(new_orders) + HashJoin(orders ⟕ new_orders)
t3: Executor: For each row in source:
         ├─ ON MATCHED → UPDATE orders SET status = new_orders.status
         └─ ON NOT MATCHED → INSERT INTO orders (...)
t4: Storage: B+Tree update + WAL write
t5: COMMIT with MERGE audit log
```

---

## 4. 状态图

```
MERGE Execution States:
─────────────────────────────────────────────────────────────
         ┌──────────────┐
    ───►│   INIT       │ ← Parse + Plan
         └──────┬───────┘
                ▼
         ┌──────────────┐
         │ SOURCE_SCAN  │ ← Scan source (new_orders)
         └──────┬───────┘
                ▼
         ┌──────────────┐
         │  HASH_BUILD  │ ← Build hash table from target (orders)
         └──────┬───────┘
                ▼
         ┌──────────────┐
         │   PROBE      │ ← Probe: for each source row, find match
         └──────┬───────┘
           ┌───┴───┐
           ▼       ▼
     ┌─────────┐ ┌─────────────┐
     │ MATCHED │ │NOT_MATCHED │
     └────┬────┘ └──────┬──────┘
          ▼              ▼
     ┌─────────┐ ┌───────────┐
     │UPDATE/  │ │  INSERT   │
     │DELETE   │ │           │
     └────┬────┘ └───────┬───┘
          └───────┬───────┘
                  ▼
           ┌─────────────┐
           │   DONE      │
           └─────────────┘
```

---

## 5. 算法实现

### 5.1 核心 Rust 实现框架

```rust
pub struct MergeExecutor {
    target: Arc<dyn Table>,
    source: Arc<dyn PhysicalPlan>,
    on_condition: Expression,
    matched: Option<MergeAction>,   // UPDATE/DELETE
    not_matched: Option<MergeAction>, // INSERT
}

pub enum MergeAction {
    Update { set: Vec<Assignment>, where: Option<Expression> },
    Insert { columns: Vec<String>, values: Vec<Expression>, where: Option<Expression> },
    Delete,
}

impl Executor for MergeExecutor {
    async fn execute(&self, ctx: &mut ExecutionContext) -> Result<RecordBatchStream> {
        // Step 1: Build hash table from target
        let mut hash_build = HashBuild::new();
        let target_rows = self.target.scan(ctx).await?;
        hash_build.build(target_rows);

        // Step 2: Probe with source rows
        let source = self.source.execute(ctx).await?;
        let mut matched_output = vec![];
        let mut not_matched_output = vec![];

        for batch in source {
            let probe_result = hash_build.probe(&batch, &self.on_condition);
            
            for (source_row, match_status) in probe_result {
                match match_status {
                    MatchStatus::Matched(ref existing) => {
                        if let Some(ref action) = self.matched {
                            self.execute_matched_action(action, existing, &source_row).await?;
                        }
                    }
                    MatchStatus::NotMatched => {
                        if let Some(ref action) = self.not_matched {
                            self.execute_not_matched_action(action, &source_row).await?;
                        }
                    }
                }
            }
        }
        Ok(Box::pin(futures::stream::empty()))
    }
}
```

### 5.2 MVCC 可见性处理

```
ON MATCHED 时的可见性：
1. 获取 source row 的快照（READ COMMITTED）
2. 获取 target row 的当前版本链
3. 如果 target row 已被其他事务锁定 → 等锁释放
4. UPDATE 时：写入新版本 + 旧版本的 next_ptr 指向新版本
5. DELETE 时：写入删除标记到 xmax

关键：MATCHED 判断基于 ON 条件，不是基于 MVCC 可见性
```

---

## 6. 测试计划

### 6.1 功能测试用例

| 测试用例 | SQL | 预期结果 |
|---------|-----|---------|
| MERGE-1 | 基本 MATCHED UPDATE | 匹配行被更新 |
| MERGE-2 | 基本 NOT MATCHED INSERT | 不匹配行被插入 |
| MERGE-3 | 条件 UPDATE | WHERE 子句生效 |
| MERGE-4 | 条件 INSERT | WHERE 子句生效 |
| MERGE-5 | 多条件 | 多个 WHEN 子句 |
| MERGE-6 | 带子查询 | USING 子查询 |
| MERGE-7 | 批量合并 | 1000 行 |

### 6.2 边界测试用例

| 测试用例 | SQL | 预期结果 |
|---------|-----|---------|
| MERGE-8 | 无匹配时 NOT MATCHED | 全部 INSERT |
| MERGE-9 | 全匹配时 MATCHED | 全部 UPDATE |
| MERGE-10 | NULL ON 条件 | 正确处理 NULL |
| MERGE-11 | 并发 MERGE | 无数据丢失 |

### 6.3 MVCC 测试

| 测试用例 | 场景 | 预期 |
|---------|------|------|
| MERGE-12 | 并发 UPDATE 同一行 | 只有一个成功 |
| MERGE-13 | 快照读取 | READ COMMITTED 行为 |

---

## 7. 覆盖率分析

| 阶段 | 当前覆盖率 | 目标 |
|------|-----------|------|
| Parser (MERGE keyword) | 0% → 100% | ✅ |
| Planner (MergeLogicalPlan) | 0% | 90% |
| Executor (MergeExecutor) | 0% | 90% |
| Storage (B+Tree update) | 0% | 85% |
| WAL (MERGE record) | 0% | 90% |

---

## 8. 相关文件

| 文件 | 作用 |
|------|------|
| `crates/parser/src/parser.rs` | MERGE 语法解析 |
| `crates/planner/src/logical_plan.rs` | MergeLogicalPlan |
| `crates/executor/src/merge.rs` | MergeExecutor |
| `crates/transaction/src/wal.rs` | WAL write |
| `crates/storage/src/bplus_tree/index.rs` | B+Tree update |
