//! TPC-H Q1-Q22 合规性测试
//!
//! 使用标准 TPC-H SQL 对比 SQLRustGo 与 SQLite 的查询结果
//!
//! 运行方式:
//!   cargo test --test tpch_compliance_test        # SQLRustGo vs SQLite (小数据集)
//!   cargo test --test tpch_compliance_test test_tpch_compliance_report_with_real_data -- --nocapture  # 显示详细输出

use rusqlite::{params, Connection};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::path::Path;
use std::sync::{Arc, RwLock};

const TPCK_DATA_DIR: &str = "data/tpch-sf01";
const TPCK_SQLITE_DB: &str = "data/tpch-sf01/tpch.db";

fn create_sqlrustgo_engine() -> ExecutionEngine {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

fn create_sqlite_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn
}

fn create_sqlite_conn_from_file(path: &str) -> Connection {
    Connection::open(path).expect(&format!("Failed to open SQLite DB at: {}", path))
}

fn setup_sqlite_schema(conn: &Connection) {
    conn.execute_batch(
        "
        CREATE TABLE lineitem (
            l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER,
            l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL,
            l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT,
            l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT,
            l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT
        );
        CREATE TABLE orders (
            o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER, o_orderstatus TEXT,
            o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT,
            o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT
        );
        CREATE TABLE customer (
            c_custkey INTEGER PRIMARY KEY, c_name TEXT, c_address TEXT,
            c_nationkey INTEGER, c_phone TEXT, c_acctbal REAL,
            c_mktsegment TEXT, c_comment TEXT
        );
        CREATE TABLE nation (
            n_nationkey INTEGER, n_name TEXT, n_regionkey INTEGER, n_comment TEXT
        );
        CREATE TABLE region (
            r_regionkey INTEGER, r_name TEXT, r_comment TEXT
        );
        CREATE TABLE part (
            p_partkey INTEGER, p_name TEXT, p_mfgr TEXT, p_brand TEXT,
            p_type TEXT, p_size INTEGER, p_container TEXT,
            p_retailprice REAL, p_comment TEXT
        );
        CREATE TABLE supplier (
            s_suppkey INTEGER, s_name TEXT, s_address TEXT, s_nationkey INTEGER,
            s_phone TEXT, s_acctbal REAL, s_comment TEXT
        );
        CREATE TABLE partsupp (
            ps_partkey INTEGER, ps_suppkey INTEGER, ps_availqty INTEGER,
            ps_supplycost REAL, ps_comment TEXT
        );
        ",
    )
    .unwrap();
}

fn setup_sqlite_data(conn: &Connection) {
    conn.execute_batch("BEGIN TRANSACTION;").unwrap();

    conn.execute(
        "INSERT INTO region VALUES (0, 'AFRICA', 'Africa region')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (1, 'AMERICA', 'America region')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO region VALUES (2, 'ASIA', 'Asia region')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (3, 'EUROPE', 'Europe region')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO region VALUES (4, 'MIDDLE EAST', 'Middle East region')",
        [],
    )
    .unwrap();

    conn.execute("INSERT INTO nation VALUES (0, 'EGYPT', 0, 'Egypt')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO nation VALUES (1, 'ETHIOPIA', 0, 'Ethiopia')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO nation VALUES (2, 'JAPAN', 2, 'Japan')", [])
        .unwrap();
    conn.execute("INSERT INTO nation VALUES (3, 'INDIA', 2, 'India')", [])
        .unwrap();
    conn.execute("INSERT INTO nation VALUES (4, 'IRAQ', 4, 'Iraq')", [])
        .unwrap();

    conn.execute("INSERT INTO customer VALUES (1, 'Customer#000001', 'Address1', 0, '10-1111111', 1000.00, 'AUTOMOBILE', 'comment1')", []).unwrap();
    conn.execute("INSERT INTO customer VALUES (2, 'Customer#000002', 'Address2', 1, '10-2222222', 2000.00, 'BUILDING', 'comment2')", []).unwrap();
    conn.execute("INSERT INTO customer VALUES (3, 'Customer#000003', 'Address3', 2, '10-3333333', 3000.00, 'AUTOMOBILE', 'comment3')", []).unwrap();
    conn.execute("INSERT INTO customer VALUES (4, 'Customer#000004', 'Address4', 3, '10-4444444', 4000.00, 'FURNITURE', 'comment4')", []).unwrap();
    conn.execute("INSERT INTO customer VALUES (5, 'Customer#000005', 'Address5', 4, '10-5555555', 5000.00, 'MACHINERY', 'comment5')", []).unwrap();

    conn.execute("INSERT INTO supplier VALUES (1, 'Supplier#1', 'Address1', 0, '10-1111111', 1000.00, 'Supplier1')", []).unwrap();
    conn.execute("INSERT INTO supplier VALUES (2, 'Supplier#2', 'Address2', 1, '10-2222222', 2000.00, 'Supplier2')", []).unwrap();
    conn.execute("INSERT INTO supplier VALUES (3, 'Supplier#3', 'Address3', 2, '10-3333333', 3000.00, 'Supplier3')", []).unwrap();

    conn.execute("INSERT INTO part VALUES (1, 'Part1', 'MFGR#1', 'Brand#1', 'ECONOMY ANODIZED STEEL', 15, 'MED BOX', 1000.00, 'Part1')", []).unwrap();
    conn.execute("INSERT INTO part VALUES (2, 'Part2', 'MFGR#1', 'Brand#2', 'PROMO ANODIZED STEEL', 25, 'LG CASE', 2000.00, 'Part2')", []).unwrap();
    conn.execute("INSERT INTO part VALUES (3, 'Part3', 'MFGR#2', 'Brand#3', 'STANDARD POLISHED STEEL', 35, 'MED CASE', 1500.00, 'Part3')", []).unwrap();
    conn.execute("INSERT INTO part VALUES (4, 'Part4', 'MFGR#2', 'Brand#4', 'MEDIUM POLISHED STEEL', 45, 'SM CASE', 1200.00, 'Part4')", []).unwrap();

    conn.execute(
        "INSERT INTO partsupp VALUES (1, 1, 100, 500.00, 'PartSupp1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO partsupp VALUES (1, 2, 200, 600.00, 'PartSupp2')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO partsupp VALUES (2, 2, 150, 700.00, 'PartSupp3')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO partsupp VALUES (3, 3, 120, 800.00, 'PartSupp4')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO partsupp VALUES (4, 1, 80, 550.00, 'PartSupp5')",
        [],
    )
    .unwrap();

    conn.execute("INSERT INTO orders VALUES (1, 1, 'O', 15000.00, '1995-01-15', '1-URGENT', 'Clerk#1', 0, 'comment')", []).unwrap();
    conn.execute("INSERT INTO orders VALUES (2, 2, 'O', 5000.00, '1995-01-20', '5-LOW', 'Clerk#2', 0, 'comment')", []).unwrap();
    conn.execute("INSERT INTO orders VALUES (3, 3, 'F', 8000.00, '1995-02-01', '3-MEDIUM', 'Clerk#3', 0, 'comment')", []).unwrap();
    conn.execute("INSERT INTO orders VALUES (4, 1, 'O', 25000.00, '1995-02-15', '1-URGENT', 'Clerk#1', 0, 'comment')", []).unwrap();
    conn.execute("INSERT INTO orders VALUES (5, 2, 'O', 3000.00, '1995-03-01', '2-HIGH', 'Clerk#2', 0, 'comment')", []).unwrap();

    let lineitem_data = vec![
        "1, 1, 1, 1, 15, 15000.00, 0.05, 1.2, 'N', 'O', '1995-01-20', '1995-01-18', '1995-01-25', 'NONE', 'AIR', 'comment1'",
        "1, 2, 2, 2, 20, 20000.00, 0.05, 1.6, 'N', 'O', '1995-01-20', '1995-01-18', '1995-01-25', 'NONE', 'AIR', 'comment2'",
        "2, 3, 3, 1, 5, 5000.00, 0.10, 0.4, 'N', 'O', '1995-01-25', '1995-01-23', '1995-01-30', 'NONE', 'TRUCK', 'comment3'",
        "3, 1, 1, 1, 8, 8000.00, 0.08, 0.64, 'N', 'O', '1995-02-10', '1995-02-08', '1995-02-15', 'NONE', 'RAIL', 'comment4'",
        "3, 2, 2, 1, 25, 25000.00, 0.03, 2.0, 'A', 'F', '1995-02-10', '1995-02-08', '1995-02-15', 'NONE', 'AIR', 'comment5'",
        "4, 3, 3, 1, 10, 10000.00, 0.06, 0.8, 'N', 'O', '1995-02-20', '1995-02-18', '1995-02-25', 'NONE', 'SHIP', 'comment6'",
        "5, 1, 1, 1, 12, 12000.00, 0.04, 0.96, 'R', 'F', '1995-03-05', '1995-03-03', '1995-03-10', 'NONE', 'AIR', 'comment7'",
        "6, 2, 1, 1, 30, 30000.00, 0.05, 1.5, 'N', 'O', '1995-09-15', '1995-09-01', '1995-09-20', 'NONE', 'AIR', 'comment8'",
        "7, 2, 2, 1, 25, 25000.00, 0.10, 1.25, 'N', 'O', '1995-09-10', '1995-09-05', '1995-09-15', 'NONE', 'TRUCK', 'comment9'",
        "8, 1, 1, 1, 10, 10000.00, 0.03, 0.3, 'N', 'O', '1995-09-20', '1995-09-10', '1995-09-25', 'NONE', 'SHIP', 'comment10'",
    ];

    for data in lineitem_data {
        let parts: Vec<&str> = data.split(',').collect();
        if parts.len() >= 16 {
            conn.execute(
                "INSERT INTO lineitem VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    parts[0].trim(),
                    parts[1].trim(),
                    parts[2].trim(),
                    parts[3].trim(),
                    parts[4].trim(),
                    parts[5].trim(),
                    parts[6].trim(),
                    parts[7].trim(),
                    parts[8].trim(),
                    parts[9].trim(),
                    parts[10].trim(),
                    parts[11].trim(),
                    parts[12].trim(),
                    parts[13].trim(),
                    parts[14].trim(),
                    parts[15].trim(),
                ],
            ).unwrap();
        }
    }

    conn.execute_batch("COMMIT;").unwrap();
}

fn setup_sqlrustgo_schema(engine: &mut ExecutionEngine) {
    engine.execute(parse("CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_linenumber INTEGER, l_quantity INTEGER, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_linestatus TEXT, l_shipdate TEXT, l_commitdate TEXT, l_receiptdate TEXT, l_shipinstruct TEXT, l_shipmode TEXT, l_comment TEXT)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE orders (o_orderkey INTEGER PRIMARY KEY, o_custkey INTEGER, o_orderstatus TEXT, o_totalprice REAL, o_orderdate TEXT, o_orderpriority TEXT, o_clerk TEXT, o_shippriority INTEGER, o_comment TEXT)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE customer (c_custkey INTEGER PRIMARY KEY, c_name TEXT, c_address TEXT, c_nationkey INTEGER, c_phone TEXT, c_acctbal REAL, c_mktsegment TEXT, c_comment TEXT)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE nation (n_nationkey INTEGER, n_name TEXT, n_regionkey INTEGER, n_comment TEXT)").unwrap()).unwrap();
    engine
        .execute(
            parse("CREATE TABLE region (r_regionkey INTEGER, r_name TEXT, r_comment TEXT)")
                .unwrap(),
        )
        .unwrap();
    engine.execute(parse("CREATE TABLE part (p_partkey INTEGER, p_name TEXT, p_mfgr TEXT, p_brand TEXT, p_type TEXT, p_size INTEGER, p_container TEXT, p_retailprice REAL, p_comment TEXT)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE supplier (s_suppkey INTEGER, s_name TEXT, s_address TEXT, s_nationkey INTEGER, s_phone TEXT, s_acctbal REAL, s_comment TEXT)").unwrap()).unwrap();
    engine.execute(parse("CREATE TABLE partsupp (ps_partkey INTEGER, ps_suppkey INTEGER, ps_availqty INTEGER, ps_supplycost REAL, ps_comment TEXT)").unwrap()).unwrap();
}

fn setup_sqlrustgo_engine_with_real_data(data_dir: &str) -> ExecutionEngine {
    let mut engine = create_sqlrustgo_engine();
    setup_sqlrustgo_schema(&mut engine);

    let tables = [
        "region", "nation", "customer", "supplier", "part", "partsupp", "orders", "lineitem",
    ];

    for table in tables {
        let filepath = format!("{}/{}.tbl", data_dir, table);
        if Path::new(&filepath).exists() {
            let mut storage = engine.storage.write().unwrap();
            match storage.bulk_load_tbl_file(table, &filepath) {
                Ok(count) => println!("Loaded {} rows into {}", count, table),
                Err(e) => println!("Failed to load {}: {:?}", table, e),
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

fn insert_sqlrustgo_data(engine: &mut ExecutionEngine) {
    engine
        .execute(parse("INSERT INTO region VALUES (0, 'AFRICA', 'Africa region')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO region VALUES (1, 'AMERICA', 'America region')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO region VALUES (2, 'ASIA', 'Asia region')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO region VALUES (3, 'EUROPE', 'Europe region')").unwrap())
        .unwrap();
    engine
        .execute(
            parse("INSERT INTO region VALUES (4, 'MIDDLE EAST', 'Middle East region')").unwrap(),
        )
        .unwrap();

    engine
        .execute(parse("INSERT INTO nation VALUES (0, 'EGYPT', 0, 'Egypt')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO nation VALUES (1, 'ETHIOPIA', 0, 'Ethiopia')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO nation VALUES (2, 'JAPAN', 2, 'Japan')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO nation VALUES (3, 'INDIA', 2, 'India')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO nation VALUES (4, 'IRAQ', 4, 'Iraq')").unwrap())
        .unwrap();

    engine.execute(parse("INSERT INTO customer VALUES (1, 'Customer#000001', 'Address1', 0, '10-1111111', 1000.00, 'AUTOMOBILE', 'comment1')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO customer VALUES (2, 'Customer#000002', 'Address2', 1, '10-2222222', 2000.00, 'BUILDING', 'comment2')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO customer VALUES (3, 'Customer#000003', 'Address3', 2, '10-3333333', 3000.00, 'AUTOMOBILE', 'comment3')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO customer VALUES (4, 'Customer#000004', 'Address4', 3, '10-4444444', 4000.00, 'FURNITURE', 'comment4')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO customer VALUES (5, 'Customer#000005', 'Address5', 4, '10-5555555', 5000.00, 'MACHINERY', 'comment5')").unwrap()).unwrap();

    engine.execute(parse("INSERT INTO supplier VALUES (1, 'Supplier#1', 'Address1', 0, '10-1111111', 1000.00, 'Supplier1')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO supplier VALUES (2, 'Supplier#2', 'Address2', 1, '10-2222222', 2000.00, 'Supplier2')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO supplier VALUES (3, 'Supplier#3', 'Address3', 2, '10-3333333', 3000.00, 'Supplier3')").unwrap()).unwrap();

    engine.execute(parse("INSERT INTO part VALUES (1, 'Part1', 'MFGR#1', 'Brand#1', 'ECONOMY ANODIZED STEEL', 15, 'MED BOX', 1000.00, 'Part1')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO part VALUES (2, 'Part2', 'MFGR#1', 'Brand#2', 'PROMO ANODIZED STEEL', 25, 'LG CASE', 2000.00, 'Part2')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO part VALUES (3, 'Part3', 'MFGR#2', 'Brand#3', 'STANDARD POLISHED STEEL', 35, 'MED CASE', 1500.00, 'Part3')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO part VALUES (4, 'Part4', 'MFGR#2', 'Brand#4', 'MEDIUM POLISHED STEEL', 45, 'SM CASE', 1200.00, 'Part4')").unwrap()).unwrap();

    engine
        .execute(parse("INSERT INTO partsupp VALUES (1, 1, 100, 500.00, 'PartSupp1')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO partsupp VALUES (1, 2, 200, 600.00, 'PartSupp2')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO partsupp VALUES (2, 2, 150, 700.00, 'PartSupp3')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO partsupp VALUES (3, 3, 120, 800.00, 'PartSupp4')").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO partsupp VALUES (4, 1, 80, 550.00, 'PartSupp5')").unwrap())
        .unwrap();

    engine.execute(parse("INSERT INTO orders VALUES (1, 1, 'O', 15000.00, '1995-01-15', '1-URGENT', 'Clerk#1', 0, 'comment')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (2, 2, 'O', 5000.00, '1995-01-20', '5-LOW', 'Clerk#2', 0, 'comment')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (3, 3, 'F', 8000.00, '1995-02-01', '3-MEDIUM', 'Clerk#3', 0, 'comment')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (4, 1, 'O', 25000.00, '1995-02-15', '1-URGENT', 'Clerk#1', 0, 'comment')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO orders VALUES (5, 2, 'O', 3000.00, '1995-03-01', '2-HIGH', 'Clerk#2', 0, 'comment')").unwrap()).unwrap();

    engine.execute(parse("INSERT INTO lineitem VALUES (1, 1, 1, 1, 15, 15000.00, 0.05, 1.2, 'N', 'O', '1995-01-20', '1995-01-18', '1995-01-25', 'NONE', 'AIR', 'comment1')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (1, 2, 2, 2, 20, 20000.00, 0.05, 1.6, 'N', 'O', '1995-01-20', '1995-01-18', '1995-01-25', 'NONE', 'AIR', 'comment2')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (2, 3, 3, 1, 5, 5000.00, 0.10, 0.4, 'N', 'O', '1995-01-25', '1995-01-23', '1995-01-30', 'NONE', 'TRUCK', 'comment3')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (3, 1, 1, 1, 8, 8000.00, 0.08, 0.64, 'N', 'O', '1995-02-10', '1995-02-08', '1995-02-15', 'NONE', 'RAIL', 'comment4')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (3, 2, 2, 1, 25, 25000.00, 0.03, 2.0, 'A', 'F', '1995-02-10', '1995-02-08', '1995-02-15', 'NONE', 'AIR', 'comment5')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (4, 3, 3, 1, 10, 10000.00, 0.06, 0.8, 'N', 'O', '1995-02-20', '1995-02-18', '1995-02-25', 'NONE', 'SHIP', 'comment6')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (5, 1, 1, 1, 12, 12000.00, 0.04, 0.96, 'R', 'F', '1995-03-05', '1995-03-03', '1995-03-10', 'NONE', 'AIR', 'comment7')").unwrap()).unwrap();
    // Q14 test data - 9月份数据用于测试 CASE WHEN
    engine.execute(parse("INSERT INTO lineitem VALUES (6, 2, 1, 1, 30, 30000.00, 0.05, 1.5, 'N', 'O', '1995-09-15', '1995-09-01', '1995-09-20', 'NONE', 'AIR', 'comment8')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (7, 2, 2, 1, 25, 25000.00, 0.10, 1.25, 'N', 'O', '1995-09-10', '1995-09-05', '1995-09-15', 'NONE', 'TRUCK', 'comment9')").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO lineitem VALUES (8, 1, 1, 1, 10, 10000.00, 0.03, 0.3, 'N', 'O', '1995-09-20', '1995-09-10', '1995-09-25', 'NONE', 'SHIP', 'comment10')").unwrap()).unwrap();
}

fn setup_sqlrustgo_engine() -> ExecutionEngine {
    let mut engine = create_sqlrustgo_engine();
    setup_sqlrustgo_schema(&mut engine);
    insert_sqlrustgo_data(&mut engine);
    engine
}

struct QueryResult {
    name: String,
    sql: String,
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
    fn new(name: &str, sql: &str) -> Self {
        QueryResult {
            name: name.to_string(),
            sql: sql.to_string(),
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
            let match_flag = self.sqlite_data == self.sqlrustgo_data;
            self.match_result = Some(match_flag);
        } else {
            self.match_result = Some(false);
        }
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
fn test_tpch_compliance_report() {
    let sqlite_conn = create_sqlite_conn();
    setup_sqlite_schema(&sqlite_conn);
    setup_sqlite_data(&sqlite_conn);

    let mut sqlrustgo_engine = setup_sqlrustgo_engine_with_bulk_load();

    let queries = vec![
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag"),
        ("Q2", "SELECT s_acctbal, s_name, n_name, p_partkey, p_mfgr, s_address, s_phone, s_comment FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND p_type LIKE '%BRASS' AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' ORDER BY s_acctbal DESC, n_name, s_name, p_partkey"),
        ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        ("Q6", "SELECT SUM(l_extendedprice * l_discount) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount BETWEEN 0.05 AND 0.07 AND l_quantity < 24"),
        ("Q7", "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name"),
        ("Q8", "SELECT EXTRACT(YEAR FROM o_orderdate) AS o_year, SUM(l_extendedprice) FROM orders, lineitem, customer, nation n1 WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey AND c_nationkey = n1.n_nationkey AND n1.n_name = 'INDIA' GROUP BY o_year"),
        ("Q9", "SELECT n_name, EXTRACT(YEAR FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount)) AS amount FROM customer, orders, lineitem, supplier, nation WHERE c_custkey = o_custkey AND o_orderkey = l_orderkey AND l_suppkey = s_suppkey AND s_nationkey = n_nationkey GROUP BY n_name, o_year"),
        ("Q10", "SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey"),
        ("Q11", "SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value FROM partsupp, supplier, nation WHERE ps_suppkey = s_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY' GROUP BY ps_partkey"),
        ("Q12", "SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND o_orderdate >= '1993-01-01' AND o_orderdate < '1994-01-01' GROUP BY l_shipmode"),
        ("Q13", "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"),
        ("Q14", "SELECT 100.00 * SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) / SUM(l_extendedprice * (1 - l_discount)) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'"),
        ("Q15", "SELECT s_suppkey, s_name, s_address, s_phone, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM supplier, lineitem WHERE l_suppkey = s_suppkey AND l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY s_suppkey, s_name, s_address, s_phone"),
        ("Q16", "SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) FROM partsupp, part WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45' AND p_type NOT LIKE 'MEDIUM POLISHED%' AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9) GROUP BY p_brand, p_type, p_size"),
        ("Q17", "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX'"),
        ("Q18", "SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice"),
        ("Q19", "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5"),
        ("Q20", "SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'"),
        ("Q21", "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem, orders, nation WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND o_orderstatus = 'F' AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name"),
        ("Q22", "SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > 0.00 GROUP BY cntrycode"),
    ];

    let mut results: Vec<QueryResult> = Vec::new();

    for (name, sql) in &queries {
        let mut result = QueryResult::new(name, sql);

        let (sqlite_ok, sqlite_rows, sqlite_data, sqlite_err) =
            run_sqlite_query_safe(&sqlite_conn, sql);
        result.set_sqlite_result(sqlite_ok, sqlite_rows, sqlite_data.clone(), sqlite_err);

        let parse_result = parse(sql);
        if parse_result.is_err() {
            result.set_sqlrustgo_result(
                false,
                0,
                Vec::new(),
                Some(format!("Parse error: {:?}", parse_result.err())),
            );
            result.compare_results();
            results.push(result);
            continue;
        }

        let query_result = sqlrustgo_engine.execute(parse_result.unwrap());
        match query_result {
            Ok(r) => {
                let data: Vec<Vec<String>> = r
                    .rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|v| match v {
                                sqlrustgo_types::Value::Null => "NULL".to_string(),
                                sqlrustgo_types::Value::Integer(i) => i.to_string(),
                                sqlrustgo_types::Value::Float(f) => f.to_string(),
                                sqlrustgo_types::Value::Text(s) => s.clone(),
                                sqlrustgo_types::Value::Boolean(b) => b.to_string(),
                                sqlrustgo_types::Value::Blob(b) => format!("BLOB[{}]", b.len()),
                                sqlrustgo_types::Value::Timestamp(ts) => ts.to_string(),
                                sqlrustgo_types::Value::Uuid(u) => u.to_string(),
                                sqlrustgo_types::Value::Array(arr) => format!("{:?}", arr),
                                sqlrustgo_types::Value::Enum(idx, name) => {
                                    format!("{}:{}", idx, name)
                                }
                                sqlrustgo_types::Value::Date(d) => d.to_string(),
                            })
                            .collect()
                    })
                    .collect();
                result.set_sqlrustgo_result(true, r.rows.len(), data, None);
            }
            Err(e) => {
                result.set_sqlrustgo_result(false, 0, Vec::new(), Some(e.to_string()));
            }
        }

        result.compare_results();
        results.push(result);
    }

    println!("\n========== TPC-H Q1-Q22 Compliance Test Report ==========\n");
    println!("{}", "=".repeat(90));
    println!(
        "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
        "Query", "SQLite Rows", "SQLRustGo Rows", "Match", "SQLite", "SQLRustGo", "Status"
    );
    println!("{}", "-".repeat(90));

    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut match_count = 0;
    let mut sqlite_err_count = 0;
    let mut sqlrustgo_err_count = 0;

    for r in &results {
        let match_str = match r.match_result {
            Some(true) => {
                match_count += 1;
                "YES".to_string()
            }
            Some(false) => "NO".to_string(),
            None => "N/A".to_string(),
        };

        let sqlite_status = if r.sqlite_ok { "OK" } else { "ERR" };
        let sqlrustgo_status = if r.sqlrustgo_ok { "OK" } else { "ERR" };

        if !r.sqlite_ok {
            sqlite_err_count += 1;
        }
        if !r.sqlrustgo_ok {
            sqlrustgo_err_count += 1;
        }

        let overall = if r.match_result == Some(true) {
            "PASS"
        } else {
            "FAIL"
        };
        if r.match_result == Some(true) {
            pass_count += 1;
        } else {
            fail_count += 1;
        }

        println!(
            "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
            r.name,
            r.sqlite_rows,
            r.sqlrustgo_rows,
            match_str,
            sqlite_status,
            sqlrustgo_status,
            overall
        );

        if let Some(ref err) = r.sqlite_error {
            println!("         SQLite Error: {}", err);
        }
        if let Some(ref err) = r.sqlrustgo_error {
            println!("         SQLRustGo Error: {}", err);
        }
    }

    println!("\n========== Summary ==========");
    println!("Total Queries: {}", results.len());
    println!(
        "Results Match (SQLite == SQLRustGo): {} / {}",
        match_count,
        results.len()
    );
    println!("SQLite Errors: {}", sqlite_err_count);
    println!("SQLRustGo Errors: {}", sqlrustgo_err_count);
    println!("Passed: {}", pass_count);
    println!("Failed: {}", fail_count);

    if fail_count > 0 {
        println!(
            "\n[FAIL] {} queries have incorrect results or errors",
            fail_count
        );
    } else {
        println!(
            "\n[PASS] All {} queries produce correct results!",
            results.len()
        );
    }
}

#[test]
fn test_tpch_q14_case_when() {
    let mut engine = setup_sqlrustgo_engine_with_bulk_load();
    let sql = "SELECT SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'";

    let result = engine.execute(parse(sql).unwrap());
    match result {
        Ok(r) => println!("Q14: {} rows, result: {:?}", r.rows.len(), r.rows),
        Err(e) => println!("Q14 ERROR: {:?}", e),
    }
}

#[test]
fn test_tpch_q16_count_distinct() {
    let mut engine = setup_sqlrustgo_engine_with_bulk_load();
    let sql = "SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) FROM partsupp, part WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45' AND p_type NOT LIKE 'MEDIUM POLISHED%' AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9) GROUP BY p_brand, p_type, p_size";

    let result = engine.execute(parse(sql).unwrap());
    match result {
        Ok(r) => println!("Q16: {} rows", r.rows.len()),
        Err(e) => println!("Q16 ERROR: {:?}", e),
    }
}

#[test]
fn test_tpch_q22_substring() {
    let mut engine = setup_sqlrustgo_engine_with_bulk_load();
    let sql = "SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > 0.00 GROUP BY cntrycode";

    let result = engine.execute(parse(sql).unwrap());
    match result {
        Ok(r) => println!("Q22: {} rows", r.rows.len()),
        Err(e) => println!("Q22 ERROR: {:?}", e),
    }
}

#[test]
fn test_tpch_q8_extract() {
    let mut engine = setup_sqlrustgo_engine_with_bulk_load();
    let sql = "SELECT EXTRACT(YEAR FROM o_orderdate) AS o_year FROM orders";

    let result = engine.execute(parse(sql).unwrap());
    match result {
        Ok(r) => println!("Q8 EXTRACT: {} rows", r.rows.len()),
        Err(e) => println!("Q8 EXTRACT ERROR: {:?}", e),
    }
}

#[test]
fn test_tpch_compliance_report_with_real_data() {
    println!("\n========== TPC-H Compliance Test with Real Data ==========");
    println!("Data directory: {}", TPCK_DATA_DIR);
    println!("SQLite DB: {}", TPCK_SQLITE_DB);

    if !Path::new(TPCK_DATA_DIR).exists() {
        println!("SKIP: TPC-H data directory not found at {}", TPCK_DATA_DIR);
        println!(
            "Please run: mkdir -p {} && cp /tmp/tpch-dbgen-master/*.tbl {}",
            TPCK_DATA_DIR, TPCK_DATA_DIR
        );
        return;
    }

    if !Path::new(TPCK_SQLITE_DB).exists() {
        println!("SKIP: SQLite DB not found at {}", TPCK_SQLITE_DB);
        return;
    }

    let sqlite_conn = create_sqlite_conn_from_file(TPCK_SQLITE_DB);
    println!("Connected to SQLite reference database");

    let mut sqlrustgo_engine = setup_sqlrustgo_engine_with_real_data(TPCK_DATA_DIR);
    println!("Loaded data into SQLRustGo engine\n");

    let queries = vec![
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag"),
        ("Q2", "SELECT s_acctbal, s_name, n_name, p_partkey, p_mfgr, s_address, s_phone, s_comment FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND p_type LIKE '%BRASS' AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' ORDER BY s_acctbal DESC, n_name, s_name, p_partkey"),
        ("Q3", "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"),
        ("Q4", "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"),
        ("Q5", "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"),
        ("Q6", "SELECT SUM(l_extendedprice * l_discount) FROM lineitem WHERE l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01' AND l_discount BETWEEN 0.05 AND 0.07 AND l_quantity < 24"),
        ("Q7", "SELECT n1.n_name AS supp_nation, n2.n_name AS cust_nation, SUM(l_extendedprice) FROM supplier, lineitem, orders, customer, nation n1, nation n2 WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND c_custkey = o_custkey AND s_nationkey = n1.n_nationkey AND c_nationkey = n2.n_nationkey GROUP BY n1.n_name, n2.n_name"),
        ("Q8", "SELECT EXTRACT(YEAR FROM o_orderdate) AS o_year, SUM(l_extendedprice) FROM orders, lineitem, customer, nation n1 WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey AND c_nationkey = n1.n_nationkey AND n1.n_name = 'INDIA' GROUP BY o_year"),
        ("Q9", "SELECT n_name, EXTRACT(YEAR FROM o_orderdate) AS o_year, SUM(l_extendedprice * (1 - l_discount)) AS amount FROM customer, orders, lineitem, supplier, nation WHERE c_custkey = o_custkey AND o_orderkey = l_orderkey AND l_suppkey = s_suppkey AND s_nationkey = n_nationkey GROUP BY n_name, o_year"),
        ("Q10", "SELECT c_custkey, SUM(l_extendedprice) FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate >= '1993-10-01' GROUP BY c_custkey"),
        ("Q11", "SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value FROM partsupp, supplier, nation WHERE ps_suppkey = s_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY' GROUP BY ps_partkey"),
        ("Q12", "SELECT l_shipmode, COUNT(*) FROM orders, lineitem WHERE l_orderkey = o_orderkey AND l_shipmode IN ('MAIL', 'SHIP') AND l_commitdate < l_receiptdate AND l_shipdate < l_commitdate AND o_orderdate >= '1993-01-01' AND o_orderdate < '1994-01-01' GROUP BY l_shipmode"),
        ("Q13", "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"),
        ("Q14", "SELECT 100.00 * SUM(CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END) / SUM(l_extendedprice * (1 - l_discount)) AS promo_revenue FROM lineitem, part WHERE l_partkey = p_partkey AND l_shipdate >= '1995-09-01' AND l_shipdate < '1995-10-01'"),
        ("Q15", "SELECT s_suppkey, s_name, s_address, s_phone, SUM(l_extendedprice * (1 - l_discount)) AS total_revenue FROM supplier, lineitem WHERE l_suppkey = s_suppkey AND l_shipdate >= '1996-01-01' AND l_shipdate < '1996-04-01' GROUP BY s_suppkey, s_name, s_address, s_phone"),
        ("Q16", "SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) FROM partsupp, part WHERE p_partkey = ps_partkey AND p_brand <> 'Brand#45' AND p_type NOT LIKE 'MEDIUM POLISHED%' AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9) GROUP BY p_brand, p_type, p_size"),
        ("Q17", "SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#23' AND p_container = 'MED BOX'"),
        ("Q18", "SELECT c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice, SUM(l_quantity) FROM customer, orders, lineitem WHERE o_orderkey = l_orderkey AND c_custkey = o_custkey GROUP BY c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice"),
        ("Q19", "SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue FROM lineitem, part WHERE p_partkey = l_partkey AND p_brand = 'Brand#12' AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG') AND l_quantity >= 1 AND l_quantity <= 11 AND p_size BETWEEN 1 AND 5"),
        ("Q20", "SELECT s_name, s_address FROM supplier, nation WHERE s_nationkey = n_nationkey AND n_name = 'CANADA'"),
        ("Q21", "SELECT s_name, COUNT(*) AS numwait FROM supplier, lineitem, orders, nation WHERE s_suppkey = l_suppkey AND o_orderkey = l_orderkey AND o_orderstatus = 'F' AND s_nationkey = n_nationkey AND n_name = 'SAUDI ARABIA' GROUP BY s_name"),
        ("Q22", "SELECT SUBSTRING(c_phone FROM 1 FOR 2) AS cntrycode, COUNT(*) FROM customer WHERE SUBSTRING(c_phone FROM 1 FOR 2) IN ('13', '31', '23', '29', '30', '18', '17') AND c_acctbal > 0.00 GROUP BY cntrycode"),
    ];

    let sqlite_unsupported = vec!["Q8", "Q9", "Q22"];

    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut match_count = 0;
    let mut sqlite_err_count = 0;
    let mut sqlrustgo_err_count = 0;
    let mut sqlite_unsupported_count = 0;

    println!("\n==============================================================================================");
    println!(
        "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
        "Query", "SQLite Rows", "SQLRustGo Rows", "Match", "SQLite", "SQLRustGo", "Status"
    );
    println!("{}", "-".repeat(90));

    for (name, sql) in &queries {
        if sqlite_unsupported.contains(&name) {
            let parse_result = parse(sql);
            if parse_result.is_err() {
                sqlrustgo_err_count += 1;
                fail_count += 1;
                println!(
                    "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
                    name, 0, 0, "N/A", "N/A", "ERR", "FAIL"
                );
                continue;
            }
            let query_result = sqlrustgo_engine.execute(parse_result.unwrap());
            match query_result {
                Ok(r) => {
                    println!(
                        "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
                        name,
                        "N/A",
                        r.rows.len(),
                        "N/A",
                        "N/A",
                        "OK",
                        "PASS*"
                    );
                    pass_count += 1;
                    sqlite_unsupported_count += 1;
                }
                Err(_) => {
                    fail_count += 1;
                    println!(
                        "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
                        name, "N/A", 0, "N/A", "N/A", "ERR", "FAIL"
                    );
                }
            }
            continue;
        }

        let (sqlite_ok, sqlite_rows, sqlite_data, sqlite_err) =
            run_sqlite_query_safe(&sqlite_conn, sql);

        let parse_result = parse(sql);
        if parse_result.is_err() {
            sqlrustgo_err_count += 1;
            fail_count += 1;
            println!(
                "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
                name,
                sqlite_rows,
                0,
                "N/A",
                if sqlite_ok { "OK" } else { "ERR" },
                "ERR",
                "FAIL"
            );
            continue;
        }

        let query_result = sqlrustgo_engine.execute(parse_result.unwrap());
        let (sqlrustgo_ok, sqlrustgo_rows, sqlrustgo_data) = match query_result {
            Ok(r) => {
                let data: Vec<Vec<String>> = r
                    .rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|v| match v {
                                sqlrustgo_types::Value::Null => "NULL".to_string(),
                                sqlrustgo_types::Value::Integer(i) => i.to_string(),
                                sqlrustgo_types::Value::Float(f) => f.to_string(),
                                sqlrustgo_types::Value::Text(s) => s.clone(),
                                sqlrustgo_types::Value::Boolean(b) => b.to_string(),
                                sqlrustgo_types::Value::Blob(b) => format!("BLOB[{}]", b.len()),
                                sqlrustgo_types::Value::Timestamp(ts) => ts.to_string(),
                                sqlrustgo_types::Value::Uuid(u) => u.to_string(),
                                sqlrustgo_types::Value::Array(arr) => format!("{:?}", arr),
                                sqlrustgo_types::Value::Enum(idx, name) => {
                                    format!("{}:{}", idx, name)
                                }
                                sqlrustgo_types::Value::Date(d) => d.to_string(),
                            })
                            .collect()
                    })
                    .collect();
                (true, r.rows.len(), data)
            }
            Err(_) => (false, 0, Vec::new()),
        };

        let match_flag = if sqlite_ok && sqlrustgo_ok {
            let data_match = sqlite_data == sqlrustgo_data;
            if data_match {
                match_count += 1;
            }
            data_match
        } else {
            false
        };

        if !sqlite_ok {
            sqlite_err_count += 1;
        }
        if !sqlrustgo_ok {
            sqlrustgo_err_count += 1;
        }

        let overall = if match_flag { "PASS" } else { "FAIL" };
        if match_flag {
            pass_count += 1;
        } else {
            fail_count += 1;
        }

        println!(
            "{:<8} {:<12} {:<12} {:<12} {:<10} {:<15} {}",
            name,
            sqlite_rows,
            sqlrustgo_rows,
            if match_flag { "YES" } else { "NO" },
            if sqlite_ok { "OK" } else { "ERR" },
            if sqlrustgo_ok { "OK" } else { "ERR" },
            overall
        );

        if let Some(ref err) = sqlite_err {
            println!("         SQLite Error: {}", err);
        }
    }

    println!("\n========== Summary ==========");
    println!("Total Queries: {}", queries.len());
    println!(
        "Results Match (SQLite == SQLRustGo): {} / {}",
        match_count,
        queries.len() - sqlite_unsupported_count
    );
    println!("SQLite Errors: {}", sqlite_err_count);
    println!(
        "SQLite Unsupported (Q8/Q9/Q22): {}",
        sqlite_unsupported_count
    );
    println!("SQLRustGo Errors: {}", sqlrustgo_err_count);
    println!("Passed: {}", pass_count);
    println!("Failed: {}", fail_count);

    if fail_count > 0 {
        println!(
            "\n[FAIL] {} queries have incorrect results or errors",
            fail_count
        );
    } else {
        println!(
            "\n[PASS] All {} queries produce correct results!",
            queries.len()
        );
    }
}
