# Server 模块设计

**版本**: v2.5.0
**模块**: Server (服务器)

---

## 一、What (是什么)

Server 是 SQLRustGo 的服务器模块，负责处理客户端连接、请求路由、连接池管理和 HTTP API 服务。

## 二、Why (为什么)

- **连接管理**: 客户端连接和认证
- **请求处理**: SQL 查询和 API 请求
- **连接池**: 高效复用连接资源
- **可观测性**: 健康检查、监控指标

## 三、核心数据结构

```rust
pub struct Server {
    config: ServerConfig,
    connection_pool: ConnectionPool,
    router: RequestRouter,
    http_server: HttpServer,
}

pub struct ConnectionPool {
    connections: Vec<Connection>,
    max_connections: usize,
    idle_timeout: Duration,
}
```

## 四、HTTP API

| 端点 | 方法 | 说明 |
|------|------|------|
| /health | GET | 健康检查 |
| /ready | GET | 就绪检查 |
| /sql | POST | SQL 查询 |
| /vector | POST | 向量搜索 |
| /metrics | GET | Prometheus 指标 |

## 五、相关文档

- [ARCHITECTURE_V2.5.md](../../architecture/ARCHITECTURE_V2.5.md)
- [DEPLOYMENT_GUIDE.md](../../../DEPLOYMENT_GUIDE.md)

---

*Server 模块设计 v2.5.0*
