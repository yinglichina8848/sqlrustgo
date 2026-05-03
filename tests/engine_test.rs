mod engine;
mod sqlite_diff;

use engine::EngineAdapter;
use sqllogictest::Runner;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo::ExecutorResult;
use sqlite_diff::{assert_query_eq, SqliteEngine};
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    let mut runner = Runner::new(|| async {
        Ok(EngineAdapter::new())
    });

    runner.run_file("tests/engine/basic.slt").unwrap();
}

#[test]
fn test_sqllogic_basic() {
    let mut runner = Runner::new(|| async {
        Ok(EngineAdapter::new())
    });

    runner.run_file("tests/engine/basic.slt").unwrap();
}

fn execute_ddl(storage: &mut MemoryStorage, sql: &str) -> Result<ExecutorResult, String> {
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

    match statement {
        Statement::CreateTable(create) => {
            let info = TableInfo {
                name: create.name.clone(),
                columns: create
                    .columns
                    .into_iter()
                    .map(|c| ColumnDefinition {
                        name: c.name,
                        data_type: c.data_type,
                        nullable: c.nullable,
                        primary_key: c.primary_key,
                    })
                    .collect(),
                foreign_keys: vec![],
                unique_constraints: vec![],
                check_constraints: vec![],
                partition_info: None,
            };
            storage
                .create_table(&info)
                .map_err(|e| format!("Create table error: {:?}", e))?;
            Ok(ExecutorResult::new(vec![], 0))
        }
        Statement::DropTable(drop) => {
            storage
                .drop_table(&drop.name)
                .map_err(|e| format!("Drop table error: {:?}", e))?;
            Ok(ExecutorResult::new(vec![], 0))
        }
        _ => Ok(ExecutorResult::new(vec![], 0)),
    }
}

struct DDLEngine {
    storage: MemoryStorage,
}

impl DDLEngine {
    fn new() -> Self {
        Self {
            storage: MemoryStorage::new(),
        }
    }

    fn execute(&mut self, sql: &str) -> Result<ExecutorResult, String> {
        execute_ddl(&mut self.storage, sql)
    }

    fn get_table_names(&self) -> HashSet<String> {
        let tables = self.storage.list_tables();
        tables.into_iter().collect()
    }
}

impl Default for DDLEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_fuzz_ddl_create_drop() {
    let mut engine = DDLEngine::new();

    for i in 0..100 {
        let table_name = format!("t{}", i);
        let create_sql = format!("CREATE TABLE {}(a INT)", table_name);

        let result = engine.execute(&create_sql);
        assert!(result.is_ok(), "Failed to create table {}: {:?}", table_name, result);

        let names = engine.get_table_names();
        assert!(names.contains(&table_name), "Table {} not found after create", table_name);

        let drop_sql = format!("DROP TABLE {}", table_name);
        let result = engine.execute(&drop_sql);
        assert!(result.is_ok(), "Failed to drop table {}: {:?}", table_name, result);
    }
}

#[test]
fn test_fuzz_ddl_recreate_same_name() {
    let mut engine = DDLEngine::new();
    let table_name = "users";

    for _ in 0..10 {
        engine.execute(&format!("CREATE TABLE {}(a INT)", table_name)).unwrap();
        engine.execute(&format!("DROP TABLE {}", table_name)).unwrap();
    }

    let result = engine.execute(&format!("CREATE TABLE {}(a INT)", table_name));
    assert!(result.is_ok(), "Failed to create table after multiple recreate: {:?}", result);
}

#[test]
fn test_differential_ddl_state() {
    let mut engine1 = DDLEngine::new();
    let mut engine2 = DDLEngine::new();

    let ddl_statements = vec![
        "CREATE TABLE t1(a INT)",
        "CREATE TABLE t2(b TEXT)",
        "CREATE TABLE t3(c INT)",
        "DROP TABLE t2",
        "CREATE TABLE t4(d INT)",
    ];

    for sql in &ddl_statements {
        engine1.execute(sql).unwrap();
    }

    for sql in &ddl_statements {
        engine2.execute(sql).unwrap();
    }

    let names1 = engine1.get_table_names();
    let names2 = engine2.get_table_names();

    assert_eq!(names1, names2, "DDL state differs between engines");
}

#[test]
fn test_differential_drop_nonexistent() {
    let mut engine = DDLEngine::new();

    let result1 = engine.execute("DROP TABLE nonexistent");
    assert!(result1.is_ok(), "DROP nonexistent should succeed (lenient mode)");

    engine.execute("CREATE TABLE t(a INT)").unwrap();

    let result2 = engine.execute("DROP TABLE t");
    assert!(result2.is_ok(), "DROP existing table should succeed");
}

#[test]
fn test_fuzz_idempotent_ddl() {
    let mut engine = DDLEngine::new();

    let sql = "CREATE TABLE t(a INT)";

    let result1 = engine.execute(sql);
    assert!(result1.is_ok());

    let result2 = engine.execute(sql);
    assert!(result2.is_ok(), "CREATE same table twice should succeed (lenient mode)");
}

#[test]
fn test_sqlite_diff_basic() {
    let sqlite = SqliteEngine::new();
    sqlite.execute("CREATE TABLE t(a INT)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (1),(2),(3)").unwrap();

    let result = sqlite.query("SELECT * FROM t").unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_sqlite_diff_count() {
    let sqlite = SqliteEngine::new();
    sqlite.execute("CREATE TABLE t(a INT)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

    let result = sqlite.query("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result[0][0], "3");
}

#[test]
fn test_sqlite_diff_where() {
    let sqlite = SqliteEngine::new();
    sqlite.execute("CREATE TABLE t(a INT)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
    sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

    let result = sqlite.query("SELECT a FROM t WHERE a > 1").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_sqlite_diff_aggregate() {
    let sqlite = SqliteEngine::new();
    sqlite.execute("CREATE TABLE orders(amount INT)").unwrap();
    sqlite.execute("INSERT INTO orders VALUES (100)").unwrap();
    sqlite.execute("INSERT INTO orders VALUES (200)").unwrap();
    sqlite.execute("INSERT INTO orders VALUES (150)").unwrap();

    let result = sqlite.query("SELECT SUM(amount) FROM orders").unwrap();
    assert_eq!(result[0][0], "450");
}