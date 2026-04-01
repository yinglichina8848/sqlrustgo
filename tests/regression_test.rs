// SQLRustGo 综合回归测试
// 运行所有测试类型并生成完整报告
//
// 运行方式:
//   cargo test --test regression_test -- --nocapture
//   cargo test --test regression_test -- --nocapture --test-threads=1  # 单线程运行

use std::process::Command;
use std::time::Instant;

/// 测试类别
#[derive(Debug, Clone)]
struct TestCategory {
    name: &'static str,
    test_files: Vec<&'static str>,
    description: &'static str,
}

/// 所有测试类别定义
fn get_test_categories() -> Vec<TestCategory> {
    vec![
        // 单元测试
        TestCategory {
            name: "单元测试 (Unit Tests)",
            test_files: vec![
                "backup_test",
                "bplus_tree_test",
                "buffer_pool_test",
                "file_storage_test",
                "local_executor_test",
                "mysqldump_test",
                "optimizer_cost_test",
                "optimizer_rules_test",
                "parser_token_test",
                "prometheus_test",
                "query_cache_config_test",
                "query_cache_test",
                "server_health_test",
                "slow_query_log_test",
                "types_value_test",
                "vectorization_test",
            ],
            description: "测试底层组件：存储、缓冲区、解析器、类型系统",
        },
        // 集成测试 - 核心
        TestCategory {
            name: "集成测试 - 核心 (Core Integration)",
            test_files: vec!["executor_test", "planner_test", "page_test"],
            description: "测试执行器、规划器、页面管理",
        },
        // 集成测试 - SQL功能
        TestCategory {
            name: "集成测试 - SQL功能 (SQL Functionality)",
            test_files: vec![
                "foreign_key_test",
                "fk_actions_test",
                "server_integration_test",
                "upsert_test",
                "mysql_compatibility_test",
                "savepoint_test",
                "session_config_test",
            ],
            description: "测试外键、服务器、UPSERT、MySQL兼容性(KILL/PROCESSLIST)、保存点",
        },
        // 集成测试 - 存储
        TestCategory {
            name: "集成测试 - 存储 (Storage)",
            test_files: vec![
                "query_cache_test",
                "optimizer_stats_test",
                "checksum_corruption_test",
                "columnar_storage_test",
                "parquet_test",
                "storage_integration_test",
            ],
            description: "测试查询缓存、优化器统计、校验和完整性、列式存储、Parquet",
        },
        // 性能测试
        TestCategory {
            name: "性能测试 (Performance)",
            test_files: vec![
                "performance_test",
                "tpch_test",
                "tpch_benchmark",
                "tpch_full_test",
                "batch_insert_test",
                "autoinc_test",
                "index_integration_test",
            ],
            description: "性能测试：批量插入、索引扫描、JOIN、缓存、向量化、TPC-H Q1-Q22",
        },
        // 异常测试 - 并发
        TestCategory {
            name: "异常测试 - 并发 (Anomaly - Concurrency)",
            test_files: vec![
                "mvcc_concurrency_test",
                "snapshot_isolation_test",
                "concurrency_stress_test",
            ],
            description: "MVCC并发、快照隔离、并发压力测试",
        },
        // 异常测试 - 隔离级别
        TestCategory {
            name: "异常测试 - 隔离级别 (Anomaly - Isolation)",
            test_files: vec!["transaction_isolation_test", "transaction_timeout_test"],
            description: "事务隔离级别、超时测试",
        },
        // 异常测试 - 数据处理
        TestCategory {
            name: "异常测试 - 数据处理 (Anomaly - Data Handling)",
            test_files: vec![
                "boundary_test",
                "null_handling_test",
                "aggregate_type_test",
                "error_handling_test",
                "datetime_type_test",
            ],
            description: "边界条件、NULL处理、聚合类型、错误处理、日期时间",
        },
        // 异常测试 - 查询
        TestCategory {
            name: "异常测试 - 查询 (Anomaly - Query)",
            test_files: vec![
                "join_test",
                "set_operations_test",
                "view_test",
                "window_function_test",
            ],
            description: "JOIN、集合操作、视图、窗口函数",
        },
        // 异常测试 - 约束
        TestCategory {
            name: "异常测试 - 约束 (Anomaly - Constraints)",
            test_files: vec!["fk_constraint_test", "catalog_consistency_test"],
            description: "外键约束、目录一致性",
        },
        // 压力测试
        TestCategory {
            name: "压力测试 (Stress)",
            test_files: vec![
                "chaos_test",
                "crash_recovery_test",
                "stress_test",
                "production_scenario_test",
                "wal_deterministic_test",
                "wal_fuzz_test",
            ],
            description: "混沌工程、崩溃恢复、压力测试、WAL确定性测试",
        },
        // 异常测试 - 稳定性
        TestCategory {
            name: "异常测试 - 稳定性 (Anomaly - Stability)",
            test_files: vec!["long_run_stability_test", "qps_benchmark_test"],
            description: "长时间运行稳定性、QPS基准测试",
        },
        // 异常测试 - 崩溃注入
        TestCategory {
            name: "异常测试 - 崩溃注入 (Anomaly - Crash Injection)",
            test_files: vec!["crash_injection_test"],
            description: "崩溃注入测试",
        },
        // CI 测试
        TestCategory {
            name: "CI 测试 (CI)",
            test_files: vec!["ci_test"],
            description: "CI 环境检查",
        },
        // 其他测试
        TestCategory {
            name: "其他测试 (Other)",
            test_files: vec![
                "binary_format_test",
                "wal_integration_test",
                "distributed_transaction_test",
            ],
            description: "二进制格式、WAL集成测试、分布式事务",
        },
        // 安全测试
        TestCategory {
            name: "安全测试 (Security)",
            test_files: vec!["auth_rbac_test", "logging_test"],
            description: "RBAC权限、日志配置",
        },
        // 教学场景测试
        TestCategory {
            name: "教学场景测试 (Teaching Scenarios)",
            test_files: vec![
                "teaching_scenario_test",
                "teaching_scenario_client_server_test",
            ],
            description: "教学场景：客户端/服务器模式",
        },
        // 工具测试
        TestCategory {
            name: "工具测试 (Tools)",
            test_files: vec!["physical_backup_test"],
            description: "物理备份、mysqldump 等工具集成测试",
        },
        // Executor 内部测试
        TestCategory {
            name: "执行器测试 (Executor)",
            test_files: vec!["PKG:sqlrustgo-executor:test_stored_proc"],
            description: "executor crate 内部测试: 存储过程 (36 tests)",
        },
    ]
}

/// 运行单个测试文件
fn run_test_file(test_file: &str) -> TestResult {
    let start = Instant::now();

    // 处理 crate 内部测试
    let output = if test_file.starts_with("PKG:") {
        let parts: Vec<&str> = test_file.split(':').collect();
        let pkg = parts[1];
        let test_path = parts.get(2).unwrap_or(&"");
        Command::new("cargo")
            .args(&["test", "-p", pkg, "--test", test_path, "--", "--nocapture"])
            .output()
    } else {
        Command::new("cargo")
            .args(&["test", "--test", test_file, "--", "--nocapture"])
            .output()
    };

    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // 解析测试结果
            let (passed, failed, ignored) = parse_test_output(&stdout);

            TestResult {
                test_file: test_file.to_string(),
                passed,
                failed,
                ignored,
                duration_ms: duration.as_millis() as u64,
                success: output.status.success() && failed == 0,
                error: if !output.status.success() && failed > 0 {
                    Some(extract_error_message(&stdout, &stderr))
                } else {
                    None
                },
            }
        }
        Err(e) => TestResult {
            test_file: test_file.to_string(),
            passed: 0,
            failed: 0,
            ignored: 0,
            duration_ms: duration.as_millis() as u64,
            success: false,
            error: Some(format!("Failed to run test: {}", e)),
        },
    }
}

/// 测试结果
#[derive(Debug, Clone)]
struct TestResult {
    test_file: String,
    passed: u32,
    failed: u32,
    ignored: u32,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
}

/// 解析测试输出
fn parse_test_output(output: &str) -> (u32, u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut ignored = 0u32;

    // 查找包含 "test result:" 的行
    // cargo 输出格式: "test result: ok. 35 passed; 0 failed; 0 ignored; ..."
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("test result:") {
            // 使用简单的字符串解析，跳过 "test result: ok. " 或 "test result: FAILED. "
            let after_status = if line.contains("ok.") {
                line.strip_prefix("test result: ok. ").unwrap_or(line)
            } else if line.contains("FAILED.") {
                line.strip_prefix("test result: FAILED. ").unwrap_or(line)
            } else {
                continue;
            };

            // 解析 "35 passed; 0 failed; 0 ignored; ..."
            let parts: Vec<&str> = after_status.split(';').collect();
            for part in parts {
                let part = part.trim();
                if part.ends_with("passed") {
                    if let Some(num_str) = part.strip_suffix("passed") {
                        if let Ok(n) = num_str.trim().parse::<u32>() {
                            passed = n;
                        }
                    }
                } else if part.ends_with("failed") {
                    if let Some(num_str) = part.strip_suffix("failed") {
                        if let Ok(n) = num_str.trim().parse::<u32>() {
                            failed = n;
                        }
                    }
                } else if part.ends_with("ignored") {
                    if let Some(num_str) = part.strip_suffix("ignored") {
                        if let Ok(n) = num_str.trim().parse::<u32>() {
                            ignored = n;
                        }
                    }
                }
            }
            break; // 只处理第一行 test result
        }
    }

    (passed, failed, ignored)
}

/// 提取错误信息
fn extract_error_message(stdout: &str, stderr: &str) -> String {
    // 查找错误信息
    for line in stdout.lines() {
        if line.contains("FAILED") || line.contains("error[") {
            return line.trim().to_string();
        }
    }
    for line in stderr.lines() {
        if line.contains("error") {
            return line.trim().to_string();
        }
    }
    "Unknown error".to_string()
}

/// 打印测试报告头部
fn print_header() {
    println!();
    println!(
        "╔══════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                     SQLRustGo v1.9.0 综合回归测试报告                              ║"
    );
    println!(
        "╚══════════════════════════════════════════════════════════════════════════════════╝"
    );
    println!();
}

/// 打印测试类别报告
fn print_category_report(category: &TestCategory, results: &[TestResult]) {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {}", pad_right(category.name, 79));
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {}", pad_right(category.description, 79));
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");

    let total_passed: u32 = results.iter().map(|r| r.passed).sum();
    let total_failed: u32 = results.iter().map(|r| r.failed).sum();
    let total_ignored: u32 = results.iter().map(|r| r.ignored).sum();
    let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();

    for result in results {
        let status = if result.success { "✅" } else { "❌" };
        let error_info = if let Some(ref e) = result.error {
            format!(" [{}]", truncate(e, 40))
        } else {
            String::new()
        };

        println!(
            "│ {} {:<35} {:>6} / {:>5} / {:>5}  ({:>4}ms){}",
            status,
            result.test_file,
            result.passed,
            result.failed,
            result.ignored,
            result.duration_ms,
            error_info
        );
    }

    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    println!(
        "│ {:>35} {:>6} / {:>5} / {:>5}  (总计: {:>4}ms)",
        "类别汇总", total_passed, total_failed, total_ignored, total_duration
    );
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
}

/// 打印最终汇总
fn print_summary(results: &[TestResult], total_duration_ms: u64) {
    let total_passed: u32 = results.iter().map(|r| r.passed).sum();
    let total_failed: u32 = results.iter().map(|r| r.failed).sum();
    let total_ignored: u32 = results.iter().map(|r| r.ignored).sum();
    let total_tests = total_passed + total_failed + total_ignored;
    let success_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!(
        "╔══════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                                    测试汇总                                      ║"
    );
    println!(
        "╠══════════════════════════════════════════════════════════════════════════════════╣"
    );
    println!(
        "║  总测试文件数: {:>10}                                                      ║",
        results.len()
    );
    println!(
        "║  总测试数:     {:>10}                                                      ║",
        total_tests
    );
    println!(
        "║  通过:         {:>10} ({:.1}%)                                             ║",
        total_passed, success_rate
    );
    println!(
        "║  失败:         {:>10}                                                      ║",
        total_failed
    );
    println!(
        "║  忽略:         {:>10}                                                      ║",
        total_ignored
    );
    println!(
        "║  总耗时:       {:>10} ms (~{:.1} 秒)                                        ║",
        total_duration_ms,
        total_duration_ms as f64 / 1000.0
    );
    println!(
        "╚══════════════════════════════════════════════════════════════════════════════════╝"
    );

    if total_failed > 0 {
        println!();
        println!("❌ 回归测试失败！请检查以下失败的测试:");
        for result in results.iter().filter(|r| !r.success) {
            println!(
                "  - {}: {}",
                result.test_file,
                result.error.clone().unwrap_or_default()
            );
        }
    } else {
        println!();
        println!("✅ 所有回归测试通过！");
    }
}

// 辅助函数
fn pad_right(s: &str, width: usize) -> String {
    let mut result = String::new();
    for c in s.chars() {
        let char_len = c.len_utf8();
        if result.len() + char_len > width {
            break;
        }
        result.push(c);
    }
    // 如果原字符串被截断了，不再添加填充
    // 如果还没达到宽度，继续填充空格
    while result.len() < width {
        result.push(' ');
    }
    result
}

fn truncate(s: &str, width: usize) -> String {
    if s.len() <= width {
        return s.to_string();
    }
    // 正确处理UTF-8字符边界
    let mut result = String::new();
    for c in s.chars() {
        if result.len() + c.len_utf8() > width - 3 {
            break;
        }
        result.push(c);
    }
    if result.len() < s.len() {
        result.push_str("...");
    }
    result
}

/// 运行回归测试
#[test]
fn test_regression_suite() {
    print_header();

    let categories = get_test_categories();
    let mut all_results = Vec::new();
    let start = Instant::now();

    // 依次运行每个类别
    for category in &categories {
        println!();
        println!("▶ 正在运行: {}", category.name);

        let mut category_results = Vec::new();
        for test_file in &category.test_files {
            print!("  ⏳ {}... ", test_file);
            let result = run_test_file(test_file);

            if result.success {
                println!("✅ ({}, {} passed)", result.duration_ms, result.passed);
            } else {
                println!("❌ ({}, {} failed)", result.duration_ms, result.failed);
            }

            all_results.push(result.clone());
            category_results.push(result);
        }

        // 打印类别汇总
        let total_passed: u32 = category_results.iter().map(|r| r.passed).sum();
        let total_failed: u32 = category_results.iter().map(|r| r.failed).sum();
        println!(
            "  📊 {}: {} passed, {} failed",
            category.name, total_passed, total_failed
        );
    }

    let total_duration_ms = start.elapsed().as_millis() as u64;

    // 打印详细报告
    for category in &categories {
        let category_results: Vec<_> = all_results
            .iter()
            .filter(|r| category.test_files.contains(&r.test_file.as_str()))
            .cloned()
            .collect();
        print_category_report(category, &category_results);
    }

    // 打印汇总
    print_summary(&all_results, total_duration_ms);

    // 断言所有测试通过
    let total_failed: u32 = all_results.iter().map(|r| r.failed).sum();
    assert_eq!(total_failed, 0, "回归测试有 {} 个测试失败", total_failed);
}
