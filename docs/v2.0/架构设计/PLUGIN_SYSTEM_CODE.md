# 插件系统真实代码结构

> 版本：v1.0
> 日期：2026-02-18
> 类型：工程可执行版本

---

## 一、目录结构

```
sqlrustgo/
├── core/
│   ├── logical/
│   │   ├── mod.rs
│   │   ├── plan.rs
│   │   └── expr.rs
│   ├── physical/
│   │   ├── mod.rs
│   │   ├── plan.rs
│   │   └── expr.rs
│   ├── optimizer/
│   │   ├── mod.rs
│   │   ├── rule.rs
│   │   └── cost.rs
│   └── execution/
│       ├── mod.rs
│       ├── engine.rs
│       └── context.rs
├── plugins/
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── memory.rs
│   │   └── file.rs
│   ├── functions/
│   │   ├── mod.rs
│   │   ├── scalar.rs
│   │   └── aggregate.rs
│   └── optimizer_rules/
│       ├── mod.rs
│       ├── predicate_pushdown.rs
│       └── projection_pruning.rs
├── registry/
│   ├── mod.rs
│   └── plugin_registry.rs
└── lib.rs
```

---

## 二、插件 trait 设计

### 2.1 基础 Plugin trait

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        ""
    }
}
```

### 2.2 Storage 插件

```rust
pub trait StoragePlugin: Plugin {
    fn scan(
        &self,
        table: &str,
        projection: Option<&[usize]>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn PhysicalPlan>>;
    
    fn insert(&self, table: &str, batch: RecordBatch) -> Result<usize>;
    
    fn create_table(&self, name: &str, schema: Schema) -> Result<()>;
    
    fn drop_table(&self, name: &str) -> Result<()>;
    
    fn table_exists(&self, name: &str) -> bool;
    
    fn table_stats(&self, name: &str) -> Result<TableStatistics>;
}
```

### 2.3 Function 插件

```rust
pub trait ScalarFunctionPlugin: Plugin {
    fn function(&self) -> Arc<dyn ScalarUDF>;
}

pub trait AggregateFunctionPlugin: Plugin {
    fn function(&self) -> Arc<dyn AggregateUDF>;
}

pub trait ScalarUDF: Send + Sync {
    fn name(&self) -> &str;
    fn signature(&self) -> &Signature;
    fn return_type(&self, args: &[DataType]) -> Result<DataType>;
    fn invoke(&self, args: &[ArrayRef]) -> Result<ArrayRef>;
}

pub trait AggregateUDF: Send + Sync {
    fn name(&self) -> &str;
    fn signature(&self) -> &Signature;
    fn return_type(&self, args: &[DataType]) -> Result<DataType>;
    fn accumulator(&self, args: &[DataType]) -> Result<Box<dyn Accumulator>>;
}
```

### 2.4 Optimizer Rule 插件

```rust
pub trait OptimizerRulePlugin: Plugin {
    fn rule(&self) -> Arc<dyn OptimizerRule>;
}

pub trait OptimizerRule: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
}
```

### 2.5 Executor 插件

```rust
pub trait ExecutorPlugin: Plugin {
    fn create_executor(&self, config: &ExecutorConfig) -> Result<Arc<dyn ExecutionEngine>>;
}

pub trait ExecutionEngine: Send + Sync {
    fn execute(&self, plan: &PhysicalPlan) -> Result<RecordBatchStream>;
    fn name(&self) -> &str;
}
```

---

## 三、注册器实现

### 3.1 PluginRegistry

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct PluginRegistry {
    storage_plugins: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    scalar_functions: RwLock<HashMap<String, Arc<dyn ScalarFunctionPlugin>>>,
    aggregate_functions: RwLock<HashMap<String, Arc<dyn AggregateFunctionPlugin>>>,
    optimizer_rules: RwLock<Vec<Arc<dyn OptimizerRulePlugin>>>,
    executors: RwLock<HashMap<String, Arc<dyn ExecutorPlugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            storage_plugins: RwLock::new(HashMap::new()),
            scalar_functions: RwLock::new(HashMap::new()),
            aggregate_functions: RwLock::new(HashMap::new()),
            optimizer_rules: RwLock::new(Vec::new()),
            executors: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn register_storage(&self, plugin: Arc<dyn StoragePlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        self.storage_plugins.write().unwrap().insert(name, plugin);
        Ok(())
    }
    
    pub fn register_scalar_function(&self, plugin: Arc<dyn ScalarFunctionPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        self.scalar_functions.write().unwrap().insert(name, plugin);
        Ok(())
    }
    
    pub fn register_aggregate_function(&self, plugin: Arc<dyn AggregateFunctionPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        self.aggregate_functions.write().unwrap().insert(name, plugin);
        Ok(())
    }
    
    pub fn register_optimizer_rule(&self, plugin: Arc<dyn OptimizerRulePlugin>) -> Result<()> {
        self.optimizer_rules.write().unwrap().push(plugin);
        Ok(())
    }
    
    pub fn register_executor(&self, plugin: Arc<dyn ExecutorPlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        self.executors.write().unwrap().insert(name, plugin);
        Ok(())
    }
    
    pub fn get_storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>> {
        self.storage_plugins.read().unwrap().get(name).cloned()
    }
    
    pub fn get_scalar_function(&self, name: &str) -> Option<Arc<dyn ScalarFunctionPlugin>> {
        self.scalar_functions.read().unwrap().get(name).cloned()
    }
    
    pub fn get_aggregate_function(&self, name: &str) -> Option<Arc<dyn AggregateFunctionPlugin>> {
        self.aggregate_functions.read().unwrap().get(name).cloned()
    }
    
    pub fn get_executor(&self, name: &str) -> Option<Arc<dyn ExecutorPlugin>> {
        self.executors.read().unwrap().get(name).cloned()
    }
    
    pub fn optimizer_rules(&self) -> Vec<Arc<dyn OptimizerRulePlugin>> {
        self.optimizer_rules.read().unwrap().clone()
    }
    
    pub fn list_storage_plugins(&self) -> Vec<String> {
        self.storage_plugins.read().unwrap().keys().cloned().collect()
    }
    
    pub fn list_executors(&self) -> Vec<String> {
        self.executors.read().unwrap().keys().cloned().collect()
    }
}
```

---

## 四、内置插件实现

### 4.1 Memory Storage Plugin

```rust
pub struct MemoryStoragePlugin {
    tables: RwLock<HashMap<String, MemoryTable>>,
}

struct MemoryTable {
    schema: Schema,
    data: Vec<RecordBatch>,
}

impl MemoryStoragePlugin {
    pub fn new() -> Self {
        Self {
            tables: RwLock::new(HashMap::new()),
        }
    }
}

impl Plugin for MemoryStoragePlugin {
    fn name(&self) -> &str {
        "memory"
    }
    
    fn description(&self) -> &str {
        "In-memory storage plugin"
    }
}

impl StoragePlugin for MemoryStoragePlugin {
    fn scan(
        &self,
        table: &str,
        projection: Option<&[usize]>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn PhysicalPlan>> {
        let tables = self.tables.read().unwrap();
        let table = tables.get(table).ok_or(Error::TableNotFound)?;
        
        Ok(Arc::new(MemoryScanExec::new(
            table.schema.clone(),
            table.data.clone(),
            projection.map(|p| p.to_vec()),
            filters.to_vec(),
            limit,
        )))
    }
    
    fn insert(&self, table: &str, batch: RecordBatch) -> Result<usize> {
        let mut tables = self.tables.write().unwrap();
        let table = tables.get_mut(table).ok_or(Error::TableNotFound)?;
        let rows = batch.row_count;
        table.data.push(batch);
        Ok(rows)
    }
    
    fn create_table(&self, name: &str, schema: Schema) -> Result<()> {
        let mut tables = self.tables.write().unwrap();
        tables.insert(name.to_string(), MemoryTable {
            schema,
            data: Vec::new(),
        });
        Ok(())
    }
    
    fn drop_table(&self, name: &str) -> Result<()> {
        let mut tables = self.tables.write().unwrap();
        tables.remove(name).ok_or(Error::TableNotFound)?;
        Ok(())
    }
    
    fn table_exists(&self, name: &str) -> bool {
        self.tables.read().unwrap().contains_key(name)
    }
    
    fn table_stats(&self, name: &str) -> Result<TableStatistics> {
        let tables = self.tables.read().unwrap();
        let table = tables.get(name).ok_or(Error::TableNotFound)?;
        
        let row_count: usize = table.data.iter().map(|b| b.row_count).sum();
        
        Ok(TableStatistics {
            row_count,
            column_stats: HashMap::new(),
        })
    }
}
```

### 4.2 Predicate Pushdown Rule Plugin

```rust
pub struct PredicatePushdownRulePlugin;

impl Plugin for PredicatePushdownRulePlugin {
    fn name(&self) -> &str {
        "predicate_pushdown"
    }
}

impl OptimizerRulePlugin for PredicatePushdownRulePlugin {
    fn rule(&self) -> Arc<dyn OptimizerRule> {
        Arc::new(PredicatePushdownRule)
    }
}

pub struct PredicatePushdownRule;

impl OptimizerRule for PredicatePushdownRule {
    fn name(&self) -> &str {
        "predicate_pushdown"
    }
    
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        match plan {
            LogicalPlan::Filter { input, predicate } => {
                match *input {
                    LogicalPlan::TableScan { table_name, source, projection, filters, limit } => {
                        Ok(LogicalPlan::TableScan {
                            table_name,
                            source,
                            projection,
                            filters: vec![predicate],
                            limit,
                        })
                    }
                    _ => {
                        let new_input = self.apply(*input)?;
                        Ok(LogicalPlan::Filter {
                            input: Box::new(new_input),
                            predicate,
                        })
                    }
                }
            }
            _ => {
                let new_children: Result<Vec<_>> = plan
                    .children()
                    .into_iter()
                    .map(|c| self.apply(c))
                    .collect();
                Ok(plan.with_new_children(new_children?))
            }
        }
    }
}
```

---

## 五、使用示例

### 5.1 初始化引擎

```rust
fn create_engine() -> Result<Engine> {
    let registry = Arc::new(PluginRegistry::new());
    
    registry.register_storage(Arc::new(MemoryStoragePlugin::new()))?;
    registry.register_storage(Arc::new(FileStoragePlugin::new()))?;
    
    registry.register_optimizer_rule(Arc::new(PredicatePushdownRulePlugin))?;
    registry.register_optimizer_rule(Arc::new(ProjectionPruningRulePlugin))?;
    
    registry.register_executor(Arc::new(DefaultExecutorPlugin))?;
    registry.register_executor(Arc::new(ParallelExecutorPlugin::new(4)))?;
    
    Ok(Engine::new(registry))
}
```

### 5.2 执行查询

```rust
fn main() -> Result<()> {
    let engine = create_engine()?;
    
    let storage = engine.registry().get_storage("memory").unwrap();
    storage.create_table("users", schema)?;
    storage.insert("users", batch)?;
    
    let executor = engine.registry().get_executor("parallel").unwrap();
    let engine = executor.create_executor(&config)?;
    
    let result = engine.execute(&plan)?;
    
    for batch in result {
        println!("{:?}", batch?);
    }
    
    Ok(())
}
```

---

## 六、插件配置

### 6.1 配置文件

```yaml
plugins:
  storage:
    - name: memory
      enabled: true
    - name: file
      enabled: true
      config:
        base_path: /data/sqlrustgo
        
  executor:
    - name: default
      enabled: true
    - name: parallel
      enabled: true
      config:
        workers: 4
        
  optimizer_rules:
    - name: predicate_pushdown
      enabled: true
    - name: projection_pruning
      enabled: true
```

### 6.2 配置加载

```rust
pub fn load_plugins_from_config(config: &Config) -> Result<Arc<PluginRegistry>> {
    let registry = Arc::new(PluginRegistry::new());
    
    for storage_config in &config.plugins.storage {
        if !storage_config.enabled {
            continue;
        }
        
        match storage_config.name.as_str() {
            "memory" => {
                registry.register_storage(Arc::new(MemoryStoragePlugin::new()))?;
            }
            "file" => {
                let base_path = storage_config.config
                    .get("base_path")
                    .map(|s| s.as_str())
                    .unwrap_or("/tmp");
                registry.register_storage(Arc::new(FileStoragePlugin::new(base_path)))?;
            }
            _ => {
                log::warn!("Unknown storage plugin: {}", storage_config.name);
            }
        }
    }
    
    Ok(registry)
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
