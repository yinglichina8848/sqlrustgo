# Unified Query 模块设计

**版本**: v2.6.0
**模块**: Unified Query (统一查询)

---

## 一、What (是什么)

Unified Query 是 SQLRustGo 的统一查询引擎，支持 SQL + 向量 + 图融合查询。

## 二、Why (为什么)

- **融合查询**: 一次查询获取多模态结果
- **简化开发**: 统一的查询接口
- **性能优化**: 跨模态优化

## 三、核心功能

| 功能 | 支持度 |
|------|--------|
| SQL 查询 | ✅ |
| 向量搜索 | ✅ |
| 图遍历 | ✅ |
| 融合查询 | ✅ |
| RRF 融合 | ✅ |

## 四、使用示例

```sql
SELECT name, score
FROM (
    SELECT name, 0.5 AS score FROM users WHERE age > 25
    UNION ALL
    SELECT name, similarity AS score FROM vector_search('embedding', '[0.1, 0.2]')
)
ORDER BY score DESC
LIMIT 10;
```

## 五、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)

---

*Unified Query 模块设计 v2.6.0*
