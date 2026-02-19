# 插件化执行引擎原型设计

> 版本：v1.0
> 日期：2026-02-18
> 目标：让执行策略可插拔，而不是写死在 executor 里

---

## 一、当前常见问题

### 1.1 典型问题代码

```rust
❌ fn execute(query: Query) {
    if query.type == SELECT {
        // SELECT 逻辑
    }
    if query.type == INSERT {
        // INSERT 逻辑
    }
    if query.type == UPDATE {
        // UPDATE 逻辑
    }
    // ... 巨型函数
}
```

**问题**：
- 强耦合
- 不可扩展
- 难以优化
- 难以测试

---

## 二、目标结构

```
Planner → LogicalPlan
    ↓
Optimizer → PhysicalPlan
    ↓
ExecutionEngine (可替换) → 执行 PhysicalPlan
    ↓
Storage
```

---

## 三、插件化执行引擎设计

### 3.1 定义核心 trait

```rust
pub trait ExecutionEngine: Send + Sync {
    fn execute(&self, plan: &PhysicalPlan) -> Result<QueryResult>;
    fn name(&self) -> &str;
}
```

### 3.2 定义执行计划

```rust
pub enum PhysicalPlan {
    TableScan {
        table_name: String,
        columns: Vec<String>,
    },
    IndexScan {
        table_name: String,
        index_name: String,
        key: Value,
    },
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Predicate,
    },
    Project {
        input: Box<PhysicalPlan>,
        columns: Vec<String>,
    },
    Insert {
        table_name: String,
        rows: Vec<Row>,
    },
    Update {
        table_name: String,
        updates: HashMap<String, Value>,
        predicate: Option<Predicate>,
    },
    Delete {
        table_name: String,
        predicate: Option<Predicate>,
    },
}
```

### 3.3 默认执行器

```rust
pub struct DefaultExecutor<S: StorageEngine> {
    storage: S,
}

impl<S: StorageEngine> ExecutionEngine for DefaultExecutor<S> {
    fn execute(&self, plan: &PhysicalPlan) -> Result<QueryResult> {
        match plan {
            PhysicalPlan::TableScan { table_name, columns } => {
                let rows = self.storage.scan(table_name)?;
                Ok(QueryResult::Rows(rows))
            }
            PhysicalPlan::Insert { table_name, rows } => {
                for row in rows {
                    self.storage.insert(table_name, row.clone())?;
                }
                Ok(QueryResult::AffectedRows(rows.len()))
            }
            PhysicalPlan::Filter { input, predicate } => {
                let result = self.execute(input)?;
                let filtered = Self::apply_filter(result, predicate)?;
                Ok(filtered)
            }
            PhysicalPlan::Project { input, columns } => {
                let result = self.execute(input)?;
                let projected = Self::apply_project(result, columns)?;
                Ok(projected)
            }
            _ => Err(Error::UnsupportedOperation),
        }
    }

    fn name(&self) -> &str {
        "default"
    }
}
```

### 3.4 插件执行器（并行执行）

```rust
pub struct ParallelExecutor<S: StorageEngine> {
    storage: S,
    workers: usize,
}

impl<S: StorageEngine> ExecutionEngine for ParallelExecutor<S> {
    fn execute(&self, plan: &PhysicalPlan) -> Result<QueryResult> {
        match plan {
            PhysicalPlan::TableScan { table_name, columns } => {
                let chunks = self.storage.scan_parallel(table_name, self.workers)?;
                let results: Vec<QueryResult> = chunks
                    .into_par_iter()
                    .map(|chunk| self.process_chunk(chunk))
                    .collect();
                Ok(Self::merge_results(results))
            }
            _ => {
                let default = DefaultExecutor::new(self.storage.clone());
                default.execute(plan)
            }
        }
    }

    fn name(&self) -> &str {
        "parallel"
    }
}
```

### 3.5 注册机制

```rust
pub struct EngineRegistry<S: StorageEngine> {
    engines: HashMap<String, Box<dyn ExecutionEngine>>,
    default: String,
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + Clone + 'static> EngineRegistry<S> {
    pub fn new(storage: S) -> Self {
        let mut registry = Self {
            engines: HashMap::new(),
            default: "default".to_string(),
            _phantom: PhantomData,
        };
        
        registry.register("default", Box::new(DefaultExecutor::new(storage.clone())));
        registry.register("parallel", Box::new(ParallelExecutor::new(storage, 4)));
        
        registry
    }

    pub fn register(&mut self, name: &str, engine: Box<dyn ExecutionEngine>) {
        self.engines.insert(name.to_string(), engine);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ExecutionEngine> {
        self.engines.get(name).map(|e| e.as_ref())
    }

    pub fn execute(&self, plan: &PhysicalPlan) -> Result<QueryResult> {
        let engine = self.engines.get(&self.default)
            .ok_or(Error::EngineNotFound)?;
        engine.execute(plan)
    }

    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if self.engines.contains_key(name) {
            self.default = name.to_string();
            Ok(())
        } else {
            Err(Error::EngineNotFound)
        }
    }
}
```

---

## 四、执行流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          执行流程                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   API                                                                       │
│    ↓                                                                        │
│   Parser                                                                    │
│    ↓                                                                        │
│   Planner ────► LogicalPlan                                                 │
│    ↓                                                                        │
│   Optimizer ──► PhysicalPlan                                                │
│    ↓                                                                        │
│   EngineRegistry.get(engine_name)                                           │
│    ↓                                                                        │
│   ExecutionEngine.execute(plan)  ← 可替换                                   │
│    ↓                                                                        │
│   Storage                                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 五、未来扩展能力

插件化后你可以支持：

| 执行器 | 用途 |
|:-------|:-----|
| DefaultExecutor | 默认单线程执行 |
| ParallelExecutor | 并行执行 |
| VectorizedExecutor | 向量化执行 |
| DistributedExecutor | 分布式执行 |
| GPUExecutor | GPU 加速 |
| TestExecutor | 测试模拟 |

**无需改核心代码**。

---

## 六、使用示例

```rust
fn main() -> Result<()> {
    let storage = MemoryStorage::new();
    let mut registry = EngineRegistry::new(storage);
    
    registry.register("vectorized", Box::new(VectorizedExecutor::new()));
    
    registry.set_default("parallel")?;
    
    let plan = planner.plan(query)?;
    let result = registry.execute(&plan)?;
    
    Ok(())
}
```

---

## 七、升级收益

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          升级收益                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ✅ 执行策略可扩展                                                         │
│   ✅ 架构层次清晰                                                           │
│   ✅ 性能优化空间大                                                         │
│   ✅ 可支持长期演进                                                         │
│   ✅ 测试更容易                                                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 八、关键一句话

```
如果不插件化 executor：
3 年后它会变成 2000 行巨型函数。

插件化后：
执行引擎成为可插拔内核。
```

---

*本文档由 TRAE (GLM-5.0) 创建*
