//! SQLite Differential Testing Framework
//!
//! Compares SQLRustGo executor results against SQLite (ground truth).
//! Any difference = bug in SQLRustGo (unless marked #[ignore]).
//!
//! =============================================================================
//! TEST LAYERS (STRICTLY enforced):
//!
//! Layer 1 — correctness (active, must PASS):
//!   These are bugs in "already supported" features. Fix these first.
//!
//! Layer 2 — capability (#[ignore] = feature gap, not bug):
//!   These are unimplemented features. Do NOT confuse for bugs.
//!   Track in sqlite_diff_bugs.md capability section.
//!
//! =============================================================================
//!
//! Usage:
//!   cargo test -p sqlrustgo-executor --test sqlite_diff -- --nocapture
//!
//! CI Integration (.github/workflows/ci-pr.yml):
//!   sqlite-diff:
//!     runs-on: ubuntu-latest
//!     steps:
//!       - uses: actions/checkout@v4
//!       - name: Setup Rust
//!         run: rustup install stable && rustup default stable
//!       - name: Install sqlite3
//!         run: apt-get update && apt-get install -y sqlite3
//!       - name: Run SQLite differential tests
//!         run: cargo test -p sqlrustgo-executor --test sqlite_diff -- --nocapture

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::process::Command;
use std::sync::{Arc, RwLock};

// =============================================================================
// SQLite runner
// =============================================================================

/// Run SQL against real SQLite and return stdout (trimmed).
/// Uses -list mode with \t separator to match SQLRustGo output format exactly.
fn sqlite_query(sql: &str) -> Result<String, String> {
    let output = Command::new("sqlite3")
        .args(["-separator", "\t", ":memory:", sql])
        .output()
        .map_err(|e| format!("sqlite3 not found: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "sqlite3 error: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// =============================================================================
// SQLRustGo runner
// =============================================================================

/// Run SQL against SQLRustGo ExecutionEngine and return formatted output.
/// Output format: \t-separated columns, \n-separated rows, no headers.
fn sqlrustgo_query(sql: &str) -> Result<String, String> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    let statements: Vec<&str> = sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut last_output = String::new();

    for stmt in statements {
        match engine.execute(stmt) {
            Ok(exec_result) => {
                if !exec_result.rows.is_empty() {
                    let lines: Vec<String> = exec_result
                        .rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .map(value_to_string)
                                .collect::<Vec<_>>()
                                .join("\t")
                        })
                        .collect();
                    last_output = lines.join("\n");
                }
                // Non-SELECT (affected_rows > 0, rows empty): output empty
            }
            Err(e) => {
                return Err(format!("SQLRustGo error on '{}': {:?}", stmt, e));
            }
        }
    }

    Ok(last_output)
}

/// Convert Value -> string matching SQLite column output
fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            let s = f.to_string();
            if !s.contains('.') {
                format!("{:.1}", f)
            } else {
                s
            }
        }
        Value::Text(s) => s.clone(),
        Value::Boolean(b) => {
            if *b { "1".to_string() } else { "0".to_string() }
        }
        Value::Blob(b) => format!("[blob {} bytes]", b.len()),
    }
}

// =============================================================================
// Row-level comparison (strict, no lossy normalization)
// =============================================================================

/// Parse tab-separated output into a Vec of Vec<String>.
fn parse_output(s: &str) -> Vec<Vec<String>> {
    s.lines()
        .map(|line| line.split('\t').map(|c| c.trim().to_lowercase()).collect())
        .filter(|row: &Vec<String>| !row.is_empty() && !(row.len() == 1 && row[0].is_empty()))
        .collect()
}

/// Compare two query outputs strictly.
///
/// RULES:
/// 1. Row count must match (no lossy count comparison)
/// 2. Per-row: column count must match
/// 3. Per-cell: values must match exactly (lowercased)
/// 4. Order: MUST match (we do NOT sort — order matters in SQL)
fn compare_results(sqlite_out: &str, sqlrustgo_out: &str) -> Result<(), String> {
    let sqlite_rows = parse_output(sqlite_out);
    let sqlrustgo_rows = parse_output(sqlrustgo_out);

    if sqlite_rows.len() != sqlrustgo_rows.len() {
        return Err(format!(
            "Row count mismatch: SQLite={}, SQLRustGo={}\n\
             SQLite:\n{}\n\nSQLRustGo:\n{}",
            sqlite_rows.len(),
            sqlrustgo_rows.len(),
            sqlite_out,
            sqlrustgo_out
        ));
    }

    for (i, (sq_row, sw_row)) in sqlite_rows.iter().zip(sqlrustgo_rows.iter()).enumerate() {
        if sq_row.len() != sw_row.len() {
            return Err(format!(
                "Row {}: column count mismatch: SQLite={} cols, SQLRustGo={} cols\n\
                 SQLite row: {:?}\n\
                 SQLRustGo row: {:?}",
                i + 1,
                sq_row.len(),
                sw_row.len(),
                sq_row,
                sw_row
            ));
        }
        for (j, (sq_cell, sw_cell)) in sq_row.iter().zip(sw_row.iter()).enumerate() {
            if sq_cell != sw_cell {
                return Err(format!(
                    "Row {}, col {}: value mismatch\n\
                     SQLite:    '{}'\n\
                     SQLRustGo: '{}'",
                    i + 1, j + 1, sq_cell, sw_cell
                ));
            }
        }
    }

    Ok(())
}

// =============================================================================
// Test macro
// =============================================================================

/// Run the same SQL on both SQLite and SQLRustGo, assert outputs match.
/// SQLite syntax errors → skip (SQLite may not support the syntax).
/// SQLRustGo errors → FAIL (regression).
macro_rules! assert_diff {
    ($name:expr, $sql:expr) => {{
        let sqlite_out = match sqlite_query($sql) {
            Ok(s) => s,
            Err(e) => {
                println!("SKIP (SQLite error): {} | {}", $name, e);
                return;
            }
        };

        let sqlrustgo_out = match sqlrustgo_query($sql) {
            Ok(s) => s,
            Err(e) => {
                panic!(
                    "SQLRustGo ERROR on '{}':\n  SQL: {}\n  Error: {}",
                    $name, $sql, e
                );
            }
        };

        if let Err(detail) = compare_results(&sqlite_out, &sqlrustgo_out) {
            panic!(
                "\n=== DIFFERENCE DETECTED ===\n\
                 Test: {}\n\
                 SQL: {}\n\n{}",
                $name, $sql, detail
            );
        }
    }};
}

// =============================================================================
// LAYER 1: correctness — bugs in already-supported features
// These tests define what "supported" means. Fix bugs here first.
// =============================================================================

/// Basic SELECT * with ORDER BY — single integer column
#[test]
fn test_order_by_int() {
    assert_diff!(
        "order by int",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (3),(1),(4),(1),(5),(9),(2),(6); \
         SELECT * FROM t ORDER BY x"
    );
}

/// Text column ORDER BY
#[test]
fn test_order_by_text() {
    assert_diff!(
        "order by text",
        "CREATE TABLE t(x TEXT); \
         INSERT INTO t VALUES ('cherry'),('apple'),('banana'); \
         SELECT * FROM t ORDER BY x"
    );
}

/// WHERE equality (simple > 15)
#[test]
fn test_where_eq() {
    assert_diff!(
        "where equality",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (10),(20),(30),(40); \
         SELECT * FROM t WHERE x > 15 ORDER BY x"
    );
}

/// WHERE with column projection — correctness bug: engine outputs full row instead of projected columns
#[test]
fn test_where_projection() {
    assert_diff!(
        "where projection",
        "CREATE TABLE t(id INTEGER, x INTEGER); \
         INSERT INTO t VALUES (1,10),(2,NULL),(3,20); \
         SELECT id FROM t WHERE x = 10 ORDER BY id"
    );
}

/// LIMIT
#[test]
fn test_limit() {
    assert_diff!(
        "limit",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (1),(2),(3),(4),(5); \
         SELECT * FROM t ORDER BY x LIMIT 3"
    );
}

/// LIMIT + OFFSET
#[test]
fn test_limit_offset() {
    assert_diff!(
        "limit offset",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (1),(2),(3),(4),(5); \
         SELECT * FROM t ORDER BY x LIMIT 3 OFFSET 2"
    );
}

/// INSERT then SELECT with ORDER BY
#[test]
fn test_insert_and_select() {
    assert_diff!(
        "insert then select",
        "CREATE TABLE t(id INTEGER, name TEXT); \
         INSERT INTO t VALUES (1,'alice'),(2,'bob'),(3,'charlie'); \
         SELECT * FROM t ORDER BY id"
    );
}

// =============================================================================
// LAYER 2: capability — unimplemented features (ignore until implemented)
//
// These are NOT bugs — they are feature gaps.
// When you implement a feature, move its test here to Layer 1.
// =============================================================================

/// DISTINCT — capability gap
#[test]
#[ignore]
fn test_distinct() {
    assert_diff!(
        "distinct",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (1),(1),(2),(NULL),(2),(3),(NULL); \
         SELECT DISTINCT x FROM t ORDER BY x"
    );
}

/// Scalar SELECT (SELECT 1, SELECT 'text') — capability gap
#[test]
#[ignore]
fn test_scalar_select() {
    assert_diff!("scalar int", "SELECT 1 AS a, 2 AS b");
    assert_diff!("scalar text", "SELECT 'hello' AS msg");
}

/// UNION ALL — capability gap
#[test]
#[ignore]
fn test_union_all() {
    assert_diff!(
        "union all",
        "SELECT 1 AS x UNION ALL SELECT 2 UNION ALL SELECT 3 ORDER BY x"
    );
}

/// GROUP BY COUNT — capability gap
#[test]
#[ignore]
fn test_group_by_count() {
    assert_diff!(
        "group by count",
        "CREATE TABLE t(a TEXT, b INTEGER); \
         INSERT INTO t VALUES ('x',10),('x',20),('y',30),('y',40),('y',50); \
         SELECT a, COUNT(*) FROM t GROUP BY a ORDER BY a"
    );
}

/// GROUP BY SUM — capability gap
#[test]
#[ignore]
fn test_group_by_sum() {
    assert_diff!(
        "group by sum",
        "CREATE TABLE t(a TEXT, b INTEGER); \
         INSERT INTO t VALUES ('x',10),('x',20),('y',30),('y',40),('z',NULL); \
         SELECT a, SUM(b) FROM t GROUP BY a ORDER BY a"
    );
}

/// COUNT(*) vs COUNT(column) — capability gap (aggregation)
#[test]
#[ignore]
fn test_count_star_vs_count_col() {
    assert_diff!(
        "count star vs count column",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (1),(NULL),(2),(NULL),(3); \
         SELECT COUNT(*), COUNT(x) FROM t"
    );
}

/// COUNT(DISTINCT x) — capability gap
#[test]
#[ignore]
fn test_count_distinct_null() {
    assert_diff!(
        "count distinct with null",
        "CREATE TABLE t(x INTEGER); \
         INSERT INTO t VALUES (1),(NULL),(2),(NULL),(3); \
         SELECT COUNT(DISTINCT x) FROM t"
    );
}

/// INNER JOIN — capability gap (high complexity)
#[test]
#[ignore]
fn test_inner_join() {
    assert_diff!(
        "inner join",
        "CREATE TABLE a(i INTEGER); INSERT INTO a VALUES (1),(2),(3); \
         CREATE TABLE b(j INTEGER); INSERT INTO b VALUES (2),(3),(4); \
         SELECT a.i, b.j FROM a JOIN b ON a.i=b.j ORDER BY a.i"
    );
}

/// LEFT JOIN — capability gap (high complexity)
#[test]
#[ignore]
fn test_left_join() {
    assert_diff!(
        "left join",
        "CREATE TABLE a(i INTEGER); INSERT INTO a VALUES (1),(2),(3); \
         CREATE TABLE b(j INTEGER); INSERT INTO b VALUES (2),(3),(4); \
         SELECT a.i, b.j FROM a LEFT JOIN b ON a.i=b.j ORDER BY a.i"
    );
}

/// LEFT JOIN with NULLs — capability gap
#[test]
#[ignore]
fn test_left_join_with_null() {
    assert_diff!(
        "left join with nulls",
        "CREATE TABLE a(i INTEGER); INSERT INTO a VALUES (1),(2),(3); \
         CREATE TABLE b(j INTEGER); INSERT INTO b VALUES (2,NULL),(3,NULL),(NULL,9); \
         SELECT a.i, b.j FROM a LEFT JOIN b ON a.i=b.j ORDER BY a.i"
    );
}

// =============================================================================
// LAYER 3: known semantic differences (ignore, document reason)
// These are SQL-standard vs SQLite behavior mismatches.
// =============================================================================

/// Known diff: NULL = NULL → SQLite=0, SQL standard=NULL
#[test]
#[ignore]
fn test_null_eq_null() {
    let sql = "SELECT NULL = NULL AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 works");
    let sqlrustgo = sqlrustgo_query(sql).expect("sqlrustgo works");
    println!("SQLite NULL=NULL: {}", sqlite);
    println!("SQLRustGo NULL=NULL: {}", sqlrustgo);
}

/// Known diff: ORDER BY NULLS LAST — SQLite extension
#[test]
#[ignore]
fn test_order_by_nulls_last() {
    let sql = "CREATE TABLE t(x INTEGER); \
               INSERT INTO t VALUES (1),(NULL),(3),(2); \
               SELECT * FROM t ORDER BY x NULLS LAST";
    let sqlite = sqlite_query(sql).expect("sqlite3 works");
    let sqlrustgo = sqlrustgo_query(sql);
    println!("SQLite NULLS LAST:\n{}", sqlite);
    match sqlrustgo {
        Ok(out) => println!("SQLRustGo NULLS LAST:\n{}", out),
        Err(e) => println!("SQLRustGo ERROR: {}", e),
    }
}

/// Known diff: IN with NULL semantics differ
#[test]
#[ignore]
fn test_in_with_null() {
    let sql = "SELECT 1 IN (1,2,NULL) AS a, 3 IN (1,2,NULL) AS b";
    let sqlite = sqlite_query(sql).expect("sqlite3 works");
    let sqlrustgo = sqlrustgo_query(sql).expect("sqlrustgo works");
    println!("SQLite IN+NULL:\n{}", sqlite);
    println!("SQLRustGo IN+NULL:\n{}", sqlrustgo);
}

/// Known diff: CASE WHEN NULL
#[test]
#[ignore]
fn test_case_when_null() {
    let sql = "SELECT CASE WHEN NULL THEN 'yes' ELSE 'no' END AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 works");
    let sqlrustgo = sqlrustgo_query(sql).expect("sqlrustgo works");
    println!("SQLite CASE WHEN NULL:\n{}", sqlite);
    println!("SQLRustGo CASE WHEN NULL:\n{}", sqlrustgo);
}

/// Known diff: EXISTS(SELECT NULL)
#[test]
#[ignore]
fn test_exists_with_null() {
    let sql = "SELECT EXISTS(SELECT NULL) AS result";
    let sqlite = sqlite_query(sql).expect("sqlite3 works");
    let sqlrustgo = sqlrustgo_query(sql).expect("sqlrustgo works");
    println!("SQLite EXISTS(NULL):\n{}", sqlite);
    println!("SQLRustGo EXISTS(NULL):\n{}", sqlrustgo);
}
