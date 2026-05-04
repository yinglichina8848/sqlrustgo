# 图检索用户指南

> **版本**: v2.9.0 (RC)
> **更新日期**: 2026-05-05

---

## 1. 概述

SQLRustGo v2.9.0 提供图检索引擎，支持属性图模型、 Cypher 查询语言和 GMP 可追溯性链。

### 1.1 核心概念

| 概念 | 说明 |
|------|------|
| **Node** | 实体节点，带标签和属性 |
| **Edge** | 关系边，连接两个节点 |
| **Label** | 标签，标识节点或边的类型 |
| **Property** | 属性，节点的键值对 |

### 1.2 GMP 可追溯性链

```
Batch → Device → Calibration → Regulation
```

---

## 2. 快速开始

### 2.1 创建节点

```rust
use sqlrustgo_graph::{Node, PropertyMap};

let mut props = PropertyMap::new();
props.insert("name".to_string(), "Batch-20260505".into());
props.insert("product".to_string(), "Vitamin-C".into());

let node = Node::new("batch".to_string(), props);
```

### 2.2 创建边

```rust
use sqlrustgo_graph::{Edge, PropertyMap};

let mut props = PropertyMap::new();
props.insert("device_id".to_string(), "DEV-001".into());
props.insert("timestamp".to_string(), "2026-05-05T10:00:00Z".into());

let edge = Edge::new(
    "manufactured_by".to_string(),
    source_node_id,
    target_node_id,
    props,
);
```

### 2.3 执行 Cypher 查询

```rust
use sqlrustgo_graph::GraphExecutor;

let executor = GraphExecutor::new(storage.clone());
let results = executor
    .execute_cypher("MATCH (b:batch)-[:manufactured_by]->(d:device) RETURN b, d")
    .unwrap();
```

---

## 3. Cypher 查询语言

### 3.1 MATCH

```sql
-- 匹配所有节点
MATCH (n) RETURN n;

-- 按标签匹配
MATCH (n:batch) RETURN n;

-- 带属性匹配
MATCH (n:batch {product: 'Vitamin-C'}) RETURN n;
```

### 3.2 WHERE

```sql
MATCH (n:batch)
WHERE n.product = 'Vitamin-C' AND n.created_at > '2026-01-01'
RETURN n;
```

### 3.3 RETURN

```sql
-- 返回指定属性
MATCH (n:batch)
RETURN n.name, n.product;

-- 别名
MATCH (n:batch)
RETURN n.name AS batch_name, n.product AS product_name;

-- 聚合
MATCH (n:batch)
RETURN COUNT(n), AVG(n.quantity);
```

### 3.4 CREATE

```sql
-- 创建节点
CREATE (n:batch {name: 'Batch-001', product: 'Vitamin-C'});

-- 创建关系
MATCH (b:batch {name: 'Batch-001'}), (d:device {name: 'DEV-001'})
CREATE (b)-[:manufactured_by]->(d);
```

### 3.5 DELETE

```sql
-- 删除节点
MATCH (n:batch {name: 'Batch-001'})
DELETE n;

-- 删除关系
MATCH (b)-[r:manufactured_by]->(d)
DELETE r;
```

---

## 4. 路径查询

### 4.1 最短路径

```sql
-- 查找两个节点间的最短路径
MATCH p = shortestPath((a:device)-[*]-(b:regulation))
WHERE a.name = 'DEV-001' AND b.name = 'REG-001'
RETURN p;
```

### 4.2 可变长度路径

```sql
-- 查找 1-3 跳的关系
MATCH (b:batch)-[:manufactured_by*1..3]->(n)
RETURN b, n;
```

---

## 5. 索引

### 5.1 创建索引

```sql
-- 为节点属性创建索引
CREATE INDEX ON :batch(name);
CREATE INDEX ON :batch(product);

-- 为关系属性创建索引
CREATE INDEX ON :manufactured_by(timestamp);
```

### 5.2 索引管理

```sql
-- 查看所有索引
SHOW INDEXES;

-- 删除索引
DROP INDEX ON :batch(name);
```

---

## 6. 事务

### 6.1 图事务

```rust
use sqlrustgo_graph::GraphTransaction;

let mut tx = executor.begin_transaction().unwrap();

// 创建节点
let batch = tx.create_node("batch", props).unwrap();

// 创建关系
tx.create_edge("manufactured_by", batch, device, edge_props).unwrap();

// 提交
tx.commit().unwrap();
```

### 6.2 回滚

```rust
// 回滚事务
tx.rollback().unwrap();
```

---

## 7. 配置

### 7.1 Cargo.toml 依赖

```toml
[dependencies]
sqlrustgo-graph = { version = "2.9", features = ["cypher", "traceability"] }
```

### 7.2 特性开关

| 特性 | 说明 |
|------|------|
| `cypher` | 启用 Cypher 查询语言 |
| `traceability` | 启用 GMP 可追溯性链 |
| `visualization` | 启用图可视化 |

---

## 8. 最佳实践

### 8.1 图建模

- 合理设计节点标签
- 避免创建过于通用的节点
- 使用属性存储核心数据，关系表达连接

### 8.2 查询优化

- 为常用查询创建索引
- 使用 LIMIT 限制结果集
- 避免深度递归查询

---

## 9. API 参考

| API | 说明 |
|-----|------|
| `GraphExecutor::new()` | 创建图执行器 |
| `execute_cypher()` | 执行 Cypher 查询 |
| `create_node()` | 创建节点 |
| `create_edge()` | 创建边 |
| `delete_node()` | 删除节点 |
| `delete_edge()` | 删除边 |

---

*图检索用户指南 v2.9.0*
*最后更新: 2026-05-05*
