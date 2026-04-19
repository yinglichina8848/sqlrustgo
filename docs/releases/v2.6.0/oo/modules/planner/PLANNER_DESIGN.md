# Planner 模块设计

**版本**: v2.6.0
**模块**: Planner (查询规划器)

---

## 一、What (是什么)

Planner 负责将 Parser 生成的 AST 转换为逻辑执行计划。

## 二、核心流程

```rust
pub struct Planner {
    catalog: Arc<Catalog>,
    functions: FunctionRegistry,
}

impl Planner {
    pub fn plan(&self, stmt: Statement) -> Result<LogicalPlan> {
        match stmt {
            Statement::Select(s) => self.plan_select(s),
            Statement::Insert(i) => self.plan_insert(i),
            // ...
        }
    }
}
```

## 三、相关文档

- [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md)

---

*Planner 模块设计 v2.6.0*
