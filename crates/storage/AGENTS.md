# crates/storage - Storage Engine

> 存储引擎核心模块。Buffer Pool、File Storage、WAL、B+ Tree 实现。

## 核心文件

| 文件 | 作用 |
|------|------|
| `src/engine.rs` | Storage engine facade |
| `src/buffer_pool.rs` | LRU buffer pool |
| `src/file_storage.rs` | 文件存储 (47k 行) |
| `src/wal.rs` | Write-Ahead Log (58k 行) |
| `src/bplus_tree/index.rs` | B+ Tree 索引 |

## 关键子模块

```
src/
├── bplus_tree/     # B+ Tree 实现
├── columnar/       # 列式存储
├── parquet/        # Parquet 支持
└── ...
```

## 常用命令

```bash
# 单 crate 测试
cargo test -p sqlrustgo-storage

# 单 crate 构建
cargo build -p sqlrustgo-storage --all-features

# 运行 storage 特定测试
cargo test buffer_pool --all-features
```

## 约定

- `PageId`, `FrameId`, `SlotId` 类型区分清晰
- `BufferPool` 提供 `fetch_page()` / `unpin_page()` 接口
- WAL 必须先写入再提交事务
