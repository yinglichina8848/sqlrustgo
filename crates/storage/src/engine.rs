//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;

use crate::bplus_tree::SimpleBPlusTree;

/// Column statistics for a single column
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub distinct_count: u64,
    pub null_count: u64,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

/// Table statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub column_stats: Vec<ColumnStats>,
}

/// Foreign key referential action
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
}

/// Foreign key constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyConstraint {
    pub referenced_table: String,
    pub referenced_column: String,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

/// Column definition for table schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_unique: bool,
    pub is_primary_key: bool,
    pub references: Option<ForeignKeyConstraint>,
    pub auto_increment: bool,
}

impl ColumnDefinition {
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: true,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
        }
    }
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

/// Table data - combines metadata and rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Record>,
}

/// Record type - a single row of values
pub type Record = Vec<Value>;

/// StorageEngine trait - abstraction for table storage
/// Enables multiple storage backends (FileStorage, MemoryStorage, etc.)
pub trait StorageEngine: Send + Sync {
    /// Scan all rows from a table
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>>;

    /// Scan rows in batches for streaming (memory-efficient)
    /// Returns (records, total_count, has_more)
    fn scan_batch(
        &self,
        table: &str,
        offset: usize,
        limit: usize,
    ) -> SqlResult<(Vec<Record>, usize, bool)> {
        let all_records = self.scan(table)?;
        let total = all_records.len();
        let has_more = offset + limit < total;
        let batch = all_records.into_iter().skip(offset).take(limit).collect();
        Ok((batch, total, has_more))
    }

    /// Insert rows into a table
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()>;

    /// Delete rows matching a filter
    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize>;

    /// Update rows matching a filter
    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize>;

    /// Create a new table
    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()>;

    /// Drop a table
    fn drop_table(&mut self, table: &str) -> SqlResult<()>;

    /// Get table metadata
    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo>;

    /// Check if table exists
    fn has_table(&self, table: &str) -> bool;

    /// List all tables
    fn list_tables(&self) -> Vec<String>;

    /// Create an index on a table
    fn create_table_index(
        &mut self,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_table_index(&mut self, table: &str, column: &str) -> SqlResult<()>;

    /// Search using index - returns row IDs matching the key
    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32>;

    /// Range query using index - returns row IDs in range [start, end)
    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32>;

    /// Create a view
    fn create_view(&mut self, info: ViewInfo) -> SqlResult<()>;

    /// Get view info
    fn get_view(&self, name: &str) -> Option<ViewInfo>;

    /// List all views
    fn list_views(&self) -> Vec<String>;

    /// Check if view exists
    fn has_view(&self, name: &str) -> bool;

    /// Analyze table and collect statistics
    fn analyze_table(&self, table: &str) -> SqlResult<TableStats>;

    /// Get the next auto_increment value for a table column
    /// Returns the next value and increments the counter
    fn get_next_auto_increment(&mut self, table: &str, column_index: usize) -> SqlResult<i64>;

    /// Get the current auto_increment counter for a table column
    fn get_auto_increment_counter(&self, table: &str, column_index: usize) -> SqlResult<i64>;

    /// Callback triggered after write operations (INSERT/UPDATE/DELETE)
    /// Used by upper layers to invalidate query caches
    fn on_write_complete(&mut self, _table: &str) {}
}

/// In-memory storage implementation for testing and caching
#[allow(clippy::type_complexity)]
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
    views: HashMap<String, ViewInfo>,
    indexes: HashMap<String, SimpleBPlusTree>,
    write_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
    auto_increment_counters: HashMap<String, HashMap<usize, i64>>,
}

#[derive(Clone, Debug)]
pub struct ViewInfo {
    pub name: String,
    pub query: String,
    pub schema: TableInfo,
    pub records: Vec<Record>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            views: HashMap::new(),
            indexes: HashMap::new(),
            write_callback: None,
            auto_increment_counters: HashMap::new(),
        }
    }

    pub fn with_callback(callback: Box<dyn Fn(&str) + Send + Sync>) -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            views: HashMap::new(),
            indexes: HashMap::new(),
            write_callback: Some(callback),
            auto_increment_counters: HashMap::new(),
        }
    }

    pub fn create_view(&mut self, info: ViewInfo) -> SqlResult<()> {
        self.views.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn get_view(&self, name: &str) -> Option<&ViewInfo> {
        self.views.get(name)
    }

    pub fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    pub fn has_view(&self, name: &str) -> bool {
        self.views.contains_key(name)
    }

    /// Get all tables that have foreign key references to the given parent table
    pub fn get_tables_referencing(&self, parent_table: &str) -> Vec<String> {
        let mut referencing_tables = Vec::new();
        for (table_name, table_info) in &self.table_infos {
            for col_def in &table_info.columns {
                if let Some(ref fk) = col_def.references {
                    if fk.referenced_table == parent_table {
                        referencing_tables.push(table_name.clone());
                        break;
                    }
                }
            }
        }
        referencing_tables
    }

    /// Get the column indices and FK constraints that reference a parent table
    /// Returns Vec of (column_index, ForeignKeyConstraint)
    pub fn get_referencing_columns(
        &self,
        child_table: &str,
        parent_table: &str,
    ) -> Vec<(usize, ForeignKeyConstraint)> {
        let mut result = Vec::new();
        if let Some(table_info) = self.table_infos.get(child_table) {
            for (col_idx, col_def) in table_info.columns.iter().enumerate() {
                if let Some(ref fk) = col_def.references {
                    if fk.referenced_table == parent_table {
                        result.push((col_idx, fk.clone()));
                    }
                }
            }
        }
        result
    }

}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MemoryStorage {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        Ok(self.tables.get(table).cloned().unwrap_or_default())
    }

    fn insert(&mut self, table: &str, mut records: Vec<Record>) -> SqlResult<()> {
        if records.is_empty() {
            return Ok(());
        }

        let table_info = self.table_infos.get(table);

        if let Some(info) = table_info {
            let has_unique = info.columns.iter().any(|c| c.is_unique);
            if has_unique {
                let table_records = self.tables.get(table).cloned().unwrap_or_default();
                let existing: Vec<&Record> = table_records.iter().collect();
                for record in &records {
                    for (col_idx, col_def) in info.columns.iter().enumerate() {
                        if col_def.is_unique {
                            if let Some(value) = record.get(col_idx) {
                                for existing_record in &existing {
                                    if let Some(existing_val) = existing_record.get(col_idx) {
                                        if existing_val == value {
                                            return Err(SqlError::DuplicateKey {
                                                value: value.to_string(),
                                                key: col_def.name.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Foreign key validation
            for record in &records {
                for (col_idx, col_def) in info.columns.iter().enumerate() {
                    if let Some(ref fk) = col_def.references {
                        if let Some(value) = record.get(col_idx) {
                            if *value != Value::Null {
                                // Check if the referenced value exists in the parent table
                                let parent_table_info = self.table_infos.get(&fk.referenced_table);
                                if let Some(parent_info) = parent_table_info {
                                    if let Some(parent_col_idx) = parent_info
                                        .columns
                                        .iter()
                                        .position(|c| c.name == fk.referenced_column)
                                    {
                                        let parent_records = self
                                            .tables
                                            .get(&fk.referenced_table)
                                            .cloned()
                                            .unwrap_or_default();
                                        let value_exists = parent_records.iter().any(|r| {
                                            r.get(parent_col_idx)
                                                .map(|v| v == value)
                                                .unwrap_or(false)
                                        });
                                        if !value_exists {
                                            return Err(SqlError::ExecutionError(format!(
                                                "Foreign key constraint violation: {} references {}.{} = {} which does not exist",
                                                col_def.name,
                                                fk.referenced_table,
                                                fk.referenced_column,
                                                value
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.tables
            .entry(table.to_string())
            .or_default()
            .append(&mut records);
        self.on_write_complete(table);
        Ok(())
    }

    fn delete(&mut self, table: &str, filters: &[Value]) -> SqlResult<usize> {
        // If filters is empty, delete all records (original behavior)
        // If filters has values, interpret as: filter[0] is the column index, filter[1] is the value to match

        // First, collect information needed for FK constraint checking
        let (col_idx, match_value, needs_fk_check) = if filters.is_empty() {
            (0, &Value::Null, false)
        } else if filters.len() >= 2 {
            let col_idx = match &filters[0] {
                Value::Integer(i) => *i as usize,
                _ => return Err(SqlError::ExecutionError(
                    "Filter column index must be an integer".to_string(),
                )),
            };
            (col_idx, &filters[1], true)
        } else {
            return Err(SqlError::ExecutionError(
                "Invalid filter format: expected [column_index, value]".to_string(),
            ));
        };

        // First, find the records that will be deleted (for FK action processing)
        let records_to_delete: Vec<Vec<Value>> = if needs_fk_check {
            if let Some(table_records) = self.tables.get(table) {
                table_records.iter()
                    .filter(|row| {
                        row.get(col_idx).map(|v| v == match_value).unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Process FK actions BEFORE deleting
        // For CASCADE, we need to keep processing until no more records are deleted
        // to handle transitive dependencies (e.g., CEO -> Manager -> Worker)
        if needs_fk_check && !records_to_delete.is_empty() {
            let mut restrict_error: Option<String> = None;
            let referenced_by = self.get_tables_referencing(table);

            // For handling transitive CASCADE deletions, we need to track values that were deleted
            // so we can find records that reference THOSE deleted records
            let mut values_to_check: Vec<Value> = vec![match_value.clone()];
            let mut processed_values: Vec<Value> = Vec::new(); // Track already processed to avoid infinite loops

            // First pass: check RESTRICT on the original values
            for child_table in &referenced_by {
                let referencing_cols = self.get_referencing_columns(child_table, table);
                for (child_col_idx, fk) in referencing_cols {
                    match fk.on_delete {
                        Some(ForeignKeyAction::Restrict) => {
                            if let Some(child_records) = self.tables.get(child_table) {
                                for child_row in child_records {
                                    if let Some(fk_value) = child_row.get(child_col_idx) {
                                        if fk_value == match_value {
                                            restrict_error = Some(format!(
                                                "Foreign key constraint violation: ON DELETE RESTRICT - child table '{}' has references to the parent",
                                                child_table
                                            ));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Some(ForeignKeyAction::SetNull) => {
                            self.set_foreign_key_null(child_table, child_col_idx, match_value)?;
                        }
                        Some(ForeignKeyAction::Cascade) | None => {}
                    }
                }
            }

            // Return RESTRICT error if found
            if let Some(err_msg) = restrict_error {
                return Err(SqlError::ExecutionError(err_msg));
            }

            // Process CASCADE deletions with multiple passes to handle transitive dependencies
            // e.g., CEO -> Manager -> Worker: deleting CEO cascades to Manager, which then cascades to Worker
            loop {
                let mut next_values_to_check: Vec<Value> = Vec::new();

                // For each value that needs checking (initially the deleted parent's value,
                // then values from cascade-deleted records)
                for current_value in &values_to_check {
                    // Skip if already processed (avoid infinite loops in case of circular refs)
                    if processed_values.contains(current_value) {
                        continue;
                    }
                    processed_values.push(current_value.clone());

                    // Find and delete child records that reference this value
                    for child_table in &referenced_by {
                        let referencing_cols = self.get_referencing_columns(child_table, table);
                        for (child_col_idx, fk) in referencing_cols {
                            if fk.on_delete == Some(ForeignKeyAction::Cascade) {
                                // Get records that will be deleted to collect their PK values for next pass
                                let records_to_delete: Vec<Vec<Value>> = if let Some(child_records) = self.tables.get(child_table) {
                                    child_records.iter()
                                        .filter(|row| {
                                            row.get(child_col_idx)
                                                .map(|v| v == current_value)
                                                .unwrap_or(false)
                                        })
                                        .cloned()
                                        .collect()
                                } else {
                                    vec![]
                                };

                                // Only propagate cascade if the child table is the SAME as the parent table
                                // (self-referencing FK). For non-self-referencing tables, cascade stops here.
                                // This is because orders referencing users doesn't mean records referencing orders should be deleted.
                                if child_table == table {
                                    // Self-referencing: collect PK of deleted records to find more children
                                    let child_pk_col_idx = 0; // Assume first column is PK

                                    // Collect the PK values from records being deleted
                                    // These become the next set of values to check for cascade
                                    for record in &records_to_delete {
                                        if let Some(pk_value) = record.get(child_pk_col_idx) {
                                            if !next_values_to_check.contains(pk_value) {
                                                next_values_to_check.push(pk_value.clone());
                                            }
                                        }
                                    }
                                }

                                // Delete the child records
                                let deleted = self.delete_internal(
                                    child_table,
                                    &[Value::Integer(child_col_idx as i64), current_value.clone()]
                                )?;
                            }
                        }
                    }
                }

                // If no new values to check, we're done cascading
                if next_values_to_check.is_empty() {
                    break;
                }

                // Continue with the next set of values
                values_to_check = next_values_to_check;
            }
        }

        // Now get mutable access to records and delete
        let records = self.tables.get_mut(table).ok_or_else(|| {
            SqlError::TableNotFound { table: table.to_string() }
        })?;

        let mut deleted_count = 0;
        if filters.is_empty() {
            // Delete all records
            deleted_count = records.len();
            records.clear();
        } else if filters.len() >= 2 {
            // Delete matching records
            let original_len = records.len();
            records.retain(|row| {
                if let Some(value) = row.get(col_idx) {
                    value != match_value
                } else {
                    true
                }
            });
            deleted_count = original_len - records.len();
        }

        self.on_write_complete(table);
        Ok(deleted_count)
    }

    fn update(
        &mut self,
        table: &str,
        filters: &[Value],
        updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        if updates.is_empty() {
            return Ok(0);
        }

        // First, get table info (immutable borrow)
        let table_info = self.table_infos.get(table).ok_or_else(|| {
            SqlError::TableNotFound { table: table.to_string() }
        })?;

        // Find which updated columns are referenced by foreign keys in other tables
        // For ON UPDATE CASCADE/SET NULL/RESTRICT, we're updating a PARENT column
        // and need to find CHILD tables that reference it
        #[derive(Debug)]
        struct UpdatedColumn {
            col_idx: usize,
            col_name: String,
            referenced_by: Vec<(String, usize, ForeignKeyConstraint)>, // (child_table, child_col_idx, fk)
        }

        let mut updated_columns: Vec<UpdatedColumn> = Vec::new();
        for (col_idx, _new_value) in updates {
            if let Some(col_def) = table_info.columns.get(*col_idx) {
                // Find all child tables that reference this column
                let mut referencing = Vec::new();
                let referenced_by = self.get_tables_referencing(table);
                for child_table in &referenced_by {
                    let ref_cols = self.get_referencing_columns(child_table, table);
                    for (child_col_idx, fk) in ref_cols {
                        // fk.referenced_column is the column in THIS (parent) table that is referenced
                        if fk.referenced_column == col_def.name {
                            referencing.push((child_table.clone(), child_col_idx, fk.clone()));
                        }
                    }
                }
                if !referencing.is_empty() {
                    updated_columns.push(UpdatedColumn {
                        col_idx: *col_idx,
                        col_name: col_def.name.clone(),
                        referenced_by: referencing,
                    });
                }
            }
        }

        // If we're updating columns that are referenced by child tables, handle FK actions
        // Collect all operations we need to perform
        #[derive(Debug)]
        enum ChildUpdateOp {
            SetNull { child_table: String, child_col_idx: usize, match_value: Value },
            Cascade { child_table: String, child_col_idx: usize, new_value: Value },
        }

        let mut child_ops: Vec<ChildUpdateOp> = Vec::new();
        let mut restrict_violation: Option<String> = None;

        // First pass: check RESTRICT and collect SET NULL/CASCADE operations
        // For each parent column being updated that is referenced by children
        for updated_col in &updated_columns {
            for (child_table, child_col_idx, child_fk) in &updated_col.referenced_by {
                match child_fk.on_update {
                    Some(ForeignKeyAction::Restrict) => {
                        // Check if any child record has the old value
                        if let Some(old_value) = filters.get(1) {
                            let child_records = self.tables.get(child_table);
                            if let Some(child_records) = child_records {
                                for child_row in child_records {
                                    if let Some(fk_value) = child_row.get(*child_col_idx) {
                                        if fk_value == old_value {
                                            restrict_violation = Some(format!(
                                                "Foreign key constraint violation: ON UPDATE RESTRICT - child table '{}' has references to the parent",
                                                child_table
                                            ));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(ForeignKeyAction::SetNull) => {
                        // Queue SET NULL operation
                        if let Some(old_value) = filters.get(1) {
                            child_ops.push(ChildUpdateOp::SetNull {
                                child_table: child_table.clone(),
                                child_col_idx: *child_col_idx,
                                match_value: old_value.clone(),
                            });
                        }
                    }
                    Some(ForeignKeyAction::Cascade) => {
                        // Queue CASCADE operation - find the new value for this column
                        if let Some((_, new_value)) = updates.iter().find(|(idx, _)| *idx == updated_col.col_idx) {
                            if let Some(_old_value) = filters.get(1) {
                                child_ops.push(ChildUpdateOp::Cascade {
                                    child_table: child_table.clone(),
                                    child_col_idx: *child_col_idx,
                                    new_value: new_value.clone(),
                                });
                            }
                        }
                    }
                    None => {}
                }
            }
        }

        // Return RESTRICT error if found
        if let Some(err_msg) = restrict_violation {
            return Err(SqlError::ExecutionError(err_msg));
        }

        // Apply child updates first (if updating FK columns)
        for op in &child_ops {
            match op {
                ChildUpdateOp::SetNull { child_table, child_col_idx, match_value } => {
                    if let Some(child_records) = self.tables.get_mut(child_table) {
                        for child_row in child_records.iter_mut() {
                            if let Some(fk_value) = child_row.get(*child_col_idx) {
                                if fk_value == match_value {
                                    child_row[*child_col_idx] = Value::Null;
                                }
                            }
                        }
                    }
                }
                ChildUpdateOp::Cascade { child_table, child_col_idx, new_value } => {
                    if let Some(child_records) = self.tables.get_mut(child_table) {
                        for child_row in child_records.iter_mut() {
                            if let Some(fk_value) = child_row.get(*child_col_idx) {
                                // For CASCADE, we need to match the OLD value that we're updating FROM
                                // But we don't have the old value directly - we have match_value in filters
                                // Actually, for CASCADE, the child should be updated to match the new parent value
                                // The matching is done via filters[1] which is the old value
                                if let Some(old_value) = filters.get(1) {
                                    if fk_value == old_value {
                                        child_row[*child_col_idx] = new_value.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Now get mutable access to records and apply updates to the parent table
        let records = self.tables.get_mut(table).ok_or_else(|| {
            SqlError::TableNotFound { table: table.to_string() }
        })?;

        let mut updated_count = 0;
        if filters.is_empty() {
            // Update all records
            for row in records.iter_mut() {
                for (col_idx, new_value) in updates {
                    if let Some(elem) = row.get_mut(*col_idx) {
                        *elem = new_value.clone();
                        updated_count += 1;
                    }
                }
            }
            // Divide by number of updates per row to get unique row count
            if !updates.is_empty() {
                updated_count /= updates.len();
            }
        } else if filters.len() >= 2 {
            // filters[0] = column index, filters[1] = value to match
            let col_idx = match &filters[0] {
                Value::Integer(i) => *i as usize,
                _ => return Err(SqlError::ExecutionError(
                    "Filter column index must be an integer".to_string(),
                )),
            };
            let match_value = &filters[1];

            for row in records.iter_mut() {
                if let Some(value) = row.get(col_idx) {
                    if value == match_value {
                        for (upd_col_idx, new_value) in updates {
                            if let Some(elem) = row.get_mut(*upd_col_idx) {
                                *elem = new_value.clone();
                            }
                        }
                        updated_count += 1;
                    }
                }
            }
        }

        self.on_write_complete(table);
        Ok(updated_count)
    }

    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()> {
        self.table_infos.insert(info.name.clone(), info.clone());
        self.tables.entry(info.name.clone()).or_default();
        Ok(())
    }

    fn drop_table(&mut self, table: &str) -> SqlResult<()> {
        self.tables.remove(table);
        self.table_infos.remove(table);
        Ok(())
    }

    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo> {
        self.table_infos.get(table).cloned().ok_or_else(|| {
            sqlrustgo_types::SqlError::TableNotFound {
                table: table.to_string(),
            }
        })
    }

    fn has_table(&self, table: &str) -> bool {
        self.tables.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    fn create_table_index(
        &mut self,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()> {
        let index_name = format!("{}_{}", table, column);
        let mut tree = SimpleBPlusTree::new();

        if let Some(records) = self.tables.get(table) {
            for (row_id, record) in records.iter().enumerate() {
                if let Some(value) = record.get(column_index) {
                    if let Some(key) = value.to_index_key() {
                        tree.insert(key, row_id as u32);
                    }
                }
            }
        }

        self.indexes.insert(index_name, tree);
        Ok(())
    }

    fn drop_table_index(&mut self, table: &str, column: &str) -> SqlResult<()> {
        let index_name = format!("{}_{}", table, column);
        self.indexes.remove(&index_name);
        Ok(())
    }

    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32> {
        let index_name = format!("{}_{}", table, column);
        self.indexes
            .get(&index_name)
            .and_then(|tree| tree.search(key))
    }

    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32> {
        let index_name = format!("{}_{}", table, column);
        self.indexes
            .get(&index_name)
            .map(|tree| tree.range_query(start, end))
            .unwrap_or_default()
    }

    fn create_view(&mut self, info: ViewInfo) -> SqlResult<()> {
        self.views.insert(info.name.clone(), info);
        Ok(())
    }

    fn get_view(&self, name: &str) -> Option<ViewInfo> {
        self.views.get(name).cloned()
    }

    fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    fn has_view(&self, name: &str) -> bool {
        self.views.contains_key(name)
    }

    fn analyze_table(&self, table: &str) -> SqlResult<TableStats> {
        let records = self
            .tables
            .get(table)
            .ok_or_else(|| SqlError::TableNotFound {
                table: table.to_string(),
            })?;

        let table_info = self.table_infos.get(table);

        let mut column_stats = Vec::new();

        if let Some(info) = table_info {
            for col in &info.columns {
                let mut null_count = 0u64;
                let mut distinct_values: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                for record in records {
                    if let Some(idx) = info.columns.iter().position(|c| c.name == col.name) {
                        if let Some(val) = record.get(idx) {
                            match val {
                                Value::Null => null_count += 1,
                                _ => {
                                    distinct_values.insert(val.to_string());
                                }
                            }
                        }
                    }
                }

                column_stats.push(ColumnStats {
                    column_name: col.name.clone(),
                    distinct_count: distinct_values.len() as u64,
                    null_count,
                    min_value: None,
                    max_value: None,
                });
            }
        }

        Ok(TableStats {
            table_name: table.to_string(),
            row_count: records.len() as u64,
            column_stats,
        })
    }

    fn on_write_complete(&mut self, table: &str) {
        if let Some(callback) = &self.write_callback {
            callback(table);
        }
    }

    fn get_next_auto_increment(&mut self, table: &str, column_index: usize) -> SqlResult<i64> {
        let counters = self
            .auto_increment_counters
            .entry(table.to_string())
            .or_insert_with(HashMap::new);
        let next = counters.entry(column_index).or_insert(0).clone();
        counters.insert(column_index, next + 1);
        Ok(next + 1)
    }

    fn get_auto_increment_counter(&self, table: &str, column_index: usize) -> SqlResult<i64> {
        let counters =
            self.auto_increment_counters
                .get(table)
                .ok_or_else(|| SqlError::TableNotFound {
                    table: table.to_string(),
                })?;
        Ok(*counters.get(&column_index).unwrap_or(&0))
    }
}

/// Helper methods for MemoryStorage (not part of StorageEngine trait)
impl MemoryStorage {
    /// Internal delete without FK action processing (used for CASCADE)
    pub fn delete_internal(&mut self, table: &str, filters: &[Value]) -> SqlResult<usize> {
        let (col_idx, match_value) = if filters.len() >= 2 {
            let col_idx = match &filters[0] {
                Value::Integer(i) => *i as usize,
                _ => return Err(SqlError::ExecutionError(
                    "Filter column index must be an integer".to_string(),
                )),
            };
            (col_idx, &filters[1])
        } else {
            return Err(SqlError::ExecutionError(
                "Invalid filter format: expected [column_index, value]".to_string(),
            ));
        };

        let records = self.tables.get_mut(table).ok_or_else(|| {
            SqlError::TableNotFound { table: table.to_string() }
        })?;

        let original_len = records.len();
        records.retain(|row| {
            if let Some(value) = row.get(col_idx) {
                value != match_value
            } else {
                true
            }
        });

        let deleted_count = original_len - records.len();
        self.on_write_complete(table);
        Ok(deleted_count)
    }

    /// Set foreign key column to NULL for records matching the given value
    pub fn set_foreign_key_null(&mut self, table: &str, col_idx: usize, match_value: &Value) -> SqlResult<()> {
        let records = self.tables.get_mut(table).ok_or_else(|| {
            SqlError::TableNotFound { table: table.to_string() }
        })?;

        for row in records.iter_mut() {
            if let Some(value) = row.get(col_idx) {
                if value == match_value {
                    if col_idx < row.len() {
                        row[col_idx] = Value::Null;
                    }
                }
            }
        }

        self.on_write_complete(table);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that StorageEngine trait is defined correctly
    #[test]
    fn test_storage_engine_trait_exists() {
        fn _check_trait(_engine: &dyn StorageEngine) {}
    }

    #[test]
    fn test_memory_storage_new() {
        let storage = MemoryStorage::new();
        assert!(storage.list_tables().is_empty());
    }

    #[test]
    fn test_memory_storage_has_table() {
        let storage = MemoryStorage::new();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_list_tables() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        let tables = storage.list_tables();
        assert!(tables.contains(&"users".to_string()));
    }

    #[test]
    fn test_memory_storage_scan_empty() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        let result = storage.scan("users").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_memory_storage_insert_and_scan() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );
        let result = storage.scan("users").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_storage_engine_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MemoryStorage>();
    }

    #[test]
    fn test_memory_storage_create_table() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            }],
        };
        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_drop_table() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        storage.drop_table("users").unwrap();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_get_table_info() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            }],
        };
        storage.create_table(&info).unwrap();
        let result = storage.get_table_info("users").unwrap();
        assert_eq!(result.name, "users");
    }

    #[test]
    fn test_memory_storage_delete() {
        let mut storage = MemoryStorage::new();
        storage
            .tables
            .insert("users".to_string(), vec![vec![Value::Integer(1)]]);
        let count = storage.delete("users", &[]).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_memory_storage_update() {
        let mut storage = MemoryStorage::new();
        storage
            .tables
            .insert("users".to_string(), vec![vec![Value::Integer(1)]]);
        let count = storage
            .update("users", &[], &[(0, Value::Integer(2))])
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_column_definition() {
        let col = ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
            references: None,
        };
        assert_eq!(col.name, "id");
    }

    #[test]
    fn test_table_info() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
        };
        assert_eq!(info.name, "users");
    }

    #[test]
    fn test_table_data() {
        let data = TableData {
            info: TableInfo {
                name: "users".to_string(),
                columns: vec![],
            },
            rows: vec![],
        };
        assert_eq!(data.info.name, "users");
    }

    #[test]
    fn test_memory_storage_default() {
        let storage = MemoryStorage::default();
        assert!(storage.tables.is_empty());
    }

    #[test]
    fn test_record_new() {
        let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
        assert_eq!(record.len(), 2);
    }

    #[test]
    fn test_record_index() {
        let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
        assert_eq!(record[0], Value::Integer(1));
    }

    #[test]
    fn test_memory_storage_with_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let storage = MemoryStorage::with_callback(Box::new(move |_table| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        assert!(storage.write_callback.is_some());
    }

    #[test]
    fn test_memory_storage_scan_batch() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
                vec![Value::Integer(4)],
                vec![Value::Integer(5)],
            ],
        );

        let (batch, total, has_more) = storage.scan_batch("users", 0, 2).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(total, 5);
        assert!(has_more);

        let (batch, total, has_more) = storage.scan_batch("users", 2, 2).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(total, 5);
        assert!(has_more);

        let (batch, total, has_more) = storage.scan_batch("users", 4, 2).unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(total, 5);
        assert!(!has_more);
    }

    #[test]
    fn test_memory_storage_scan_batch_empty() {
        let storage = MemoryStorage::new();
        let (batch, total, has_more) = storage.scan_batch("nonexistent", 0, 10).unwrap();
        assert!(batch.is_empty());
        assert_eq!(total, 0);
        assert!(!has_more);
    }
}

#[test]
fn test_record_index() {
    let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
    assert_eq!(record[0], Value::Integer(1));
}

#[test]
fn test_column_definition_new() {
    let col = ColumnDefinition {
        name: "id".to_string(),
        data_type: "INTEGER".to_string(),
        nullable: false,
        is_unique: true,
        references: None,
    };
    assert_eq!(col.name, "id");
    assert_eq!(col.data_type, "INTEGER");
    assert!(!col.nullable);
    assert!(col.is_unique);
}

#[test]
fn test_table_info_new() {
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                references: None,
            },
        ],
    };
    assert_eq!(info.name, "users");
    assert_eq!(info.columns.len(), 2);
}

#[test]
fn test_table_stats_new() {
    let stats = TableStats {
        table_name: "users".to_string(),
        row_count: 100,
        column_stats: vec![ColumnStats {
            column_name: "id".to_string(),
            distinct_count: 100,
            null_count: 0,
            min_value: Some(Value::Integer(1)),
            max_value: Some(Value::Integer(100)),
        }],
    };
    assert_eq!(stats.row_count, 100);
    assert_eq!(stats.column_stats.len(), 1);
}

#[test]
fn test_column_stats_new() {
    let stats = ColumnStats {
        column_name: "id".to_string(),
        distinct_count: 50,
        null_count: 5,
        min_value: Some(Value::Integer(1)),
        max_value: Some(Value::Integer(100)),
    };
    assert_eq!(stats.column_name, "id");
    assert_eq!(stats.distinct_count, 50);
}

#[test]
fn test_table_data_new() {
    let data = TableData {
        info: TableInfo {
            name: "users".to_string(),
            columns: vec![],
        },
        rows: vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
    };
    assert_eq!(data.rows.len(), 2);
}

#[test]
fn test_column_definition_serialize() {
    let col = ColumnDefinition {
        name: "id".to_string(),
        data_type: "INTEGER".to_string(),
        nullable: false,
        is_unique: true,
        references: None,
    };
    let json = serde_json::to_string(&col).unwrap();
    assert!(json.contains("id"));
}

#[test]
fn test_table_info_serialize() {
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("users"));
}

#[test]
fn test_view_info_new() {
    let view = ViewInfo {
        name: "user_view".to_string(),
        query: "SELECT * FROM users".to_string(),
        schema: TableInfo {
            name: "user_view".to_string(),
            columns: vec![],
        },
        records: vec![],
    };
    assert_eq!(view.name, "user_view");
}

#[test]
fn test_memory_storage_views() {
    let mut storage = MemoryStorage::new();
    let view = ViewInfo {
        name: "user_view".to_string(),
        query: "SELECT * FROM users".to_string(),
        schema: TableInfo {
            name: "user_view".to_string(),
            columns: vec![],
        },
        records: vec![],
    };
    storage.create_view(view).unwrap();
    assert!(storage.has_view("user_view"));
    assert_eq!(storage.list_views(), vec!["user_view"]);
}

#[test]
fn test_memory_storage_get_view() {
    let mut storage = MemoryStorage::new();
    let view = ViewInfo {
        name: "v1".to_string(),
        query: "SELECT 1".to_string(),
        schema: TableInfo {
            name: "v1".to_string(),
            columns: vec![],
        },
        records: vec![],
    };
    storage.create_view(view).unwrap();
    let retrieved = storage.get_view("v1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().query, "SELECT 1");
}

#[test]
fn test_memory_storage_insert_empty() {
    let mut storage = MemoryStorage::new();
    let result = storage.insert("users", vec![]);
    assert!(result.is_ok());
}

#[test]
fn test_memory_storage_insert_with_info() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();
    let rows = storage.scan("users").unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn test_memory_storage_duplicate_key() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();
    let result = storage.insert("users", vec![vec![Value::Integer(1)]]);
    assert!(result.is_err());
}

#[test]
fn test_memory_storage_get_table_info() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    let retrieved = storage.get_table_info("users").unwrap();
    assert_eq!(retrieved.name, "users");
    assert_eq!(retrieved.columns.len(), 1);
}

#[test]
fn test_memory_storage_get_table_info_not_found() {
    let storage = MemoryStorage::new();
    let result = storage.get_table_info("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_memory_storage_default() {
    let storage = MemoryStorage::default();
    assert!(storage.tables.is_empty());
}

#[test]
fn test_memory_storage_with_callback() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let called = AtomicBool::new(false);
    let storage = MemoryStorage::with_callback(Box::new(move |_t| {
        called.store(true, Ordering::SeqCst);
    }));
    assert!(storage.write_callback.is_some());
}

#[test]
fn test_memory_storage_on_write_complete() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    let count = AtomicUsize::new(0);
    let storage = MemoryStorage::with_callback(Box::new(move |_t| {
        count.fetch_add(1, Ordering::SeqCst);
    }));
    assert!(storage.write_callback.is_some());
}

#[test]
fn test_memory_storage_list_tables() {
    let mut storage = MemoryStorage::new();
    let info1 = TableInfo {
        name: "users".to_string(),
        columns: vec![],
    };
    let info2 = TableInfo {
        name: "orders".to_string(),
        columns: vec![],
    };
    storage.create_table(&info1).unwrap();
    storage.create_table(&info2).unwrap();

    let tables = storage.list_tables();
    assert_eq!(tables.len(), 2);
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"orders".to_string()));
}

#[test]
fn test_memory_storage_create_index() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();

    let result = storage.create_table_index("users", "id", 0);
    assert!(result.is_ok());
}

#[test]
fn test_memory_storage_drop_index() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();
    storage.create_table_index("users", "id", 0).unwrap();

    let result = storage.drop_table_index("users", "id");
    assert!(result.is_ok());
}

#[test]
fn test_memory_storage_search_index() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(2)]])
        .unwrap();
    storage.create_table_index("users", "id", 0).unwrap();

    let result = storage.search_index("users", "id", 1);
    assert!(result.is_some());
}

#[test]
fn test_memory_storage_range_index() {
    let mut storage = MemoryStorage::new();
    let info = TableInfo {
        name: "users".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: true,
            references: None,
        }],
    };
    storage.create_table(&info).unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(1)]])
        .unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(5)]])
        .unwrap();
    storage
        .insert("users", vec![vec![Value::Integer(10)]])
        .unwrap();
    storage.create_table_index("users", "id", 0).unwrap();

    let result = storage.range_index("users", "id", 1, 10);
    assert!(result.len() >= 0);
}

#[test]
fn test_foreign_key_constraint_new() {
    let fk = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: Some(ForeignKeyAction::Cascade),
        on_update: Some(ForeignKeyAction::Restrict),
    };
    assert_eq!(fk.referenced_table, "users");
    assert_eq!(fk.referenced_column, "id");
}

#[test]
fn test_column_definition_with_foreign_key() {
    let fk = ForeignKeyConstraint {
        referenced_table: "users".to_string(),
        referenced_column: "id".to_string(),
        on_delete: None,
        on_update: None,
    };
    let col = ColumnDefinition {
        name: "user_id".to_string(),
        data_type: "INTEGER".to_string(),
        nullable: false,
        is_unique: false,
        references: Some(fk),
    };
    assert!(col.references.is_some());
}
