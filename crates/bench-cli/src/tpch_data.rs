use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::Arc;

pub fn generate_tpch_data(scale_factor: f64) -> Arc<MemoryStorage> {
    let mut storage = MemoryStorage::new();

    let customer_count = (1500.0 * scale_factor) as i64;
    let order_count = (15000.0 * scale_factor) as i64;
    let lineitem_count = (60000.0 * scale_factor) as i64;
    let part_count = (2000.0 * scale_factor) as i64;
    let supplier_count = (100.0 * scale_factor) as i64;
    let partsupp_count = part_count * supplier_count / 10;

    println!("Generating TPC-H data for SF{}...", scale_factor);
    println!("  customer: {} rows", customer_count);
    println!("  orders: {} rows", order_count);
    println!("  lineitem: {} rows", lineitem_count);
    println!("  part: {} rows", part_count);
    println!("  supplier: {} rows", supplier_count);
    println!("  partsupp: {} rows", partsupp_count);
    println!("  nation: 25 rows");
    println!("  region: 5 rows");

    let mut customers = Vec::new();
    for i in 0..customer_count {
        let c_custkey = i + 1;
        let c_name = format!("Customer#{:07}", c_custkey);
        let c_address = format!("{} Oak Street", i);
        let c_nationkey = i % 25;
        let c_phone = format!("{:02}-{:08}", i % 100, i % 100000000);
        let c_acctbal = ((i % 10000) as f64 / 100.0) - 500.0;
        let c_mktsegment = match i % 5 {
            0 => "AUTOMOBILE",
            1 => "BUILDING",
            2 => "FURNITURE",
            3 => "MACHINERY",
            _ => "HOUSEHOLD",
        };

        customers.push(vec![
            Value::Integer(c_custkey),
            Value::Text(c_name),
            Value::Text(c_address),
            Value::Integer(c_nationkey),
            Value::Text(c_phone),
            Value::Float(c_acctbal),
            Value::Text(c_mktsegment.to_string()),
        ]);
    }

    storage.insert("customer", customers).ok();

    let mut orders = Vec::new();
    for i in 0..order_count {
        let o_orderkey = i + 1;
        let o_custkey = i % customer_count + 1;
        let o_orderstatus = if i % 3 == 0 { "F" } else { "O" };
        let o_totalprice = ((i % 100000) as f64) / 100.0 + 1000.0;
        let o_orderdate = 78000 + (i % 2400);
        let o_orderpriority = match i % 5 {
            0 => "1-URGENT",
            1 => "2-HIGH",
            2 => "3-MEDIUM",
            3 => "4-LOW",
            _ => "5-NOT SPECIFIED",
        };
        let o_clerk = format!("Clerk#{:06}", i % 1000);

        orders.push(vec![
            Value::Integer(o_orderkey),
            Value::Integer(o_custkey),
            Value::Text(o_orderstatus.to_string()),
            Value::Float(o_totalprice),
            Value::Integer(o_orderdate),
            Value::Text(o_orderpriority.to_string()),
            Value::Text(o_clerk),
        ]);
    }

    storage.insert("orders", orders).ok();

    let mut lineitems = Vec::new();
    for i in 0..lineitem_count {
        let l_orderkey = i % order_count + 1;
        let l_partkey = i % part_count + 1;
        let l_suppkey = i % supplier_count + 1;
        let l_linenumber = (i % 4) as i64 + 1;
        let l_quantity = (i % 50) as i64 + 1;
        let l_extendedprice = ((i % 100000) as f64) / 100.0 + 100.0;
        let l_discount = ((i % 10) as f64) / 100.0;
        let l_tax = ((i % 8) as f64) / 100.0;
        let l_returnflag = if i % 3 == 0 { "R" } else { "N" };
        let l_linestatus = if i % 2 == 0 { "O" } else { "F" };
        let l_shipdate = 78000 + (i % 2400);
        let l_commitdate = l_shipdate + 30;
        let l_receiptdate = l_shipdate + 60;

        lineitems.push(vec![
            Value::Integer(l_orderkey),
            Value::Integer(l_partkey),
            Value::Integer(l_suppkey),
            Value::Integer(l_linenumber),
            Value::Integer(l_quantity),
            Value::Float(l_extendedprice),
            Value::Float(l_discount),
            Value::Float(l_tax),
            Value::Text(l_returnflag.to_string()),
            Value::Text(l_linestatus.to_string()),
            Value::Integer(l_shipdate),
            Value::Integer(l_commitdate),
            Value::Integer(l_receiptdate),
        ]);
    }

    storage.insert("lineitem", lineitems).ok();

    let mut parts = Vec::new();
    for i in 0..part_count {
        let p_partkey = i + 1;
        let p_name = format!("Part#{}", p_partkey);
        let p_mfgr = match i % 5 {
            0 => "Manufacturer#1",
            1 => "Manufacturer#2",
            2 => "Manufacturer#3",
            3 => "Manufacturer#4",
            _ => "Manufacturer#5",
        };
        let p_brand = format!("Brand#{}", (i % 50) + 1);
        let p_type = match i % 10 {
            0 => "ECONOMY ANODIZED STEEL",
            1 => "ECONOMY BURNISHED STEEL",
            2 => "LARGE BRUSHED STEEL",
            3 => "MEDIUM ANODIZED STEEL",
            4 => "MEDIUM BURNISHED STEEL",
            _ => "SMALL POLISHED STEEL",
        };
        let p_size = (i % 50) as i64 + 1;
        let p_container = match i % 10 {
            0 => "SM CASE",
            1 => "SM BOX",
            2 => "SM PACK",
            3 => "SM PKG",
            4 => "MED CASE",
            _ => "MED BOX",
        };
        let p_retailprice = ((i % 10000) as f64) / 100.0 + 100.0;
        let p_comment = format!("Part comment {}", i);

        parts.push(vec![
            Value::Integer(p_partkey),
            Value::Text(p_name),
            Value::Text(p_mfgr.to_string()),
            Value::Text(p_brand),
            Value::Text(p_type.to_string()),
            Value::Integer(p_size),
            Value::Text(p_container.to_string()),
            Value::Float(p_retailprice),
            Value::Text(p_comment),
        ]);
    }

    storage.insert("part", parts).ok();

    let mut suppliers = Vec::new();
    for i in 0..supplier_count {
        let s_suppkey = i + 1;
        let s_name = format!("Supplier#{:07}", s_suppkey);
        let s_address = format!("{} Avenue", i * 17);
        let s_nationkey = i % 25;
        let s_phone = format!("{:02}-{:08}", i % 100, (i * 17) % 100000000);
        let s_acctbal = ((i % 10000) as f64 / 100.0) - 500.0;
        let s_comment = format!("Supplier comment {}", i);

        suppliers.push(vec![
            Value::Integer(s_suppkey),
            Value::Text(s_name),
            Value::Text(s_address),
            Value::Integer(s_nationkey),
            Value::Text(s_phone),
            Value::Float(s_acctbal),
            Value::Text(s_comment),
        ]);
    }

    storage.insert("supplier", suppliers).ok();

    let mut partsupps = Vec::new();
    for i in 0..partsupp_count {
        let ps_partkey = i % part_count + 1;
        let ps_suppkey = i % supplier_count + 1;
        let ps_availqty = (i % 9999) as i64 + 1;
        let ps_supplycost = ((i % 10000) as f64) / 100.0 + 1.0;
        let ps_comment = format!("Partsupp comment {}", i);

        partsupps.push(vec![
            Value::Integer(ps_partkey),
            Value::Integer(ps_suppkey),
            Value::Integer(ps_availqty),
            Value::Float(ps_supplycost),
            Value::Text(ps_comment),
        ]);
    }

    storage.insert("partsupp", partsupps).ok();

    let nations = vec![
        vec![
            Value::Integer(0),
            Value::Text("ALGERIA".to_string()),
            Value::Integer(0),
            Value::Text("algolia".to_string()),
        ],
        vec![
            Value::Integer(1),
            Value::Text("ARGENTINA".to_string()),
            Value::Integer(1),
            Value::Text("argentina".to_string()),
        ],
        vec![
            Value::Integer(2),
            Value::Text("BRAZIL".to_string()),
            Value::Integer(1),
            Value::Text("brazil".to_string()),
        ],
        vec![
            Value::Integer(3),
            Value::Text("CANADA".to_string()),
            Value::Integer(1),
            Value::Text("canada".to_string()),
        ],
        vec![
            Value::Integer(4),
            Value::Text("CHINA".to_string()),
            Value::Integer(2),
            Value::Text("china".to_string()),
        ],
        vec![
            Value::Integer(5),
            Value::Text("EGYPT".to_string()),
            Value::Integer(0),
            Value::Text("egypt".to_string()),
        ],
        vec![
            Value::Integer(6),
            Value::Text("ETHIOPIA".to_string()),
            Value::Integer(0),
            Value::Text("ethiopia".to_string()),
        ],
        vec![
            Value::Integer(7),
            Value::Text("FRANCE".to_string()),
            Value::Integer(3),
            Value::Text("france".to_string()),
        ],
        vec![
            Value::Integer(8),
            Value::Text("GERMANY".to_string()),
            Value::Integer(3),
            Value::Text("germany".to_string()),
        ],
        vec![
            Value::Integer(9),
            Value::Text("INDIA".to_string()),
            Value::Integer(2),
            Value::Text("india".to_string()),
        ],
        vec![
            Value::Integer(10),
            Value::Text("INDONESIA".to_string()),
            Value::Integer(2),
            Value::Text("indonesia".to_string()),
        ],
        vec![
            Value::Integer(11),
            Value::Text("IRAN".to_string()),
            Value::Integer(2),
            Value::Text("iran".to_string()),
        ],
        vec![
            Value::Integer(12),
            Value::Text("IRAQ".to_string()),
            Value::Integer(0),
            Value::Text("iraq".to_string()),
        ],
        vec![
            Value::Integer(13),
            Value::Text("JAPAN".to_string()),
            Value::Integer(2),
            Value::Text("japan".to_string()),
        ],
        vec![
            Value::Integer(14),
            Value::Text("JORDAN".to_string()),
            Value::Integer(0),
            Value::Text("jordan".to_string()),
        ],
        vec![
            Value::Integer(15),
            Value::Text("KENYA".to_string()),
            Value::Integer(0),
            Value::Text("kenya".to_string()),
        ],
        vec![
            Value::Integer(16),
            Value::Text("MOROCCO".to_string()),
            Value::Integer(0),
            Value::Text("morocco".to_string()),
        ],
        vec![
            Value::Integer(17),
            Value::Text("MOZAMBIQUE".to_string()),
            Value::Integer(0),
            Value::Text("mozambique".to_string()),
        ],
        vec![
            Value::Integer(18),
            Value::Text("PERU".to_string()),
            Value::Integer(1),
            Value::Text("peru".to_string()),
        ],
        vec![
            Value::Integer(19),
            Value::Text("ROMANIA".to_string()),
            Value::Integer(3),
            Value::Text("romania".to_string()),
        ],
        vec![
            Value::Integer(20),
            Value::Text("RUSSIA".to_string()),
            Value::Integer(3),
            Value::Text("russia".to_string()),
        ],
        vec![
            Value::Integer(21),
            Value::Text("SAUDI ARABIA".to_string()),
            Value::Integer(0),
            Value::Text("saudi".to_string()),
        ],
        vec![
            Value::Integer(22),
            Value::Text("UNITED KINGDOM".to_string()),
            Value::Integer(3),
            Value::Text("uk".to_string()),
        ],
        vec![
            Value::Integer(23),
            Value::Text("UNITED STATES".to_string()),
            Value::Integer(1),
            Value::Text("united states".to_string()),
        ],
        vec![
            Value::Integer(24),
            Value::Text("VIETNAM".to_string()),
            Value::Integer(2),
            Value::Text("vietnam".to_string()),
        ],
    ];
    storage.insert("nation", nations).ok();

    let regions = vec![
        vec![
            Value::Integer(0),
            Value::Text("AFRICA".to_string()),
            Value::Text("africa".to_string()),
        ],
        vec![
            Value::Integer(1),
            Value::Text("AMERICA".to_string()),
            Value::Text("america".to_string()),
        ],
        vec![
            Value::Integer(2),
            Value::Text("ASIA".to_string()),
            Value::Text("asia".to_string()),
        ],
        vec![
            Value::Integer(3),
            Value::Text("EUROPE".to_string()),
            Value::Text("europe".to_string()),
        ],
        vec![
            Value::Integer(4),
            Value::Text("MIDDLE EAST".to_string()),
            Value::Text("middle east".to_string()),
        ],
    ];
    storage.insert("region", regions).ok();

    println!("Data generation complete!");

    Arc::new(storage)
}
