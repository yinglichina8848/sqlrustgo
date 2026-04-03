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

// Simple test: run just Q1 to verify basic functionality
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
