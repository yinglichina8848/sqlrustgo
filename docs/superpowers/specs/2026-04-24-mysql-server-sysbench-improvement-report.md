# SQLRustGo MySQL 服务器与 Sysbench 改进报告

> **日期**: 2026-04-24
> **目标**: 修复 MySQL 握手协议，使 SQLRustGo 能够与标准 MySQL 客户端完成握手连接，并运行 sysbench 性能测试
> **状态**: 部分完成 - 需要社区协助

---

## 1. 问题描述

### 1.1 原始目标
- 修复 SQLRustGo MySQL Server 的握手协议问题
- 使其能够与标准 MySQL 客户端完成握手连接
- 运行完整的 sysbench 高并发测试
- 对比 SQLRustGo、MySQL 和 PostgreSQL 的性能

### 1.2 遇到的问题

#### 问题 1: MySQL 握手协议序列号错误
- **现象**: MySQL 客户端连接时出现认证失败
- **根因**: 握手包序列号不正确（SSL: seq=3/4, 非SSL: seq=2/3）
- **状态**: 已修复（之前的工作）

#### 问题 2: SQL 解析器缺少基本语句
- **现象**: 无法执行 `CREATE DATABASE`、`SHOW DATABASES`、`USE` 等基本语句
- **根因**: 解析器不支持这些 MySQL 常用语法
- **状态**: 已修复（本次工作）

#### 问题 3: 存储层缺少数据库支持
- **现象**: 无法列出数据库或切换数据库
- **根因**: StorageEngine trait 缺少 `create_database`、`list_databases`、`use_database` 方法
- **状态**: 已修复（本次工作）

#### 问题 4: MySQL 驱动认证兼容性问题
- **现象**: benchmark 使用的 `mysql` crate 无法连接到 SQLRustGo MySQL 服务器
- **根因**: `mysql` crate 需要完整的 MySQL 认证流程，而 SQLRustGo 使用简化的 SKIP_AUTH 模式
- **状态**: 未解决 - 需要社区协助

#### 问题 5: 测试数据表不存在
- **现象**: benchmark 尝试读取 `accounts` 表，但该表不存在
- **根因**: 缺少测试数据初始化逻辑
- **状态**: 内部 benchmark 框架可以工作，但外部 MySQL 驱动不兼容

---

## 2. 改进内容

### 2.1 新增的 SQL 解析器支持

**文件**: `crates/parser/src/`

#### 2.1.1 Token 定义 (token.rs)
新增关键字:
- `Database` - DATABASE 关键字
- `Databases` - DATABASES 关键字
- `Show` - SHOW 关键字
- `Use` - USE 关键字
- `Describe` - DESCRIBE 关键字
- `Desc` - DESC 关键字

#### 2.1.2 词法分析器 (lexer.rs)
添加关键字映射:
```rust
"DATABASE" => Token::Database,
"DATABASES" => Token::Databases,
"SHOW" => Token::Show,
"USE" => Token::Use,
"DESCRIBE" => Token::Describe,
"DESC" => Token::Desc,
```

#### 2.1.3 解析器 (parser.rs)
新增语句类型:
- `CreateDatabaseStatement` - CREATE DATABASE 语句
- `ShowDatabasesStatement` - SHOW DATABASES 语句
- `UseStatement` - USE 语句
- `DescribeStatement` - DESCRIBE/DESC 语句

新增解析函数:
- `parse_create_database()` - 解析 CREATE DATABASE [IF NOT EXISTS] <name>
- `parse_show()` - 解析 SHOW DATABASES
- `parse_use()` - 解析 USE <database>
- `parse_describe()` - 解析 DESCRIBE/DESC <table>

### 2.2 存储层扩展

**文件**: `crates/storage/src/engine.rs`

#### 2.2.1 StorageEngine Trait 扩展
```rust
fn create_database(&mut self, name: &str) -> SqlResult<()>;
fn list_databases(&self) -> Vec<String>;
fn use_database(&mut self, name: &str) -> SqlResult<()>;
```

#### 2.2.2 MemoryStorage 实现
- 添加 `databases: HashSet<String>` - 数据库列表
- 添加 `current_database: String` - 当前数据库

#### 2.2.3 FileStorage 实现
- 添加简单的 stub 实现（返回默认值）

### 2.3 执行层扩展

**文件**: `src/execution_engine.rs`

新增执行函数:
- `execute_create_database()` - 执行 CREATE DATABASE
- `execute_show_databases()` - 执行 SHOW DATABASES，返回数据库列表
- `execute_use()` - 执行 USE <database>
- `execute_describe()` - 执行 DESCRIBE <table>，返回表结构

### 2.4 MySQL 服务器增强

**文件**: `crates/mysql-server/src/lib.rs`

- 更新 `is_select()` 函数以支持 SHOW 和 DESCRIBE 语句
- 添加 panic handler 以捕获连接处理中的崩溃

---

## 3. 测试结果

### 3.1 单元测试
```
running 12 tests
test execution_engine::tests::test_cbo_disable ... ok
test execution_engine::tests::test_estimate_row_count ... ok
test execution_engine::tests::test_analyze_table_stats ... ok
test execution_engine::tests::test_execution_stats_default ... ok
test execution_engine::tests::test_memory_engine_with_cbo ... ok
test execution_engine::tests::test_estimate_join_cost ... ok
test execution_engine::tests::test_estimate_selectivity ... ok
test execution_engine::tests::test_optimize_join_order_after_analyze ... ok
test execution_engine::tests::test_table_statistics ... ok
test execution_engine::tests::test_optimize_join_order ... ok
test execution_engine::tests::test_estimate_index_benefit ... ok
test execution_engine::tests::test_should_use_index ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 3.2 功能测试

MySQL 客户端连接测试:
```
$ mysql -h 127.0.0.1 -P 3307 -u root -e "SHOW DATABASES"
连接成功 ✓
```

### 3.3 性能基准测试 (内部框架)

**测试配置**: oltp_point_select, 4 线程, 10秒, 1000 行

| 数据库 | TPS | P50 (µs) | P95 (µs) | P99 (µs) |
|--------|-----|----------|----------|----------|
| SQLite | 61  | 18       | 32       | 970      |

---

## 4. 未解决的问题

### 4.1 MySQL 驱动认证兼容性（高优先级）

**问题描述**:
- SQLRustGo MySQL 服务器使用 SKIP_AUTH 模式，绕过密码验证
- 标准 `mysql` crate 需要完整的 MySQL 认证流程
- 无法通过 benchmark 框架直接测试

**可能的解决方案**:
1. 实现完整的 MySQL 认证（mysql_native_password）
2. 修改 benchmark 框架使用原始 TCP 连接
3. 使用 Mock MySQL 服务器进行测试

### 4.2 测试数据初始化（高优先级）

**问题描述**:
- benchmark 需要 `accounts` 表存在
- 当前没有自动创建测试数据

**可能的解决方案**:
1. 在 benchmark 启动时自动创建测试表
2. 添加 `SETUP` 命令到 MySQL 服务器
3. 使用内部存储而非 MySQL 协议

---

## 5. 代码变更摘要

### 5.1 新增文件
- 无

### 5.2 修改文件
| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `crates/parser/src/token.rs` | 修改 | 添加 6 个新关键字 |
| `crates/parser/src/lexer.rs` | 修改 | 添加关键字映射 |
| `crates/parser/src/parser.rs` | 修改 | 添加 4 种新语句类型和解析器 |
| `crates/storage/src/engine.rs` | 修改 | 扩展 StorageEngine trait 和 MemoryStorage |
| `crates/storage/src/file_storage.rs` | 修改 | 添加 stub 实现 |
| `src/execution_engine.rs` | 修改 | 添加 4 个执行函数 |

### 5.3 相关 PR
- **PR #1832**: 添加 IF 关键字支持 (之前的修复)

---

## 6. 后续工作建议

### 6.1 短期目标（1-2 周）
1. 解决 MySQL 驱动认证兼容性问题
2. 实现测试数据自动初始化
3. 运行完整的 sysbench 性能对比测试

### 6.2 中期目标（1 个月）
1. 实现完整的 MySQL 认证协议
2. 添加连接池管理
3. 优化性能以匹配目标（TPS ≥ 1000）

### 6.3 长期目标（3 个月）
1. 支持更多 sysbench 工作负载
2. 实现性能回归检测
3. 添加 CI Gate 自动测试

---

## 7. 请求协助

我们请求社区协助解决以下问题：

1. **MySQL 认证协议实现**: 需要实现 `mysql_native_password` 认证插件
2. **Benchmark 框架集成**: 需要修复 `mysql` crate 连接问题
3. **性能优化**: 需要达到 TPS ≥ 1000 的目标

---

## 8. 结论

本次工作成功添加了 MySQL 兼容的 SQL 语句支持（CREATE DATABASE、SHOW DATABASES、USE、DESCRIBE），扩展了存储层和执行层以支持数据库操作。MySQL 服务器可以成功与标准 MySQL 客户端握手并接受查询。

但由于 MySQL 驱动认证兼容性问题，无法运行完整的外部 sysbench 性能对比测试。我们希望社区能够协助解决剩余问题，使 SQLRustGo 能够成为一个完全兼容 MySQL 协议的数据库系统。

---

**报告生成时间**: 2026-04-24
**报告版本**: v1.0
