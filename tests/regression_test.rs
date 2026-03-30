// SQLRustGo 综合回归测试
// 运行所有测试类型并生成完整报告
//
// 运行方式:
//   cargo test --test regression_test -- --nocapture
//   cargo test --test regression_test -- --nocapture --test-threads=1  # 单线程运行

use std::process::Command;
use std::time::Instant;

/// 测试类型
#[derive(Debug, Clone, PartialEq)]
enum TestType {
    IntegrationTest,
    CrateTest { crate_name: &'static str },
}

/// 测试类别
#[derive(Debug, Clone)]
struct TestCategory {
    name: &'static str,
    test_files: Vec<(&'static str, TestType)>,
    description: &'static str,
}

/// 所有测试类别定义
fn get_test_categories() -> Vec<TestCategory> {
    vec![
        // 单元测试
        TestCategory {
            name: "单元测试 (Unit Tests)",
            test_files: vec![
                ("bplus_tree_test", TestType::IntegrationTest),
                ("buffer_pool_test", TestType::IntegrationTest),
                ("file_storage_test", TestType::IntegrationTest),
                ("local_executor_test", TestType::IntegrationTest),
                ("optimizer_cost_test", TestType::IntegrationTest),
                ("optimizer_rules_test", TestType::IntegrationTest),
                ("parser_token_test", TestType::IntegrationTest),
                ("query_cache_config_test", TestType::IntegrationTest),
                ("query_cache_test", TestType::IntegrationTest),
                ("server_health_test", TestType::IntegrationTest),
                ("types_value_test", TestType::IntegrationTest),
                ("vectorization_test", TestType::IntegrationTest),
            ],
            description: "测试底层组件：存储、缓冲区、解析器、类型系统",
        },
        // 集成测试 - 核心
        TestCategory {
            name: "集成测试 - 核心 (Core Integration)",
            test_files: vec![
                ("executor_test", TestType::IntegrationTest),
                ("planner_test", TestType::IntegrationTest),
                ("page_test", TestType::IntegrationTest),
                ("index_integration_test", TestType::IntegrationTest),
                ("session_config_test", TestType::IntegrationTest),
                // ("integration_test", TestType::IntegrationTest), // TODO: 缺少 harness 模块
            ],
            description: "测试执行器、规划器、页面管理、索引、会话配置",
        },
        // 集成测试 - SQL功能
        TestCategory {
            name: "集成测试 - SQL功能 (SQL Functionality)",
            test_files: vec![
                ("foreign_key_test", TestType::IntegrationTest),
                ("fk_actions_test", TestType::IntegrationTest),
                ("server_integration_test", TestType::IntegrationTest),
                ("upsert_test", TestType::IntegrationTest),
                ("autoinc_test", TestType::IntegrationTest),
                ("savepoint_test", TestType::IntegrationTest),
            ],
            description: "测试外键、服务器、UPSERT、自增、保存点",
        },
        // 集成测试 - 存储
        TestCategory {
            name: "集成测试 - 存储 (Storage)",
            test_files: vec![
                ("query_cache_test", TestType::IntegrationTest),
                ("optimizer_stats_test", TestType::IntegrationTest),
                ("checksum_corruption_test", TestType::IntegrationTest),
                ("storage_integration_test", TestType::IntegrationTest),
                ("batch_insert_test", TestType::IntegrationTest),
            ],
            description: "测试查询缓存、优化器统计、校验和完整性、批量插入",
        },
        // 教学场景测试
        TestCategory {
            name: "教学场景测试 (Teaching Scenarios)",
            test_files: vec![
                ("teaching_scenario_test", TestType::IntegrationTest),
                (
                    "teaching_scenario_client_server_test",
                    TestType::IntegrationTest,
                ),
            ],
            description: "35+ 教学场景测试：CRUD、事务、JOIN、聚合、子查询、视图、优化器",
        },
        // 性能测试
        TestCategory {
            name: "性能测试 (Performance)",
            test_files: vec![("performance_test", TestType::IntegrationTest)],
            description: "22 性能测试：批量插入、索引扫描、JOIN、缓存、向量化",
        },
        // v2.0.0 新功能测试
        TestCategory {
            name: "v2.0.0 新功能 - 列式存储 (Columnar Storage)",
            test_files: vec![("columnar_storage_test", TestType::IntegrationTest)],
            description:
                "Epic-12 列式存储测试：ColumnChunk/ColumnSegment/ColumnarScan/ProjectionPushdown",
        },
        TestCategory {
            name: "v2.0.0 新功能 - 窗口函数与高级SQL (Window Functions)",
            test_files: vec![("window_function_test", TestType::IntegrationTest)],
            description: "Phase 2 窗口函数测试：ROW_NUMBER/RANK/SUM OVER",
        },
        TestCategory {
            name: "v2.0.0 新功能 - 分布式事务 (Distributed Transaction)",
            test_files: vec![("distributed_transaction_test", TestType::IntegrationTest)],
            description: "Phase 3 分布式事务测试：Sharding/2PC",
        },
        TestCategory {
            name: "v2.0.0 新功能 - TPC-H基准测试 (TPC-H Benchmark)",
            test_files: vec![
                ("tpch_test", TestType::IntegrationTest),
                ("mysql_tpch_test", TestType::IntegrationTest),
            ],
            description: "Epic-12 TPC-H 基准测试：Parquet 导入导出",
        },
        // 异常测试 - 并发
        TestCategory {
            name: "异常测试 - 并发 (Anomaly - Concurrency)",
            test_files: vec![
                ("mvcc_concurrency_test", TestType::IntegrationTest),
                ("snapshot_isolation_test", TestType::IntegrationTest),
                ("concurrency_stress_test", TestType::IntegrationTest),
            ],
            description: "MVCC并发、快照隔离、并发压力测试",
        },
        // 异常测试 - 隔离级别
        TestCategory {
            name: "异常测试 - 隔离级别 (Anomaly - Isolation)",
            test_files: vec![
                ("transaction_isolation_test", TestType::IntegrationTest),
                ("transaction_timeout_test", TestType::IntegrationTest),
            ],
            description: "事务隔离级别、超时测试",
        },
        // 异常测试 - 数据处理
        TestCategory {
            name: "异常测试 - 数据处理 (Anomaly - Data Handling)",
            test_files: vec![
                ("boundary_test", TestType::IntegrationTest),
                ("null_handling_test", TestType::IntegrationTest),
                ("aggregate_type_test", TestType::IntegrationTest),
                ("error_handling_test", TestType::IntegrationTest),
                ("datetime_type_test", TestType::IntegrationTest),
            ],
            description: "边界条件、NULL处理、聚合类型、错误处理、日期时间",
        },
        // 异常测试 - 查询
        TestCategory {
            name: "异常测试 - 查询 (Anomaly - Query)",
            test_files: vec![
                ("join_test", TestType::IntegrationTest),
                ("set_operations_test", TestType::IntegrationTest),
                ("view_test", TestType::IntegrationTest),
            ],
            description: "JOIN、集合操作、视图",
        },
        // 异常测试 - 约束
        TestCategory {
            name: "异常测试 - 约束 (Anomaly - Constraints)",
            test_files: vec![
                ("fk_constraint_test", TestType::IntegrationTest),
                ("catalog_consistency_test", TestType::IntegrationTest),
            ],
            description: "外键约束、目录一致性",
        },
        // 压力测试
        TestCategory {
            name: "压力测试 (Stress)",
            test_files: vec![
                ("chaos_test", TestType::IntegrationTest),
                ("crash_recovery_test", TestType::IntegrationTest),
                ("stress_test", TestType::IntegrationTest),
                ("production_scenario_test", TestType::IntegrationTest),
                ("wal_deterministic_test", TestType::IntegrationTest),
                ("wal_fuzz_test", TestType::IntegrationTest),
            ],
            description: "混沌工程、崩溃恢复、压力测试、WAL确定性测试",
        },
        // 异常测试 - 稳定性
        TestCategory {
            name: "异常测试 - 稳定性 (Anomaly - Stability)",
            test_files: vec![
                ("long_run_stability_test", TestType::IntegrationTest),
                ("qps_benchmark_test", TestType::IntegrationTest),
            ],
            description: "长时间运行稳定性、QPS基准测试",
        },
        // 异常测试 - 崩溃注入
        TestCategory {
            name: "异常测试 - 崩溃注入 (Anomaly - Crash Injection)",
            test_files: vec![("crash_injection_test", TestType::IntegrationTest)],
            description: "崩溃注入测试",
        },
        // CI 测试
        TestCategory {
            name: "CI 测试 (CI)",
            test_files: vec![("ci_test", TestType::IntegrationTest)],
            description: "CI 环境检查",
        },
        // 其他测试
        TestCategory {
            name: "其他测试 (Other)",
            test_files: vec![
                ("binary_format_test", TestType::IntegrationTest),
                ("wal_integration_test", TestType::IntegrationTest),
            ],
            description: "二进制格式、WAL集成测试",
        },
        // SQL Firewall 模块测试 (Issue #1134)
        TestCategory {
            name: "SQL Firewall 模块 (SQL Firewall)",
            test_files: vec![(
                "sqlrustgo-security",
                TestType::CrateTest {
                    crate_name: "sqlrustgo-security",
                },
            )],
            description: "SQL防火墙测试：SQL注入防护、批量操作拦截、告警系统",
        },
        // 版本升级CLI模块测试 (Issue #1132)
        TestCategory {
            name: "版本升级CLI模块 (Upgrade CLI)",
            test_files: vec![(
                "sqlrustgo-tools",
                TestType::CrateTest {
                    crate_name: "sqlrustgo-tools",
                },
            )],
            description: "版本升级CLI工具测试：版本检测、升级执行、回滚支持",
        },
        // AgentSQL模块测试 (Issue #1128)
        TestCategory {
            name: "AgentSQL模块 (AgentSQL)",
            test_files: vec![(
                "sqlrustgo-agentsql",
                TestType::CrateTest {
                    crate_name: "sqlrustgo-agentsql",
                },
            )],
            description: "AgentSQL NL2SQL、内存管理、REST API测试",
        },
        // 存储过程与触发器测试
        TestCategory {
            name: "存储过程与触发器 (Stored Procedures & Triggers)",
            test_files: vec![("stored_procedure_trigger_test", TestType::IntegrationTest)],
            description: "存储过程和触发器解析与执行测试",
        },
    ]
}

/// 运行单个测试文件
fn run_test_file(test_file: &str, test_type: &TestType) -> TestResult {
    let start = Instant::now();

    let output = match test_type {
        TestType::IntegrationTest => Command::new("cargo")
            .args(&["test", "--test", test_file, "--", "--nocapture"])
            .output(),
        TestType::CrateTest { crate_name } => Command::new("cargo")
            .args(&["test", "-p", crate_name, "--", "--nocapture"])
            .output(),
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
        "║                     SQLRustGo v2.1.0 综合回归测试报告                              ║"
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
        for (test_file, test_type) in &category.test_files {
            print!("  ⏳ {}... ", test_file);
            let result = run_test_file(test_file, test_type);

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
            .filter(|r| {
                category
                    .test_files
                    .iter()
                    .any(|(name, _)| name == &r.test_file)
            })
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
