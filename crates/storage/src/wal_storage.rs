use crate::bplus_tree::index::CompositeKey;
use crate::engine::{ColumnDefinition, Record, SqlResult, StorageEngine, TableInfo, Value};
use crate::wal::{WalEntry, WalEntryType, WalManager};
use std::path::PathBuf;

pub struct WalStorage<S: StorageEngine> {
    inner: S,
    wal: WalManager,
    current_tx_id: u64,
    wal_enabled: bool,
}

impl<S: StorageEngine> WalStorage<S> {
    pub fn new(inner: S, wal_path: PathBuf) -> SqlResult<Self> {
        Ok(Self {
            inner,
            wal: WalManager::new(wal_path),
            current_tx_id: 0,
            wal_enabled: true,
        })
    }

    pub fn new_without_wal(inner: S) -> Self {
        Self {
            inner,
            wal: WalManager::new(PathBuf::from("/dev/null")),
            current_tx_id: 0,
            wal_enabled: false,
        }
    }

    pub fn set_wal_enabled(&mut self, enabled: bool) {
        self.wal_enabled = enabled;
    }

    pub fn begin_transaction(&mut self) -> SqlResult<u64> {
        if self.current_tx_id != 0 {
            return Err(crate::engine::SqlError::ExecutionError(
                "Transaction already in progress".to_string(),
            )
            .into());
        }
        let tx_id = self.generate_tx_id();
        if self.wal_enabled {
            self.wal.log_begin(tx_id)?;
        }
        self.current_tx_id = tx_id;
        Ok(tx_id)
    }

    pub fn commit_transaction(&mut self) -> SqlResult<()> {
        if self.current_tx_id == 0 {
            return Err(crate::engine::SqlError::ExecutionError(
                "No transaction in progress".to_string(),
            )
            .into());
        }
        let tx_id = self.current_tx_id;
        if self.wal_enabled {
            self.wal.log_commit(tx_id)?;
            self.wal.sync()?;
        }
        self.current_tx_id = 0;
        Ok(())
    }

    pub fn rollback_transaction(&mut self) -> SqlResult<()> {
        if self.current_tx_id == 0 {
            return Err(crate::engine::SqlError::ExecutionError(
                "No transaction in progress".to_string(),
            )
            .into());
        }
        let tx_id = self.current_tx_id;
        if self.wal_enabled {
            self.wal.log_rollback(tx_id)?;
            self.wal.sync()?;
        }
        self.current_tx_id = 0;
        Ok(())
    }

    pub fn current_tx_id(&self) -> u64 {
        self.current_tx_id
    }

    pub fn in_transaction(&self) -> bool {
        self.current_tx_id != 0
    }

    pub fn recover(&self) -> SqlResult<Vec<WalEntry>> {
        Ok(self.wal.recover()?)
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    fn log_insert(&self, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> SqlResult<()> {
        if !self.wal_enabled || self.current_tx_id == 0 {
            return Ok(());
        }
        self.wal
            .log_insert(self.current_tx_id, table_id, key, data)?;
        Ok(())
    }

    fn log_update(&self, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> SqlResult<()> {
        if !self.wal_enabled || self.current_tx_id == 0 {
            return Ok(());
        }
        self.wal
            .log_update(self.current_tx_id, table_id, key, data)?;
        Ok(())
    }

    fn log_delete(&self, table_id: u64, key: Vec<u8>) -> SqlResult<()> {
        if !self.wal_enabled || self.current_tx_id == 0 {
            return Ok(());
        }
        self.wal.log_delete(self.current_tx_id, table_id, key)?;
        Ok(())
    }

    fn generate_tx_id(&self) -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use monotonic counter to avoid clock skew issues
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Get time-based component with error handling
        let time_component = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or_else(|e| {
                // Fallback for clock skew (time went backwards)
                // Use a fixed base + elapsed time
                e.duration().as_nanos() as u64
            });

        // Mix with atomic counter to ensure uniqueness even with fast calls
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        // XOR time and counter components for better distribution
        time_component.wrapping_add(counter.wrapping_mul(0x5F3759D0))
    }

    fn table_name_to_id(table: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        table.hash(&mut hasher);
        hasher.finish()
    }

    fn record_key(record: &[Value]) -> Vec<u8> {
        if record.is_empty() {
            return Vec::new();
        }
        match &record[0] {
            Value::Integer(i) => i.to_le_bytes().to_vec(),
            Value::Text(s) => s.as_bytes().to_vec(),
            Value::Boolean(b) => [*b as u8].to_vec(),
            Value::Null => Vec::new(),
            Value::Float(f) => f.to_bits().to_le_bytes().to_vec(),
            Value::Decimal(d) => format!("{:?}", d).into_bytes(),
            Value::Blob(b) => b.clone(),
            Value::Date(d) => d.to_le_bytes().to_vec(),
            Value::Timestamp(ts) => ts.to_le_bytes().to_vec(),
            Value::Uuid(u) => u.to_le_bytes().to_vec(),
            Value::Array(arr) => format!("{:?}", arr).into_bytes(),
            Value::Enum(idx, _) => idx.to_le_bytes().to_vec(),
        }
    }

    fn record_to_bytes(record: &[Value]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for value in record {
            match value {
                Value::Integer(i) => {
                    bytes.extend_from_slice(b"i:");
                    bytes.extend_from_slice(&i.to_le_bytes());
                }
                Value::Text(s) => {
                    bytes.extend_from_slice(b"s:");
                    bytes.extend_from_slice(s.as_bytes());
                    bytes.push(0);
                }
                Value::Boolean(b) => {
                    bytes.extend_from_slice(b"b:");
                    bytes.push(*b as u8);
                }
                Value::Null => {
                    bytes.extend_from_slice(b"n:");
                }
                Value::Float(f) => {
                    bytes.extend_from_slice(b"f:");
                    bytes.extend_from_slice(&f.to_bits().to_le_bytes());
                }
                Value::Decimal(d) => {
                    bytes.extend_from_slice(b"d:");
                    bytes.extend_from_slice(format!("{:?}", d).as_bytes());
                    bytes.push(0);
                }
                Value::Blob(b) => {
                    bytes.extend_from_slice(b"B:");
                    bytes.extend_from_slice(b);
                    bytes.push(0);
                }
                Value::Date(d) => {
                    bytes.extend_from_slice(b"D:");
                    bytes.extend_from_slice(&d.to_le_bytes());
                }
                Value::Timestamp(ts) => {
                    bytes.extend_from_slice(b"T:");
                    bytes.extend_from_slice(&ts.to_le_bytes());
                }
                Value::Uuid(u) => {
                    bytes.extend_from_slice(b"U:");
                    bytes.extend_from_slice(&u.to_le_bytes());
                }
                Value::Array(arr) => {
                    bytes.extend_from_slice(b"A:");
                    bytes.extend_from_slice(format!("{:?}", arr).as_bytes());
                    bytes.push(0);
                }
                Value::Enum(idx, name) => {
                    bytes.extend_from_slice(b"E:");
                    bytes.extend_from_slice(&idx.to_le_bytes());
                    bytes.extend_from_slice(name.as_bytes());
                    bytes.push(0);
                }
            }
        }
        bytes
    }
}

impl<S: StorageEngine> StorageEngine for WalStorage<S> {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        self.inner.scan(table)
    }

    fn get_row(&self, table: &str, row_index: usize) -> SqlResult<Option<Record>> {
        self.inner.get_row(table, row_index)
    }

    fn scan_batch(
        &self,
        table: &str,
        offset: usize,
        limit: usize,
    ) -> SqlResult<(Vec<Record>, usize, bool)> {
        self.inner.scan_batch(table, offset, limit)
    }

    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        let table_id = Self::table_name_to_id(table);
        for record in &records {
            let key = Self::record_key(record);
            let data = Self::record_to_bytes(record);
            self.log_insert(table_id, key, data)?;
        }
        self.inner.insert(table, records)
    }

    fn bulk_load_tbl_file(&mut self, table_name: &str, filepath: &str) -> SqlResult<usize> {
        self.inner.bulk_load_tbl_file(table_name, filepath)
    }

    fn delete(&mut self, table: &str, filters: &[Value]) -> SqlResult<usize> {
        let table_id = Self::table_name_to_id(table);
        let key = format!("{:?}", filters).into_bytes();
        self.log_delete(table_id, key)?;
        self.inner.delete(table, filters)
    }

    fn update(
        &mut self,
        table: &str,
        filters: &[Value],
        updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        let table_id = Self::table_name_to_id(table);
        let key = format!("{:?}", filters).into_bytes();
        let data = format!("{:?}", updates).into_bytes();
        self.log_update(table_id, key, data)?;
        self.inner.update(table, filters, updates)
    }

    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()> {
        self.inner.create_table(info)
    }

    fn drop_table(&mut self, table: &str) -> SqlResult<()> {
        self.inner.drop_table(table)
    }

    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo> {
        self.inner.get_table_info(table)
    }

    fn has_table(&self, table: &str) -> bool {
        self.inner.has_table(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.inner.list_tables()
    }

    fn create_table_index(
        &mut self,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()> {
        self.inner.create_table_index(table, column, column_index)
    }

    fn create_hash_index(
        &mut self,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()> {
        self.inner.create_hash_index(table, column, column_index)
    }

    fn drop_table_index(&mut self, table: &str, column: &str) -> SqlResult<()> {
        self.inner.drop_table_index(table, column)
    }

    fn search_index(&self, table: &str, column: &str, key: i64) -> Vec<u32> {
        self.inner.search_index(table, column, key)
    }

    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32> {
        self.inner.range_index(table, column, start, end)
    }

    fn create_composite_index(
        &mut self,
        table: &str,
        columns: Vec<String>,
    ) -> SqlResult<crate::engine::IndexId> {
        self.inner.create_composite_index(table, columns)
    }

    fn search_composite_index(
        &self,
        index_id: crate::engine::IndexId,
        key: &CompositeKey,
    ) -> SqlResult<Vec<u32>> {
        self.inner.search_composite_index(index_id, key)
    }

    fn range_composite_index(
        &self,
        index_id: crate::engine::IndexId,
        start: &CompositeKey,
        end: &CompositeKey,
    ) -> SqlResult<Vec<u32>> {
        self.inner.range_composite_index(index_id, start, end)
    }

    fn create_view(&mut self, info: crate::engine::ViewInfo) -> SqlResult<()> {
        self.inner.create_view(info)
    }

    fn get_view(&self, name: &str) -> Option<crate::engine::ViewInfo> {
        self.inner.get_view(name)
    }

    fn list_views(&self) -> Vec<String> {
        self.inner.list_views()
    }

    fn has_view(&self, name: &str) -> bool {
        self.inner.has_view(name)
    }

    fn create_trigger(&mut self, info: crate::engine::TriggerInfo) -> SqlResult<()> {
        self.inner.create_trigger(info)
    }

    fn drop_trigger(&mut self, name: &str) -> SqlResult<()> {
        self.inner.drop_trigger(name)
    }

    fn get_trigger(&self, name: &str) -> Option<crate::engine::TriggerInfo> {
        self.inner.get_trigger(name)
    }

    fn list_triggers(&self, table: &str) -> Vec<crate::engine::TriggerInfo> {
        self.inner.list_triggers(table)
    }

    fn analyze_table(&self, table: &str) -> SqlResult<crate::engine::TableStats> {
        self.inner.analyze_table(table)
    }

    fn get_next_auto_increment(&mut self, table: &str, column_index: usize) -> SqlResult<i64> {
        self.inner.get_next_auto_increment(table, column_index)
    }

    fn get_auto_increment_counter(&self, table: &str, column_index: usize) -> SqlResult<i64> {
        self.inner.get_auto_increment_counter(table, column_index)
    }

    fn on_write_complete(&mut self, table: &str) {
        self.inner.on_write_complete(table)
    }

    fn scan_columns(&self, table: &str, column_indices: &[usize]) -> SqlResult<Vec<Record>> {
        self.inner.scan_columns(table, column_indices)
    }

    fn set_cancel_flag(&mut self, flag: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        self.inner.set_cancel_flag(flag)
    }

    fn clear_cancel_flag(&mut self) {
        self.inner.clear_cancel_flag()
    }

    fn cancel_flag(&self) -> Option<std::sync::Arc<std::sync::atomic::AtomicBool>> {
        self.inner.cancel_flag()
    }

    fn check_cancelled(&self) -> SqlResult<()> {
        self.inner.check_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MemoryStorage;
    use tempfile::TempDir;

    #[test]
    fn test_wal_storage_basic_insert() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        let tx_id = storage.begin_transaction().unwrap();
        assert!(tx_id > 0);
        assert!(storage.in_transaction());

        let records = vec![vec![Value::Integer(1), Value::Text("test".to_string())]];
        storage.insert("t1", records).unwrap();

        storage.commit_transaction().unwrap();
        assert!(!storage.in_transaction());

        let entries = storage.recover().unwrap();
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .collect();
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn test_wal_storage_rollback() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        let records = vec![vec![Value::Integer(1)]];
        storage.insert("t1", records).unwrap();
        storage.rollback_transaction().unwrap();

        let entries = storage.recover().unwrap();
        let rollbacks: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Rollback)
            .collect();
        assert_eq!(rollbacks.len(), 1);
    }

    #[test]
    fn test_wal_storage_multiple_transactions() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        storage.insert("t1", vec![vec![Value::Integer(1)]]).unwrap();
        storage.commit_transaction().unwrap();

        storage.begin_transaction().unwrap();
        storage.insert("t1", vec![vec![Value::Integer(2)]]).unwrap();
        storage.commit_transaction().unwrap();

        let entries = storage.recover().unwrap();
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .collect();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_wal_storage_disabled() {
        let inner = MemoryStorage::new();
        let mut storage = WalStorage::new_without_wal(inner);

        storage.begin_transaction().unwrap();
        storage.insert("t1", vec![vec![Value::Integer(1)]]).unwrap();
        storage.commit_transaction().unwrap();

        let entries = storage.recover().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_wal_storage_without_transaction() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.insert("t1", vec![vec![Value::Integer(1)]]).unwrap();

        let entries = storage.recover().unwrap_or_default();
        let inserts: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert)
            .collect();
        assert_eq!(inserts.len(), 0);
    }

    #[test]
    fn test_wal_storage_error_no_transaction() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        let result = storage.commit_transaction();
        assert!(result.is_err());

        let result = storage.rollback_transaction();
        assert!(result.is_err());
    }

    #[test]
    fn test_update_without_begin_error() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage
            .create_table(&TableInfo {
                name: "t1".to_string(),
                columns: vec![
                    ColumnDefinition::new("id", "INTEGER"),
                    ColumnDefinition::new("value", "INTEGER"),
                ],
            })
            .unwrap();

        let result = storage.update("t1", &[Value::Integer(1)], &[(1, Value::Integer(200))]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_without_begin_error() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage
            .create_table(&TableInfo {
                name: "t1".to_string(),
                columns: vec![ColumnDefinition::new("id", "INTEGER")],
            })
            .unwrap();

        let result = storage.delete("t1", &[Value::Integer(1)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tx_id_uniqueness_rapid_generation() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        let mut tx_ids = Vec::new();
        for _ in 0..100 {
            let tx_id = storage.begin_transaction().unwrap();
            tx_ids.push(tx_id);
            storage.commit_transaction().unwrap();
        }

        let mut unique_ids = tx_ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(
            tx_ids.len(),
            unique_ids.len(),
            "Transaction IDs must be unique"
        );
    }

    #[test]
    fn test_rapid_commit_rollback_cycles() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        for i in 0..50 {
            storage.begin_transaction().unwrap();
            storage
                .insert("t1", vec![vec![Value::Integer(i as i64)]])
                .unwrap();
            if i % 2 == 0 {
                storage.commit_transaction().unwrap();
            } else {
                storage.rollback_transaction().unwrap();
            }
        }

        let entries = storage.recover().unwrap();
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .collect();
        let rollbacks: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Rollback)
            .collect();
        assert_eq!(commits.len(), 25);
        assert_eq!(rollbacks.len(), 25);
    }

    #[test]
    fn test_update_in_transaction() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage
            .create_table(&TableInfo {
                name: "t1".to_string(),
                columns: vec![
                    ColumnDefinition::new("id", "INTEGER"),
                    ColumnDefinition::new("value", "INTEGER"),
                ],
            })
            .unwrap();

        storage.begin_transaction().unwrap();
        storage
            .insert("t1", vec![vec![Value::Integer(1), Value::Integer(100)]])
            .unwrap();
        storage.commit_transaction().unwrap();

        storage.begin_transaction().unwrap();
        let result = storage.update("t1", &[Value::Integer(1)], &[(1, Value::Integer(200))]);
        assert!(result.is_ok());
        if result.is_ok() {
            storage.commit_transaction().unwrap();
        } else {
            storage.rollback_transaction().unwrap();
        }

        let entries = storage.recover().unwrap();
        let updates: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Update)
            .collect();
        assert_eq!(updates.len(), 1);
    }

    #[test]
    fn test_multi_table_transactions() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .unwrap();
        storage
            .insert("orders", vec![vec![Value::Integer(100), Value::Integer(1)]])
            .unwrap();
        storage
            .insert(
                "products",
                vec![vec![
                    Value::Integer(1000),
                    Value::Text("Widget".to_string()),
                ]],
            )
            .unwrap();
        storage.commit_transaction().unwrap();

        let entries = storage.recover().unwrap();
        let inserts: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert)
            .collect();
        assert_eq!(inserts.len(), 3);
    }

    #[test]
    fn test_zero_value_handling() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        storage
            .insert(
                "t1",
                vec![
                    vec![Value::Integer(0)],
                    vec![Value::Integer(-1)],
                    vec![Value::Text(String::new())],
                ],
            )
            .unwrap();
        storage.commit_transaction().unwrap();

        let entries = storage.recover().unwrap();
        assert!(entries.iter().any(|e| e.entry_type == WalEntryType::Commit));
    }

    #[test]
    fn test_delete_in_transaction() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        storage.insert("t1", vec![vec![Value::Integer(1)]]).unwrap();
        storage.commit_transaction().unwrap();

        storage.begin_transaction().unwrap();
        storage.delete("t1", &[Value::Integer(1)]).unwrap();
        storage.commit_transaction().unwrap();

        let entries = storage.recover().unwrap();
        let deletes: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Delete)
            .collect();
        assert_eq!(deletes.len(), 1);
    }

    #[test]
    fn test_nested_transaction_prevents() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        storage.begin_transaction().unwrap();
        let result = storage.begin_transaction();
        assert!(result.is_err());
        storage.commit_transaction().unwrap();
    }

    #[test]
    fn test_wal_sync_after_crash_simulation() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");

        {
            let inner = MemoryStorage::new();
            let mut storage = WalStorage::new(inner, wal_path.clone()).unwrap();
            storage.begin_transaction().unwrap();
            storage
                .insert("t1", vec![vec![Value::Integer(1), Value::Integer(100)]])
                .unwrap();
            storage.commit_transaction().unwrap();

            storage.begin_transaction().unwrap();
            storage
                .insert("t1", vec![vec![Value::Integer(2), Value::Integer(200)]])
                .unwrap();
        }

        {
            let inner = MemoryStorage::new();
            let storage = WalStorage::new(inner, wal_path).unwrap();
            let entries = storage.recover().unwrap();

            let commits: Vec<_> = entries
                .iter()
                .filter(|e| e.entry_type == WalEntryType::Commit)
                .collect();
            assert_eq!(commits.len(), 1);
        }
    }

    #[test]
    fn test_tx_id_counter_rollover_safety() {
        let dir = TempDir::new().unwrap();
        let inner = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        let mut storage = WalStorage::new(inner, wal_path).unwrap();

        let mut tx_ids = Vec::new();
        for _ in 0..1000 {
            let tx_id = storage.begin_transaction().unwrap();
            tx_ids.push(tx_id);
            storage.commit_transaction().unwrap();
        }

        let mut sorted = tx_ids.clone();
        sorted.sort();
        assert_eq!(
            tx_ids, sorted,
            "Transaction IDs should be generated in rough order"
        );
    }

    #[test]
    fn test_wal_recovery_preserves_all_entry_types() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("test.wal");
        let manager = crate::wal::WalManager::new(wal_path.clone());

        manager.log_begin(1).unwrap();
        manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
        manager.log_update(1, 1, vec![1], vec![20]).unwrap();
        manager.log_delete(1, 1, vec![1]).unwrap();
        manager.log_commit(1).unwrap();

        let entries = manager.recover().unwrap();
        assert_eq!(entries.len(), 5);

        let begins = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Begin)
            .count();
        let inserts = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Insert)
            .count();
        let updates = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Update)
            .count();
        let deletes = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Delete)
            .count();
        let commits = entries
            .iter()
            .filter(|e| e.entry_type == WalEntryType::Commit)
            .count();

        assert_eq!(begins, 1);
        assert_eq!(inserts, 1);
        assert_eq!(updates, 1);
        assert_eq!(deletes, 1);
        assert_eq!(commits, 1);
    }
}
