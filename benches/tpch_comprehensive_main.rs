//! TPC-H 综合性能测试入口

mod tpch_comprehensive {
    include!("tpch_comprehensive.rs");
}

fn main() {
    println!("TPC-H Comprehensive Benchmark");
    println!("==============================\n");

    // 运行 SF=0.1 测试
    println!("Running SF=0.1 benchmarks...");
    tpch_comprehensive::run_all_scenarios(tpch_comprehensive::ScaleFactor::SF01);

    // 运行 SF 对比
    println!("\nRunning scale factor comparison...");
    tpch_comprehensive::run_sf_comparison();

    // 生成 JSON 报告
    tpch_comprehensive::generate_json_report(
        "benchmark_results/sf01_report.json",
        tpch_comprehensive::ScaleFactor::SF01,
    );

    println!("\nBenchmark complete!");
}
