# Graph 模块设计

**版本**: v2.6.0
**模块**: Graph (图引擎)

---

## 一、What (是什么)

Graph 是 SQLRustGo 的图查询引擎，支持属性图存储和 Cypher 查询。

## 二、Why (为什么)

- **关系数据处理**: 社交网络、推荐系统等
- **高效遍历**: 图结构数据的路径查询
- **Cypher 支持**: 友好的图查询语法

## 三、核心功能

| 功能 | 支持度 |
|------|--------|
| MATCH | ✅ |
| CREATE | ✅ |
| Cypher 查询 | ✅ |
| BFS/DFS | ✅ |
| 多跳查询 | ✅ |

## 四、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)

---

*Graph 模块设计 v2.6.0*
