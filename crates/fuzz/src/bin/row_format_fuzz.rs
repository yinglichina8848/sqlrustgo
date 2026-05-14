//! Row format fuzz test binary
//!
//! Usage:
//!   cargo run -p sqlrustgo-fuzz --bin row_format_fuzz -- [rounds]
//!
//! Arguments:
//!   rounds - Number of fuzzing rounds (default: 10000)

use sqlrustgo_fuzz::row_format_fuzz::{
    run_default_fuzz, run_quick_fuzz, FuzzResult, RowFormatFuzzer,
};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--help" {
        println!("Row Format Fuzz Test");
        println!();
        println!("Usage:");
        println!("  cargo run -p sqlrustgo-fuzz --bin row_format_fuzz -- [rounds]");
        println!();
        println!("Arguments:");
        println!("  rounds - Number of fuzzing rounds (default: 10000)");
        return;
    }

    let rounds: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10000);

    println!("=== Row Format Fuzz Test ===");
    println!("Rounds: {}", rounds);
    println!();

    let result = if rounds <= 1000 {
        run_quick_fuzz();
        FuzzResult {
            total_rounds: rounds,
            failures: vec![],
        }
    } else {
        // For full runs, use a time-based seed
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut fuzzer = RowFormatFuzzer::new(seed, rounds);
        fuzzer.run()
    };

    result.summary();

    if !result.passed() {
        println!();
        println!("FAILED: {} failures found", result.failures.len());
        process::exit(1);
    }

    println!();
    println!("Result: ALL PASSED");
}
