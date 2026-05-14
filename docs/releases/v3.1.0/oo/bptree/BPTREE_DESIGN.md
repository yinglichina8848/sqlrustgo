# B+Tree 索引设计 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> 磁盘持久化 B+Tree 索引的完整操作链路分析

## 1. B+Tree 架构

### 1.1 节点结构

```rust
const BTREE_ORDER: usize = 64;
const MAX_KEYS_PER_NODE: usize = 63;
const PAGE_DATA_SIZE: usize = 4032;

pub struct BTreeNode {
    pub is_leaf: bool,
    pub num_keys: u16,
    pub keys: Vec<i64>,
    pub values: Vec<u32>,
    pub children: Vec<u32>,
    pub next_leaf: Option<u32>,
}

pub struct BTreeIndex {
    pub metadata: BTreeMetadata,
    nodes: Vec<Option<BTreeNode>>,
    dirty: bool,
}

pub struct CompositeKey {
    pub values: Vec<Value>,
}
```

### 1.2 B+Tree 特性

| 特性 | 值 | 说明 |
|------|-----|------|
| 阶数 (d) | 64 | 每个内部节点最多 64 个子节点 |
| MAX_KEYS | 63 | 每个节点最多 63 个键 |
| PAGE_DATA_SIZE | 4032 | 4096 - 64 (页头) |
| 节点类型 | Internal(1) / Leaf(2) | 磁盘标记 |
| 叶子链表 | next_leaf | 范围查询支持 |

### 1.3 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [bplus_tree/index.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/bplus_tree/index.rs) | 1985 | B+Tree 磁盘版核心 |
| [bplus_tree/mod.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/bplus_tree/mod.rs) | 127 | 简化版 (BTreeMap) |
| [buffer_pool.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/buffer_pool.rs) | - | 页面缓存 |

## 2. Search 操作链路

### 2.1 搜索算法

```rust
pub fn search(&self, key: i64) -> Option<u32> {
    self.search_node(self.root_page_id, key)
}

fn search_node(&self, node_id: u32, key: i64) -> Option<u32> {
    let node = self.load_node(node_id)?;
    if node.is_leaf {
        node.find_key_index(key).map(|idx| node.values[idx])
    } else {
        let child_idx = node.find_child_index(key);
        self.search_node(node.children[child_idx], key)
    }
}
```

### 2.2 搜索时序图

```
Search(key=42)
    │
    ▼
LoadNode(root_page_id=1)
    │
    ├── is_leaf=false
    │
    ▼
find_child_index(42) → child_idx=2
    │
    ▼
LoadNode(page_id=5)
    │
    ├── is_leaf=false
    │
    ▼
find_child_index(42) → child_idx=3
    │
    ▼
LoadNode(page_id=9)
    │
    ├── is_leaf=true
    │
    ▼
binary_search(42) → Some(3)
    │
    ▼
Return values[3] → Some(row_id)
```

### 2.3 范围查询

```rust
pub fn range_query(&self, start: i64, end: i64) -> Vec<u32> {
    let mut results = Vec::new();
    let mut current = self.find_leaf_page(start);
    loop {
        for (idx, &key) in current.keys.iter().enumerate() {
            if key >= start && key < end {
                results.push(current.values[idx]);
            } else if key >= end {
                return results;
            }
        }
        match current.next_leaf {
            Some(next_id) => current = self.load_node(next_id)?,
            None => break,
        }
    }
    results
}
```

## 3. Insert 操作链路

### 3.1 插入时序图

```
Insert(key=42, value=100)
    │
    ▼
FindLeafPage(42) → page_id=9
    │
    ▼
LoadNode(page_id=9) [leaf]
    │
    ▼
key exists? → NO
    │
    ▼
IsNodeFull? → NO
    │
    ▼
BinarySearch position → pos=3
    │
    ▼
Shift keys[3..] and values[3..]
    │
    ▼
Insert key at keys[3], value at values[3]
    │
    ▼
node.num_keys++
    │
    ▼
MarkPageDirty(page_id=9)
    │
    ▼
Return
```

### 3.2 节点分裂时序图

```
Insert(key=95) - Node Full!
    │
    ▼
SplitNode(node, key=95, value=195)
    │
    ├── Create new_node
    │
    ▼
Copy upper half keys/values to new_node
    │
    ▼
median_key = keys[31] = 50
    │
    ▼
Insert median_key into parent
    │
    ▼
Parent.isFull? → YES
    │
    ▼
SplitParent() → recurse up to root
    │
    ▼
Create new_root with two children
    │
    ▼
Update root_page_id
```

### 3.3 Insert 状态图

```
                ┌─────────────┐
                │   START     │
                └──────┬──────┘
                       │
                FindLeafPage(key)
                       │
                ┌──────▼──────┐
                │   LOAD      │
                │   LEAF      │
                └──────┬──────┘
                       │
                ┌──────▼──────────┐
                │ key exists?     │
                └──┬──────────┬───┘
                YES│          │NO
                   ▼          │
            ┌──────────┐     │
            │  UPDATE   │     │
            │  VALUE    │     │
            └─────┬────┘     │
                  │          │
                  ▼          ▼
            ┌────────────────────┐
            │    IS FULL?        │
            └──┬─────────────┬───┘
            YES│             │NO
               ▼             │
        ┌──────────┐        │
        │  SPLIT    │        │
        │  NODE     │        │
        └────┬─────┘        │
             │               │
        ┌────▼──────┐       │
        │PROPAGATE  │       │
        │TO PARENT? │       │
        └──┬────┬───┘       │
        YES│    │NO         │
           ▼    │           │
    ┌──────────┐│          │
    │PROPAGATE ││          │
    │UPWARD    ││          │
    └────┬─────┘│          │
         │      │          │
         ▼      ▼          ▼
       ┌──────────────────────┐
       │       END            │
       └──────────────────────┘
```

## 4. Delete 操作链路

### 4.1 删除时序图

```
Delete(key=42)
    │
    ▼
FindLeafPage(42) → page_id=9
    │
    ▼
LoadNode(page_id=9) [leaf]
    │
    ▼
find_key_index(42) → Some(3)
    │
    ▼
Shift keys[4..] left, Shift values[4..] left
    │
    ▼
num_keys--
    │
    ▼
IsUnderflow? (num_keys < MIN_KEYS=31)
    │
    ├─NO: MarkDirty → Return true
    │
    └─YES: ⚠️ 当前实现无再平衡
            直接返回，节点保持稀疏
```

## 5. 算法复杂度与性能分析

### 5.1 操作复杂度

| 操作 | 时间复杂度 | IO 复杂度 | 说明 |
|------|------------|-----------|------|
| Search | O(log₆₄ N) | O(Height) | 二分搜索 + 递归下降 |
| Insert | O(log₆₄ N) | O(Height + 1) | 含分裂 |
| Delete | O(log₆₄ N) | O(Height + 1) | 无再平衡 |
| Range Query | O(log₆₄ N + K) | O(Height + K/63) | 叶子链表遍历 |
| Bulk Load | O(N log N) | O(N/63) | 排序 + 批量构建 |

### 5.2 B+Tree 高度估算

```
Height = ceil(log_d(N/d))

示例 (d=64):
- 1,000,000 键    → Height ≈ 3
- 100,000,000 键  → Height ≈ 4
- 10,000,000,000 键 → Height ≈ 5
```

### 5.3 ⚠️ 已知正确性问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **分裂不向上传播** | 🔴 严重 | 大量数据后树退化 | 实现 handle_split 递归传播 |
| **删除无再平衡** | 🟡 中等 | 长期运行节点稀疏 | 实现 underflow 处理 + 合并 |
| **CompositeKey 只取首列** | 🔴 严重 | 多列索引退化为单列 | 实现完整多列编码 |
| **基数估算硬编码 80%** | 🟡 中等 | CBO 选择错误 | 接入统计信息 |

### 5.4 性能瓶颈

```
瓶颈1: 分裂不向上传播
  当前: insert_at() 分裂叶节点后不处理父节点
  结果: 树高度不增长，大量数据时退化为宽叶节点
  修复: 实现 handle_split() 递归向上传播

瓶颈2: CompositeKey 编码
  当前: encode_composite_key() 只取 values[0].as_integer()
  结果: (a, b) 索引退化为 (a) 索引
  修复: 实现多列拼接编码 (如 PostgreSQL tuple encoding)

瓶颈3: 基数估算
  当前: estimate_cardinality() 返回 total * 0.8
  结果: CBO 无法区分选择性高的索引
  修复: 使用 NDV (Number of Distinct Values) 统计
```

## 6. 与其他模块的依赖

```
BTreeIndex
  ├── 依赖: sqlrustgo_types::Value (键值类型)
  ├── 依赖: serde (序列化/反序列化)
  ├── 依赖: storage::page (页面管理)
  ├── 被依赖: FileStorage (索引管理)
  ├── 被依赖: CboOptimizer (代价估算)
  └── 被依赖: ClusteredIndex (聚簇索引底层)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充正确性问题和性能瓶颈 |
