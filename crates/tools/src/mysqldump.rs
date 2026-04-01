//! MySQL Dump Import Tool
//!
//! Provides functionality to import SQL dump files in mysqldump format.
//! Supports:
//! - CREATE TABLE statements
//! - INSERT INTO statements
//! - SET statements (key=value pairs)
//! - DROP TABLE statements
//! - USE database statements

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub tables_created: usize,
    pub tables_dropped: usize,
    pub rows_inserted: usize,
    pub queries_executed: usize,
    pub errors: usize,
    pub warnings: Vec<String>,
}

impl Default for ImportStats {
    fn default() -> Self {
        Self {
            tables_created: 0,
            tables_dropped: 0,
            rows_inserted: 0,
            queries_executed: 0,
            errors: 0,
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SqlStatement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
    },
    DropTable {
        name: String,
        if_exists: bool,
    },
    Use {
        database: String,
    },
    Set {
        key: String,
        value: String,
    },
    LockTables {
        tables: Vec<String>,
    },
    UnlockTables,
    Begin,
    Commit,
    Rollback,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
    pub unique: bool,
    pub default: Option<String>,
    pub references: Option<ForeignKeyRef>,
}

#[derive(Debug, Clone)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone)]
pub enum ImportMode {
    Full,
    SchemaOnly,
    DataOnly,
    ContinueOnError,
}

pub struct DumpImporter {
    current_database: Option<String>,
    statements: Vec<SqlStatement>,
    stats: ImportStats,
    mode: ImportMode,
    verbose: bool,
}

impl DumpImporter {
    pub fn new(mode: ImportMode, verbose: bool) -> Self {
        Self {
            current_database: None,
            statements: Vec::new(),
            stats: ImportStats::default(),
            mode,
            verbose,
        }
    }

    pub fn stats(&self) -> &ImportStats {
        &self.stats
    }

    pub fn statements(&self) -> &[SqlStatement] {
        &self.statements
    }

    pub fn current_database(&self) -> &Option<String> {
        &self.current_database
    }

    pub fn import_file(&mut self, path: &Path) -> Result<ImportStats> {
        let file = File::open(path).context(format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);

        self.import_reader(reader)?;
        Ok(self.stats.clone())
    }

    pub fn import_reader<R: BufRead>(&mut self, reader: R) -> Result<()> {
        let mut current_statement = String::new();
        let mut in_string = false;
        let mut string_char = ' ';

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context(format!("Failed to read line {}", line_num + 1))?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("/*") {
                continue;
            }

            for (i, c) in line.chars().enumerate() {
                if !in_string && (c == '\'' || c == '"') {
                    in_string = true;
                    string_char = c;
                } else if in_string && c == string_char {
                    if i + 1 < line.len() && line.chars().nth(i + 1) == Some(string_char) {
                        current_statement.push(c);
                        current_statement.push(c);
                    } else {
                        in_string = false;
                    }
                } else {
                    current_statement.push(c);
                }
            }

            if current_statement.ends_with(';') && !in_string {
                let stmt = current_statement.trim().trim_end_matches(';');
                if !stmt.is_empty() {
                    if let Err(e) = self.execute_statement(stmt) {
                        self.stats.errors += 1;
                        eprintln!("Error executing statement: {}", e);
                        if matches!(self.mode, ImportMode::ContinueOnError) {
                            self.stats.warnings.push(format!(
                                "Line {}: Error executing: {}",
                                line_num + 1,
                                e
                            ));
                        } else {
                            anyhow::bail!(
                                "Failed to execute statement at line {}: {}",
                                line_num + 1,
                                e
                            );
                        }
                    }
                }
                current_statement.clear();
            }
        }

        if !current_statement.trim().is_empty() && !current_statement.trim().ends_with(';') {
            if let Err(e) = self.execute_statement(current_statement.trim()) {
                self.stats.errors += 1;
                eprintln!("Warning: Final statement missing semicolon: {}", e);
            }
        }

        Ok(())
    }

    fn execute_statement(&mut self, stmt: &str) -> Result<()> {
        self.stats.queries_executed += 1;

        if self.verbose {
            let preview = if stmt.len() > 100 {
                format!("{}...", &stmt[..100])
            } else {
                stmt.to_string()
            };
            println!("Executing: {}", preview);
        }

        let stmt_upper = stmt.to_uppercase();

        if stmt_upper.starts_with("CREATE TABLE") {
            self.execute_create_table(stmt)?;
        } else if stmt_upper.starts_with("INSERT") {
            self.execute_insert(stmt)?;
        } else if stmt_upper.starts_with("DROP TABLE") {
            self.parse_drop_table(stmt)?;
        } else if stmt_upper.starts_with("USE ") {
            self.parse_use(stmt)?;
        } else if stmt_upper.starts_with("SET ") {
            self.parse_set(stmt)?;
        } else if stmt_upper.starts_with("LOCK TABLES") {
            self.parse_lock_tables(stmt)?;
        } else if stmt_upper.starts_with("UNLOCK TABLES") {
            self.statements.push(SqlStatement::UnlockTables);
        } else if stmt_upper == "BEGIN" || stmt_upper == "BEGIN WORK" {
            self.statements.push(SqlStatement::Begin);
        } else if stmt_upper == "COMMIT" || stmt_upper == "COMMIT WORK" {
            self.statements.push(SqlStatement::Commit);
        } else if stmt_upper == "ROLLBACK" || stmt_upper == "ROLLBACK WORK" {
            self.statements.push(SqlStatement::Rollback);
        } else {
            self.statements
                .push(SqlStatement::Unknown(stmt.to_string()));
        }

        Ok(())
    }

    fn execute_create_table(&mut self, stmt: &str) -> Result<()> {
        let table_re = Regex::new(r"CREATE TABLE `?(\w+)`?\s*\(").unwrap();
        if let Some(caps) = table_re.captures(stmt) {
            let table_name = caps.get(1).unwrap().as_str().to_string();
            let columns = self.parse_column_definitions(stmt)?;

            self.statements.push(SqlStatement::CreateTable {
                name: table_name.clone(),
                columns: columns.clone(),
            });
            self.stats.tables_created += 1;

            if self.verbose {
                println!(
                    "  Created table: {} ({} columns)",
                    table_name,
                    columns.len()
                );
            }
        }
        Ok(())
    }

    fn parse_column_definitions(&self, stmt: &str) -> Result<Vec<ColumnDef>> {
        let mut columns = Vec::new();

        let col_re = Regex::new(r"(\w+)\s+([A-Za-z0-9_()]+(?:\s*\([^)]+\))?)").unwrap();

        let after_create = stmt.find('(').map(|pos| &stmt[pos + 1..]).unwrap_or("");
        let before_ending = after_create
            .rfind(')')
            .map(|pos| &after_create[..pos])
            .unwrap_or(after_create);

        let col_strs: Vec<&str> = before_ending.split(',').collect();

        for col_str in col_strs {
            let col_str = col_str.trim();
            if col_str.is_empty() {
                continue;
            }

            if let Some(cap) = col_re.captures(col_str) {
                let name = cap.get(1).unwrap().as_str().to_string();
                let data_type_full = cap.get(2).unwrap().as_str().to_uppercase();

                let rest = col_str[cap.get(0).unwrap().len()..].to_uppercase();
                let nullable = !rest.contains("NOT NULL");
                let primary_key = rest.contains("PRIMARY KEY");
                let auto_increment =
                    rest.contains("AUTO_INCREMENT") || rest.contains("AUTOINCREMENT");
                let unique = rest.contains("UNIQUE");

                columns.push(ColumnDef {
                    name,
                    data_type: data_type_full,
                    nullable,
                    primary_key,
                    auto_increment,
                    unique,
                    default: None,
                    references: None,
                });
            }
        }

        Ok(columns)
    }

    fn execute_insert(&mut self, stmt: &str) -> Result<()> {
        let insert_re =
            Regex::new(r"INSERT INTO `?(\w+)`?\s*(?:\(([^)]+)\))?\s*VALUES\s*(.+)").unwrap();

        if let Some(caps) = insert_re.captures(stmt) {
            let table_name = caps.get(1).unwrap().as_str().to_string();
            let columns_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let values_str = caps.get(3).unwrap().as_str();

            let columns: Vec<String> = if columns_str.is_empty() {
                vec![]
            } else {
                columns_str
                    .split(',')
                    .map(|s| s.trim().trim_matches('`').to_string())
                    .collect()
            };

            let row_values = self.parse_multi_row_values(values_str)?;

            self.statements.push(SqlStatement::Insert {
                table: table_name.clone(),
                columns: columns.clone(),
                values: row_values.clone(),
            });
            self.stats.rows_inserted += row_values.len();

            if self.verbose {
                println!("  Inserted {} rows into: {}", row_values.len(), table_name);
            }
        }
        Ok(())
    }

    fn parse_multi_row_values(&self, values_str: &str) -> Result<Vec<Vec<String>>> {
        let mut rows = Vec::new();
        let mut current_row = String::new();
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        for c in values_str.chars() {
            match c {
                '\'' | '"' if !in_string => {
                    in_string = true;
                    string_char = c;
                    current_row.push(c);
                }
                '\'' | '"' if in_string => {
                    if c == string_char {
                        in_string = false;
                    }
                    current_row.push(c);
                }
                '(' if !in_string => {
                    paren_depth += 1;
                    if paren_depth == 1 {
                        current_row.clear();
                    }
                    current_row.push(c);
                }
                ')' if !in_string => {
                    paren_depth -= 1;
                    if paren_depth > 0 {
                        current_row.push(c);
                    }
                    if paren_depth == 0 {
                        let row = self.parse_row_values(&current_row)?;
                        if !row.is_empty() {
                            rows.push(row);
                        }
                    }
                }
                ',' if !in_string && paren_depth == 0 => {
                    continue;
                }
                _ => {
                    if paren_depth > 0 || in_string {
                        current_row.push(c);
                    }
                }
            }
        }

        Ok(rows)
    }

    fn parse_row_values(&self, row: &str) -> Result<Vec<String>> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut chars = row.chars().peekable();

        while let Some(c) = chars.next() {
            if !in_string && (c == '\'' || c == '"') {
                in_string = true;
                string_char = c;
                current.push(c);
            } else if in_string && c == string_char {
                let next = chars.peek();
                if next == Some(&string_char) {
                    current.push(string_char);
                    chars.next();
                } else {
                    in_string = false;
                    current.push(c);
                }
            } else if !in_string && c == ',' {
                let val = current.trim().to_string();
                if !val.is_empty() {
                    values.push(val);
                }
                current.clear();
            } else {
                current.push(c);
            }
        }

        let final_val = current.trim().to_string();
        if !final_val.is_empty() {
            values.push(final_val);
        }

        Ok(values)
    }

    fn parse_drop_table(&mut self, stmt: &str) -> Result<()> {
        let drop_re = Regex::new(r#"(?i)DROP TABLE\s+(?:IF EXISTS\s+)?[`"']?(\w+)[`"']?"#).unwrap();

        if let Some(caps) = drop_re.captures(stmt) {
            let table_name = caps.get(1).unwrap().as_str().to_string();
            let if_exists = stmt.to_uppercase().contains("IF EXISTS");

            self.statements.push(SqlStatement::DropTable {
                name: table_name.clone(),
                if_exists,
            });
            self.stats.tables_dropped += 1;
            if self.verbose {
                println!("  Dropped table: {}", table_name);
            }
        }
        Ok(())
    }

    fn parse_use(&mut self, stmt: &str) -> Result<()> {
        let use_re = Regex::new(r#"(?i)USE\s+[`"']?(\w+)[`"']?"#).unwrap();

        if let Some(caps) = use_re.captures(stmt) {
            let database = caps.get(1).unwrap().as_str().to_string();
            self.current_database = Some(database.clone());
            self.statements.push(SqlStatement::Use {
                database: database.clone(),
            });
            if self.verbose {
                println!("  Using database: {}", database);
            }
        }
        Ok(())
    }

    fn parse_set(&mut self, stmt: &str) -> Result<()> {
        let set_re = Regex::new(r#"(?i)SET\s+(?:@(\w+)\s*=\s*)?(.+)"#).unwrap();

        if let Some(caps) = set_re.captures(stmt) {
            let key = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let value = caps.get(2).unwrap().as_str().trim_matches(';').to_string();

            self.statements.push(SqlStatement::Set {
                key: key.clone(),
                value: value.clone(),
            });

            if self.verbose {
                println!(
                    "  SET {} = {}",
                    if key.is_empty() { "@" } else { &key },
                    value
                );
            }
        }
        Ok(())
    }

    fn parse_lock_tables(&mut self, stmt: &str) -> Result<()> {
        let lock_type_keywords = ["READ", "WRITE", "LOCAL"];
        let words: Vec<&str> = stmt.split_whitespace().collect();
        let mut tables = Vec::new();

        let mut i = 2;
        while i < words.len() {
            let word = words[i].trim_matches(',').trim_matches(';').to_uppercase();
            if lock_type_keywords.contains(&word.as_str()) {
                break;
            }
            if !word.is_empty() {
                tables.push(words[i].trim_matches(',').trim_matches(';').to_string());
            }
            i += 1;
        }

        self.statements.push(SqlStatement::LockTables { tables });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_create_table() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let columns = importer
            .parse_column_definitions(
                "id INT NOT NULL PRIMARY KEY, name VARCHAR(255), email VARCHAR(255) UNIQUE",
            )
            .unwrap();
        assert!(!columns.is_empty(), "Expected at least 1 column");
    }

    #[test]
    fn test_parse_insert() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer.parse_row_values("1, 2, 3").unwrap();
        assert_eq!(values.len(), 3, "Expected 3 values but got {:?}", values);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "2");
        assert_eq!(values[2], "3");
    }

    #[test]
    fn test_parse_use_statement() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let mut importer = importer;
        importer.parse_use("USE mydb").unwrap();
        assert_eq!(importer.current_database, Some("mydb".to_string()));
    }

    #[test]
    fn test_import_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.tables_created, 0);
        assert_eq!(stats.rows_inserted, 0);
        assert_eq!(stats.errors, 0);
    }

    #[test]
    fn test_parse_multi_row_values() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values_str = "(1, 'Alice'), (2, 'Bob'), (3, 'Charlie')";
        let rows = importer.parse_multi_row_values(values_str).unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_parse_single_quoted_string_with_comma() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer.parse_row_values("1, 2, 3").unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "2");
        assert_eq!(values[2], "3");
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer
            .parse_row_values(r#"1, 'It''s a test', 3"#)
            .unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_parse_set_statement() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let mut importer = importer;
        importer.parse_set("SET FOREIGN_KEY_CHECKS = 0").unwrap();
        let set_stmt = &importer.statements[0];
        if let SqlStatement::Set { key, value } = set_stmt {
            assert_eq!(key, "");
            assert_eq!(value, "FOREIGN_KEY_CHECKS = 0");
        } else {
            panic!("Expected Set statement");
        }
    }

    #[test]
    fn test_parse_drop_table() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let mut importer = importer;
        importer.parse_drop_table("DROP TABLE users").unwrap();
        assert_eq!(importer.stats.tables_dropped, 1);
    }

    #[test]
    fn test_parse_drop_table_if_exists() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let mut importer = importer;
        importer
            .parse_drop_table("DROP TABLE IF EXISTS users")
            .unwrap();
        assert_eq!(importer.stats.tables_dropped, 1);
    }

    #[test]
    fn test_import_reader_single_insert() {
        let sql = "CREATE TABLE users (id INT);\nINSERT INTO users VALUES (1);";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
        assert_eq!(importer.stats.queries_executed, 2);
    }

    #[test]
    fn test_import_reader_multi_row_insert() {
        let sql = "CREATE TABLE users (id INT, name VARCHAR(255));\nINSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
        assert_eq!(importer.stats.rows_inserted, 2);
    }

    #[test]
    fn test_import_reader_with_comments() {
        let sql = "-- comment\nCREATE TABLE t (id INT); -- end\nINSERT INTO t VALUES (1);";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
    }

    #[test]
    fn test_import_reader_lock_unlock() {
        let sql = "LOCK TABLES t WRITE;\nUNLOCK TABLES;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 2);
    }

    #[test]
    fn test_import_reader_begin_commit() {
        let sql = "BEGIN;\nINSERT INTO t VALUES (1);\nCOMMIT;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 3);
    }

    #[test]
    fn test_import_stats_accumulation() {
        let sql =
            "CREATE TABLE t (id INT);\nINSERT INTO t VALUES (1);\nINSERT INTO t VALUES (2), (3);";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
        assert_eq!(importer.stats.rows_inserted, 3);
        assert_eq!(importer.stats.queries_executed, 3);
    }

    #[test]
    fn test_mysqldump_format_comments_stripped() {
        let sql = "/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;\nCREATE TABLE t (id INT);";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
    }

    #[test]
    fn test_null_values_in_insert() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer.parse_row_values("1, NULL, 'test'").unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "NULL");
    }

    #[test]
    fn test_empty_string_value() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer.parse_row_values("1, 2, 3").unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_various_data_types() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let columns = importer
            .parse_column_definitions(
                "id INT, name VARCHAR(255), price DECIMAL(10,2), active BOOLEAN",
            )
            .unwrap();
        assert!(!columns.is_empty());
    }

    #[test]
    fn test_parse_column_definitions_all_types() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let columns = importer
            .parse_column_definitions(
                "id INT PRIMARY KEY AUTO_INCREMENT,
                 name VARCHAR(255) NOT NULL,
                 age INT UNIQUE,
                 email VARCHAR(100) UNIQUE NOT NULL,
                 created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                 balance DECIMAL(10,2) DEFAULT 0.00,
                 active BOOLEAN DEFAULT TRUE,
                 bio TEXT,
                 data BLOB",
            )
            .unwrap();
        assert!(!columns.is_empty());
    }

    #[test]
    fn test_parse_multi_row_values_single_row() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values_str = "(1, 'Alice')";
        let rows = importer.parse_multi_row_values(values_str).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    }

    #[test]
    fn test_parse_multi_row_values_empty() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values_str = "";
        let rows = importer.parse_multi_row_values(values_str).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_row_values_with_quotes() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer
            .parse_row_values(r#"'hello', 'world', 123"#)
            .unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_parse_row_values_mixed_types() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer
            .parse_row_values(r#"1, 'string', 3.14, NULL, 'another string'"#)
            .unwrap();
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_parse_row_values_numeric_only() {
        let importer = DumpImporter::new(ImportMode::Full, false);
        let values = importer.parse_row_values("1, 2, 3, 4, 5").unwrap();
        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_import_reader_transactions() {
        let sql = "BEGIN;\nINSERT INTO t VALUES (1);\nCOMMIT;\nBEGIN;\nINSERT INTO t VALUES (2);\nROLLBACK;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 6);
    }

    #[test]
    fn test_import_reader_multiple_drops() {
        let sql = "DROP TABLE IF EXISTS t1;\nDROP TABLE t2;\nDROP TABLE IF EXISTS t3;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_dropped, 3);
    }

    #[test]
    fn test_import_reader_multiple_uses() {
        let sql = "USE db1;\nUSE db2;\nUSE db3;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.current_database, Some("db3".to_string()));
    }

    #[test]
    fn test_import_reader_set_variables() {
        let sql = "SET FOREIGN_KEY_CHECKS = 0;\nSET UNIQUE_CHECKS = 0;\nSET SESSION sql_mode = 'STRICT_TRANS_TABLES';";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 3);
    }

    #[test]
    fn test_import_stats_update() {
        let sql = "CREATE TABLE t1 (id INT);\nCREATE TABLE t2 (id INT);\nDROP TABLE t1;\nINSERT INTO t2 VALUES (1), (2), (3);";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();

        assert_eq!(importer.stats.tables_created, 2);
        assert_eq!(importer.stats.tables_dropped, 1);
        assert_eq!(importer.stats.rows_inserted, 3);
        assert_eq!(importer.stats.queries_executed, 4);
        assert_eq!(importer.stats.errors, 0);
    }

    #[test]
    fn test_parse_lock_tables_write() {
        let sql = "LOCK TABLES t1 WRITE, t2 READ;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 1);
    }

    #[test]
    fn test_skip_empty_lines() {
        let sql = "\n\nCREATE TABLE t (id INT);\n\n\nINSERT INTO t VALUES (1);\n\n";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.tables_created, 1);
        assert_eq!(importer.stats.rows_inserted, 1);
    }

    #[test]
    fn test_sql_statement_begin() {
        let sql = "BEGIN;\n";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert!(matches!(importer.statements[0], SqlStatement::Begin));
    }

    #[test]
    fn test_sql_statement_commit() {
        let sql = "COMMIT;\n";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert!(matches!(importer.statements[0], SqlStatement::Commit));
    }

    #[test]
    fn test_sql_statement_rollback() {
        let sql = "ROLLBACK;\n";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert!(matches!(importer.statements[0], SqlStatement::Rollback));
    }

    #[test]
    fn test_sql_statement_unlock() {
        let sql = "UNLOCK TABLES;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert!(matches!(importer.statements[0], SqlStatement::UnlockTables));
    }

    #[test]
    fn test_sql_statement_lock_tables() {
        let sql = "LOCK TABLES t READ;\n";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        match &importer.statements[0] {
            SqlStatement::LockTables { tables } => {
                assert_eq!(tables.len(), 1);
                assert_eq!(tables[0], "t");
            }
            _ => panic!("Expected LockTables statement"),
        }
    }

    #[test]
    fn test_create_table_statistics_realistic() {
        let sql = r#"
            CREATE TABLE orders (
                id INT PRIMARY KEY AUTO_INCREMENT,
                customer_id INT NOT NULL,
                product_id INT NOT NULL,
                quantity INT DEFAULT 1,
                price DECIMAL(10,2) NOT NULL,
                status VARCHAR(20) DEFAULT 'pending',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            );
            
            INSERT INTO orders (customer_id, product_id, quantity, price, status) VALUES
                (1, 101, 2, 29.99, 'completed'),
                (1, 102, 1, 49.99, 'completed'),
                (2, 101, 5, 29.99, 'pending'),
                (3, 103, 1, 99.99, 'shipped');
        "#;
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();

        assert_eq!(importer.stats.tables_created, 1);
        assert_eq!(importer.stats.rows_inserted, 4);
        assert_eq!(importer.stats.queries_executed, 2);
    }

    #[test]
    fn test_import_reader_unknown_statement() {
        let sql = "SHOW TABLES;";
        let reader = Cursor::new(sql);
        let mut importer = DumpImporter::new(ImportMode::Full, false);
        importer.import_reader(reader).unwrap();
        assert_eq!(importer.stats.queries_executed, 1);
        assert!(matches!(importer.statements[0], SqlStatement::Unknown(_)));
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "import", about = "Import mysqldump format SQL files")]
pub enum ImportCommand {
    File {
        #[structopt(short = "f", long = "file")]
        file: std::path::PathBuf,

        #[structopt(short = "m", long = "mode", default_value = "full")]
        mode: String,

        #[structopt(short = "c", long = "continue-on-error")]
        continue_on_error: bool,

        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
}

pub fn run_import(cmd: ImportCommand) -> Result<()> {
    match cmd {
        ImportCommand::File {
            file,
            mode: _,
            continue_on_error: _,
            verbose,
        } => {
            let mut importer = DumpImporter::new(ImportMode::Full, verbose);

            println!("Importing from: {}", file.display());
            println!("Mode: Full");

            let stats = importer.import_file(&file)?;

            println!("\n=== Import Summary ===");
            println!("Tables created: {}", stats.tables_created);
            println!("Tables dropped: {}", stats.tables_dropped);
            println!("Rows inserted: {}", stats.rows_inserted);
            println!("Queries executed: {}", stats.queries_executed);
            println!("Errors: {}", stats.errors);

            if !stats.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &stats.warnings {
                    println!("  - {}", warning);
                }
            }

            if stats.errors > 0 {
                anyhow::bail!("Import completed with {} errors", stats.errors);
            }

            println!("\nImport completed successfully!");
            Ok(())
        }
    }
}
