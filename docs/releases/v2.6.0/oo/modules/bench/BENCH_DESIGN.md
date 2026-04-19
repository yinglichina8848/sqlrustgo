# Bench 模块设计

**版本**: v2.6.0
**模块**: Bench (基准测试)

---

## 一、What (是什么)

Bench 是 SQLRustGo 的基准测试模块，支持 TPC-H、OLTP、向量搜索等多种基准测试。

## 二、支持的基准

| 基准 | 说明 | 状态 |
|------|------|------|
| TPC-H | 决策支持系统 | ✅ SF=0.1~100 |
| OLTP | 事务处理 | ✅ |
| Vector Search | 向量搜索 | ✅ |
| MySQL 兼容 | MySQL 5.7 兼容 | ✅ |

## 三、相关文档

- [TEST_PLAN.md](../../../TEST_PLAN.md)
- [PERFORMANCE_ANALYSIS.md](../../reports/PERFORMANCE_ANALYSIS.md)

---

*Bench 模块设计 v2.6.0*
