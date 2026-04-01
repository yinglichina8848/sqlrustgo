# 连接池

## 概述

SQLRustGo 实现 executor 连接池，复用执行器实例以减少创建开销，提升并发性能。

## 功能特性

- ✅ **连接池管理** - 预创建并复用会话
- ✅ **Executor 复用** - 减少 executor 创建开销
- ✅ **空闲连接回收** - 自动回收空闲连接
- ✅ **最大连接数限制** - 可配置的池大小
- ✅ **并发安全** - 线程安全的获取/释放

## 概念说明

### PoolConfig

连接池配置。

```rust
pub struct PoolConfig {
    pub size: usize,        // 连接池大小
    pub timeout_ms: u64,   // 获取连接超时(ms)
}
```

### ConnectionPool

连接池管理器。

```rust
let config = PoolConfig {
    size: 10,
    timeout_ms: 5000,
};
let pool = ConnectionPool::new(config);
```

### PooledConnection

池化连接，通过 `acquire()` 获取，`drop` 时自动归还池中。

```rust
let conn = pool.acquire();  // 获取连接
// 使用 conn.executor() 执行查询
drop(conn);  // 自动释放回连接池
```

### PooledSession

池化会话，包含 executor 和 storage。

```rust
pub struct PooledSession {
    pub executor: LocalExecutor<'static>,
    pub storage: Arc<MemoryStorage>,
    pub transaction_id: Option<u64>,
}
```

## 使用方法

### 创建连接池

```rust
use sqlrustgo_server::connection_pool::{ConnectionPool, PoolConfig};

let config = PoolConfig {
    size: 10,
    timeout_ms: 5000,
};
let pool = ConnectionPool::new(config);
```

### 获取连接

```rust
let conn = pool.acquire();  // 阻塞直到有可用连接
let executor = conn.executor();

// 使用 executor 执行查询
let result = executor.execute(&plan)?;
```

### 自动释放

```rust
{
    let conn = pool.acquire();
    let result = conn.executor().execute(&plan)?;
} // conn 在作用域结束时自动释放回池中
```

### 非阻塞获取

```rust
if let Some(conn) = pool.try_acquire() {
    let result = conn.executor().execute(&plan)?;
} else {
    println!("连接池已满，请稍后重试");
}
```

### 并发使用

```rust
use std::sync::Arc;
use std::thread;

let pool = Arc::new(ConnectionPool::new(config));

let handles: Vec<_> = (0..10).map(|_| {
    let pool = Arc::clone(&pool);
    thread::spawn(move || {
        let conn = pool.acquire();
        let result = conn.executor().execute(&plan)?;
        Ok::<_, String>(result)
    })
}).collect();

for handle in handles {
    handle.join().unwrap()?;
}
```

## 配置建议

| 应用场景 | 池大小 | 超时(ms) |
|----------|--------|----------|
| 开发/测试 | 5-10 | 5000 |
| 小规模生产 | 10-20 | 3000 |
| 大规模生产 | 50-100 | 1000 |

## 性能收益

- **减少开销** - 复用 executor 避免重复创建
- **提升 QPS** - 目标: +20% QPS
- **并发支持** - 支持高并发连接请求

## 测试

运行连接池测试：

```bash
cargo test connection_pool
```

测试结果：✅ 全部通过

| 测试 | 描述 |
|------|------|
| `test_connection_pool_basic_acquire_release` | 基本获取/释放 |
| `test_connection_pool_concurrent_50_connections` | 50并发连接 |
| `test_connection_pool_concurrent_100_connections` | 100并发连接 |
| `test_connection_pool_basic` | 基础功能测试 |
| `test_connection_pool_exhaustion` | 池耗尽测试 |
| `test_connection_pool_min_size` | 最小连接数测试 |
| `test_connection_pool_max_size` | 最大连接数测试 |

## 相关文档

- [性能测试报告](./performance-test-report.md)
