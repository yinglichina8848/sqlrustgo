use sqllogictest::{Runner, default_extension};
use std::path::PathBuf;

fn get_test_path(category: &str, test_name: &str) -> PathBuf {
    let mut path = PathBuf::from("tests/sql");
    path.push(category);
    path.push(test_name);
    path.set_extension("test");
    path
}

fn run_test_file(category: &str, test_name: &str) -> Result<(), String> {
    let path = get_test_path(category, test_name);
    if !path.exists() {
        return Err(format!("Test file not found: {:?}", path));
    }
    let mut runner = Runner::new(tests::engine::mod::EngineAdapter::default());
    runner.insert_file(path.clone(), default_extension);
    Ok(())
}

#[test]
fn test_sql_syntax_select() {
    run_test_file("syntax", "select").unwrap();
}

#[test]
fn test_sql_syntax_where() {
    run_test_file("syntax", "where").unwrap();
}

#[test]
fn test_sql_syntax_join() {
    run_test_file("syntax", "join").unwrap();
}

#[test]
fn test_sql_syntax_subquery() {
    run_test_file("syntax", "subquery").unwrap();
}

#[test]
fn test_sql_syntax_order_by() {
    run_test_file("syntax", "order_by").unwrap();
}

#[test]
fn test_sql_dml_insert() {
    run_test_file("dml", "insert").unwrap();
}

#[test]
fn test_sql_dml_update() {
    run_test_file("dml", "update").unwrap();
}

#[test]
fn test_sql_dml_delete() {
    run_test_file("dml", "delete").unwrap();
}

#[test]
fn test_sql_ddl_create_table() {
    run_test_file("ddl", "create_table").unwrap();
}

#[test]
fn test_sql_ddl_index() {
    run_test_file("ddl", "index").unwrap();
}

#[test]
fn test_sql_ddl_alter() {
    run_test_file("ddl", "alter").unwrap();
}

#[test]
fn test_sql_transaction_basic() {
    run_test_file("transaction", "basic_txn").unwrap();
}

#[test]
fn test_sql_transaction_mvcc() {
    run_test_file("transaction", "mvcc_snapshot").unwrap();
}

#[test]
fn test_sql_transaction_wal() {
    run_test_file("transaction", "wal_recovery").unwrap();
}

#[test]
fn test_sql_types_numeric() {
    run_test_file("types", "numeric").unwrap();
}

#[test]
fn test_sql_types_string() {
    run_test_file("types", "string").unwrap();
}

#[test]
fn test_sql_types_datetime() {
    run_test_file("types", "datetime").unwrap();
}

#[test]
fn test_sql_compatibility_pg() {
    run_test_file("compatibility", "pg_compat").unwrap();
}