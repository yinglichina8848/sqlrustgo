# Transaction 模块设计

**版本**: v2.6.0
**模块**: Transaction (事务管理)

---

## 一、What (是什么)

Transaction 是 SQLRustGo 的事务管理模块，负责事务的开启、提交、回滚。

## 二、隔离级别

| 隔离级别 | 支持 | 说明 |
|----------|------|------|
| READ COMMITTED | ✅ | 已提交读 |
| REPEATABLE READ | ✅ | 可重复读 |
| SNAPSHOT | ✅ | 快照隔离 |
| SERIALIZABLE | ✅ | 可串行化 (SSI) |

## 三、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)
- [MVCC_DESIGN.md](../mvcc/MVCC_DESIGN.md)

---

*Transaction 模块设计 v2.6.0*
