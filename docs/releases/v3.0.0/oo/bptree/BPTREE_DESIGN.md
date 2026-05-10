# B+Tree 索引设计

> 磁盘持久化 B+Tree 索引的完整操作链路分析

## 1. B+Tree 架构

### 1.1 节点结构

```rust
// B+Tree 阶数: 64
const BTREE_ORDER: usize = 64;
const MAX_KEYS_PER_NODE: usize = BTREE_ORDER - 1;
const MAX_CHILDREN_PER_NODE: usize = BTREE_ORDER;

// 节点类型
const NODE_TYPE_INTERNAL: u8 = 1;  // 内部节点
const NODE_TYPE_LEAF: u8 = 2;     // 叶子节点

pub struct BTreeNode {
    pub is_leaf: bool,           // 是否为叶子节点
    pub num_keys: u16,           // 当前键数量
    pub keys: Vec<i64>,          // 键数组
    pub values: Vec<u32>,        // 值数组 (行ID)
    pub children: Vec<u32>,      // 子节点页ID数组
    pub next_leaf: Option<u32>, // 下一个叶子节点 (叶子节点专用)
}
```

### 1.2 B+Tree 特性

| 特性 | 说明 |
|------|------|
| 阶数 (d) | 每个内部节点最多 d 个子节点 |
| 键数量 | 内部节点: d-1 个键, 叶子节点: d-1 个键 |
| 高度平衡 | 所有叶子节点在同一层 |
| 叶子链表 | 叶子节点通过 next_leaf 连接 |

### 1.3 关键文件

| 文件 | 作用 |
|------|------|
| `crates/storage/src/bplus_tree/index.rs` | B+Tree 核心实现 (~1985行) |
| `crates/storage/src/engine.rs` | Storage engine facade |
| `crates/storage/src/buffer_pool.rs` | Buffer pool 管理 |

## 2. Search 操作链路

### 2.1 搜索算法

```rust
pub fn search(&self, key: i64) -> Option<u32> {
    self.search_node(self.root_page_id, key)
}

fn search_node(&self, node_id: u32, key: i64) -> Option<u32> {
    let node = self.load_node(node_id)?;

    if node.is_leaf {
        // 叶子节点: 二分查找键
        node.find_key_index(key)
            .map(|idx| node.values[idx])
    } else {
        // 内部节点: 找到第一个大于 key 的子节点
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
    ├──is_leaf=false
    │
    ▼
find_child_index(42) → 返回 child_idx=2
    │
    ▼
LoadNode(page_id=5)
    │
    ├──is_leaf=false
    │
    ▼
find_child_index(42) → 返回 child_idx=3
    │
    ▼
LoadNode(page_id=9)
    │
    ├──is_leaf=true
    │
    ▼
find_key_index(42) → 返回 Some(3)
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

### 3.1 插入算法

```rust
pub fn insert(&mut self, key: i64, value: u32) {
    // 1. 找到目标叶子节点
    let leaf_id = self.find_leaf_page(key);

    // 2. 加载叶子节点
    let mut leaf = self.load_node_mut(leaf_id);

    // 3. 检查是否已存在
    if leaf.find_key_index(key).is_some() {
        // 更新现有值
        leaf.values[leaf.find_key_index(key).unwrap()] = value;
        return;
    }

    // 4. 插入键值对
    let (split_key, new_node) = leaf.insert_at(pos, key, value);

    // 5. 如果节点需要分裂，处理向上传播
    if let Some((median_key, new_node)) = split_key {
        self.handle_split(leaf_id, median_key, new_node);
    }
}
```

### 3.2 插入时序图

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

### 3.3 节点分裂时序图

```
Insert(key=95, value=195) - Node Full!
    │
    ▼
SplitNode(node, key=95, value=195)
    │
    ├──Create new_node
    │
    ▼
Copy upper half of keys/values to new_node
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

### 3.4 Insert 完整状态图

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
                    ┌──────▼──────────────┐
                    │ key exists?         │
                    └──────┬──────────────┘
                     YES/   \NO
                    ┌──────▼──────┐
                    │   UPDATE    │
                    │   VALUE     │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  IS FULL?   │
                    └──────┬──────┘
                     YES/   \NO
                    ┌──────▼──────┐
                    │   SPLIT     │
                    │   NODE      │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  NEED       │
                    │  PROPAGATE? │
                    └──────┬──────┘
                     YES/   \NO
                    ┌──────▼──────┐
                    │  PROPAGATE   │
                    │  TO PARENT  │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │    END       │
                    └─────────────┘
```

## 4. Delete 操作链路

### 4.1 删除算法

```rust
pub fn delete(&mut self, key: i64) -> bool {
    let leaf_id = self.find_leaf_page(key);
    let mut leaf = self.load_node_mut(leaf_id);

    if let Some(idx) = leaf.find_key_index(key) {
        // 1. 从叶子节点删除
        leaf.remove_at(idx);

        // 2. 检查是否需要合并或借用
        if leaf.num_keys < MIN_KEYS && leaf_id != self.root_page_id {
            self.handle_underflow(leaf_id);
        }
        true
    } else {
        false
    }
}
```

### 4.2 删除时序图

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
Shift keys[4..] left
    │
    ▼
Shift values[4..] left
    │
    ▼
num_keys--
    │
    ▼
IsUnderflow? (num_keys < MIN_KEYS=31)
    │
    ├─NO: MarkDirty → Return true
    │
    └─YES: HandleUnderflow()
             │
             ▼
        TryBorrow from sibling
             │
        ├─SUCCESS: MarkDirty → Return true
        │
        └─FAIL: Merge with sibling
                  │
                  ▼
             Delete key from parent
                  │
                  ▼
             Recursive handle underflow
```

### 4.3 节点合并状态图

```
                ┌─────────────┐
                │   START     │
                └──────┬──────┘
                       │
               HandleUnderflow(leaf_id)
                       │
                ┌──────▼──────┐
                │  TryBorrow  │
                │  from       │
                │  sibling    │
                └──────┬──────┘
                 YES/   \NO
                ┌──────▼──────┐
                │  Borrow     │
                │  Success?   │
                └──────┬──────┘
                 YES/   \NO
          ┌──────┴──────┐
          ▼             ▼
    ┌─────────┐   ┌─────────────┐
    │ Redist- │   │   Merge      │
    │  ribute │   │   with       │
    └────┬────┘   │   sibling   │
         │         └──────┬──────┘
         │                │
         │         ┌──────▼──────┐
         │         │  Delete     │
         │         │  key from   │
         │         │  parent     │
         │         └──────┬──────┘
         │                │
         │         ┌──────▼──────┐
         │         │  Parent     │
         │         │  underflow? │
         │         └──────┬──────┘
         │          YES/   \NO
         │    ┌─────────┴─────────┐
         │    ▼                   ▼
         │┌─────────┐      ┌─────────┐
         ││Recursive│      │   END   │
         ││Handle   │      └─────────┘
         │└────┬────┘
         │     │
         └─────┘
```

## 5. B+Tree 测试计划

### 5.1 覆盖率目标

| 操作 | 当前覆盖率 | 目标覆盖率 |
|------|------------|------------|
| Search | 100% | 100% |
| Insert | 85% | 98% |
| Delete | 75% | 95% |
| Split | 80% | 95% |
| Merge | 60% | 90% |
| Range Query | 70% | 95% |

### 5.2 关键测试用例

```rust
// T1: 基本插入和搜索
fn test_insert_and_search() {
    let mut tree = BPlusTree::new();
    for i in 0..1000 {
        tree.insert(i as i64, i);
    }
    assert_eq!(tree.search(500), Some(500));
}

// T2: 节点分裂触发
fn test_node_split() {
    let mut tree = BPlusTree::new();
    // 插入足够多的键触发分裂
    for i in 0..10000 {
        tree.insert(i as i64, i);
    }
    // 验证根节点分裂
    assert!(tree.height > 1);
}

// T3: 范围查询
fn test_range_query() {
    let mut tree = BPlusTree::new();
    for i in 0..100 {
        tree.insert(i as i64, i);
    }
    let results = tree.range_query(10, 20);
    assert_eq!(results.len(), 10);
}

// T4: 删除和合并
fn test_delete_and_merge() {
    let mut tree = BPlusTree::new();
    for i in 0..100 {
        tree.insert(i as i64, i);
    }
    for i in 0..50 {
        tree.delete(i as i64);
    }
    // 验证合并后结构正确
}

// T5: 边界条件
fn test_boundary_conditions() {
    let mut tree = BPlusTree::new();
    tree.insert(i64::MIN, 1);
    tree.insert(i64::MAX, 2);
    tree.insert(0, 3);
    assert_eq!(tree.search(i64::MIN), Some(1));
}
```

## 6. 性能评估

### 6.1 B+Tree 高度计算

```
Height = ceil(log_d(N/d))

其中:
- d = BTREE_ORDER = 64 (阶数)
- N = 总键数
- d = 每个叶子节点键数 ≈ 63

示例:
- 1,000,000 键 → Height ≈ 3
- 100,000,000 键 → Height ≈ 4
```

### 6.2 操作复杂度

| 操作 | 时间复杂度 | IO 复杂度 |
|------|------------|------------|
| Search | O(log_d N) | Height |
| Insert | O(log_d N) | Height + 1 (写) |
| Delete | O(log_d N) | Height + 1 (写) |
| Range Query | O(log_d N + k) | Height + k/d |

### 6.3 Buffer Pool 影响

```
Hit Rate = (Total - Misses) / Total
Avg IO Cost = Hit Rate * 0 + (1 - Hit Rate) * DiskLatency

期望 Hit Rate > 95% for good performance
```
