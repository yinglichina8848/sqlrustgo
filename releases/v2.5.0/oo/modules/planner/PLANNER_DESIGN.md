# Planner 模块设计

**版本**: v2.5.0
**模块**: Planner (查询规划器)

---

## 一、What (是什么)

Planner 负责将 Parser 生成的 AST 转换为逻辑执行计划，处理子查询、视图、物化视图等复杂查询场景。

## 二、Why (为什么)

- **逻辑转换**: AST → 逻辑计划
- **语义分析**: 名称解析、类型推导
- **查询分解**: 子查询展开
- **视图处理**: 视图展开

## 三、核心流程

```rust
pub struct Planner {
    catalog: Arc<Catalog>,
    functions: FunctionRegistry,
}

impl Planner {
    // AST → 逻辑计划
    pub fn plan(&self, stmt: Statement) -> Result<LogicalPlan> {
        match stmt {
            Statement::Select(s) => self.plan_select(s),
            Statement::Insert(i) => self.plan_insert(i),
            Statement::Update(u) => self.plan_update(u),
            Statement::Delete(d) => self.plan_delete(d),
        }
    }
}
```

## 四、相关文档

- [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md)
- [OPTIMIZER_DESIGN.md](./optimizer/OPTIMIZER_DESIGN.md)

---

*Planner 模块设计 v2.5.0*
