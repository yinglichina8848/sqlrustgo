mod sqlite_diff;

use sqlite_diff::{RustEngine, SqliteEngine};

fn basic_setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t(a INT, b INT, c TEXT);",
        "INSERT INTO t VALUES (1,100,'foo'),(2,200,'bar'),(3,300,'baz'),(NULL,400,NULL);",
    ]
}

fn two_table_setup() -> Vec<&'static str> {
    vec![
        "CREATE TABLE t1(a INT, b INT);",
        "CREATE TABLE t2(c INT, d INT);",
        "INSERT INTO t1 VALUES (1,10),(2,20),(NULL,30);",
        "INSERT INTO t2 VALUES (100,1000),(200,2000);",
    ]
}

const QUERY_TEMPLATES: &[&str] = &[
    "SELECT * FROM t",
    "SELECT a, b FROM t",
    "SELECT a + 1 FROM t",
    "SELECT a * 2, b + 10 FROM t",
    "SELECT * FROM t WHERE a > 1",
    "SELECT * FROM t WHERE a IS NULL",
    "SELECT * FROM t WHERE a IS NOT NULL",
    "SELECT * FROM t WHERE a > 1 AND b < 200",
    "SELECT * FROM t WHERE a > 1 OR b = 100",
    "SELECT * FROM t WHERE (a > 1 AND b < 200) OR a IS NULL",
    "SELECT COUNT(*) FROM t",
    "SELECT COUNT(a) FROM t",
    "SELECT SUM(a) FROM t",
    "SELECT * FROM t1, t2",
    "SELECT * FROM t1 JOIN t2 ON t1.a = t2.c",
    "SELECT * FROM t WHERE a IN (SELECT a FROM t WHERE a > 1)",
    "SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t WHERE a > 100)",
];

#[test]
fn fuzz_all_queries() {
    for setup_fn in &[basic_setup, two_table_setup] {
        let sqlite = SqliteEngine::new();
        let mut rust = RustEngine::new();

        for stmt in setup_fn() {
            sqlite.execute(stmt).unwrap();
            rust.execute(stmt).unwrap();
        }

        for sql in QUERY_TEMPLATES {
            let sqlite_result = sqlite.query(sql);
            let rust_result = rust.query(sql);

            match (sqlite_result, rust_result) {
                (Ok(sq), Ok(rq)) => {
                    assert_eq!(sq, rq, "Query failed: {}", sql);
                }
                (Err(_), Err(_)) => {}
                (Ok(_sq), Err(e)) => {
                    panic!("SQLite ok but Rust failed: {} - Error: {}", sql, e);
                }
                (Err(e), Ok(_rq)) => {
                    panic!("Rust ok but SQLite failed: {} - Error: {}", sql, e);
                }
            }
        }
    }
}

mod rand_gen {
    use rand::{Rng, SeedableRng};

    pub fn generate_random_insert(table_name: &str, num_rows: usize, seed: u64) -> Vec<String> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut inserts = Vec::new();

        for _ in 0..num_rows {
            let a: Option<i64> = if rng.gen_bool(0.2) {
                None
            } else {
                Some(rng.gen_range(1..100))
            };
            let b: i64 = rng.gen_range(1..1000);

            let a_str = match a {
                Some(v) => v.to_string(),
                None => "NULL".to_string(),
            };

            inserts.push(format!(
                "INSERT INTO {} VALUES ({}, {})",
                table_name, a_str, b
            ));
        }

        inserts
    }
}

#[test]
fn fuzz_random_data() {
    use rand_gen::generate_random_insert;

    let setup_stmts = vec!["CREATE TABLE t(a INT, b INT);"];
    let insert_stmts = generate_random_insert("t", 20, 42);

    let sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();

    for stmt in &setup_stmts {
        sqlite.execute(stmt).unwrap();
        rust.execute(stmt).unwrap();
    }

    for stmt in &insert_stmts {
        sqlite.execute(stmt).unwrap();
        rust.execute(stmt).unwrap();
    }

    let queries = &[
        "SELECT * FROM t",
        "SELECT * FROM t WHERE a > 50",
        "SELECT COUNT(*) FROM t",
        "SELECT SUM(a) FROM t",
    ];

    for sql in queries {
        let sqlite_result = sqlite.query(sql);
        let rust_result = rust.query(sql);

        match (sqlite_result, rust_result) {
            (Ok(sq), Ok(rq)) => {
                assert_eq!(sq, rq, "Query failed: {}", sql);
            }
            (Err(_), Err(_)) => {}
            (Ok(_sq), Err(e)) => {
                panic!("SQLite ok but Rust failed: {} - Error: {}", sql, e);
            }
            (Err(e), Ok(_rq)) => {
                panic!("Rust ok but SQLite failed: {} - Error: {}", sql, e);
            }
        }
    }
}
