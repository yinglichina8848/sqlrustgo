# 配置热更新工具

## 概述

SQLRustGo 提供运行时配置热更新能力，支持在不重启服务的情况下修改配置。

## 功能特性

- ✅ **文件监听** - 自动检测配置文件变化
- ✅ **原子更新** - 配置更新是原子性的
- ✅ **变更回调** - 支持配置变更监听器
- ✅ **验证机制** - 更新前验证配置合法性
- ✅ **多配置支持** - Database、Log、Cache 分别管理

## 概念说明

### ConfigManager

配置管理器，负责加载、保存和更新配置。

```rust
let manager = ConfigManager::new(PathBuf::from("config.json"))?;
```

### ConfigWatcher

配置监视器，轮询检测配置文件变化并自动重载。

```rust
let watcher = ConfigWatcher::new(config_path, poll_interval_secs)?;
watcher.start_watching();
```

### ConfigListener

配置变更监听器接口。

```rust
pub trait ConfigListener: Send + Sync {
    fn on_config_change(&self, change: ConfigChange);
}
```

## 配置结构

### AppConfig

```json
{
  "database": {
    "host": "localhost",
    "port": 5432,
    "max_connections": 100,
    "timeout_seconds": 30
  },
  "log": {
    "level": "info",
    "rotation_size_mb": 100,
    "retention_days": 7,
    "format": "json"
  },
  "cache": {
    "enabled": true,
    "max_size_mb": 512,
    "ttl_seconds": 3600
  },
  "version": "2.1.0"
}
```

## 使用方法

### Rust API

```rust
use sqlrustgo_tools::config_hot_reload::{ConfigManager, ConfigWatcher, ConfigChange};

// 创建配置管理器
let manager = ConfigManager::new(PathBuf::from("config.json"))?;

// 获取当前配置
let config = manager.get_config();

// 添加变更监听器
manager.add_listener(Box::new(|change: ConfigChange| {
    println!("Config changed: {:?}", change);
}));

// 手动重载配置
manager.reload()?;

// 动态更新配置
manager.update_database_config(DatabaseConfig {
    host: "localhost".to_string(),
    port: 5432,
    max_connections: 200,
    timeout_seconds: 60,
})?;
```

### 启动配置监视

```rust
let watcher = ConfigWatcher::new(config_path, 5)?; // 5秒轮询
let handle = watcher.start_watching();

// 保持运行...
```

## 配置变更类型

```rust
pub enum ConfigChange {
    Database(DatabaseConfig),
    Log(LogConfig),
    Cache(CacheConfig),
    Full(AppConfig),
}
```

## 测试

运行配置热更新测试：

```bash
cargo test -p sqlrustgo-tools config
```

测试结果：✅ 全部通过

| 测试 | 描述 |
|------|------|
| `test_default_config` | 默认配置创建 |
| `test_database_config_default` | 数据库配置默认值 |
| `test_cache_config_default` | 缓存配置默认值 |

## 相关文档

- [物理备份指南](./physical-backup-guide.md)
- [mysqldump导入指南](./mysqldump-import-guide.md)
