use std::fs::File;
use std::io::Write;

fn write_line(w: &mut dyn Write, fields: &[&str]) {
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, "|").unwrap();
        }
        write!(w, "{}", f).unwrap();
    }
    writeln!(w, "|").unwrap();
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "/home/openclaw/sqlrustgo-tpch/data".to_string());
    
    println!("Generating TPC-H SF=1 data to: {}", out_dir);

    let region_file = File::create(format!("{}/region.tbl", out_dir)).unwrap();
    let mut region_file = std::io::BufWriter::new(region_file);
    for i in 0..5 {
        let name = match i {
            0 => "AFRICA",
            1 => "AMERICA",
            2 => "ASIA",
            3 => "EUROPE",
            _ => "MIDDLE EAST",
        };
        let comment = name.to_lowercase();
        write_line(&mut region_file, &[&i.to_string(), name, &comment]);
    }
    println!("  region: 5 rows");

    let nation_file = File::create(format!("{}/nation.tbl", out_dir)).unwrap();
    let mut nation_file = std::io::BufWriter::new(nation_file);
    let nations: Vec<(&str, i32, &str)> = vec![
        ("ALGERIA", 0, "algolia"),
        ("ARGENTINA", 1, "argentina"),
        ("BRAZIL", 1, "brazil"),
        ("CANADA", 1, "canada"),
        ("CHINA", 2, "china"),
        ("EGYPT", 0, "egypt"),
        ("ETHIOPIA", 0, "ethiopia"),
        ("FRANCE", 3, "france"),
        ("GERMANY", 3, "germany"),
        ("INDIA", 2, "india"),
        ("INDONESIA", 2, "indonesia"),
        ("IRAN", 2, "iran"),
        ("IRAQ", 0, "iraq"),
        ("JAPAN", 2, "japan"),
        ("JORDAN", 0, "jordan"),
        ("KENYA", 0, "kenya"),
        ("MOROCCO", 0, "morocco"),
        ("MOZAMBIQUE", 0, "mozambique"),
        ("PERU", 1, "peru"),
        ("ROMANIA", 3, "romania"),
        ("RUSSIA", 3, "russia"),
        ("SAUDI ARABIA", 0, "saudi"),
        ("UNITED KINGDOM", 3, "uk"),
        ("UNITED STATES", 1, "united states"),
        ("VIETNAM", 2, "vietnam"),
    ];
    for (i, (name, region_key, comment)) in nations.iter().enumerate() {
        write_line(&mut nation_file, &[&i.to_string(), name, &region_key.to_string(), comment]);
    }
    println!("  nation: 25 rows");

    let supplier_count = 100;
    let supplier_file = File::create(format!("{}/supplier.tbl", out_dir)).unwrap();
    let mut supplier_file = std::io::BufWriter::new(supplier_file);
    for i in 0..supplier_count {
        let s_suppkey = i + 1;
        let s_name = format!("Supplier#{:07}", s_suppkey);
        let s_address = format!("{} Avenue", i * 17);
        let s_nationkey = i % 25;
        let s_phone = format!("{:02}-{:08}", i % 100, (i * 17) % 100000000);
        let s_acctbal = format!("{:.2}", ((i % 10000) as f64 / 100.0) - 500.0);
        let s_comment = format!("Supplier comment {}", i);
        write_line(&mut supplier_file, &[&s_suppkey.to_string(), &s_name, &s_address, &s_nationkey.to_string(), &s_phone, &s_acctbal, &s_comment]);
    }
    println!("  supplier: {} rows", supplier_count);

    let part_count = 2000;
    let part_file = File::create(format!("{}/part.tbl", out_dir)).unwrap();
    let mut part_file = std::io::BufWriter::new(part_file);
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
        let p_retailprice = format!("{:.2}", ((i % 10000) as f64) / 100.0 + 100.0);
        let p_comment = format!("Part comment {}", i);
        write_line(&mut part_file, &[&p_partkey.to_string(), &p_name, p_mfgr, &p_brand, p_type, &p_size.to_string(), p_container, &p_retailprice, &p_comment]);
    }
    println!("  part: {} rows", part_count);

    let partsupp_count = part_count * supplier_count / 10;
    let partsupp_file = File::create(format!("{}/partsupp.tbl", out_dir)).unwrap();
    let mut partsupp_file = std::io::BufWriter::new(partsupp_file);
    for i in 0..partsupp_count {
        let ps_partkey = i % part_count + 1;
        let ps_suppkey = i % supplier_count + 1;
        let ps_availqty = (i % 9999) as i64 + 1;
        let ps_supplycost = format!("{:.2}", ((i % 10000) as f64) / 100.0 + 1.0);
        let ps_comment = format!("Partsupp comment {}", i);
        write_line(&mut partsupp_file, &[&ps_partkey.to_string(), &ps_suppkey.to_string(), &ps_availqty.to_string(), &ps_supplycost, &ps_comment]);
    }
    println!("  partsupp: {} rows", partsupp_count);

    println!("Data generation complete!");
}
