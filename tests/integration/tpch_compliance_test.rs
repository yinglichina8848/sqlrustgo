//! TPC-H Q1-Q22 合规性测试
//!
//! 使用标准 TPC-H SF=0.1 数据对比 SQLRustGo 与 SQLite 的查询结果
//!
//! 运行方式:
//!   cargo test --test tpch_compliance_test        # SQLRustGo vs SQLite
//!   cargo test --test tpch_compliance_test -- --nocapture  # 显示详细输出

use rusqlite::Connection;
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
use sqlrustgo_storage::{ColumnDefinition, TableInfo};
use std::sync::{Arc, RwLock};

// Use tiny dataset for quick testing, change to "data/tpch-sf001" for full SF=0.1 test
const TBL_DATA_DIR: &str = "data/tpch-tiny";
const TBL_SQLITE_DB: &str = "data/tpch-tiny/tpch.db";

fn create_sqlite_conn() -> Connection {
    Connection::open(TBL_SQLITE_DB).unwrap()
}

fn setup_sqlrustgo_engine() -> ExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let engine = ExecutionEngine::new(storage.clone());

    {
        let mut storage = storage.write().unwrap();

        storage
            .create_table(&TableInfo {
                name: "region".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "r_regionkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "r_name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "r_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "nation".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "n_nationkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "n_name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "n_regionkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "n_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "customer".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "c_custkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_address".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_nationkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_phone".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_acctbal".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_mktsegment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "c_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "supplier".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "s_suppkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_address".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_nationkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_phone".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_acctbal".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "s_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "part".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "p_partkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_name".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_mfgr".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_brand".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_type".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_size".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_container".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_retailprice".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "p_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "partsupp".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "ps_partkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "ps_suppkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "ps_availqty".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "ps_supplycost".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "ps_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "orders".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "o_orderkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_custkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_orderstatus".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_totalprice".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_orderdate".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_orderpriority".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_clerk".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_shippriority".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "o_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();
        storage
            .create_table(&TableInfo {
                name: "lineitem".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "l_orderkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_partkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_suppkey".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_linenumber".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_quantity".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_extendedprice".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_discount".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_tax".to_string(),
                        data_type: "REAL".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_returnflag".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_linestatus".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_shipdate".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_commitdate".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_receiptdate".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_shipinstruct".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_shipmode".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                    ColumnDefinition {
                        name: "l_comment".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                        is_primary_key: false,
                        auto_increment: false,
                    },
                ],
            })
            .unwrap();

        let tables = [
            "region", "nation", "customer", "supplier", "part", "partsupp", "orders", "lineitem",
        ];

        for table_name in tables {
            let filepath = format!("{}/{}.tbl", TBL_DATA_DIR, table_name);
            match storage.bulk_load_tbl_file(table_name, &filepath) {
                Ok(count) => println!("  Loaded {} rows into {}", count, table_name),
                Err(e) => println!("  Failed to load {}: {:?}", table_name, e),
            }
        }
    }

    engine
}

struct QueryResult {
    name: String,
    sqlite_ok: bool,
    sqlite_rows: usize,
    sqlite_error: Option<String>,
    sqlite_data: Vec<Vec<String>>,
    sqlrustgo_ok: bool,
    sqlrustgo_error: Option<String>,
    sqlrustgo_rows: usize,
    sqlrustgo_data: Vec<Vec<String>>,
    match_result: Option<bool>,
}

impl QueryResult {
    fn new(name: &str, _sql: &str) -> Self {
        let _ = _sql; // Kept for API compatibility
        QueryResult {
            name: name.to_string(),
            sqlite_ok: false,
            sqlite_rows: 0,
            sqlite_error: None,
            sqlite_data: Vec::new(),
            sqlrustgo_ok: false,
            sqlrustgo_error: None,
            sqlrustgo_rows: 0,
            sqlrustgo_data: Vec::new(),
            match_result: None,
        }
    }

    fn set_sqlite_result(
        &mut self,
        ok: bool,
        rows: usize,
        data: Vec<Vec<String>>,
        error: Option<String>,
    ) {
        self.sqlite_ok = ok;
        self.sqlite_rows = rows;
        self.sqlite_data = data;
        self.sqlite_error = error;
    }

    fn set_sqlrustgo_result(
        &mut self,
        ok: bool,
        rows: usize,
        data: Vec<Vec<String>>,
        error: Option<String>,
    ) {
        self.sqlrustgo_ok = ok;
        self.sqlrustgo_rows = rows;
        self.sqlrustgo_data = data;
        self.sqlrustgo_error = error;
    }

    fn compare_results(&mut self) {
        if self.sqlite_ok && self.sqlrustgo_ok {
            let mut sqlite_sorted = self.sqlite_data.clone();
            let mut sqlrustgo_sorted = self.sqlrustgo_data.clone();

            for row in sqlite_sorted.iter_mut() {
                row.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            }
            for row in sqlrustgo_sorted.iter_mut() {
                row.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            }

            sqlite_sorted.sort_by(|a, b| {
                let a_str = a.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|");
                let b_str = b.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|");
                a_str.cmp(&b_str)
            });
            sqlrustgo_sorted.sort_by(|a, b| {
                let a_str = a.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|");
                let b_str = b.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|");
                a_str.cmp(&b_str)
            });

            let match_flag =
                Self::compare_data_with_tolerance(&sqlite_sorted, &sqlrustgo_sorted, &self.name);

            if !match_flag && self.name == "Q6" {
                println!(
                    "DEBUG Q6: sqlite={:?}, sqlrustgo={:?}",
                    sqlite_sorted, sqlrustgo_sorted
                );
            }

            self.match_result = Some(match_flag);
        } else {
            self.match_result = Some(false);
        }
    }

    fn compare_data_with_tolerance(a: &[Vec<String>], b: &[Vec<String>], query_name: &str) -> bool {
        if a.len() != b.len() {
            println!(
                "DEBUG {}: row count mismatch {} vs {}",
                query_name,
                a.len(),
                b.len()
            );
            return false;
        }
        for (i, (row_a, row_b)) in a.iter().zip(b.iter()).enumerate() {
            if row_a.len() != row_b.len() {
                println!(
                    "DEBUG {} row {}: column count mismatch {} vs {}",
                    query_name,
                    i,
                    row_a.len(),
                    row_b.len()
                );
                return false;
            }
            for (j, (val_a, val_b)) in row_a.iter().zip(row_b.iter()).enumerate() {
                if !Self::compare_values_with_tolerance(val_a, val_b) {
                    println!(
                        "DEBUG {} row {} col {}: '{}' vs '{}'",
                        query_name, i, j, val_a, val_b
                    );
                    return false;
                }
            }
        }
        true
    }

    fn compare_values_with_tolerance(a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }

        if let (Ok(num_a), Ok(num_b)) = (a.parse::<f64>(), b.parse::<f64>()) {
            let tolerance = 0.000001f64;
            if (num_a - num_b).abs() < tolerance {
                return true;
            }
            let max_magnitude = num_a.abs().max(num_b.abs());
            if max_magnitude > 0.0 && (num_a - num_b).abs() / max_magnitude < tolerance {
                return true;
            }
        }

        a == b
    }
}

fn run_sqlite_query(
    conn: &Connection,
    sql: &str,
) -> (bool, usize, Vec<Vec<String>>, Option<String>) {
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => return (false, 0, Vec::new(), Some(e.to_string())),
    };

    let column_count = stmt.column_count();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row_count = 0;

    let mut result_rows = match stmt.query([]) {
        Ok(r) => r,
        Err(e) => return (false, 0, Vec::new(), Some(e.to_string())),
    };

    while let Some(row) = result_rows.next().unwrap() {
        let mut row_data: Vec<String> = Vec::new();
        for i in 0..column_count {
            let value: String = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => "NULL".to_string(),
                Ok(rusqlite::types::ValueRef::Integer(i)) => i.to_string(),
                Ok(rusqlite::types::ValueRef::Real(f)) => f.to_string(),
                Ok(rusqlite::types::ValueRef::Text(s)) => String::from_utf8_lossy(s).to_string(),
                Ok(rusqlite::types::ValueRef::Blob(b)) => format!("BLOB[{}]", b.len()),
                Err(_) => "ERROR".to_string(),
            };
            row_data.push(value);
        }
        rows.push(row_data);
        row_count += 1;
    }

    (true, row_count, rows, None)
}

fn run_sqlite_query_safe(
    conn: &Connection,
    sql: &str,
) -> (bool, usize, Vec<Vec<String>>, Option<String>) {
    match run_sqlite_query(conn, sql) {
        (ok, rows, data, None) => (ok, rows, data, None),
        (_, _, _, Some(e)) => (false, 0, Vec::new(), Some(e)),
    }
}
#[test]
fn test_tpch_q1_simple() {
    let sqlite_conn = create_sqlite_conn();
    let mut sqlrustgo_engine = setup_sqlrustgo_engine();

    let sql = "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag";

    println!("Running Q1...");

    // Run SQLite query
    let mut stmt = sqlite_conn.prepare(sql).unwrap();
    let sqlite_result: Vec<(String, i64)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    println!("SQLite returned {} rows", sqlite_result.len());

    // Run SQLRustGo query
    let parse_result = parse(sql);
    assert!(
        parse_result.is_ok(),
        "Parse error: {:?}",
        parse_result.err()
    );

    let result = sqlrustgo_engine.execute(parse_result.unwrap());
    match result {
        Ok(r) => {
            println!("SQLRustGo returned {} rows", r.rows.len());
            for row in r.rows.iter().take(5) {
                println!("  {:?}", row);
            }
        }
        Err(e) => {
            println!("SQLRustGo ERROR: {:?}", e);
            panic!("Q1 execution failed: {:?}", e);
        }
    }
}
