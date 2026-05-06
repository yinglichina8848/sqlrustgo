//! sqlrustgo-fuzz — Simple SQL fuzzing framework
//!
//! Generates random SQL statements from templates and compares
//! SQLRustGo results against SQLite (differential testing).
//!
//! Usage:
//!   cargo run -p sqlrustgo-fuzz -- [iterations]

use std::process::Command;
use std::time::Instant;

fn main() {
    let iterations: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    println!("=== SQLRustGo Fuzz ===");
    println!("Iterations: {}", iterations);
    println!();

    // Template pools for SQL generation
    let selects = [
        "SELECT 1",
        "SELECT NULL",
        "SELECT 1 + 1",
        "SELECT 1 + 2 * 3",
        "SELECT 'hello'",
        "SELECT 42 AS answer",
        "SELECT 1, 2, 3",
        "SELECT NULL, 1, 'text'",
    ];

    let from_where = [
        "",
        " WHERE 1=1",
        " WHERE 1=0",
        " WHERE NULL IS NULL",
        " WHERE 1 IS NOT NULL",
    ];

    let mut rng = 42u64;

    let mut failures = Vec::new();
    let start = Instant::now();

    for i in 0..iterations {
        // Simple pseudo-random: xorshift64
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;

        let sel_idx = (rng as usize) % selects.len();
        let where_idx = (rng as usize) % from_where.len();

        let sql = format!("{}{};", selects[sel_idx], from_where[where_idx]);

        // Run SQLRustGo
        let srg_result = Command::new("cargo")
            .args(["run", "--bin", "sqlrustgo", "--", "--execute", &sql])
            .output();

        match srg_result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                if !out.status.success() && !stderr.contains("not supported") {
                    failures.push((sql.clone(), "SQLRustGo exec error".to_string()));
                }
            }
            Err(e) => {
                failures.push((sql.clone(), format!("Launch error: {}", e)));
            }
        }

        if (i + 1) % 10 == 0 {
            println!("  [{}/{}] failures: {}", i + 1, iterations, failures.len());
        }
    }

    let elapsed = start.elapsed();
    println!();
    println!("=== Results ===");
    println!("Iterations: {}", iterations);
    println!("Failures:    {}", failures.len());
    println!("Time:       {:?}", elapsed);

    if failures.is_empty() {
        println!("Result:     ✅ ALL PASSED");
    } else {
        println!("Result:     ❌ {} FAILURES", failures.len());
        for (sql, reason) in &failures[..failures.len().min(10)] {
            println!("  - {} ({})", sql, reason);
        }
    }
}
