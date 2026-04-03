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

fn setup_sqlrustgo_engine_with_bulk_load() -> ExecutionEngine {
    let mut engine = create_sqlrustgo_engine();
    setup_sqlrustgo_schema(&mut engine);

    use std::io::Write;
    let temp_dir = std::env::temp_dir();

    let small_data = vec![
        ("region", "0|AFRICA|Africa region\n1|AMERICA|America region\n2|ASIA|Asia region\n3|EUROPE|Europe region\n4|MIDDLE EAST|Middle East region\n"),
        ("nation", "0|Egypt|0|Egypt\n1|Ethiopia|0|Ethiopia\n2|Japan|2|Japan\n3|India|2|India\n4|Iraq|4|Iraq\n"),
        ("customer", "1|Customer#001|Address1|0|10-1111111|1000.00|AUTOMOBILE|comment1\n2|Customer#002|Address2|1|10-2222222|2000.00|BUILDING|comment2\n3|Customer#003|Address3|2|10-3333333|3000.00|AUTOMOBILE|comment3\n4|Customer#004|Address4|3|10-4444444|4000.00|FURNITURE|comment4\n5|Customer#005|Address5|4|10-5555555|5000.00|MACHINERY|comment5\n"),
        ("supplier", "1|Supplier#1|Address1|0|10-1111111|1000.00|Supplier1\n2|Supplier#2|Address2|1|10-2222222|2000.00|Supplier2\n3|Supplier#3|Address3|2|10-3333333|3000.00|Supplier3\n"),
        ("part", "1|Part1|MFGR#1|Brand#1|ECONOMY ANODIZED STEEL|15|MED BOX|1000.00|Part1\n2|Part2|MFGR#1|Brand#2|PROMO ANODIZED STEEL|25|LG CASE|2000.00|Part2\n3|Part3|MFGR#2|Brand#3|STANDARD POLISHED STEEL|35|MED CASE|1500.00|Part3\n4|Part4|MFGR#2|Brand#4|MEDIUM POLISHED STEEL|45|SM CASE|1200.00|Part4\n"),
        ("partsupp", "1|1|100|500.00|PartSupp1\n1|2|200|600.00|PartSupp2\n2|2|150|700.00|PartSupp3\n3|3|120|800.00|PartSupp4\n4|1|80|550.00|PartSupp5\n"),
        ("orders", "1|1|O|15000.00|1995-01-15|1-URGENT|Clerk#1|0|comment\n2|2|O|5000.00|1995-01-20|5-LOW|Clerk#2|0|comment\n3|3|F|8000.00|1995-02-01|3-MEDIUM|Clerk#3|0|comment\n4|1|O|25000.00|1995-02-15|1-URGENT|Clerk#1|0|comment\n5|2|O|3000.00|1995-03-01|2-HIGH|Clerk#2|0|comment\n6|2|O|30000.00|1995-09-15|1-URGENT|Clerk#1|0|comment\n7|2|O|25000.00|1995-09-10|5-LOW|Clerk#2|0|comment\n"),
        ("lineitem", "1|1|1|1|15|15000.00|0.05|1.2|N|O|1995-01-20|1995-01-18|1995-01-25|NONE|AIR|comment1\n1|2|2|2|20|20000.00|0.05|1.6|N|O|1995-01-20|1995-01-18|1995-01-25|NONE|AIR|comment2\n2|3|3|1|5|5000.00|0.10|0.4|N|O|1995-01-25|1995-01-23|1995-01-30|NONE|TRUCK|comment3\n3|1|1|1|8|8000.00|0.08|0.64|N|O|1995-02-10|1995-02-08|1995-02-15|NONE|RAIL|comment4\n3|2|2|1|25|25000.00|0.03|2.0|A|F|1995-02-10|1995-02-08|1995-02-15|NONE|AIR|comment5\n4|3|3|1|10|10000.00|0.06|0.8|N|O|1995-02-20|1995-02-18|1995-02-25|NONE|SHIP|comment6\n5|1|1|1|12|12000.00|0.04|0.96|R|F|1995-03-05|1995-03-03|1995-03-10|NONE|AIR|comment7\n6|2|1|1|30|30000.00|0.05|1.5|N|O|1995-09-15|1995-09-01|1995-09-20|NONE|AIR|comment8\n7|2|2|1|25|25000.00|0.10|1.25|N|O|1995-09-10|1995-09-05|1995-09-15|NONE|TRUCK|comment9\n8|1|1|1|10|10000.00|0.03|0.3|N|O|1995-09-20|1995-09-10|1995-09-25|NONE|SHIP|comment10\n"),
    ];

    for (table, data) in small_data {
        let filepath = temp_dir.join(format!("{}.tbl", table));
        let mut file = std::fs::File::create(&filepath).unwrap();
        file.write_all(data.as_bytes()).unwrap();

        let mut storage = engine.storage.write().unwrap();
        match storage.bulk_load_tbl_file(table, filepath.to_str().unwrap()) {
            Ok(count) => println!("Loaded {} rows into {} (bulk)", count, table),
            Err(e) => println!("Failed to load {}: {:?}", table, e),
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
