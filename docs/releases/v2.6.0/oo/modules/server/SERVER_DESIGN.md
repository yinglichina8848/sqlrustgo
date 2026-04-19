# Server 模块设计

**版本**: v2.6.0
**模块**: Server (服务器)

---

## 一、What (是什么)

Server 是 SQLRustGo 的服务器模块，负责处理客户端连接、请求路由、连接池管理和 HTTP API 服务。

## 二、HTTP API

| 端点 | 方法 | 说明 |
|------|------|------|
| /health | GET | 健康检查 |
| /ready | GET | 就绪检查 |
| /sql | POST | SQL 查询 |
| /vector | POST | 向量搜索 |
| /metrics | GET | Prometheus 指标 |

## 三、相关文档

- [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md)
- [DEPLOYMENT_GUIDE.md](../../DEPLOYMENT_GUIDE.md)

---

*Server 模块设计 v2.6.0*
