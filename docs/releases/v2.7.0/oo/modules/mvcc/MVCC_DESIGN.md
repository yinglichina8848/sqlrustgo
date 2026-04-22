# MVCC 模块设计

**版本**: v2.7.0
**模块**: MVCC (Multi-Version Concurrency Control)

---

## 一、概述

MVCC 提供快照隔离级别，支持并发事务而互不阻塞。

## 二、核心组件

| 组件 | 职责 |
|------|------|
| VersionChain | 版本链管理 |
| Snapshot | 事务快照 |
| ReadView | 读视图 |

## 三、相关文档

- [ARCHITECTURE_V2.7.md](../../architecture/ARCHITECTURE_V2.7.md)

---

*MVCC 模块设计 v2.7.0*
