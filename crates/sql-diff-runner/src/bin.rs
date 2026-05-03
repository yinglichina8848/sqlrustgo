//! SQL Differential Testing Runner
//! 
//! Compares SQLRustGo results with SQLite reference implementation
//! to verify semantic correctness.
//!
//! Features:
//! - CSV normalization
//! - NULL/empty handling
//! - Order-independent comparison
//! - Automatic diff classification
//! - SQL shrinking

fn main() {
    println!("SQLite Differential Testing Runner v0.1.0");
    println!("=========================================");
    
    let differ = SqlDiffer::new();
    
    // Load test cases
    let test_cases = load_test_cases();
    
    println!("\nRunning {} test cases...\n", test_cases.len());
    
    let mut passed = 0;
    let mut failed = 0;
    let mut by_category: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for sql in test_cases {
        match differ.diff(&sql) {
            None => {
                println!("✅ PASS");
                passed += 1;
            }
            Some(diff) => {
                println!("❌ FAIL: {:?}", diff.category);
                failed += 1;
                
                let cat = format!("{:?}", diff.category);
                *by_category.entry(cat).or_insert(0) += 1;
                
                // Auto-shrink and save regression
                if failed < 10 {
                    let minimal = differ.shrink(&sql);
                    save_regression(&minimal, &diff.category);
                }
            }
        }
    }
    
    println!("\n========== Summary ==========");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("\nBy Category:");
    for (cat, count) in &by_category {
        println!("  {}: {}", cat, count);
    }
    
    if failed > 0 {
        std::process::exit(1);
    }
}

fn load_test_cases() -> Vec<String> {
    vec![
        "SELECT 1".to_string(),
        "SELECT 1+1".to_string(),
        "SELECT * FROM users".to_string(),
        "SELECT name, age FROM users".to_string(),
        "SELECT * FROM users WHERE age > 30".to_string(),
        "SELECT COUNT(*) FROM users".to_string(),
        "SELECT DISTINCT department FROM employees".to_string(),
    ]
}

fn save_regression(sql: &str, category: &DiffCategory) {
    let id = uuid::Uuid::new_v4().to_string();
    let path = format!("tests/regression/{}_{}.sql", 
        format!("{:?}", category).to_lowercase().replace(" ", "_"),
        id.split('-').next().unwrap()
    );
    
    // Create directory if needed
    let _ = std::fs::create_dir_all("tests/regression");
    let _ = std::fs::write(&path, sql);
    
    println!("📁 Saved regression: {}", path);
}