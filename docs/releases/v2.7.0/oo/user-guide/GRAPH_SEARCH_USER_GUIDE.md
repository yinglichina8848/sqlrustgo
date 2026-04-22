# 图检索用户指南

> **版本**: v2.7.0 (GA)
> **更新日期**: 2026-04-22

---

## 1. 概述

SQLRustGo v2.7.0 提供图检索引擎，支持属性图模型、 Cypher 查询语言和 GMP 可追溯性链。

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
props.insert("name".to_string(), "Batch-20260422".into());
props.insert("product".to_string(), "Vitamin-C".into());

let node = Node::new("batch".to_string(), props);
```

### 2.2 创建边

```rust
use sqlrustgo_graph::{Edge, PropertyMap};

let mut props = PropertyMap::new();
props.insert("device_id".to_string(), "DEV-001".into());
props.insert("timestamp".to_string(), "2026-04-22T10:00:00Z".into());

let edge = Edge::new(
    "manufactured_by".to_string(),
    props,
);
```

### 2.3 图遍历

```rust
use sqlrustgo_graph::GraphStore;

let store = GraphStore::new();
store.insert_node(batch_node)?;
store.insert_edge(&batch_node, &device_node, edge)?;
```

---

## 3. Cypher 查询

### 3.1 匹配查询

```sql
-- 查找所有批次
MATCH (b:batch)
RETURN b.name, b.product;

-- 查找特定产品的批次
MATCH (b:batch {product: 'Vitamin-C'})
RETURN b.name, b.created_at;
```

### 3.2 关系查询

```sql
-- 查找批次对应的设备
MATCH (b:batch)-[:manufactured_by]->(d:device)
WHERE b.name = 'Batch-20260422'
RETURN d.name, d.model;
```

### 3.3 多跳查询

```sql
-- 三跳查询：批次 → 设备 → 校准 → 法规
MATCH (b:batch)-[:manufactured_by]->(d:device)-[:calibrated_by]->(c:calibration)-[:follows]->(r:regulation)
WHERE b.name = 'Batch-20260422'
RETURN b.name, d.name, c.date, r.standard;
```

---

## 4. GMP 应用场景

### 4.1 批记录追溯

```sql
-- 追溯批次全生命周期
MATCH path = (b:batch {batch_id: 'B20260422'})-[:manufactured_by|processed_by|tested_by*1..5]->(end)
RETURN path;
```

### 4.2 设备校准链

```sql
-- 设备校准历史
MATCH (d:device {device_id: 'DEV-001'})-[:calibrated_by]->(c:calibration)
RETURN c.calibration_date, c.result, c.next_due_date
ORDER BY c.calibration_date DESC;
```

### 4.3 偏差调查

```sql
-- 查找与偏差相关的批次和设备
MATCH (b:batch)-[:involved_in]->(dev:deviation)<-[:caused_by]-(d:device)
WHERE dev.deviation_id = 'DEV-2026-001'
RETURN b.name, d.name, dev.description;
```

---

## 5. 图引擎配置

### 5.1 分片配置

```rust
use sqlrustgo_graph::sharded_graph::{MultiShardGraphStore, GraphShardId};

let shards = vec![
    GraphShardId::new(0),
    GraphShardId::new(1),
    GraphShardId::new(2),
];

let store = MultiShardGraphStore::new(shards, hash_func);
```

### 5.2 遍历配置

```rust
use sqlrustgo_graph::traversal::TraversalBuilder;

let traversal = TraversalBuilder::new()
    .max_depth(5)
    .with_weights(true)
    .build();
```

---

## 6. 性能优化

### 6.1 索引优化

```sql
-- 为常用查询字段创建索引
CREATE INDEX idx_batch_product ON batch(product);
CREATE INDEX idx_device_type ON device(device_type);
```

### 6.2 批量导入

```rust
use sqlrustgo_graph::GraphStore;

let mut batch = Vec::new();
// 准备节点...
store.batch_insert_nodes(&batch)?;
```

---

## 7. 最佳实践

### 7.1 图设计

- 节点标签命名统一采用小写
- 边标签采用动词短语
- 属性名采用 snake_case

### 7.2 查询优化

- 避免全图遍历
- 使用限制条件提前过滤
- 合理设置遍历深度

### 7.3 GMP 合规

- 所有批次必须可追溯
- 设备校准记录完整
- 偏差调查证据链完整

---

## 8. API 参考

| API | 说明 |
|-----|------|
| `Node::new()` | 创建节点 |
| `Edge::new()` | 创建边 |
| `GraphStore::insert_node()` | 插入节点 |
| `GraphStore::insert_edge()` | 插入边 |
| `GraphStore::traverse()` | 图遍历 |
| `MultiShardGraphStore` | 分片图存储 |

---

## 9. 故障排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 节点未找到 | 标签或ID错误 | 检查查询条件 |
| 边创建失败 | 节点不存在 | 先创建节点 |
| 遍历超时 | 图过大或深度过深 | 增加限制条件 |

---

*图检索用户指南 v2.7.0*
*最后更新: 2026-04-22*