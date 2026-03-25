//! Data Backup and Export Module
//!
//! Supports exporting data in CSV, JSON, and SQL formats

use crate::{Record, StorageEngine, TableInfo};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackupFormat {
    Csv,
    Json,
    Sql,
}

impl BackupFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(BackupFormat::Csv),
            "json" => Some(BackupFormat::Json),
            "sql" => Some(BackupFormat::Sql),
            _ => None,
        }
    }
}

pub struct BackupExporter;

impl BackupExporter {
    pub fn export_table(
        storage: &dyn StorageEngine,
        table: &str,
        path: &Path,
        format: BackupFormat,
    ) -> SqlResult<usize> {
        let table_info = storage.get_table_info(table)?;
        let rows = storage.scan(table)?;

        let file =
            File::create(path).map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        let count = match format {
            BackupFormat::Csv => Self::export_csv(&table_info, &rows, &mut writer),
            BackupFormat::Json => Self::export_json(&table_info, &rows, &mut writer),
            BackupFormat::Sql => Self::export_sql(&table_info, &rows, &mut writer),
        }?;

        writer
            .flush()
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        Ok(count)
    }

    fn export_csv(
        info: &TableInfo,
        rows: &[Record],
        writer: &mut BufWriter<File>,
    ) -> SqlResult<usize> {
        let headers: Vec<&str> = info.columns.iter().map(|c| c.name.as_str()).collect();
        writeln!(writer, "{}", headers.join(","))
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        for row in rows {
            let values: Vec<String> = row.iter().map(|v| Self::csv_escape(v)).collect();
            writeln!(writer, "{}", values.join(","))
                .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
        }

        Ok(rows.len())
    }

    fn csv_escape(value: &Value) -> String {
        match value {
            Value::Null => "".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => {
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.clone()
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::Blob(b) => format!("[BLOB: {} bytes]", b.len()),
            Value::Date(d) => d.to_string(),
            Value::Timestamp(t) => t.to_string(),
        }
    }

    fn export_json(
        info: &TableInfo,
        rows: &[Record],
        writer: &mut BufWriter<File>,
    ) -> SqlResult<usize> {
        writeln!(writer, "{{").map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
        writeln!(writer, "  \"table\": \"{}\",", info.name)
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        let headers: Vec<&str> = info.columns.iter().map(|c| c.name.as_str()).collect();
        writeln!(writer, "  \"columns\": {:?},", headers)
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        writeln!(writer, "  \"rows\": [")
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        for (i, row) in rows.iter().enumerate() {
            let obj: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(j, v)| {
                    format!(
                        "    \"{}\": {}",
                        headers.get(j).unwrap_or(&""),
                        Self::json_value(v)
                    )
                })
                .collect();

            if i == rows.len() - 1 {
                writeln!(writer, "    {{{}}}", obj.join(", "))
                    .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
            } else {
                writeln!(writer, "    {{{}}}", obj.join(", "))
                    .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
            }
        }

        writeln!(writer, "  ]").map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
        writeln!(writer, "}}").map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        Ok(rows.len())
    }

    fn json_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Boolean(b) => b.to_string(),
            Value::Blob(b) => format!("\"[BLOB: {} bytes]\"", b.len()),
            Value::Date(d) => format!("\"{}\"", d),
            Value::Timestamp(t) => format!("\"{}\"", t),
        }
    }

    fn export_sql(
        info: &TableInfo,
        rows: &[Record],
        writer: &mut BufWriter<File>,
    ) -> SqlResult<usize> {
        let table_name = &info.name;

        for row in rows {
            let values: Vec<String> = row.iter().map(|v| Self::sql_value(v)).collect();
            writeln!(
                writer,
                "INSERT INTO {} ({}) VALUES ({});",
                table_name,
                info.columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                values.join(", ")
            )
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;
        }

        Ok(rows.len())
    }

    fn sql_value(value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
            Value::Boolean(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            Value::Blob(b) => format!("X'{}'", use_hex::encode(b)),
            Value::Date(d) => format!("'{}'", d),
            Value::Timestamp(t) => format!("'{}'", t),
        }
    }
}

mod use_hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

pub struct DataRestorer;

impl DataRestorer {
    pub fn restore_from_backup(
        storage: &mut dyn StorageEngine,
        path: &Path,
        format: BackupFormat,
    ) -> SqlResult<usize> {
        let content = std::fs::read_to_string(path)
            .map_err(|e: std::io::Error| SqlError::IoError(e.to_string()))?;

        match format {
            BackupFormat::Csv => Self::restore_csv(storage, &content),
            BackupFormat::Json => Self::restore_json(storage, &content),
            BackupFormat::Sql => Self::restore_sql(storage, &content),
        }
    }

    fn restore_csv(storage: &mut dyn StorageEngine, content: &str) -> SqlResult<usize> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(0);
        }

        let headers: Vec<&str> = lines[0].split(',').collect();
        let mut rows = Vec::new();

        for line in &lines[1..] {
            let values: Vec<Value> = line.split(',').map(|s| Self::parse_csv_value(s)).collect();
            if !values.is_empty() {
                rows.push(values);
            }
        }

        let table_info = TableInfo {
            name: "restored".to_string(),
            columns: headers
                .iter()
                .map(|name| crate::ColumnDefinition {
                    name: name.to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                })
                .collect(),
        };

        let count = rows.len();
        storage.create_table(&table_info)?;
        storage.insert("restored", rows)?;

        Ok(count)
    }

    fn parse_csv_value(s: &str) -> Value {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Value::Null;
        }
        if let Ok(i) = trimmed.parse::<i64>() {
            return Value::Integer(i);
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return Value::Float(f);
        }
        if trimmed == "true" || trimmed == "false" {
            return Value::Boolean(trimmed == "true");
        }
        Value::Text(trimmed.to_string())
    }

    fn restore_json(storage: &mut dyn StorageEngine, content: &str) -> SqlResult<usize> {
        let data: serde_json::Value =
            serde_json::from_str(content).map_err(|e| SqlError::ParseError(e.to_string()))?;

        let table_name = data["table"].as_str().unwrap_or("restored");
        let columns: Vec<String> = data["columns"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let rows_array = data["rows"].as_array();

        let mut rows = Vec::new();
        if let Some(arr) = rows_array {
            for row_obj in arr {
                let mut row = Vec::new();
                for col in &columns {
                    let val = &row_obj[col];
                    row.push(Self::json_to_value(val));
                }
                rows.push(row);
            }
        }

        let table_info = TableInfo {
            name: table_name.to_string(),
            columns: columns
                .iter()
                .map(|name| crate::ColumnDefinition {
                    name: name.clone(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                })
                .collect(),
        };

        let count = rows.len();
        storage.create_table(&table_info)?;
        storage.insert(table_name, rows)?;

        Ok(count)
    }

    fn json_to_value(v: &serde_json::Value) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::Text(s.clone()),
            _ => Value::Null,
        }
    }

    fn restore_sql(storage: &mut dyn StorageEngine, content: &str) -> SqlResult<usize> {
        let mut total_rows = 0;

        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("INSERT INTO") {
                continue;
            }

            if let Some((table_name, values_part)) = Self::parse_insert(line) {
                let rows = Self::parse_insert_values(&values_part);

                if !storage.has_table(&table_name) {
                    let table_info = TableInfo {
                        name: table_name.clone(),
                        columns: vec![],
                    };
                    storage.create_table(&table_info)?;
                }

                storage.insert(&table_name, rows)?;
                total_rows += 1;
            }
        }

        Ok(total_rows)
    }

    fn parse_insert(line: &str) -> Option<(String, &str)> {
        let line = line.trim_end_matches(';').trim();
        if !line.starts_with("INSERT INTO") {
            return None;
        }

        let rest = &line[12..];
        let paren_pos = rest.find('(')?;
        let table_name = rest[..paren_pos].trim().trim_matches('`').to_string();
        let values_pos = rest[paren_pos..].find("VALUES")? + paren_pos;

        Some((table_name, &rest[values_pos..]))
    }

    fn parse_insert_values(values_part: &str) -> Vec<Record> {
        let mut rows = Vec::new();

        let values_str = values_part.trim();
        let value_groups: Vec<&str> = values_str.split("),").collect();

        for group in value_groups {
            let group = group.trim().trim_matches('(').trim_matches(')');
            let values: Vec<Value> = group
                .split(',')
                .map(|s| Self::parse_sql_value(s.trim()))
                .collect();
            rows.push(values);
        }

        rows
    }

    fn parse_sql_value(s: &str) -> Value {
        let s = s.trim();
        if s.eq_ignore_ascii_case("NULL") {
            return Value::Null;
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            let inner = &s[1..s.len() - 1];
            return Value::Text(inner.replace("''", "'"));
        }
        if s.starts_with("X'") && s.ends_with('\'') {
            return Value::Blob(vec![]);
        }
        if let Ok(i) = s.parse::<i64>() {
            return Value::Integer(i);
        }
        if let Ok(f) = s.parse::<f64>() {
            return Value::Float(f);
        }
        Value::Text(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStorage;
    use std::env::temp_dir;

    #[test]
    fn test_backup_format_from_str() {
        assert_eq!(BackupFormat::from_str("csv"), Some(BackupFormat::Csv));
        assert_eq!(BackupFormat::from_str("json"), Some(BackupFormat::Json));
        assert_eq!(BackupFormat::from_str("sql"), Some(BackupFormat::Sql));
        assert_eq!(BackupFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_export_csv() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                crate::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                crate::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                },
            ],
        };
        storage.create_table(&info).unwrap();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let path = temp_dir().join("test_backup.csv");
        let count =
            BackupExporter::export_table(&storage, "users", &path, BackupFormat::Csv).unwrap();
        assert_eq!(count, 2);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("id,name"));
        assert!(content.contains("1,Alice"));
        assert!(content.contains("2,Bob"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_sql() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                crate::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                },
                crate::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                },
            ],
        };
        storage.create_table(&info).unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .unwrap();

        let path = temp_dir().join("test_backup.sql");
        let count =
            BackupExporter::export_table(&storage, "users", &path, BackupFormat::Sql).unwrap();
        assert_eq!(count, 1);

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("INSERT INTO users"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(BackupExporter::csv_escape(&Value::Integer(42)), "42");
        assert_eq!(
            BackupExporter::csv_escape(&Value::Text("hello".to_string())),
            "hello"
        );
        assert_eq!(
            BackupExporter::csv_escape(&Value::Text("a,b".to_string())),
            "\"a,b\""
        );
    }

    #[test]
    fn test_sql_value() {
        assert_eq!(BackupExporter::sql_value(&Value::Integer(42)), "42");
        assert_eq!(
            BackupExporter::sql_value(&Value::Text("O'Reilly".to_string())),
            "'O''Reilly'"
        );
        assert_eq!(BackupExporter::sql_value(&Value::Null), "NULL");
    }
}
