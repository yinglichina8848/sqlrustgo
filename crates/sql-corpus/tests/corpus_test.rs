//! SQL Corpus Integration Test
//!
//! Runs the SQL regression corpus against SQLRustGo in memory-efficient batches

use sqlrustgo_sql_corpus::{CorpusFileResult, SqlCorpus};
use std::collections::HashMap;
use std::path::PathBuf;

fn get_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sql_corpus")
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
fn test_sql_corpus_batch_dml_insert() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("DML/INSERT");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch DML/INSERT Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
}

#[test]
fn test_sql_corpus_batch_dml_update() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("DML/UPDATE");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch DML/UPDATE Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
}

#[test]
fn test_sql_corpus_batch_dml_delete() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("DML/DELETE");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch DML/DELETE Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
}

#[test]
#[ignore]
fn test_sql_corpus_batch_select() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("DML/SELECT");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch DML/SELECT Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
}

#[test]
fn test_sql_corpus_single_file() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root.clone());

    let result = corpus.execute_file(&corpus_root.join("DML/SELECT/aggregates.sql"));

    println!("\n=== Single File: aggregates.sql ===");
    println!(
        "Total: {} cases, {} passed, {} failed",
        result.total_cases, result.passed, result.failed
    );
}

#[test]
fn test_sql_corpus_batch_ddl() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("DDL");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch DDL Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );

    const PASS_RATE_THRESHOLD: f64 = 80.0;
    if summary.pass_rate < PASS_RATE_THRESHOLD {
        panic!(
            "DDL batch pass rate {:.1}% is below threshold {:.1}%",
            summary.pass_rate, PASS_RATE_THRESHOLD
        );
    }
}

#[test]
fn test_sql_corpus_batch_expressions() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("EXPRESSIONS");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch EXPRESSIONS Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
}

#[test]
fn test_sql_corpus_batch_advanced() {
    let corpus_root = get_corpus_root();
    let mut corpus = SqlCorpus::new(corpus_root);

    let results = corpus.execute_batch("ADVANCED");
    print_results(&results);

    let summary = corpus.summary(&results);
    println!(
        "\nBatch ADVANCED Summary: {} files, {} cases, {:.1}% pass rate",
        summary.total_files, summary.total_cases, summary.pass_rate
    );
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
