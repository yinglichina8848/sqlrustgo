# SQLRustGo

Rust 实现的关系型数据库系统，支持 SQL-92 子集。

## 项目目标

- 从零开始构建 Rust 数据库系统
- 现代化架构设计
- AI 工具链深度集成

## 项目结构

```
sqlrustgo/
├── Cargo.toml
├── README.md
├── .gitignore
├── .github/workflows/
├── docs/
│   ├── 2026-02-13-sqlcc-rust-redesign-design.md
│   └── 2026-02-13-sqlcc-rust-impl-plan.md
└── src/
```

## 功能特性

- ✅ SQL-92 子集支持 (SELECT, INSERT, UPDATE, DELETE)
- ✅ 存储引擎 (Buffer Pool, FileStorage)
- ✅ B+ Tree 索引
- ✅ 事务支持 (WAL)
- ✅ 网络协议支持

## 文档

- [设计文档](docs/2026-02-13-sqlcc-rust-redesign-design.md)
- [实施计划](docs/2026-02-13-sqlcc-rust-impl-plan.md)

## 构建

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example demo
```

## 参与贡献

欢迎提交 PR！请参考 GitHub 上的 Issue 和 PR 列表。
