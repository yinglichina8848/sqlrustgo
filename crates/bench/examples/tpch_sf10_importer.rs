//! TPC-H SF=10 Data Importer
//!
//! Efficiently imports SF=10 TPC-H data into SQLite for SQLRustGo
//!
//! Usage:
//!   cargo run -p sqlrustgo-bench --example tpch_sf10_importer -- --scale 10 --output /path/to/data.db

use rusqlite::{Connection, params};
use std::time::Instant;
use rand::Rng;
use std::fs;

/// TPC-H SF=10 data scale factors
const SF10_LINEITEM: usize = 60_000_000;
const SF10_ORDERS: usize = 15_000_000;
const SF10_CUSTOMER: usize = 1_500_000;
const SF10_PART: usize = 2_000_000;
const SF10_SUPPLIER: usize = 100_000;
const SF10_PARTSUPP: usize = 8_000_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scale = if args.len() > 2 && args[1] == "--scale" {
        args[2].parse().unwrap_or(10)
    } else {
        10
    };
    let output_path = if args.len() > 4 && args[3] == "--output" {
        args[4].clone()
    } else {
        format!("tpch_sf{}.db", scale)
    };

    println!("==============================================");
    println!("  TPC-H SF{} Data Importer", scale);
    println!("==============================================");
    println!("Output: {}", output_path);
    println!();

    // Remove existing database
    if fs::metadata(&output_path).is_ok() {
        println!("Removing existing database...");
        fs::remove_file(&output_path).unwrap();
    }

    // Create connection
    println!("Creating database...");
    let conn = Connection::open(&output_path).unwrap();
    
    // Set performance pragmas
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = OFF;
         PRAGMA cache_size = -2000000;
         PRAGMA temp_store = MEMORY;
         PRAGMA main.page_size = 4096;
         PRAGMA main.freelist_threshold = 0;"
    ).unwrap();

    let total_start = Instant::now();

    // Create tables
    create_tables(&conn);
    
    // Import data
    import_nation_region(&conn);
    import_part(&conn, scale);
    import_supplier(&conn, scale);
    import_partsupp(&conn, scale);
    import_customer(&conn, scale);
    import_orders(&conn, scale);
    import_lineitem(&conn, scale);

    println!();
    println!("==============================================");
    println!("  Total time: {:.2}s", total_start.elapsed().as_secs_f64());
    println!("==============================================");
}

fn create_tables(conn: &Connection) {
    println!("Creating tables...");
    
    conn.execute_batch(
        "CREATE TABLE region (
            r_regionkey INTEGER PRIMARY KEY,
            r_name TEXT NOT NULL,
            r_comment TEXT
        );
        
        CREATE TABLE nation (
            n_nationkey INTEGER PRIMARY KEY,
            n_name TEXT NOT NULL,
            n_regionkey INTEGER NOT NULL,
            n_comment TEXT,
            FOREIGN KEY (n_regionkey) REFERENCES region(r_regionkey)
        );
        
        CREATE TABLE supplier (
            s_suppkey INTEGER PRIMARY KEY,
            s_name TEXT NOT NULL,
            s_address TEXT NOT NULL,
            s_nationkey INTEGER NOT NULL,
            s_phone TEXT NOT NULL,
            s_acctbal REAL NOT NULL,
            s_comment TEXT,
            FOREIGN KEY (s_nationkey) REFERENCES nation(n_nationkey)
        );
        
        CREATE TABLE part (
            p_partkey INTEGER PRIMARY KEY,
            p_name TEXT NOT NULL,
            p_mfgr TEXT NOT NULL,
            p_brand TEXT NOT NULL,
            p_type TEXT NOT NULL,
            p_size INTEGER NOT NULL,
            p_container TEXT NOT NULL,
            p_retailprice REAL NOT NULL,
            p_comment TEXT
        );
        
        CREATE TABLE partsupp (
            ps_partkey INTEGER NOT NULL,
            ps_suppkey INTEGER NOT NULL,
            ps_availqty INTEGER NOT NULL,
            ps_supplycost REAL NOT NULL,
            ps_comment TEXT,
            PRIMARY KEY (ps_partkey, ps_suppkey),
            FOREIGN KEY (ps_partkey) REFERENCES part(p_partkey),
            FOREIGN KEY (ps_suppkey) REFERENCES supplier(s_suppkey)
        );
        
        CREATE TABLE customer (
            c_custkey INTEGER PRIMARY KEY,
            c_name TEXT NOT NULL,
            c_address TEXT NOT NULL,
            c_nationkey INTEGER NOT NULL,
            c_phone TEXT NOT NULL,
            c_acctbal REAL NOT NULL,
            c_mktsegment TEXT NOT NULL,
            c_comment TEXT,
            FOREIGN KEY (c_nationkey) REFERENCES nation(n_nationkey)
        );
        
        CREATE TABLE orders (
            o_orderkey INTEGER PRIMARY KEY,
            o_custkey INTEGER NOT NULL,
            o_orderstatus TEXT NOT NULL,
            o_totalprice REAL NOT NULL,
            o_orderdate TEXT NOT NULL,
            o_orderpriority TEXT NOT NULL,
            o_clerk TEXT NOT NULL,
            o_shippriority INTEGER NOT NULL,
            o_comment TEXT,
            FOREIGN KEY (o_custkey) REFERENCES customer(c_custkey)
        );
        
        CREATE TABLE lineitem (
            l_orderkey INTEGER NOT NULL,
            l_partkey INTEGER NOT NULL,
            l_suppkey INTEGER NOT NULL,
            l_linenumber INTEGER NOT NULL,
            l_quantity REAL NOT NULL,
            l_extendedprice REAL NOT NULL,
            l_discount REAL NOT NULL,
            l_tax REAL NOT NULL,
            l_returnflag TEXT NOT NULL,
            l_linestatus TEXT NOT NULL,
            l_shipdate TEXT NOT NULL,
            l_commitdate TEXT NOT NULL,
            l_receiptdate TEXT NOT NULL,
            l_shipinstruct TEXT NOT NULL,
            l_shipmode TEXT NOT NULL,
            l_comment TEXT,
            PRIMARY KEY (l_orderkey, l_linenumber),
            FOREIGN KEY (l_orderkey) REFERENCES orders(o_orderkey),
            FOREIGN KEY (l_partkey) REFERENCES part(p_partkey),
            FOREIGN KEY (l_suppkey) REFERENCES supplier(s_suppkey)
        );
        
        CREATE INDEX idx_lineitem_orderkey ON lineitem(l_orderkey);
        CREATE INDEX idx_lineitem_partkey ON lineitem(l_partkey);
        CREATE INDEX idx_orders_custkey ON orders(o_custkey);
        CREATE INDEX idx_partsupp_partkey ON partsupp(ps_partkey);
        CREATE INDEX idx_partsupp_suppkey ON partsupp(ps_suppkey);"
    ).unwrap();
}

fn import_nation_region(conn: &Connection) {
    println!("Importing nation and region...");
    
    conn.execute("INSERT INTO region VALUES (0, 'AFRICA', 'lar deposits')", []).unwrap();
    conn.execute("INSERT INTO region VALUES (1, 'AMERICA', 'alanced eage')", []).unwrap();
    conn.execute("INSERT INTO region VALUES (2, 'ASIA', 'ent deposits')", []).unwrap();
    conn.execute("INSERT INTO region VALUES (3, 'EUROPE', 'ically cold')", []).unwrap();
    conn.execute("INSERT INTO region VALUES (4, 'MIDDLE EAST', 'quires acros')", []).unwrap();
    
    let nations = [
        (0, "ALGERIA", 0), (1, "ARGENTINA", 1), (2, "BRAZIL", 1),
        (3, "CANADA", 1), (4, "EGYPT", 4), (5, "ETHIOPIA", 0),
        (6, "FRANCE", 3), (7, "GERMANY", 3), (8, "INDIA", 2),
        (9, "INDONESIA", 2), (10, "IRAN", 4), (11, "IRAQ", 4),
        (12, "JAPAN", 2), (13, "JORDAN", 4), (14, "KENYA", 0),
        (15, "MOROCCO", 0), (16, "MOZAMBIQUE", 0), (17, "PERU", 1),
        (18, "CHINA", 2), (19, "ROMANIA", 3), (20, "SAUDI ARABIA", 4),
        (21, "VIETNAM", 2), (22, "RUSSIA", 3), (23, "UNITED KINGDOM", 3),
        (24, "UNITED STATES", 1),
    ];
    
    for (key, name, region) in nations {
        conn.execute(
            "INSERT INTO nation VALUES (?1, ?2, ?3, '')",
            params![key, name, region],
        ).unwrap();
    }
}

fn import_part(conn: &Connection, scale: usize) {
    let count = SF10_PART * scale / 10;
    println!("Importing part ({} rows)...", count);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let mfg_parts = ["MFGR#1", "MFGR#2", "MFGR#3", "MFGR#4", "MFGR#5"];
    let brands = ["Brand#10", "Brand#20", "Brand#30", "Brand#40", "Brand#50"];
    let types = ["STANDARD", "SMALL", "MEDIUM", "LARGE", "ECONOMY"];
    let containers = ["SM CASE", "LG CASE", "LG BOX", "SM BOX", "WRAP"];
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, String, String, String, String, i32, String, f64, String)> = Vec::with_capacity(batch_size);
    
    for i in 1..=count {
        batch.push((
            i as i32,
            format!("Part{}", i),
            mfg_parts[rng.gen_range(0..5)].to_string(),
            brands[rng.gen_range(0..5)].to_string(),
            types[rng.gen_range(0..5)].to_string(),
            rng.gen_range(1..51),
            containers[rng.gen_range(0..5)].to_string(),
            rng.gen_range(900.0..1050.0),
            format!("Comment{}", i),
        ));
        
        if batch.len() >= batch_size {
            insert_part_batch(conn, &batch);
            batch.clear();
            print_progress(i, count, start.elapsed().as_secs_f64());
        }
    }
    if !batch.is_empty() {
        insert_part_batch(conn, &batch);
    }
    println!();
}

fn insert_part_batch(conn: &Connection, batch: &[(i32, String, String, String, String, i32, String, f64, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO part VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8
        ]).unwrap();
    }
}

fn import_supplier(conn: &Connection, scale: usize) {
    let count = SF10_SUPPLIER * scale / 10;
    println!("Importing supplier ({} rows)...", count);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, String, String, i32, String, f64, String)> = Vec::with_capacity(batch_size);
    
    for i in 1..=count {
        batch.push((
            i as i32,
            format!("Supplier#{:08}", i),
            format!("Address{}", i),
            rng.gen_range(0..25),
            format!("{}-####-####", i % 10000),
            rng.gen_range(-5000.0..11000.0),
            format!("Comment{}", i),
        ));
        
        if batch.len() >= batch_size {
            insert_supplier_batch(conn, &batch);
            batch.clear();
            print_progress(i, count, start.elapsed().as_secs_f64());
        }
    }
    if !batch.is_empty() {
        insert_supplier_batch(conn, &batch);
    }
    println!();
}

fn insert_supplier_batch(conn: &Connection, batch: &[(i32, String, String, i32, String, f64, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO supplier VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![
            row.0, row.1, row.2, row.3, row.4, row.5, row.6
        ]).unwrap();
    }
}

fn import_partsupp(conn: &Connection, scale: usize) {
    let num_parts = SF10_PART * scale / 10;
    println!("Importing partsupp ({} parts, 1-4 suppliers each)...", num_parts);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, i32, i32, f64, String)> = Vec::with_capacity(batch_size);
    let mut row_count = 0;
    
    for partkey in 1..=num_parts as i32 {
        // Each part has 1-4 suppliers
        let num_supps = rng.gen_range(1..5);
        let mut used_supps = Vec::with_capacity(num_supps);
        
        for _ in 0..num_supps {
            // Generate unique supplier for this part
            let mut suppkey;
            loop {
                suppkey = rng.gen_range(1..=(SF10_SUPPLIER * scale / 10) as i32);
                if !used_supps.contains(&suppkey) {
                    used_supps.push(suppkey);
                    break;
                }
            }
            
            batch.push((
                partkey,
                suppkey,
                rng.gen_range(1..10000),
                rng.gen_range(1.0..1000.0),
                format!("Comment{}", row_count),
            ));
            row_count += 1;
            
            if batch.len() >= batch_size {
                insert_partsupp_batch(conn, &batch);
                batch.clear();
                print_progress(partkey as usize, num_parts, start.elapsed().as_secs_f64());
            }
        }
    }
    if !batch.is_empty() {
        insert_partsupp_batch(conn, &batch);
    }
    println!(" ({} partsupp rows generated)", row_count);
}

fn insert_partsupp_batch(conn: &Connection, batch: &[(i32, i32, i32, f64, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO partsupp VALUES (?1, ?2, ?3, ?4, ?5)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![row.0, row.1, row.2, row.3, row.4]).unwrap();
    }
}

fn import_customer(conn: &Connection, scale: usize) {
    let count = SF10_CUSTOMER * scale / 10;
    println!("Importing customer ({} rows)...", count);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let segments = ["AUTOMOBILE", "BUILDING", "FURNITURE", "HOUSEHOLD", "MACHINERY"];
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, String, String, i32, String, f64, String, String)> = Vec::with_capacity(batch_size);
    
    for i in 1..=count {
        batch.push((
            i as i32,
            format!("Customer#{:09}", i),
            format!("Address{}", i),
            rng.gen_range(0..25),
            format!("{}-###-####", i % 10000),
            rng.gen_range(-5000.0..11000.0),
            segments[rng.gen_range(0..5)].to_string(),
            format!("Comment{}", i),
        ));
        
        if batch.len() >= batch_size {
            insert_customer_batch(conn, &batch);
            batch.clear();
            print_progress(i, count, start.elapsed().as_secs_f64());
        }
    }
    if !batch.is_empty() {
        insert_customer_batch(conn, &batch);
    }
    println!();
}

fn insert_customer_batch(conn: &Connection, batch: &[(i32, String, String, i32, String, f64, String, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO customer VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7
        ]).unwrap();
    }
}

fn import_orders(conn: &Connection, scale: usize) {
    let count = SF10_ORDERS * scale / 10;
    println!("Importing orders ({} rows)...", count);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let statuses = ["O", "P", "F"];
    let priorities = ["1-URGENT", "2-HIGH", "3-MEDIUM", "4-LOW", "5-NOT SPECIFIED"];
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, i32, String, f64, String, String, String, i32, String)> = Vec::with_capacity(batch_size);
    
    for i in 1..=count {
        let base_date = 87600; // 1992-01-01
        let days_offset = rng.gen_range(0..2500);
        let orderdate = format!("{:04}-{:02}-{:02}", 
            1992 + days_offset / 365,
            (days_offset % 365) / 30 + 1,
            days_offset % 30 + 1
        );
        
        batch.push((
            i as i32,
            rng.gen_range(1..=(SF10_CUSTOMER * scale / 10) as i32),
            statuses[rng.gen_range(0..3)].to_string(),
            (rng.gen_range(0..100000) as f64) * 100.0,
            orderdate,
            priorities[rng.gen_range(0..5)].to_string(),
            format!("Clerk#{:08}", rng.gen_range(0..100)),
            0,
            format!("Comment{}", i),
        ));
        
        if batch.len() >= batch_size {
            insert_orders_batch(conn, &batch);
            batch.clear();
            print_progress(i, count, start.elapsed().as_secs_f64());
        }
    }
    if !batch.is_empty() {
        insert_orders_batch(conn, &batch);
    }
    println!();
}

fn insert_orders_batch(conn: &Connection, batch: &[(i32, i32, String, f64, String, String, String, i32, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO orders VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8
        ]).unwrap();
    }
}

fn import_lineitem(conn: &Connection, scale: usize) {
    let count = SF10_LINEITEM * scale / 10;
    println!("Importing lineitem ({} rows)...", count);
    
    let start = Instant::now();
    let mut rng = rand::thread_rng();
    
    let return_flags = ["N", "R", "A"];
    let ship_instrs = ["DELIVER IN PERSON", "COLLECT COD", "NONE", "TAKE BACK RETURN"];
    let ship_modes = ["AIR", "TRUCK", "RAIL", "SHIP", "REG AIR", "FOB"];
    
    let batch_size = 10000;
    let mut batch: Vec<(i32, i32, i32, i32, f64, f64, f64, f64, String, String, String, String, String, String, String, String)> = Vec::with_capacity(batch_size);
    
    for i in 1..=count {
        let orderkey = rng.gen_range(1..=(SF10_ORDERS * scale / 10) as i32);
        let linenumber = rng.gen_range(1..8);
        
        let base_date = 87600;
        let ship_offset = rng.gen_range(0..500);
        let shipdate = format!("{:04}-{:02}-{:02}",
            1992 + ship_offset / 365,
            (ship_offset % 365) / 30 + 1,
            ship_offset % 30 + 1
        );
        let commit_offset = ship_offset + rng.gen_range(0..30);
        let commitdate = format!("{:04}-{:02}-{:02}",
            1992 + commit_offset / 365,
            (commit_offset % 365) / 30 + 1,
            commit_offset % 30 + 1
        );
        let receipt_offset = commit_offset + rng.gen_range(0..30);
        let receiptdate = format!("{:04}-{:02}-{:02}",
            1992 + receipt_offset / 365,
            (receipt_offset % 365) / 30 + 1,
            receipt_offset % 30 + 1
        );
        
        batch.push((
            orderkey,
            rng.gen_range(1..=(SF10_PART * scale / 10) as i32),
            rng.gen_range(1..=(SF10_SUPPLIER * scale / 10) as i32),
            linenumber,
            rng.gen_range(1.0..50.0),
            rng.gen_range(100.0..10000.0),
            rng.gen_range(0.0..0.10),
            rng.gen_range(0.0..0.10),
            return_flags[rng.gen_range(0..3)].to_string(),
            "O".to_string(),
            shipdate,
            commitdate,
            receiptdate,
            ship_instrs[rng.gen_range(0..4)].to_string(),
            ship_modes[rng.gen_range(0..6)].to_string(),
            format!("Comment{}", i),
        ));
        
        if batch.len() >= batch_size {
            insert_lineitem_batch(conn, &batch);
            batch.clear();
            print_progress(i, count, start.elapsed().as_secs_f64());
        }
    }
    if !batch.is_empty() {
        insert_lineitem_batch(conn, &batch);
    }
    println!();
}

fn insert_lineitem_batch(conn: &Connection, batch: &[(i32, i32, i32, i32, f64, f64, f64, f64, String, String, String, String, String, String, String, String)]) {
    let mut stmt = conn.prepare(
        "INSERT INTO lineitem VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
    ).unwrap();
    
    for row in batch {
        stmt.execute(params![
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9,
            row.10, row.11, row.12, row.13, row.14, row.15
        ]).unwrap();
    }
}

fn print_progress(current: usize, total: usize, elapsed_secs: f64) {
    let pct = (current as f64) / (total as f64) * 100.0;
    let rate = current as f64 / elapsed_secs;
    let remaining = (total - current) as f64 / rate;
    print!("\r  Progress: {:6.2}% | {:>12} / {:>12} rows | {:.0} rows/s | ETA: {:.0}s  ", 
        pct, current, total, rate, remaining);
}
