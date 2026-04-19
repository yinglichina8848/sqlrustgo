# Optimizer 模块设计

**版本**: v2.6.0
**模块**: Optimizer (查询优化器)

---

## 一、What (是什么)

Optimizer 是 SQLRustGo 的基于成本的查询优化器，将逻辑计划转换为最优的物理执行计划。

## 二、优化策略

| 优化类型 | 说明 |
|----------|------|
| Predicate Pushdown | 谓词下推 |
| Projection Pushdown | 投影下推 |
| Join Reordering | 连接重排 |
| Index Selection | 索引选择 |

## 三、相关文档

- [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md)

---

*Optimizer 模块设计 v2.6.0*
