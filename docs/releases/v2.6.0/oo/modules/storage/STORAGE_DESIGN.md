# Storage 模块设计

**版本**: v2.6.0
**模块**: Storage (存储引擎)

---

## 一、What (是什么)

Storage 是 SQLRustGo 的存储引擎核心，负责数据的持久化存储、索引管理、缓冲池管理。

## 二、核心组件

| 组件 | 说明 |
|------|------|
| BufferPool | 缓冲池管理 |
| PageManager | 页面调度 |
| IndexManager | 索引管理 |
| BPlusTree | B+树索引 |

## 三、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)

---

*Storage 模块设计 v2.6.0*
