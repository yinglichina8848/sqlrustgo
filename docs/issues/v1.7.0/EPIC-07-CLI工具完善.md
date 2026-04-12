# [Epic-07] CLI 工具完善

## 概述

完善 CLI 工具，支持 MySQL 风格的交互命令和协议。

**优先级**: P0
**来源**: 原 v1.8

---

## Issues

### CLI-01: .tables 命令

**优先级**: P0

**描述**: 实现 `.tables` 命令显示所有表

**Acceptance Criteria**:
- [ ] `.tables` 输出当前数据库所有表名
- [ ] 输出格式清晰

---

### CLI-02: .schema table_name

**优先级**: P0

**描述**: 实现 `.schema` 命令显示表结构

**Acceptance Criteria**:
- [ ] `.schema orders` 输出表的 DDL
- [ ] 显示列名、类型、约束

---

### CLI-03: .indexes table_name

**优先级**: P1

**描述**: 实现 `.indexes` 命令显示索引信息

**Acceptance Criteria**:
- [ ] `.indexes orders` 输出表的所有索引
- [ ] 显示索引名、列、唯一性

---

### CLI-04: MySQL 协议支持（可选）

**优先级**: P2

**描述**: 支持 MySQL 协议，让 DBeaver 等工具可连接

**Acceptance Criteria**:
- [ ] 监听 MySQL 默认端口 3306
- [ ] 处理 MySQL 认证协议
- [ ] DBeaver 可成功连接

---

## 实现步骤

1. **CLI 交互命令**
   - 解析 `.` 开头的命令
   - 实现 help 命令

2. **MySQL 协议**
   - 实现 MySQL handshake
   - 实现简单查询执行

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/bench-cli/src/cli.rs` | CLI 主程序 |
| `crates/server/src/mysql.rs` | MySQL 协议处理 |

---

**关联 Issue**: CLI-01, CLI-02, CLI-03, CLI-04
