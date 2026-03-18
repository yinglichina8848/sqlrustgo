//! TPC-H 综合性能测试入口

mod tpch_comprehensive {
    include!("tpch_comprehensive.rs");
}

fn main() {
    println!("TPC-H Comprehensive Benchmark");
    println!("==============================\n");

    let default_sf = tpch_comprehensive::ScaleFactor::safe_default();
    let mem_estimate = default_sf.estimate_memory_mb();

    println!("Memory safety check:");
    println!("  Default scale factor: {:?}", default_sf);
    println!("  Estimated memory: {} MB", mem_estimate);
    println!("  Max allowed: {} MB\n", tpch_comprehensive::MAX_MEMORY_MB);

    // 运行安全的默认测试
    println!("Running benchmarks with safe default (SF=0.1)...");
    tpch_comprehensive::run_all_scenarios(tpch_comprehensive::ScaleFactor::SF01);

    // 仅在内存充足时运行 SF1 测试
    if tpch_comprehensive::ScaleFactor::SF1.is_safe() {
        println!("\nRunning scale factor comparison (SF=1)...");
        tpch_comprehensive::run_sf_comparison();
    } else {
        println!("\nSkipping SF=1 test - insufficient memory");
    }

    // 生成 JSON 报告
    tpch_comprehensive::generate_json_report(
        "benchmark_results/sf01_report.json",
        tpch_comprehensive::ScaleFactor::SF01,
    );

    println!("\nBenchmark complete!");
}
