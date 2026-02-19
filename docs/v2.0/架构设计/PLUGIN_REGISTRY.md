# 插件注册机制完整实现

> 版本：v1.0
> 日期：2026-02-18
> 目标：内核级可插拔架构

---

## 一、插件类型分类

```rust
pub trait StoragePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn create_source(&self, config: &str) -> Result<Arc<dyn TableSource>>;
}

pub trait ExecutionPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn create_executor(&self, config: &str) -> Result<Arc<dyn ExecutionEngine>>;
}

pub trait OptimizerRule: Send + Sync {
    fn name(&self) -> &str;
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
}

pub trait FunctionPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn functions(&self) -> Vec<Arc<dyn ScalarUDF>>;
    fn aggregates(&self) -> Vec<Arc<dyn AggregateUDF>>;
}
```

---

## 二、注册中心

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct PluginRegistry {
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    executor: RwLock<HashMap<String, Arc<dyn ExecutionPlugin>>>,
    optimizer_rules: RwLock<Vec<Arc<dyn OptimizerRule>>>,
    functions: RwLock<HashMap<String, Arc<dyn FunctionPlugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
            executor: RwLock::new(HashMap::new()),
            optimizer_rules: RwLock::new(Vec::new()),
            functions: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn register_storage(&self, name: &str, plugin: Arc<dyn StoragePlugin>) {
        self.storage.write().unwrap().insert(name.to_string(), plugin);
    }
    
    pub fn register_executor(&self, name: &str, plugin: Arc<dyn ExecutionPlugin>) {
        self.executor.write().unwrap().insert(name.to_string(), plugin);
    }
    
    pub fn register_optimizer_rule(&self, rule: Arc<dyn OptimizerRule>) {
        self.optimizer_rules.write().unwrap().push(rule);
    }
    
    pub fn register_function(&self, name: &str, plugin: Arc<dyn FunctionPlugin>) {
        self.functions.write().unwrap().insert(name.to_string(), plugin);
    }
    
    pub fn get_storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>> {
        self.storage.read().unwrap().get(name).cloned()
    }
    
    pub fn get_executor(&self, name: &str) -> Option<Arc<dyn ExecutionPlugin>> {
        self.executor.read().unwrap().get(name).cloned()
    }
    
    pub fn get_optimizer_rules(&self) -> Vec<Arc<dyn OptimizerRule>> {
        self.optimizer_rules.read().unwrap().clone()
    }
    
    pub fn list_storage_plugins(&self) -> Vec<String> {
        self.storage.read().unwrap().keys().cloned().collect()
    }
    
    pub fn list_executor_plugins(&self) -> Vec<String> {
        self.executor.read().unwrap().keys().cloned().collect()
    }
}
```

---

## 三、动态注册

### 3.1 存储插件注册

```rust
pub struct MemoryStoragePlugin;

impl StoragePlugin for MemoryStoragePlugin {
    fn name(&self) -> &str {
        "memory"
    }
    
    fn create_source(&self, config: &str) -> Result<Arc<dyn TableSource>> {
        Ok(Arc::new(MemorySource::new()))
    }
}

pub struct FileStoragePlugin;

impl StoragePlugin for FileStoragePlugin {
    fn name(&self) -> &str {
        "file"
    }
    
    fn create_source(&self, config: &str) -> Result<Arc<dyn TableSource>> {
        Ok(Arc::new(FileSource::new(config)))
    }
}

impl PluginRegistry {
    pub fn register_default_plugins(&self) {
        self.register_storage("memory", Arc::new(MemoryStoragePlugin));
        self.register_storage("file", Arc::new(FileStoragePlugin));
        self.register_executor("default", Arc::new(DefaultExecutorPlugin));
        self.register_executor("parallel", Arc::new(ParallelExecutorPlugin));
    }
}
```

### 3.2 执行器插件注册

```rust
pub struct DefaultExecutorPlugin;

impl ExecutionPlugin for DefaultExecutorPlugin {
    fn name(&self) -> &str {
        "default"
    }
    
    fn create_executor(&self, config: &str) -> Result<Arc<dyn ExecutionEngine>> {
        Ok(Arc::new(DefaultExecutor::new()))
    }
}

pub struct ParallelExecutorPlugin;

impl ExecutionPlugin for ParallelExecutorPlugin {
    fn name(&self) -> &str {
        "parallel"
    }
    
    fn create_executor(&self, config: &str) -> Result<Arc<dyn ExecutionEngine>> {
        let workers = config.parse().unwrap_or(4);
        Ok(Arc::new(ParallelExecutor::new(workers)))
    }
}
```

### 3.3 优化器规则注册

```rust
pub struct PredicatePushdownRule;

impl OptimizerRule for PredicatePushdownRule {
    fn name(&self) -> &str {
        "predicate_pushdown"
    }
    
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        PredicatePushdown::new().optimize(plan)
    }
}

pub struct ProjectionPruningRule;

impl OptimizerRule for ProjectionPruningRule {
    fn name(&self) -> &str {
        "projection_pruning"
    }
    
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        ProjectionPruning::new().optimize(plan)
    }
}

impl PluginRegistry {
    pub fn register_default_rules(&self) {
        self.register_optimizer_rule(Arc::new(PredicatePushdownRule));
        self.register_optimizer_rule(Arc::new(ProjectionPruningRule));
        self.register_optimizer_rule(Arc::new(ConstantFoldingRule));
        self.register_optimizer_rule(Arc::new(JoinReorderRule));
    }
}
```

---

## 四、启动加载机制

### 4.1 静态编译插件

```rust
pub fn create_engine() -> Engine {
    let registry = PluginRegistry::new();
    
    registry.register_storage("memory", Arc::new(MemoryStoragePlugin));
    registry.register_storage("file", Arc::new(FileStoragePlugin));
    
    Engine::new(registry)
}
```

### 4.2 Feature Flag 插件

```rust
pub fn create_engine() -> Engine {
    let registry = PluginRegistry::new();
    
    registry.register_storage("memory", Arc::new(MemoryStoragePlugin));
    
    #[cfg(feature = "file-storage")]
    registry.register_storage("file", Arc::new(FileStoragePlugin));
    
    #[cfg(feature = "s3-storage")]
    registry.register_storage("s3", Arc::new(S3StoragePlugin));
    
    Engine::new(registry)
}
```

### 4.3 配置文件加载

```yaml
plugins:
  storage:
    - name: memory
      enabled: true
    - name: file
      enabled: true
      config:
        base_path: /data
    - name: s3
      enabled: false
      
  executor:
    - name: default
      enabled: true
    - name: parallel
      enabled: true
      config:
        workers: 4
```

```rust
pub fn load_plugins_from_config(config: &Config) -> Result<PluginRegistry> {
    let registry = PluginRegistry::new();
    
    for storage_config in &config.plugins.storage {
        if storage_config.enabled {
            match storage_config.name.as_str() {
                "memory" => registry.register_storage("memory", Arc::new(MemoryStoragePlugin)),
                "file" => registry.register_storage("file", Arc::new(FileStoragePlugin::new(&storage_config.config))),
                _ => log::warn!("Unknown storage plugin: {}", storage_config.name),
            }
        }
    }
    
    Ok(registry)
}
```

---

## 五、插件生命周期

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          插件生命周期                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 注册阶段                                                               │
│      registry.register_xxx(name, plugin)                                    │
│                                                                              │
│   2. 初始化阶段                                                             │
│      plugin.initialize(config)                                              │
│                                                                              │
│   3. 运行阶段                                                               │
│      plugin.create_xxx()                                                    │
│                                                                              │
│   4. 关闭阶段                                                               │
│      plugin.shutdown()                                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 六、使用示例

```rust
fn main() -> Result<()> {
    let registry = PluginRegistry::new();
    registry.register_default_plugins();
    
    let storage = registry.get_storage("memory").unwrap();
    let source = storage.create_source("")?;
    
    let executor = registry.get_executor("parallel").unwrap();
    let engine = executor.create_executor("4")?;
    
    let result = engine.execute(plan)?;
    
    Ok(())
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
