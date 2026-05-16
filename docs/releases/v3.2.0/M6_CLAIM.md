# M6 任务认领声明

> **认领人**: Sisyphus (AI Agent)
> **认领时间**: 2026-05-15
> **任务**: M6 - 多表DML/HashJoin/MERGE
> **总控Issue**: #972 (v3.2.0 SQLRustGo Trusted GMP Data Platform)

---

## 任务范围

根据 Issue #972 (v3.2.0 总控) 的规划：

| Milestone | 日期 | 内容 |
|-----------|------|------|
| M6 | 2026-12-15 | 多表DML/HashJoin/MERGE |

### 三大子任务

1. **HashJoin 实现**
   - 状态: ✅ 已完整实现
   - 支持 INNER/LEFT/RIGHT/FULL/CROSS/SEMI/ANTI
   - CBO 代价模型已集成

2. **MERGE 增强**
   - 状态: ✅ 基本功能已实现
   - 需增强: 多表 MERGE 源支持

3. **多表 DML**
   - 状态: ⚠️ Parser 已支持, Executor 未实现
   - 需实现: 多表 UPDATE/DELETE 执行器

---

## 现有实现分析

### HashJoin (crates/planner/src/physical_plan.rs)

```rust
pub struct HashJoinExec {
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    join_type: crate::JoinType,
    condition: Option<Expr>,
    schema: Schema,
}
```

- ✅ Inner, Left, Right, Full, Cross, LeftSemi, LeftAnti, RightSemi, RightAnti
- ✅ CBO 代价模型 `crates/optimizer/src/cost.rs`
- ✅ 并行执行 `crates/executor/src/parallel_executor.rs`

### MERGE (tests/e2e/e2e_merge_test.rs)

```sql
MERGE INTO target USING source ON target.id = source.id
WHEN MATCHED THEN UPDATE SET target.value = source.value
WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)
```

- ✅ 基本语法支持
- ✅ WHEN MATCHED THEN UPDATE
- ✅ WHEN NOT MATCHED THEN INSERT
- ⚠️ 多表源可能需增强

### 多表 DML (tests/execution_chain_regression_test.rs)

```sql
-- 多表 UPDATE (Parser 支持, 执行器未实现)
UPDATE t1, t2 SET t1.val = t2.val WHERE t1.id = t2.id

-- 多表 DELETE (Parser 支持, 执行器未实现)
DELETE t1 FROM t1 INNER JOIN t2 ON t1.id = t2.id
```

- ✅ Parser: `crates/parser/src/parser.rs` 支持
- ❌ Executor: 执行逻辑缺失
- ⚠️ 测试被标记为 `#[ignore]`

---

## 开发计划

### Phase 1: 多表 UPDATE 执行器 (P0 - 核心缺口)

**目标**: 实现 `UPDATE t1, t2 SET ... WHERE ...` 执行

**实现位置**: `crates/executor/src/local_executor.rs`

**步骤**:
1. 添加 `execute_multi_table_update()` 方法
2. 解析多表引用，构建更新计划
3. 实现行级更新逻辑
4. 添加单元测试

**验收标准**:
- [ ] `UPDATE t1, t2 SET t1.x = t2.y WHERE t1.id = t2.id` 可执行
- [ ] 测试通过，无 panic

### Phase 2: 多表 DELETE 执行器 (P0 - 核心缺口)

**目标**: 实现 `DELETE t1 FROM t1 JOIN t2 ON ...` 执行

**实现位置**: `crates/executor/src/local_executor.rs`

**步骤**:
1. 添加 `execute_multi_table_delete()` 方法
2. 支持 JOIN 语法的 DELETE
3. 实现行级删除逻辑
4. 添加单元测试

**验收标准**:
- [ ] `DELETE t1 FROM t1 INNER JOIN t2 ON t1.id = t2.id` 可执行
- [ ] 测试通过，无 panic

### Phase 3: MERGE 多表源增强 (P1)

**目标**: 支持 MERGE 使用复杂 JOIN 作为源

**实现位置**: `crates/planner/src/planner.rs`

**步骤**:
1. 增强 MERGE 的 USING 子句支持 subquery/JOIN
2. 验证 planner 处理多表源
3. 添加测试用例

**验收标准**:
- [ ] `MERGE INTO t USING (SELECT * FROM s1 JOIN s2 ON ...)` 支持

### Phase 4: HashJoin 评估 (P2)

**目标**: 评估现有 HashJoin 是否满足需求

**检查项**:
- [ ] 性能基准测试
- [ ] 内存使用优化
- [ ] 必要时增强

---

## 实现细节

### 多表 UPDATE 执行器设计

```rust
// crates/executor/src/local_executor.rs

async fn execute_multi_table_update(
    &mut self,
    tables: &[TableRef],      // t1, t2
    set_clause: Vec<SetItem>, // t1.x = t2.y
    where_clause: Option<Expr>,
) -> Result<Vec<HashMap<String, Value>>, String> {
    // 1. 构建表依赖图
    // 2. 按依赖顺序处理更新
    // 3. 对于每个更新行, 执行 SET
    // 4. 处理 WHERE 条件
}
```

### 多表 DELETE 执行器设计

```rust
async fn execute_multi_table_delete(
    &mut self,
    target_table: TableRef,   // t1
    from_clause: TableRef,     // t1 JOIN t2
    where_clause: Option<Expr>,
) -> Result<Vec<HashMap<String, Value>>, String> {
    // 1. 解析 JOIN 条件
    // 2. 找到匹配行
    // 3. 从 target_table 删除
    // 4. 处理 WHERE 条件
}
```

---

## 分支策略

- 基于分支: `develop/v3.1.0`
- 功能分支: `feature/v320-m6-multitable-dml`
- 目标分支: `develop/v3.2.0` (创建后)

---

## 状态追踪

| 日期 | 状态 | 说明 |
|------|------|------|
| 2026-05-15 | ✅ 完成 | 分析现有实现状态 |
| 2026-05-15 | ✅ 完成 | 制定详细开发计划 |
| 2026-05-15 | 🔄 进行中 | 创建功能分支 |
| - | ⏳ 待开始 | Phase 1: 多表 UPDATE |
| - | ⏳ 待开始 | Phase 2: 多表 DELETE |
| - | ⏳ 待开始 | Phase 3: MERGE 增强 |
| - | ⏳ 待开始 | Phase 4: HashJoin 评估 |

---

## 验收条件

1. 多表 UPDATE 执行成功，测试通过
2. 多表 DELETE 执行成功，测试通过
3. MERGE 多表源增强通过测试
4. Clippy/Format 检查通过
5. 门禁测试通过

---

*Auto-declared and planned by Sisyphus AI Agent*