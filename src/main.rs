//! SQLRustGo Database System

fn main() {
    sqlrustgo::init();
    println!("SQLRustGo v1.5.0");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_module_exists() {
        // Verify main module compiles correctly
        let _ = env!("CARGO_PKG_NAME");
    }

    #[test]
    fn test_sqlrustgo_init() {
        sqlrustgo::init();
    }

    #[test]
    fn test_sqlrustgo_module_exports() {
        // Verify that sqlrustgo module has expected exports
        let _ = sqlrustgo::init;
    }

    #[test]
    fn test_main_runs_init() {
        // Test that init function runs without panicking
        sqlrustgo::init();
    }
}
