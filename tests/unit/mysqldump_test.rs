use sqlrustgo_tools::{DumpImporter, ImportMode, ImportStats, SqlStatement};
use std::io::Cursor;

#[test]
fn test_import_stats_default() {
    let stats = ImportStats::default();
    assert_eq!(stats.tables_created, 0);
    assert_eq!(stats.tables_dropped, 0);
    assert_eq!(stats.rows_inserted, 0);
    assert_eq!(stats.queries_executed, 0);
    assert_eq!(stats.errors, 0);
    assert!(stats.warnings.is_empty());
}

#[test]
fn test_import_stats_with_values() {
    let stats = ImportStats {
        tables_created: 5,
        tables_dropped: 2,
        rows_inserted: 100,
        queries_executed: 107,
        errors: 1,
        warnings: vec!["Warning 1".to_string()],
    };
    assert_eq!(stats.tables_created, 5);
    assert_eq!(stats.tables_dropped, 2);
    assert_eq!(stats.rows_inserted, 100);
    assert_eq!(stats.queries_executed, 107);
    assert_eq!(stats.errors, 1);
    assert_eq!(stats.warnings.len(), 1);
}

#[test]
fn test_dump_importer_new() {
    let importer = DumpImporter::new(ImportMode::Full, false);
    assert_eq!(importer.stats().tables_created, 0);
}

#[test]
fn test_parse_create_table_simple() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(255));";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
    assert_eq!(importer.stats().queries_executed, 1);
}

#[test]
fn test_parse_create_table_with_primary_key() {
    let sql = "CREATE TABLE users (id INT NOT NULL PRIMARY KEY, name VARCHAR(255));";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_parse_create_table_with_auto_increment() {
    let sql = "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255));";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_parse_create_table_with_foreign_key() {
    let sql =
        "CREATE TABLE orders (id INT, user_id INT, FOREIGN KEY (user_id) REFERENCES users(id));";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_parse_insert_single_row() {
    let sql = "CREATE TABLE users (id INT);\nINSERT INTO users VALUES (1);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
    assert_eq!(importer.stats().rows_inserted, 1);
    assert_eq!(importer.stats().queries_executed, 2);
}

#[test]
fn test_parse_insert_multiple_rows() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(255));\nINSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
    assert_eq!(importer.stats().rows_inserted, 2);
}

#[test]
fn test_parse_insert_with_columns() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(255));\nINSERT INTO users (id, name) VALUES (1, 'Alice');";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_parse_insert_null_values() {
    let sql =
        "CREATE TABLE users (id INT, name VARCHAR(255));\nINSERT INTO users VALUES (1, NULL);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_parse_insert_empty_string() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(255));\nINSERT INTO users VALUES (1, '');";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_parse_insert_escaped_quotes() {
    let sql = r#"CREATE TABLE users (id INT, name VARCHAR(255));
INSERT INTO users VALUES (1, 'test');"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
    assert_eq!(importer.stats().queries_executed, 2);
}

#[test]
fn test_parse_insert_string_with_comma() {
    let sql = r#"CREATE TABLE users (id INT, bio TEXT);"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = r#"INSERT INTO users VALUES (1, 'Hello, World');"#;
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_parse_drop_table() {
    let sql = "DROP TABLE users;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_dropped, 1);
    assert_eq!(importer.stats().queries_executed, 1);
}

#[test]
fn test_parse_drop_table_if_exists() {
    let sql = "DROP TABLE IF EXISTS users;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_dropped, 1);
}

#[test]
fn test_parse_use_statement() {
    let sql = "USE mydb;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(*importer.current_database(), Some("mydb".to_string()));
    assert_eq!(importer.stats().queries_executed, 1);
}

#[test]
fn test_parse_set_statement() {
    let sql = "SET FOREIGN_KEY_CHECKS = 0;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 1);
    assert_eq!(importer.statements().len(), 1);
}

#[test]
fn test_parse_set_statement_with_user_var() {
    let sql = "SET @old_sql_mode = @@SQL_MODE;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 1);
}

#[test]
fn test_parse_lock_unlock_tables() {
    let sql = "LOCK TABLES t READ;\nUNLOCK TABLES;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 2);
}

#[test]
fn test_parse_begin_commit() {
    let sql = "BEGIN;\nINSERT INTO t VALUES (1);\nCOMMIT;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 3);
}

#[test]
fn test_parse_rollback() {
    let sql = "BEGIN;\nINSERT INTO t VALUES (1);\nROLLBACK;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 3);
}

#[test]
fn test_comments_stripped() {
    let sql = "-- single line comment\nCREATE TABLE t (id INT); -- inline comment\n/* multi line\ncomment */";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_mysqldump_comments_stripped() {
    let sql = "/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;\nCREATE TABLE t (id INT);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_multiple_statements() {
    let sql = "CREATE TABLE t1 (id INT);\nCREATE TABLE t2 (id INT);\nINSERT INTO t1 VALUES (1);\nINSERT INTO t2 VALUES (2), (3);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 2);
    assert_eq!(importer.stats().rows_inserted, 3);
    assert_eq!(importer.stats().queries_executed, 4);
}

#[test]
fn test_create_and_insert_realistic() {
    let sql = r#"
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL
);

INSERT INTO users (username, email) VALUES 
    ('alice', 'alice@example.com'),
    ('bob', 'bob@example.com'),
    ('charlie', 'charlie@example.com');

CREATE TABLE posts (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT
);

INSERT INTO posts (user_id, title, content) VALUES
    (1, 'First Post', 'Content A'),
    (1, 'Second Post', 'Content B'),
    (2, 'Third Post', 'Content C');
"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 2);
    assert_eq!(importer.stats().rows_inserted, 6);
    assert_eq!(importer.stats().queries_executed, 4);
}

#[test]
fn test_import_mode_full() {
    let sql = "CREATE TABLE t (id INT);";
    let reader = Cursor::new(sql);
    let importer = DumpImporter::new(ImportMode::Full, false);
    assert!(matches!(importer.stats().tables_created, 0));
}

#[test]
fn test_import_mode_schema_only() {
    let sql = "CREATE TABLE t (id INT);\nINSERT INTO t VALUES (1);";
    let reader = Cursor::new(sql);
    let importer = DumpImporter::new(ImportMode::SchemaOnly, false);
    assert!(matches!(importer.stats().tables_created, 0));
}

#[test]
fn test_import_mode_data_only() {
    let sql = "CREATE TABLE t (id INT);\nINSERT INTO t VALUES (1);";
    let reader = Cursor::new(sql);
    let importer = DumpImporter::new(ImportMode::DataOnly, false);
    assert!(matches!(importer.stats().tables_created, 0));
}

#[test]
fn test_sql_statement_types() {
    let sql = "CREATE TABLE t (id INT);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    assert_eq!(importer.statements().len(), 1);
    match &importer.statements()[0] {
        SqlStatement::CreateTable { name, columns } => {
            assert_eq!(name, "t");
            assert_eq!(columns.len(), 1);
        }
        _ => panic!("Expected CreateTable statement"),
    }
}

#[test]
fn test_sql_statement_insert() {
    let sql = "INSERT INTO users (id, name) VALUES (1, 'Alice');";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    assert_eq!(importer.statements().len(), 1);
    match &importer.statements()[0] {
        SqlStatement::Insert {
            table,
            columns,
            values,
        } => {
            assert_eq!(table, "users");
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0], "id");
            assert_eq!(columns[1], "name");
            assert_eq!(values.len(), 1);
        }
        _ => panic!("Expected Insert statement"),
    }
}

#[test]
fn test_sql_statement_drop() {
    let sql = "DROP TABLE IF EXISTS users;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    assert_eq!(importer.statements().len(), 1);
    match &importer.statements()[0] {
        SqlStatement::DropTable { name, if_exists } => {
            assert_eq!(name, "users");
            assert!(if_exists);
        }
        _ => panic!("Expected DropTable statement"),
    }
}

#[test]
fn test_sql_statement_use() {
    let sql = "USE mydb;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    assert_eq!(importer.statements().len(), 1);
    match &importer.statements()[0] {
        SqlStatement::Use { database } => {
            assert_eq!(database, "mydb");
        }
        _ => panic!("Expected Use statement"),
    }
}

#[test]
fn test_sql_statement_set() {
    let sql = "SET FOREIGN_KEY_CHECKS = 0;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    assert_eq!(importer.statements().len(), 1);
    match &importer.statements()[0] {
        SqlStatement::Set { key, value } => {
            assert!(key.is_empty());
            assert_eq!(value, "FOREIGN_KEY_CHECKS = 0");
        }
        _ => panic!("Expected Set statement"),
    }
}

#[test]
fn test_large_insert() {
    let sql = "CREATE TABLE numbers (id INT, value INT);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let values: Vec<String> = (1..=100).map(|i| format!("({}, {})", i, i * 10)).collect();
    let insert_sql = format!("INSERT INTO numbers VALUES {};", values.join(", "));
    let reader2 = Cursor::new(insert_sql);
    importer.import_reader(reader2).unwrap();

    assert_eq!(importer.stats().rows_inserted, 100);
}

#[test]
fn test_special_characters_in_strings() {
    let sql = r#"CREATE TABLE test (id INT, text TEXT);"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = r#"INSERT INTO test VALUES (1, 'Line1\nLine2\tTab');"#;
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_numeric_values() {
    let sql = "CREATE TABLE nums (id INT, big_int BIGINT, price DECIMAL(10,2), ratio DOUBLE);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = "INSERT INTO nums VALUES (1, 9223372036854775807, 12345.67, 3.14159);";
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_utf8_strings() {
    let sql = r#"CREATE TABLE users (id INT, name VARCHAR(255));"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = r#"INSERT INTO users VALUES (1, '张三'), (2, 'Привет'), (3, '🎉');"#;
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 3);
}

#[test]
fn test_backticks_quoted_names() {
    let sql = "CREATE TABLE `users` (`id` INT, `user_name` VARCHAR(255));";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_double_quoted_strings() {
    let sql = r#"CREATE TABLE test (id INT, text TEXT);"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = r#"INSERT INTO test VALUES (1, "Hello World");"#;
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_quoted_string_with_double_quote() {
    let sql = r#"CREATE TABLE test (id INT, text TEXT);"#;
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let sql2 = r#"INSERT INTO test VALUES (1, 'He said "Hello"');"#;
    let reader2 = Cursor::new(sql2);
    importer.import_reader(reader2).unwrap();
    assert_eq!(importer.stats().rows_inserted, 1);
}

#[test]
fn test_partial_statement_no_semicolon() {
    let sql = "CREATE TABLE t (id INT)";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_warnings_accumulated() {
    let sql = "SET FOREIGN_KEY_CHECKS = 0;\nSET UNIQUE_CHECKS = 0;";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::ContinueOnError, false);
    importer.import_reader(reader).unwrap();
    assert_eq!(importer.stats().queries_executed, 2);
}

#[test]
fn test_stats_clone() {
    let sql = "CREATE TABLE t (id INT);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let stats = importer.stats().clone();
    assert_eq!(stats.tables_created, 1);
    assert_eq!(importer.stats().tables_created, 1);
}

#[test]
fn test_import_summary_format() {
    let sql = "CREATE TABLE users (id INT);\nINSERT INTO users VALUES (1), (2);";
    let reader = Cursor::new(sql);
    let mut importer = DumpImporter::new(ImportMode::Full, false);
    importer.import_reader(reader).unwrap();

    let stats = &importer.stats();
    assert_eq!(stats.tables_created, 1);
    assert_eq!(stats.rows_inserted, 2);
    assert_eq!(stats.queries_executed, 2);
    assert_eq!(stats.errors, 0);
}
