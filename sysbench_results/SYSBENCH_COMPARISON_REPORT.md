# Sysbench OLTP 性能对比报告

**测试日期**: 2026-04-23
**测试环境**: Ubuntu 24.04, 8GB RAM

## 1. 测试环境

| 配置项 | 值 |
|--------|---|
| 操作系统 | Ubuntu 24.04 LTS |
| CPU | 8 核心 |
| 内存 | 8 GB |
| MySQL 版本 | 8.0.45 |
| PostgreSQL 版本 | 16.13 |
| sysbench | 1.0.20 |
| SQLRustGo | v2.6.0 (编译版本) |

## 2. 测试结果汇总

### 2.1 Point Select (主键点查) - 32 线程, 30秒

| 数据库 | QPS/TPS | 平均延迟(ms) | p95延迟(ms) |
|--------|---------|--------------|-------------|
| **PostgreSQL** | **257,384** | 0.12 | 0.17 |
| MySQL | 164,784 | 0.19 | 0.28 |
| SQLRustGo | N/A | N/A | N/A |

**分析**: PostgreSQL 点查性能比 MySQL 高约 56%

### 2.2 Read Write (读写混合) - 32 线程, 30秒

| 数据库 | TPS | QPS | 平均延迟(ms) | p95延迟(ms) |
|--------|-----|-----|--------------|-------------|
| **MySQL** | **1,003** | 20,138 | 31.62 | 74.46 |
| PostgreSQL | 260 | 5,606 | 123.00 | 995.51 |
| SQLRustGo | N/A | N/A | N/A | N/A |

**分析**: MySQL 读写混合性能约为 PostgreSQL 的 4 倍

### 2.3 Insert (插入) - 16 线程, 20秒

| 数据库 | TPS | 平均延迟(ms) | p95延迟(ms) |
|--------|-----|--------------|-------------|
| **PostgreSQL** | **6,210** | 2.57 | 2.71 |
| MySQL | 2,262 | 7.07 | 12.52 |
| SQLRustGo | N/A | N/A | N/A |

**分析**: PostgreSQL 插入性能约为 MySQL 的 2.7 倍

## 3. SQLRustGo MySQL Server 修复状态

### 修复尝试 (2026-04-23)

已尝试修复 MySQL Wire Protocol 兼容性：

1. **更新认证协议**: 从 `mysql_old_password` 改为 `caching_sha2_password`
2. **修复握手包格式**: 符合 MySQL 8.0 握手协议
3. **简化认证逻辑**: 开发模式下跳过密码验证
4. **改进错误处理**: 更友好的连接关闭处理

### 当前问题

- 认证握手成功（客户端发送握手响应，服务器返回 OK）
- 客户端在收到 OK 包后立即断开连接
- 错误: `Lost connection to MySQL server during query`
- 原因分析中：可能是客户端期望额外的认证交换数据

### 相关 Issue

- #1785: 触发器存储实现 + OLTP 服务器集成

### 待解决

- 完成 MySQL 协议认证流程
- 支持 caching_sha2_password 完整认证

## 4. 结论

| 场景 | 推荐数据库 |
|------|------------|
| 纯点查 (OLTP) | PostgreSQL |
| 读写混合 (OLTP) | MySQL |
| 批量写入 | PostgreSQL |

PostgreSQL 在纯读和纯写场景下性能更优，MySQL 在读写混合场景下更稳定。

---

*本报告由 Sysbench 自动化测试生成*
