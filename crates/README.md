# 工作区成员

该目录包含 SQLRustGo v1.2.0+ 的模块化 crate 结构。

## 目前状态

crates/ 目录正在从整体 src/ 结构迁移。

### 迁移进度

- [x] 第 1 阶段：创建工作区结构
- [ ] 第 2 阶段：迁移类型模块
- [ ] 第 3 阶段：迁移解析器模块
- [ ] 第 4 阶段：迁移其他模块

### 板条箱依赖项

```
sqlrustgo-server (depends on all)
├── sqlrustgo-planner
│   ├── sqlrustgo-parser
│   └── sqlrustgo-common
├── sqlrustgo-optimizer
│   └── sqlrustgo-types
├── sqlrustgo-executor
│   └── sqlrustgo-storage
├── sqlrustgo-storage
├── sqlrustgo-catalog
└── sqlrustgo-transaction
```

### 迁移命令

```bash
# Build workspace
cargo build --workspace

# Build single crate
cargo build -p sqlrustgo-types
```
