# v1.1.0-Beta 阶段执行手册（学生版）

## 概述

本手册面向学生，指导如何在本地环境搭建、运行和验证 SQLRustGo 项目。

**版本**: v1.1.0-beta
**目标读者**: 学生 / 教学演示参与者

---

## 1. 环境准备

### 1.1 系统要求

| 项目 | 要求 |
|------|------|
| 操作系统 | macOS / Linux / Windows (WSL2) |
| Rust | 1.75+ |
| 内存 | 4GB+ |
| 磁盘 | 1GB+ |

### 1.2 安装 Rust

```bash
# 安装 Rust (如果没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 1.3 克隆项目

```bash
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo
```

---

## 2. 构建与测试

### 2.1 编译项目

```bash
cargo build --all-features
```

**预期输出**: 编译成功，无错误

### 2.2 运行测试

```bash
cargo test --all-features
```

**预期输出**: 所有测试通过

### 2.3 代码质量检查

```bash
# Clippy 检查
cargo clippy --all-features -- -D warnings

# 格式化检查
cargo fmt --check
```

---

## 3. 运行 REPL

### 3.1 启动交互式环境

```bash
cargo run --bin sqlrustgo
```

**预期输出**:
```
SQLRustGo v1.1.0-beta
Type "help" for more information.
sql>
```

### 3.2 常用命令

| 命令 | 说明 |
|------|------|
| `help` | 显示帮助 |
| `exit` / `quit` | 退出 |

---

## 4. 基本 SQL 操作

### 4.1 创建表

```sql
CREATE TABLE users (id INT, name TEXT, age INT);
```

### 4.2 插入数据

```sql
INSERT INTO users VALUES (1, 'Alice', 25);
INSERT INTO users VALUES (2, 'Bob', 30);
INSERT INTO users VALUES (3, 'Charlie', 28);
```

### 4.3 查询数据

```sql
SELECT * FROM users;
SELECT name, age FROM users WHERE age > 25;
```

### 4.4 聚合函数

```sql
SELECT COUNT(*) FROM users;
SELECT COUNT(id) FROM users;
SELECT AVG(age) FROM users;
SELECT MIN(age) FROM users;
SELECT MAX(age) FROM users;
SELECT SUM(age) FROM users;
```

### 4.5 更新数据

```sql
UPDATE users SET age = 26 WHERE id = 1;
```

### 4.6 删除数据

```sql
DELETE FROM users WHERE id = 3;
```

### 4.7 删除表

```sql
DROP TABLE users;
```

---

## 5. 事务操作

### 5.1 开启事务

```sql
BEGIN;
```

### 5.2 提交事务

```sql
COMMIT;
```

### 5.3 回滚事务

```sql
ROLLBACK;
```

---

## 6. 验证任务完成

### 6.1 门禁检查清单

完成开发任务后，必须通过以下检查：

| 检查项 | 命令 | 预期结果 |
|--------|------|----------|
| 编译 | `cargo build --all-features` | 成功 |
| 测试 | `cargo test --all-features` | 全部通过 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --check` | 通过 |

### 6.2 覆盖率检查（可选）

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features
```

**目标**: ≥ 80%

---

## 7. 常见问题

### 7.1 编译错误

**问题**: `error: failed to run custom build command`

**解决**: 确保 Rust 工具链已正确安装
```bash
rustup update
rustup default stable
```

### 7.2 测试失败

**问题**: 部分测试失败

**解决**:
1. 检查是否有未提交的更改
2. 确保在正确的分支上
3. 运行 `cargo clean` 后重新编译

### 7.3 Clippy 警告

**问题**: Clippy 检查失败

**解决**: 修复代码中的 lint 警告，或使用 `#[allow(clippy::xxx)]` 标注（仅在必要时）

---

## 8. 附录

### 8.1 项目结构

```
sqlrustgo/
├── src/
│   ├── executor/    # 查询执行引擎
│   ├── parser/      # SQL 解析器
│   ├── lexer/       # 词法分析器
│   ├── storage/     # 存储引擎
│   ├── transaction/ # 事务管理
│   ├── network/    # 网络协议
│   └── types/      # 类型系统
├── data/           # 测试数据
├── docs/           # 文档
└── tests/         # 集成测试
```

### 8.2 相关文档

- [README.md](../README.md) - 项目介绍
- [architecture.md](../architecture.md) - 架构设计
- [v1.1.0-beta/README.md](./README.md) - Beta 版本索引

---

*手册版本: v1.0*
*最后更新: 2026-02-20*
