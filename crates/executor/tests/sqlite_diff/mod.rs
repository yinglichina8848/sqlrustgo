//! SQLite Differential Testing Framework
//! 
//! Compares SQLRustGo executor results against SQLite for the same SQL input.
//! This catches NULL semantics, JOIN corner cases, GROUP BY bugs, etc.
//!
//! Usage:
//!   cargo test -p sqlrustgo-executor --test sqlite_diff

use std::process::Command;

/// Run a SQL query against SQLite and return the output
fn sqlite_query(sql: &str) -> Result<String, String> {
    let output = Command::new("sqlite3")
        .args([":memory:", ".mode column", ".headers on", sql])
        .output()
        .map_err(|e| format!("sqlite3 not found: {e}"))?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run a SQL query against SQLRustGo executor and return the output
fn sqlrustgo_query(sql: &str) -> Result<String, String> {
    // TODO: connect to SQLRustGo server and execute query
    // For now, return unimplemented
    Err("SQLRustGo runner not yet implemented".to_string())
}

/// Compare outputs, ignoring ordering differences in SELECT results
fn compare_results(sqlite: &str, sqlrustgo: &str) -> bool {
    // Normalize: split into lines, sort, re-join
    let mut sqlite_lines: Vec<&str> = sqlite.lines().collect();
    let mut sqlrustgo_lines: Vec<&str> = sqlrustgo.lines().collect();
    
    // Remove header line if present
    sqlite_lines.retain(|l| !l.trim().is_empty());
    sqlrustgo_lines.retain(|l| !l.trim().is_empty());
    
    sqlite_lines.sort();
    sqlrustgo_lines.sort();
    
    sqlite_lines == sqlrustgo_lines
}

// =====================================================================
// Test Cases: SQL-92 Core Queries
// =====================================================================

#[test]
fn test_simple_select() {
    let sql = "SELECT 1 AS a, 2 AS b UNION ALL SELECT 3, 4 ORDER BY a";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    // sqlrustgo comparison will be enabled once runner is implemented
    println!("SQLite result:\n{}", sqlite);
}

#[test]
fn test_null_semantics() {
    // NULL = NULL should return NULL (not TRUE) in standard SQL
    let sql = "SELECT NULL = NULL AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("NULL semantics SQLite:\n{}", sqlite);
    // SQLite returns 0 for NULL=NULL, standard SQL returns NULL
}

#[test]
fn test_group_by_with_aggregate() {
    let sql = "CREATE TABLE t(a INTEGER, b INTEGER); \
               INSERT INTO t VALUES (1,10),(1,20),(2,30),(2,40),(3,NULL); \
               SELECT a, SUM(b) FROM t GROUP BY a ORDER BY a";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("GROUP BY SQLite:\n{}", sqlite);
}

#[test]
fn test_left_join_with_null() {
    let sql = "CREATE TABLE a(i INTEGER); INSERT INTO a VALUES (1),(2),(3); \
               CREATE TABLE b(j INTEGER); INSERT INTO b VALUES (2),(3),(4); \
               SELECT a.i, b.j FROM a LEFT JOIN b ON a.i=b.j ORDER BY a.i";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("LEFT JOIN:\n{}", sqlite);
}

#[test]
fn test_order_by_with_nulls() {
    let sql = "CREATE TABLE t(x INTEGER); INSERT INTO t VALUES (1),(NULL),(3),(2); \
               SELECT * FROM t ORDER BY x NULLS LAST";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("ORDER BY NULLS LAST:\n{}", sqlite);
}

#[test]
fn test_distinct_with_null() {
    let sql = "SELECT DISTINCT x FROM (SELECT NULL AS x UNION ALL SELECT NULL UNION ALL SELECT 1)";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("DISTINCT NULL:\n{}", sqlite);
}

#[test]
fn test_case_when_null() {
    let sql = "SELECT CASE WHEN NULL THEN 'yes' ELSE 'no' END AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("CASE NULL:\n{}", sqlite);
}

#[test]
fn test_in_with_null() {
    let sql = "SELECT 1 IN (1,2,NULL) AS a, 3 IN (1,2,NULL) AS b";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("IN with NULL:\n{}", sqlite);
}

#[test]
fn test_exists_with_null() {
    let sql = "SELECT EXISTS(SELECT NULL) AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("EXISTS NULL:\n{}", sqlite);
}

#[test]
fn test_count_null() {
    let sql = "CREATE TABLE t(x INTEGER); INSERT INTO t VALUES (1),(NULL),(2),(NULL),(3); \
               SELECT COUNT(*), COUNT(x), COUNT(DISTINCT x) FROM t";
    let sqlite = sqlite_query(sql).expect("sqlite3 should work");
    println!("COUNT NULL:\n{}", sqlite);
}
