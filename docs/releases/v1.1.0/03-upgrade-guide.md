# SQLRustGo v1.1.0 升级指南

## 概述

本指南帮助用户从 v1.0.0 升级到 v1.1.0。v1.1.0 包含多项新功能和性能改进，建议所有用户升级。

## 主要变更

### 1. 新增 Planner 模块

v1.1.0 引入了完整的查询规划器架构，包括：

- `LogicalPlan`: 逻辑计划表示
- `PhysicalPlan`: 物理执行计划
- `Analyzer`: 查询分析和优化

**兼容性说明**: 此变更对 API 用户透明，现有查询语句无需修改。

### 2. 执行引擎抽象

新增 `ExecutionEngine` trait 和 `EngineRegistry`:

```rust
// 新增: 引擎注册表
let mut registry = EngineRegistry::new();
registry.register("custom", Box::new(CustomEngine::new()));

// 设置默认引擎
registry.set_default("custom")?;
```

**兼容性说明**: 现有代码自动使用默认引擎，无需修改。

### 3. 聚合函数增强

完整支持聚合操作:

```sql
SELECT COUNT(*), SUM(amount), AVG(price), MIN(id), MAX(name) FROM orders;
```

**兼容性说明**: 语法完全兼容，仅增加新功能。

### 4. 网络层改进

- 异步服务器支持
- 连接池管理
- 配置系统增强

**兼容性说明**: 协议兼容现有客户端。

### 5. 认证与授权

基于角色的访问控制 (RBAC):

```rust
// 用户角色
enum Role {
    Admin,    // 完全访问
    User,     // Select/Insert/Update/Delete
    Readonly, // 仅 Select
}

// 权限检查
if user.role.can_execute(&Operation::Insert) {
    // 执行操作
}
```

**兼容性说明**: 新功能，不影响现有未使用认证的部署。

## 迁移步骤

### 1. 更新依赖

确保 Cargo.toml 中的 Rust 版本:

```toml
[package]
rust-version = "1.75"
```

### 2. 更新依赖库

```bash
cargo update
```

### 3. 重新编译

```bash
cargo build --release
```

### 4. 运行测试

```bash
cargo test --all-features
```

## 已知问题

### 1. 测试覆盖率

v1.1.0 提升了测试覆盖率到 90%+, 运行测试时可能会发现一些之前未覆盖的边界情况。

### 2. 性能基准

建议运行性能基准测试以评估新版本对您工作负载的影响:

```bash
cargo bench
```

## 回滚方案

如遇到问题，可回滚到 v1.0.0:

```bash
git checkout v1.0.0
cargo build --release
```

## 获取帮助

- 文档: https://github.com/minzuuniversity/sqlrustgo/docs
- Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- 讨论: https://github.com/minzuuniversity/sqlrustgo/discussions

## 变更日志

完整变更请参考 [CHANGELOG.md](../../CHANGELOG.md)
