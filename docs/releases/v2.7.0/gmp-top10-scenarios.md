# GMP Top 10 应用场景

> 版本: `v2.7.0`  
> 日期: 2026-05-XX  
> 相关 Issue: T-07

---

## 1. 概述

GMP (Graph Memory Processor) 是 SQLRustGo 的图谱处理引擎，支持在关系型数据库中直接进行图模式匹配与遍历。本文档定义 v2.7.0 的 Top 10 应用场景，作为场景化开发的依据。

### 1.1 Top 10 场景列表

| 排名 | 场景名称 | 场景描述 | 优先级 |
|------|----------|----------|--------|
| 1 | 社交网络好友推荐 | 基于二度人脉的好友推荐 | P0 |
| 2 | 知识图谱问答 | 多跳关系查询 | P0 |
| 3 | 欺诈检测 | 异常交易模式识别 | P0 |
| 4 | 推荐系统 | 用户-物品-行为三元组分析 | P1 |
| 5 | 供应链追踪 | 多层级供应链路径分析 | P1 |
| 6 | 组织架构分析 | 跨部门汇报链查询 | P1 |
| 7 | 安全威胁分析 | 攻击路径识别 | P2 |
| 8 | 生物信息检索 | 蛋白质相互作用网络 | P2 |
| 9 | 金融风控 | 关联企业担保链分析 | P2 |
| 10 | 物流优化 | 最短路径与网络分析 | P2 |

---

## 2. 场景详细定义

### 2.1 场景1: 社交网络好友推荐

#### 2.1.1 业务描述
基于社交网络的二度人脉关系，为用户推荐可能认识的人。

#### 2.1.2 数据模型
```sql
-- 用户表
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100)
);

-- 好友关系表 (边)
CREATE TABLE friendships (
    user_id BIGINT,
    friend_id BIGINT,
    created_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (friend_id) REFERENCES users(id)
);

-- 启用图谱
ALTER TABLE friendships ADD GRAPH (user_id, friend_id);
```

#### 2.1.3 GMP 查询示例
```sql
-- 查找用户A的二度好友（可能认识的人）
SELECT friend_of_friend.name
FROM friendships f1
JOIN friendships f2 ON f1.friend_id = f2.user_id
WHERE f1.user_id = @current_user_id
  AND f2.friend_id != @current_user_id
  AND f2.friend_id NOT IN (
    SELECT friend_id FROM friendships WHERE user_id = @current_user_id
  )
LIMIT 10;

-- GMP 原生查询
GRAPH MATCH (me)-[:friendship]->(friend)-[:friendship]->(suggestion)
WHERE me.id = @current_user_id
  AND suggestion.id != @current_user_id
RETURN suggestion.name, COUNT(*) AS common_friends
ORDER BY common_friends DESC
LIMIT 10;
```

#### 2.1.4 性能要求
- 二度查询延迟: < 100ms
- 支持 100万+ 用户规模

---

### 2.2 场景2: 知识图谱问答

#### 2.2.1 业务描述
支持多跳关系的知识图谱查询，如"谁是谁的老师的学生"。

#### 2.2.2 数据模型
```sql
CREATE TABLE entities (
    id BIGINT PRIMARY KEY,
    name VARCHAR(200),
    entity_type VARCHAR(50)  -- person, organization, concept
);

CREATE TABLE relations (
    subject_id BIGINT,
    relation_type VARCHAR(50),  -- teaches, works_for, located_in
    object_id BIGINT,
    weight FLOAT DEFAULT 1.0,
    FOREIGN KEY (subject_id) REFERENCES entities(id),
    FOREIGN KEY (object_id) REFERENCES entities(id)
);

ALTER TABLE relations ADD GRAPH (subject_id, object_id);
```

#### 2.2.3 GMP 查询示例
```sql
-- 三跳查询: 查找"清华大学老师教过的学生的工作单位"
GRAPH MATCH (university)-[:located_in]->(city)<-[:located_in]-(org)<-[:works_for]-(student)<-[:teaches]-(professor)
WHERE university.name = '清华大学' AND professor.name = @professor_name
RETURN DISTINCT org.name AS company, COUNT(DISTINCT student) AS num_students
ORDER BY num_students DESC;
```

---

### 2.3 场景3: 欺诈检测

#### 2.3.1 业务描述
通过异常交易模式识别潜在欺诈行为。

#### 2.3.2 数据模型
```sql
CREATE TABLE accounts (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100),
    account_type VARCHAR(20)  -- personal, corporate
);

CREATE TABLE transactions (
    id BIGINT PRIMARY KEY,
    from_account BIGINT,
    to_account BIGINT,
    amount DECIMAL(15,2),
    transaction_type VARCHAR(20),
    created_at TIMESTAMP
);

ALTER TABLE transactions ADD GRAPH (from_account, to_account);
```

#### 2.3.3 GMP 查询示例
```sql
-- 检测资金快速转移模式（黑钱洗白）
GRAPH MATCH (source)-[:transaction*1..3]->(middleman)-[:transaction]->(destination)
WHERE source.account_type = 'personal'
  AND destination.account_type = 'corporate'
  AND ALL(r IN relationships WHERE r.amount > 10000)
RETURN source.name, destination.name, COUNT(*) AS hop_count
ORDER BY hop_count DESC;

-- 检测环形交易（洗钱特征）
GRAPH MATCH path = (a)-[:transaction]->(b)-[:transaction]->(c)-[:transaction]->(a)
WHERE a.id < c.id  -- 避免重复
RETURN a.name, b.name, c.name, 
       [r IN relationships(path) | r.amount] AS amounts;
```

---

### 2.4 场景4: 推荐系统

#### 2.4.1 业务描述
基于用户-物品-行为的异构图谱进行推荐。

#### 2.4.2 数据模型
```sql
CREATE TABLE items (
    id BIGINT PRIMARY KEY,
    name VARCHAR(200),
    category VARCHAR(100)
);

CREATE TABLE user_item_actions (
    user_id BIGINT,
    item_id BIGINT,
    action_type VARCHAR(20),  -- view, cart, purchase, rating
    rating INT,  -- 1-5, nullable
    created_at TIMESTAMP
);

ALTER TABLE user_item_actions ADD GRAPH (user_id, item_id);
```

#### 2.4.3 GMP 查询示例
```sql
-- 协同过滤: 查找相似用户喜欢的物品
GRAPH MATCH (me)-[:action{purchase}]->(item)<-[:action{purchase}]-(similar_user)-[:action{purchase}]->(recommended)
WHERE me.id = @current_user_id
  AND recommended.id NOT IN (
    SELECT item_id FROM user_item_actions WHERE user_id = @current_user_id AND action_type = 'purchase'
  )
RETURN recommended.name, COUNT(DISTINCT similar_user) AS popularity
ORDER BY popularity DESC
LIMIT 10;
```

---

### 2.5 场景5: 供应链追踪

#### 2.5.1 业务描述
追踪商品从原材料到最终产品的全链路。

#### 2.5.2 数据模型
```sql
CREATE TABLE supply_nodes (
    id BIGINT PRIMARY KEY,
    name VARCHAR(200),
    node_type VARCHAR(50)  -- supplier, manufacturer, warehouse, retailer
);

CREATE TABLE supply_edges (
    from_node BIGINT,
    to_node BIGINT,
    lead_time_days INT,
    cost DECIMAL(15,2)
);

ALTER TABLE supply_edges ADD GRAPH (from_node, to_node);
```

#### 2.5.3 GMP 查询示例
```sql
-- 查找原材料到最终产品的所有路径
GRAPH MATCH path = (raw_material)-[:supply_edge*1..5]->(final_product)
WHERE raw_material.node_type = 'supplier'
  AND final_product.node_type = 'retailer'
  AND final_product.name = @product_name
RETURN [n IN nodes(path) | n.name] AS supply_chain,
       REDUCE(cost = 0, r IN relationships(path) | cost + r.cost) AS total_cost
ORDER BY total_cost ASC;
```

---

### 2.6 场景6: 组织架构分析

#### 2.6.1 业务描述
查询跨部门汇报链和协作关系。

#### 2.6.2 数据模型
```sql
CREATE TABLE employees (
    id BIGINT PRIMARY KEY,
    name VARCHAR(100),
    department VARCHAR(100),
    title VARCHAR(100)
);

CREATE TABLE org_edges (
    manager_id BIGINT,
    report_id BIGINT,
    relationship VARCHAR(20)  -- reports_to, works_with
);

ALTER TABLE org_edges ADD GRAPH (manager_id, report_id);
```

#### 2.6.3 GMP 查询示例
```sql
-- 查找跨部门协作路径
GRAPH MATCH (employee)-[:org_edge*1..3]->(target)
WHERE employee.department = 'Engineering'
  AND target.department = 'Sales'
  AND target.title CONTAINS 'Manager'
RETURN DISTINCT target.name, target.department, target.title,
       LENGTH(path) AS distance;
```

---

### 2.7 场景7: 安全威胁分析

#### 2.7.1 业务描述
识别网络攻击路径和横向移动特征。

#### 2.7.2 数据模型
```sql
CREATE TABLE hosts (
    id BIGINT PRIMARY KEY,
    hostname VARCHAR(200),
    ip_address VARCHAR(50),
    criticality VARCHAR(20)  -- low, medium, high, critical
);

CREATE TABLE network_edges (
    source_host BIGINT,
    target_host BIGINT,
    port INT,
    protocol VARCHAR(10),
    connection_count INT
);

ALTER TABLE network_edges ADD GRAPH (source_host, target_host);
```

#### 2.7.3 GMP 查询示例
```sql
-- 查找从低安全域到高安全域的攻击路径
GRAPH MATCH path = (entry_point)-[:network_edge*1..5]->(target)
WHERE entry_point.criticality = 'low'
  AND target.criticality IN ('high', 'critical')
  AND ALL(e IN relationships(path) WHERE e.connection_count > 100)
RETURN [h IN nodes(path) | h.hostname] AS attack_path,
       LENGTH(path) AS hops
ORDER BY hops ASC;
```

---

### 2.8 场景8: 生物信息检索

#### 2.8.1 业务描述
查询蛋白质相互作用网络。

#### 2.8.2 数据模型
```sql
CREATE TABLE proteins (
    id BIGINT PRIMARY KEY,
    uniprot_id VARCHAR(20),
    gene_name VARCHAR(50),
    organism VARCHAR(100)
);

CREATE TABLE protein_interactions (
    protein_a BIGINT,
    protein_b BIGINT,
    interaction_type VARCHAR(50),  -- binding, activation, inhibition
    confidence_score FLOAT
);

ALTER TABLE protein_interactions ADD GRAPH (protein_a, protein_b);
```

#### 2.8.3 GMP 查询示例
```sql
-- 查找疾病相关蛋白的相互作用网络
GRAPH MATCH (disease_protein)-[:interaction*1..2]-(target)
WHERE disease_protein.gene_name = @disease_gene
  AND target.organism = 'Homo sapiens'
RETURN target.gene_name, 
       [r IN relationships(path) | r.interaction_type] AS interaction_types,
       AVG([r IN relationships(path) | r.confidence_score]) AS avg_confidence
ORDER BY avg_confidence DESC;
```

---

### 2.9 场景9: 金融风控

#### 2.9.1 业务描述
分析关联企业担保链，识别连锁风险。

#### 2.9.2 数据模型
```sql
CREATE TABLE companies (
    id BIGINT PRIMARY KEY,
    name VARCHAR(200),
    credit_rating VARCHAR(10)
);

CREATE TABLE guarantee_edges (
    guarantor BIGINT,    -- 担保方
    guaranteed BIGINT,    -- 被担保方
    amount DECIMAL(15,2),
    expire_date DATE
);

ALTER TABLE guarantee_edges ADD GRAPH (guarantor, guaranteed);
```

#### 2.9.3 GMP 查询示例
```sql
-- 查找担保链中的核心企业（担保额度最高）
GRAPH MATCH (company)-[:guarantee_edge*1..3]->(connected)
WHERE company.credit_rating IN ('AAA', 'AA')
RETURN company.name AS guarantor,
       COLLECT(DISTINCT connected.name) AS guaranteed_companies,
       SUM([r IN relationships(path) | r.amount]) AS total_guaranteed
GROUP BY company.name
ORDER BY total_guaranteed DESC
LIMIT 10;
```

---

### 2.10 场景10: 物流优化

#### 2.10.1 业务描述
计算配送网络的最短路径和最优路线。

#### 2.10.2 数据模型
```sql
CREATE TABLE locations (
    id BIGINT PRIMARY KEY,
    name VARCHAR(200),
    location_type VARCHAR(50)  -- warehouse, store, hub
);

CREATE TABLE delivery_edges (
    from_location BIGINT,
    to_location BIGINT,
    distance_km FLOAT,
    transit_hours FLOAT,
    cost DECIMAL(10,2)
);

ALTER TABLE delivery_edges ADD GRAPH (from_location, to_location);
```

#### 2.10.3 GMP 查询示例
```sql
-- 查找最低成本配送路径
GRAPH MATCH path = (warehouse)-[:delivery_edge*1..4]->(store)
WHERE warehouse.location_type = 'warehouse'
  AND store.name = @store_name
RETURN [l IN nodes(path) | l.name] AS route,
       REDUCE(dist = 0, r IN relationships(path) | dist + r.distance_km) AS total_distance,
       REDUCE(cost = 0, r IN relationships(path) | cost + r.cost) AS total_cost
ORDER BY total_cost ASC
LIMIT 5;
```

---

## 3. 场景优先级与交付计划

### Phase 1 (v2.7.0-alpha): P0 场景
- [ ] 场景1: 社交网络好友推荐
- [ ] 场景2: 知识图谱问答
- [ ] 场景3: 欺诈检测

### Phase 2 (v2.7.0-beta): P1 场景
- [ ] 场景4: 推荐系统
- [ ] 场景5: 供应链追踪
- [ ] 场景6: 组织架构分析

### Phase 3 (v2.7.0-RC): P2 场景
- [ ] 场景7: 安全威胁分析
- [ ] 场景8: 生物信息检索
- [ ] 场景9: 金融风控
- [ ] 场景10: 物流优化

---

## 4. 性能基准

| 场景 | 数据规模 | 查询延迟目标 | 吞吐量目标 |
|------|----------|--------------|------------|
| 社交网络 | 100万用户, 1000万关系 | < 100ms | > 1000 QPS |
| 知识图谱 | 100万实体, 500万关系 | < 200ms | > 500 QPS |
| 欺诈检测 | 10万账户, 100万交易 | < 50ms | > 5000 QPS |
| 推荐系统 | 100万用户, 1000万物品 | < 100ms | > 1000 QPS |
| 供应链 | 10万节点, 50万边 | < 200ms | > 500 QPS |

---

## 5. 测试用例

每个场景需包含：
- ✅ 正常路径测试
- ✅ 边界条件测试
- ✅ 性能基准测试
- ✅ 压力测试

测试脚本位置: `tests/gmp_scenarios/`
