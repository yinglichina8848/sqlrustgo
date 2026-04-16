//! SQL Corpus Integration Test
//!
//! Runs the SQL regression corpus against SQLRustGo

use sqlrustgo_sql_corpus::{CorpusFileResult, SqlCorpus};
use std::collections::HashMap;
use std::path::PathBuf;

fn get_corpus_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/sql-corpus
    // We need to join "sql_corpus" to get crates/sql-corpus/sql_corpus
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sql_corpus")
}

fn run_corpus_test() -> HashMap<String, CorpusFileResult> {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);
    corpus.execute_all()
}

fn print_results(results: &HashMap<String, CorpusFileResult>) {
    let mut total_cases = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    println!("\n=== SQL Corpus Results ===\n");

    let mut files: Vec<_> = results.iter().collect();
    files.sort_by_key(|(k, _)| *k);

    for (file, result) in files {
        println!("{}: {}/{} passed", file, result.passed, result.total_cases);
        for case in &result.results {
            if case.success {
                println!("  ✓ {}", case.case_name);
            } else {
                println!(
                    "  ✗ {} - {}",
                    case.case_name,
                    case.error_message.as_deref().unwrap_or("failed")
                );
            }
        }
        total_cases += result.total_cases;
        total_passed += result.passed;
        total_failed += result.failed;
    }

    println!("\n=== Summary ===");
    println!(
        "Total: {} cases, {} passed, {} failed",
        total_cases, total_passed, total_failed
    );
    if total_cases > 0 {
        println!(
            "Pass rate: {:.1}%",
            (total_passed as f64 / total_cases as f64) * 100.0
        );
    }
}

#[test]
fn test_sql_corpus_all() {
    let results = run_corpus_test();
    print_results(&results);

    let corpus = SqlCorpus::new(get_corpus_root());
    let summary = corpus.summary(&results);

    println!(
        "\nFinal Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );

    if summary.failed > 0 {
        panic!("SQL Corpus had {} failing tests", summary.failed);
    }
}

#[test]
fn test_sql_corpus_joins() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root.join("DML/SELECT/joins.sql"));

    let results = corpus.execute_file(&corpus_root.join("DML/SELECT/joins.sql"));

    println!("\n=== JOIN Tests ===");
    for case in &results.results {
        if case.success {
            println!("  ✓ {}", case.case_name);
        } else {
            println!(
                "  ✗ {} - {}",
                case.case_name,
                case.error_message.as_deref().unwrap_or("failed")
            );
        }
    }

    assert_eq!(results.failed, 0, "JOIN tests had failures");
}

#[test]
fn test_sql_corpus_subqueries() {
    let corpus_root = get_corpus_root();
    let results = SqlCorpus::new(corpus_root.clone())
        .execute_file(&corpus_root.join("DML/SELECT/subqueries.sql"));

    println!("\n=== Subquery Tests ===");
    for case in &results.results {
        if case.success {
            println!("  ✓ {}", case.case_name);
        } else {
            println!(
                "  ✗ {} - {}",
                case.case_name,
                case.error_message.as_deref().unwrap_or("failed")
            );
        }
    }

    assert_eq!(results.failed, 0, "Subquery tests had failures");
}

#[test]
fn test_sql_corpus_aggregates() {
    let corpus_root = get_corpus_root();
    let results = SqlCorpus::new(corpus_root.clone())
        .execute_file(&corpus_root.join("DML/SELECT/aggregates.sql"));

    println!("\n=== Aggregate Tests ===");
    for case in &results.results {
        if case.success {
            println!("  ✓ {}", case.case_name);
        } else {
            println!(
                "  ✗ {} - {}",
                case.case_name,
                case.error_message.as_deref().unwrap_or("failed")
            );
        }
    }

    assert_eq!(results.failed, 0, "Aggregate tests had failures");
}
