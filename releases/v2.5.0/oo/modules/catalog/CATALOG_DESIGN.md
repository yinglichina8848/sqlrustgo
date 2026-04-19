# Catalog 模块设计

**版本**: v2.5.0
**模块**: Catalog (元数据管理)

---

## 一、What (是什么)

Catalog 是 SQLRustGo 的元数据管理系统，负责管理数据库 schema、表结构、索引、视图等元数据信息。

## 二、Why (为什么)

- **元数据管理**: 统一管理所有数据库对象的元数据
- **名称解析**: SQL 中的表名、列名解析
- **权限管理**: 用户和权限信息管理
- **系统表**: 提供 Information Schema 支持

## 三、核心数据结构

```rust
pub struct Catalog {
    tables: HashMap<TableId, TableSchema>,
    indexes: HashMap<IndexId, IndexSchema>,
    views: HashMap<ViewId, ViewDef>,
    users: HashMap<UserId, User>,
    schemas: HashMap<SchemaId, Schema>,
}
```

## 四、相关文档

- [ARCHITECTURE_V2.5.md](../../architecture/ARCHITECTURE_V2.5.md)

---

*Catalog 模块设计 v2.5.0*
