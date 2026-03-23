//! SQL-92 Compliance Test Runner
//! 
//! Runs SQL-92 test cases and generates compliance reports.

use clap::Parser;
use sqlrustgo_parser::parse;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to test cases directory
    #[arg(short, long, default_value = "cases")]
    cases_dir: PathBuf,
    
    /// Output report file (Markdown)
    #[arg(short, long, default_value = "sql92-compliance-report.md")]
    output: PathBuf,
    
    /// Filter tests by category (e.g., "ddl", "queries")
    #[arg(short, long)]
    filter: Option<String>,
    
    /// Update expected outputs
    #[arg(long)]
    update_expected: bool,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    category: String,
    sql_path: PathBuf,
    expected_path: PathBuf,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    category: String,
    passed: bool,
    message: Option<String>,
}

fn find_test_cases(dir: &PathBuf, filter: &Option<String>) -> Vec<TestCase> {
    let mut cases = Vec::new();
    
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.is_dir() {
            let category = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Apply filter
            if let Some(ref f) = filter {
                if &category != f {
                    continue;
                }
            }
            
            // Find all .sql files
            if let Ok(entries) = fs::read_dir(&path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().map(|e| e == "sql").unwrap_or(false) {
                        let name = p.file_stem().unwrap().to_string_lossy().to_string();
                        let expected_path = p.with_extension("expected");
                        
                        cases.push(TestCase {
                            name,
                            category: category.clone(),
                            sql_path: p,
                            expected_path,
                        });
                    }
                }
            }
        }
    }
    
    cases
}

fn run_test_case(test: &TestCase, _verbose: bool) -> TestResult {
    // Read SQL
    let sql = match fs::read_to_string(&test.sql_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult {
                name: test.name.clone(),
                category: test.category.clone(),
                passed: false,
                message: Some(format!("Failed to read SQL: {}", e)),
            };
        }
    };
    
    // Parse SQL
    let result = parse(&sql);
    
    match result {
        Ok(_stmt) => {
            // Check if expected file exists
            if !test.expected_path.exists() {
                return TestResult {
                    name: test.name.clone(),
                    category: test.category.clone(),
                    passed: false,
                    message: Some("Expected output file not found".to_string()),
                };
            }
            
            TestResult {
                name: test.name.clone(),
                category: test.category.clone(),
                passed: true,
                message: Some("Parsed successfully".to_string()),
            }
        }
        Err(e) => {
            TestResult {
                name: test.name.clone(),
                category: test.category.clone(),
                passed: false,
                message: Some(format!("Parse error: {}", e)),
            }
        }
    }
}

fn generate_report(results: &[TestResult]) -> String {
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    let pass_rate = if results.is_empty() {
        0.0
    } else {
        (passed as f64 / results.len() as f64) * 100.0
    };
    
    let mut report = String::new();
    report.push_str("# SQL-92 Compliance Test Report\n\n");
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Tests**: {}\n", results.len()));
    report.push_str(&format!("- **Passed**: {}\n", passed));
    report.push_str(&format!("- **Failed**: {}\n", failed));
    report.push_str(&format!("- **Pass Rate**: {:.2}%\n\n", pass_rate));
    
    // Group by category
    let mut categories: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for r in results {
        categories.entry(r.category.clone()).or_default().push(r);
    }
    
    report.push_str("## Results by Category\n\n");
    
    // Sort categories
    let mut cat_keys: Vec<_> = categories.keys().collect();
    cat_keys.sort();
    
    for cat in cat_keys {
        let cat_results = categories.get(cat).unwrap();
        let cat_passed = cat_results.iter().filter(|r| r.passed).count();
        report.push_str(&format!("### {} ({}/{})\n\n", cat, cat_passed, cat_results.len()));
        
        for r in cat_results {
            let status = if r.passed { "✅ PASS" } else { "❌ FAIL" };
            report.push_str(&format!("- {} **{}**\n", status, r.name));
            if let Some(ref msg) = r.message {
                report.push_str(&format!("  - {}\n", msg));
            }
        }
        report.push_str("\n");
    }
    
    report
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let args = Args::parse();
    
    println!("SQL-92 Compliance Test Suite");
    println!("============================\n");
    
    // Find test cases
    let cases_dir = args.cases_dir.clone();
    let test_cases = find_test_cases(&cases_dir, &args.filter);
    
    if test_cases.is_empty() {
        println!("No test cases found!");
        return;
    }
    
    println!("Found {} test cases\n", test_cases.len());
    
    // Run tests in parallel
    let results: Vec<TestResult> = test_cases
        .par_iter()
        .map(|test| {
            if args.verbose {
                println!("Running: {}/{}", test.category, test.name);
            }
            run_test_case(test, args.verbose)
        })
        .collect();
    
    // Print summary
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    
    println!("\n============================");
    println!("Summary:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Pass rate: {:.2}%", if results.is_empty() { 0.0 } else { (passed as f64 / results.len() as f64) * 100.0 });
    
    // Generate report
    let report = generate_report(&results);
    
    if let Err(e) = fs::write(&args.output, &report) {
        eprintln!("Failed to write report: {}", e);
    } else {
        println!("\nReport written to: {}", args.output.display());
    }
    
    // Exit with error if any tests failed
    if failed > 0 {
        std::process::exit(1);
    }
}
