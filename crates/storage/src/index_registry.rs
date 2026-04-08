// storage/src/index_registry.rs

use crate::engine::IndexId;
use std::collections::HashMap;

/// 索引类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// B+Tree 单列索引
    BTree,
    /// Hash 索引
    Hash,
    /// 复合 B+Tree 索引
    CompositeBTree,
}

/// 索引元数据
#[derive(Debug, Clone, PartialEq)]
pub struct IndexMeta {
    /// 索引 ID
    pub id: IndexId,
    /// 表名
    pub table_name: String,
    /// 列名列表
    pub column_names: Vec<String>,
    /// 索引类型
    pub index_type: IndexType,
}

/// 索引注册表 - 管理所有索引元数据
#[derive(Debug, Clone, Default)]
pub struct IndexRegistry {
    indexes: HashMap<IndexId, IndexMeta>,
    name_to_id: HashMap<String, IndexId>,
}

impl IndexRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册新索引
    pub fn register(&mut self, meta: IndexMeta) {
        let id = meta.id;
        let name = Self::make_index_name(&meta.table_name, &meta.column_names);
        self.name_to_id.insert(name, id);
        self.indexes.insert(id, meta);
    }

    /// 根据 ID 获取索引元数据
    pub fn get(&self, id: IndexId) -> Option<&IndexMeta> {
        self.indexes.get(&id)
    }

    /// 根据表名和列名获取索引 ID
    pub fn get_by_columns(&self, table: &str, columns: &[String]) -> Option<IndexId> {
        let name = Self::make_index_name(table, columns);
        self.name_to_id.get(&name).copied()
    }

    /// 获取下一个可用的索引 ID
    pub fn next_id(&self) -> IndexId {
        IndexId(self.indexes.len() as u32)
    }

    /// 生成索引名称
    fn make_index_name(table: &str, columns: &[String]) -> String {
        format!("{}_{}", table, columns.join("_"))
    }

    /// 获取所有索引
    pub fn all_indexes(&self) -> impl Iterator<Item = &IndexMeta> {
        self.indexes.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_registry_register_and_get() {
        let mut registry = IndexRegistry::new();

        let meta = IndexMeta {
            id: IndexId(0),
            table_name: "users".to_string(),
            column_names: vec!["id".to_string()],
            index_type: IndexType::BTree,
        };

        registry.register(meta.clone());

        assert_eq!(registry.get(IndexId(0)), Some(&meta));
    }

    #[test]
    fn test_index_registry_get_by_columns() {
        let mut registry = IndexRegistry::new();

        let meta = IndexMeta {
            id: IndexId(0),
            table_name: "users".to_string(),
            column_names: vec!["id".to_string()],
            index_type: IndexType::BTree,
        };

        registry.register(meta);

        assert_eq!(
            registry.get_by_columns("users", &["id".to_string()]),
            Some(IndexId(0))
        );
    }
}
