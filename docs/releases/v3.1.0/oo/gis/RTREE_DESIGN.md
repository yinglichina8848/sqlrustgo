# GIS 空间索引 R-Tree (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> R-Tree 空间索引 / MBR / 二次分裂 / ST_* 空间函数

## 1. R-Tree 架构

### 1.1 核心数据结构

```rust
pub struct RTree {
    root: RTreeNode,
    max_entries: usize,   // 4
    min_entries: usize,   // 2
    next_geometry_id: u64,
}

pub enum RTreeNode {
    Leaf { entries: Vec<Entry> },
    Internal { children: Vec<RTreeNode>, mbr: MBR },
}

pub struct MBR {
    pub min_x: f64, pub max_x: f64,
    pub min_y: f64, pub max_y: f64,
}

pub struct Entry {
    pub id: u64,
    pub geometry: Geometry,
    pub mbr: MBR,
}
```

### 1.2 支持的几何类型

| 类型 | 说明 | WKT 示例 |
|------|------|---------|
| Point | 点 | `POINT(1 2)` |
| LineString | 线 | `LINESTRING(0 0, 1 1, 2 2)` |
| Polygon | 多边形 | `POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))` |
| MultiPoint | 多点 | `MULTIPOINT(0 0, 1 1)` |
| MultiLineString | 多线 | `MULTILINESTRING((0 0, 1 1), (2 2, 3 3))` |
| MultiPolygon | 多多边形 | `MULTIPOLYGON(((0 0, 1 0, 1 1, 0 0)))` |
| GeometryCollection | 几何集合 | `GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(0 0, 1 1))` |

### 1.3 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [rtree.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/gis/src/rtree.rs) | 1115 | R-Tree 空间索引 |
| [spatial.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/gis/src/spatial.rs) | 1328 | ST_* 空间函数 |
| [geometry.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/gis/src/geometry.rs) | - | 几何类型定义 |
| [wkt.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/gis/src/wkt.rs) | - | WKT 解析 |
| [wkb.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/gis/src/wkb.rs) | - | WKB 编解码 |

## 2. R-Tree 插入链路

### 2.1 插入时序图

```
insert(geometry=POINT(5 5))
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. 计算 MBR                                      │
│    ├── Point → MBR(5,5,5,5)                      │
│    └── Polygon → 计算所有顶点的 min/max           │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. choose_leaf(mbr)                               │
│    ├── 从根节点开始                                │
│    ├── 对每个子节点计算 MBR 扩展面积              │
│    ├── 选择扩展最小的子节点                       │
│    └── 递归直到叶节点                              │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 3. 添加到叶节点                                   │
│    ├── entries.push(Entry { id, geometry, mbr })  │
│    └── 检查 entries.len() > max_entries           │
└──────────────────┬───────────────────────────────┘
                   │
        ┌──────────┴──────────┐
        │ 需要分裂?           │
        └──┬──────────────┬───┘
        YES│              │NO
           ▼              │
┌──────────────────────┐  │
│ quadratic_split()    │  │
│ ├── pick_seeds()     │  │
│ │   O(n²) ⚠️        │  │
│ ├── 分配条目到两组   │  │
│ └── pick_next()      │  │
└──────────┬───────────┘  │
           │              │
           ▼              ▼
┌──────────────────────────────────────────────────┐
│ 4. adjust_tree(parent)                            │
│    ├── 更新 MBR                                   │
│    └── 如果分裂，向父节点添加新节点               │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
              Return geom_id
```

### 2.2 插入活动图

```
    ┌──────────────┐
    │ insert(geom) │
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 计算 MBR     │
    └──────┬───────┘
           │
           ▼
    ┌──────────────────┐
    │ choose_leaf(mbr) │
    └──────┬───────────┘
           │
           ▼
    ┌──────────────────┐
    │ 添加到叶节点      │
    └──────┬───────────┘
           │
    ┌──────┴──────────┐
    │ entries > max?   │
    └──┬──────────┬───┘
      YES       NO
       │         │
       ▼         │
┌──────────────┐  │
│quadratic_split│  │
│O(n²) ⚠️     │  │
└──────┬───────┘  │
       │          │
       ▼          ▼
┌──────────────────────┐
│ adjust_tree(parent)  │
└──────┬───────────────┘
       │
       ▼
┌──────────────┐
│ 返回 geom_id │
└──────────────┘
```

## 3. 搜索链路

### 3.1 MBR 搜索

```
search(mbr=MBR(0,0,10,10))
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. 从根节点开始                                   │
│    ├── 检查根 MBR 是否与查询 MBR 重叠            │
│    └── 不重叠 → 返回空                            │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. 递归搜索                                       │
│    ├── Internal: 对每个子节点检查 MBR 重叠        │
│    │   └── 重叠 → 递归搜索子节点                 │
│    └── Leaf: 对每个 Entry 检查 MBR 重叠          │
│        └── 重叠 → 加入结果集                      │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
            返回匹配的 geom_id 列表
```

## 4. 算法复杂度与性能分析

### 4.1 操作复杂度

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| 插入 | O(log n) 平均 | 分裂时 O(n) |
| 搜索 (MBR) | O(n^0.5) | 2D R-Tree |
| search_contains | O(n^0.5) + 精确检查 | MBR 预过滤 + 精确 |
| search_intersects | O(n^0.5) + 精确检查 | MBR 预过滤 + 精确 |
| 删除 | O(log n) | 无重新插入 |
| 二次分裂 | O(n²) | ⚠️ pick_seeds |
| 批量加载 | O(n log n) | Sort-Tile-Recursive |

### 4.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **二次分裂 O(n²)** | 🟡 中等 | 大节点分裂慢 | R*-Tree 线性分裂 O(n) |
| **删除无重新插入** | 🟡 中等 | 长期运行后 MBR 膨胀 | 条件重新插入 |
| **max_entries=4 太小** | 🟡 中等 | 树深度大 | 调整为 50-100 |
| **无批量加载** | 🟡 中等 | 初始构建慢 | 实现 STR 批量加载 |

### 4.3 性能优化建议

```
优化1: R*-Tree 分裂算法
  当前: Quadratic Split O(n²)
  建议: R*-Tree 选择最小重叠/面积扩展的分裂
  预期: 搜索性能提升 30-50%, 分裂 O(n)

优化2: 增大 max_entries
  当前: max_entries=4, min_entries=2
  建议: max_entries=50, min_entries=25 (磁盘页对齐)
  预期: 树深度减少, IO 减少

优化3: 批量加载 (STR)
  当前: 逐条插入
  建议: Sort-Tile-Recursive 批量构建
  预期: 构建时间 O(n log n), 搜索性能更优
```

## 5. 与其他模块的依赖

```
RTree
  ├── 依赖: gis::geometry (几何类型)
  ├── 依赖: gis::wkt (WKT 解析)
  ├── 被依赖: gis::spatial (ST_* 函数)
  └── 被依赖: GIS 集成测试
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充二次分裂 O(n²) 问题、R* 优化建议 |
