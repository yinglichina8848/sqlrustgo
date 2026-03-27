# SQLRustGo Server 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-server

---

## 1. 模块概述

Server 模块负责数据库服务器的连接管理和网络服务。

### 1.1 模块职责

- 连接池管理
- 会话管理
- HTTP API
- 健康检查
- 指标端点

### 1.2 模块结构

```
crates/server/
├── src/
│   ├── lib.rs                 # 模块入口
│   ├── connection_pool.rs     # 连接池
│   ├── http_server.rs         # HTTP 服务器
│   ├── health.rs              # 健康检查
│   ├── metrics_endpoint.rs   # 指标端点
│   ├── teaching_endpoints.rs # 教学端点
│   └── lib.rs
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 连接池

```uml
@startuml

class ConnectionPool {
  -config: PoolConfig
  -available: Vec<Connection>
  -in_use: HashSet<ConnectionId>
  -wait_queue: Vec<Waiter>
  --
  +acquire(): Result<PooledConnection>
  +release(conn)
  +try_acquire(): Option<PooledConnection>
  +close()
}

class PooledConnection {
  -id: ConnectionId
  -connection: Connection
  -pool: Weak<ConnectionPool>
  -created_at: Timestamp
  -last_used: Timestamp
}

class Waiter {
  -tx: Sender<PooledConnection>
  -timeout: Duration
}

ConnectionPool --> PooledConnection
ConnectionPool --> Waiter

@enduml
```

### 2.2 HTTP 服务器

```uml
@startuml

class HttpServer {
  -address: SocketAddr
  -router: Router
  -state: ServerState
  --
  +start(): Result<()>
  +stop(): Result<()>
}

class Router {
  -routes: HashMap<MethodPath, Handler>
  --
  +register(method, path, handler)
  +route(request): Response
}

class Handler {
  -callback: fn(Request) -> Response
  -middleware: Vec<Middleware>
}

HttpServer --> Router
Router --> Handler

@enduml
```

---

## 3. 与代码对应检查

### 3.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 连接池 | `connection_pool.rs` | ✅ 对应 |
| HTTP 服务器 | `http_server.rs` | ✅ 对应 |
| 健康检查 | `health.rs` | ✅ 对应 |
| 指标端点 | `metrics_endpoint.rs` | ✅ 对应 |
| 教学端点 | `teaching_endpoints.rs` | ✅ 对应 |

---

## 4. 测试设计

### 4.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_pool() {
        let pool = ConnectionPool::new(PoolConfig {
            size: 5,
            timeout_ms: 1000,
        });
        
        let conn = pool.acquire();
        assert!(conn.is_ok());
    }
    
    #[test]
    fn test_pool_exhaustion() {
        let pool = ConnectionPool::new(PoolConfig {
            size: 2,
            timeout_ms: 100,
        });
        
        let _c1 = pool.acquire().unwrap();
        let _c2 = pool.acquire().unwrap();
        
        let result = pool.try_acquire();
        assert!(result.is_none());
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
