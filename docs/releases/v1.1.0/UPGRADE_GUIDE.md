# SQLRustGo v1.0.0 → v1.1.0 升级迁移指南

> 版本：v1.1.0
> 发布日期：2026-03-03
> 目标用户：v1.0.0 用户升级到 v1.1.0

---

## 一、升级概述

### 1.1 版本对比

| 项目 | v1.0.0 | v1.1.0 |
|------|--------|--------|
| 架构模式 | 单体嵌入式 | Client-Server + 嵌入式 |
| 执行器 | 单一实现 | 插件化 ExecutionEngine |
| 查询计划 | 无分离 | LogicalPlan/PhysicalPlan 分离 |
| 网络协议 | 基础实现 | 异步 MySQL 协议 |
| Join 算法 | Nested Loop | HashJoin + Nested Loop |
| 测试覆盖率 | 84% | 94.18% |

### 1.2 升级收益

- ✅ **性能提升**: HashJoin 算法显著提升 JOIN 查询性能
- ✅ **可扩展性**: 插件化执行器支持自定义实现
- ✅ **网络支持**: 完整的 Client-Server 架构
- ✅ **代码质量**: 测试覆盖率提升至 94%+

---

## 二、升级前准备

### 2.1 系统要求

| 要求 | v1.0.0 | v1.1.0 |
|------|--------|--------|
| Rust 版本 | ≥1.75 | ≥1.85 |
| 操作系统 | Linux/macOS/Windows | Linux/macOS/Windows |
| 依赖项 | 基础 | + Tokio 异步运行时 |

### 2.2 备份数据

```bash
# 备份数据文件
cp -r data/ data_backup_v1.0.0/

# 备份配置文件（如有）
cp config.toml config.toml.bak
```

### 2.3 检查依赖

```bash
# 更新 Rust 版本
rustup update stable

# 检查项目依赖
cargo outdated
```

---

## 三、升级步骤

### 3.1 更新代码

```bash
# 拉取最新代码
git fetch origin
git checkout v1.1.0

# 或切换到 release 分支
git checkout release/v1.1.0
```

### 3.2 更新依赖

```bash
# 更新 Cargo.lock
cargo update

# 构建项目
cargo build --release
```

### 3.3 运行测试

```bash
# 运行所有测试
cargo test --all

# 检查覆盖率
cargo llvm-cov --all-features
```

---

## 四、API 变更

### 4.1 执行器接口变更

#### v1.0.0

```rust
// 旧 API
use sqlrustgo::executor::Executor;

let mut executor = Executor::new();
let result = executor.execute(sql)?;
```

#### v1.1.0

```rust
// 新 API
use sqlrustgo::executor::ExecutionEngine;
use sqlrustgo::planner::PhysicalPlan;

let mut engine = ExecutionEngine::new();
let result = engine.execute(plan)?;
```

### 4.2 查询计划接口

#### 新增 LogicalPlan

```rust
use sqlrustgo::planner::LogicalPlan;
use sqlrustgo::parser::parse;

// 解析 SQL 为逻辑计划
let logical_plan = parse("SELECT * FROM users")?;

// 转换为物理计划
let physical_plan = logical_plan.to_physical()?;
```

### 4.3 网络服务接口

#### 新增 Server 模式

```rust
use sqlrustgo::network::Server;

// 启动服务器
let server = Server::bind("0.0.0.0:3306")?;
server.run().await?;
```

#### 新增 Client 连接

```rust
use sqlrustgo::network::Client;

// 连接服务器
let mut client = Client::connect("127.0.0.1:3306").await?;

// 执行查询
let result = client.query("SELECT * FROM users").await?;
```

---

## 五、配置变更

### 5.1 新增配置项

```toml
# config.toml

[server]
# 服务器绑定地址
bind = "0.0.0.0:3306"

# 最大连接数
max_connections = 100

# 连接超时（秒）
timeout = 30

[executor]
# 默认执行器
default_engine = "hash"

# 是否启用查询优化
enable_optimizer = true
```

### 5.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_BIND` | 服务器绑定地址 | `0.0.0.0:3306` |
| `SQLRUSTGO_MAX_CONN` | 最大连接数 | `100` |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 | `info` |

---

## 六、数据迁移

### 6.1 数据兼容性

v1.1.0 **完全兼容** v1.0.0 数据格式，无需迁移数据。

| 数据类型 | 兼容性 |
|----------|--------|
| 表数据文件 | ✅ 兼容 |
| 索引文件 | ✅ 兼容 |
| WAL 日志 | ✅ 兼容 |

### 6.2 数据验证

```bash
# 启动服务后验证数据
cargo run --release

# 执行验证查询
SELECT COUNT(*) FROM users;
```

---

## 七、性能优化建议

### 7.1 HashJoin 使用

```sql
-- v1.1.0 自动选择 HashJoin 处理大表 JOIN
SELECT * FROM orders o JOIN users u ON o.user_id = u.id;
```

### 7.2 连接池配置

```rust
// 推荐连接池配置
let pool = ConnectionPool::new()
    .max_connections(50)
    .min_connections(5)
    .idle_timeout(Duration::from_secs(300));
```

### 7.3 异步查询

```rust
// 使用异步 API 提升吞吐量
async fn query_handler() {
    let results = futures::future::join_all(queries.map(|q| client.query(q))).await;
}
```

---

## 八、回滚方案

### 8.1 回滚步骤

```bash
# 1. 停止服务
pkill sqlrustgo

# 2. 切换回 v1.0.0
git checkout v1.0.0

# 3. 恢复数据（如有变更）
cp -r data_backup_v1.0.0/ data/

# 4. 重新构建
cargo build --release
```

### 8.2 兼容性保证

- v1.1.0 数据文件可被 v1.0.0 读取
- 新增功能（如 HashJoin）不影响基础查询
- 网络协议向后兼容

---

## 九、常见问题

### Q1: 升级后编译失败？

**A**: 确保 Rust 版本 ≥ 1.85

```bash
rustup update stable
rustc --version
```

### Q2: 旧代码 API 不兼容？

**A**: 参考 [API 变更](#四api-变更) 章节更新代码

### Q3: 性能没有提升？

**A**: 检查是否正确配置：
- 确保 HashJoin 被启用
- 检查连接池配置
- 验证查询是否使用 JOIN

### Q4: 网络连接失败？

**A**: 检查防火墙和端口配置：
```bash
# 检查端口
netstat -tlnp | grep 3306

# 检查防火墙
sudo ufw status
```

---

## 十、获取帮助

### 10.1 文档资源

- [Release Notes](./RELEASE_NOTES.md)
- [CHANGELOG](../../CHANGELOG.md)
- [API 文档](https://docs.rs/sqlrustgo)

### 10.2 社区支持

- GitHub Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- GitHub Discussions: https://github.com/minzuuniversity/sqlrustgo/discussions

---

*本文档由 TRAE (GLM-5.0) 创建*
*最后更新: 2026-03-03*
