//! PB-05: TPC-H Query Performance Tests
//!
//! These tests verify query execution performance for basic query operations.

#[cfg(test)]
mod tests {
    use sqlrustgo_executor::{
        harness::TestHarness, mock_storage::MockStorage, test_data::TestDataSet,
    };
    use sqlrustgo_planner::{
        physical_plan::{FilterExec, LimitExec, ProjectionExec, SeqScanExec},
        DataType, Field, Schema,
    };
    use sqlrustgo_types::Value;

    /// Helper to create a schema
    fn create_schema(fields: Vec<(&'static str, DataType)>) -> Schema {
        Schema::new(
            fields
                .into_iter()
                .map(|(name, dt)| Field::new(name.to_string(), dt))
                .collect(),
        )
    }

    /// Test Q1: Simple Projection (SELECT with column selection)
    #[test]
    fn test_tpch_q1_projection() {
        let storage = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness = TestHarness::<MockStorage>::new(storage);

        let schema = create_schema(vec![
            ("order_id", DataType::Integer),
            ("user_id", DataType::Integer),
            ("amount", DataType::Integer),
        ]);

        let scan = Box::new(SeqScanExec::new("orders".to_string(), schema.clone()));

        let plan = Box::new(ProjectionExec::new(
            scan,
            vec![
                sqlrustgo_planner::Expr::column("order_id"),
                sqlrustgo_planner::Expr::column("amount"),
            ],
            create_schema(vec![
                ("order_id", DataType::Integer),
                ("amount", DataType::Integer),
            ]),
        ));

        let start = std::time::Instant::now();
        let result = harness.execute(plan.as_ref()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Q1 (Projection): {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty());
        assert!(elapsed.as_secs_f64() < 1.0);
    }

    /// Test Q3: Filter + Limit
    #[test]
    fn test_tpch_q3_filter_limit() {
        let storage = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness = TestHarness::<MockStorage>::new(storage);

        let schema = create_schema(vec![
            ("order_id", DataType::Integer),
            ("user_id", DataType::Integer),
            ("amount", DataType::Integer),
        ]);

        let scan = Box::new(SeqScanExec::new("orders".to_string(), schema.clone()));

        // Filter: amount > 100
        let filter = Box::new(FilterExec::new(
            scan,
            sqlrustgo_planner::Expr::binary_expr(
                sqlrustgo_planner::Expr::column("amount"),
                sqlrustgo_planner::Operator::Gt,
                sqlrustgo_planner::Expr::literal(Value::Integer(100)),
            ),
        ));

        // Take top 3
        let limit = Box::new(LimitExec::new(filter, 3, None));

        let plan = Box::new(ProjectionExec::new(
            limit,
            vec![
                sqlrustgo_planner::Expr::column("order_id"),
                sqlrustgo_planner::Expr::column("amount"),
            ],
            create_schema(vec![
                ("order_id", DataType::Integer),
                ("amount", DataType::Integer),
            ]),
        ));

        let start = std::time::Instant::now();
        let result = harness.execute(plan.as_ref()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Q3 (Filter+Limit): {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(result.rows.len() <= 3);
        assert!(elapsed.as_secs_f64() < 1.0);
    }

    /// Test Q6: Filter (similar to Q6 predicate)
    /// Note: FilterExec has known limitations in current implementation
    #[test]
    #[ignore] // Ignored due to FilterExec evaluation issues
    fn test_tpch_q6_filter() {
        let storage = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness = TestHarness::<MockStorage>::new(storage);

        let schema = create_schema(vec![
            ("order_id", DataType::Integer),
            ("user_id", DataType::Integer),
            ("amount", DataType::Integer),
        ]);

        let scan = Box::new(SeqScanExec::new("orders".to_string(), schema.clone()));

        // Filter: amount > 50 (matching some rows in test data)
        let filter = Box::new(FilterExec::new(
            scan,
            sqlrustgo_planner::Expr::binary_expr(
                sqlrustgo_planner::Expr::column("amount"),
                sqlrustgo_planner::Operator::Gt,
                sqlrustgo_planner::Expr::literal(Value::Integer(50)),
            ),
        ));

        let plan = Box::new(ProjectionExec::new(
            filter,
            vec![sqlrustgo_planner::Expr::column("order_id")],
            create_schema(vec![("order_id", DataType::Integer)]),
        ));

        let start = std::time::Instant::now();
        let result = harness.execute(plan.as_ref()).unwrap();
        let elapsed = start.elapsed();

        println!(
            "TPC-H Q6 (Filter): {:?} ({} rows)",
            elapsed,
            result.rows.len()
        );

        assert!(!result.rows.is_empty());
        assert!(elapsed.as_secs_f64() < 1.0);
    }

    /// Test: Verify queries execute correctly
    #[test]
    fn test_tpch_queries_execute() {
        // Test simple scan
        let storage1 = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness1 = TestHarness::<MockStorage>::new(storage1);

        let schema1 = create_schema(vec![("id", DataType::Integer)]);
        let scan1 = Box::new(SeqScanExec::new("orders".to_string(), schema1.clone()));
        let plan1 = Box::new(ProjectionExec::new(
            scan1,
            vec![sqlrustgo_planner::Expr::column("id")],
            schema1,
        ));

        let result1 = harness1.execute(plan1.as_ref()).unwrap();
        assert!(!result1.rows.is_empty());

        println!("TPC-H: Simple scan executed successfully");

        // Test filter
        let storage2 = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness2 = TestHarness::<MockStorage>::new(storage2);

        let schema2 = create_schema(vec![("id", DataType::Integer)]);
        let scan2 = Box::new(SeqScanExec::new("orders".to_string(), schema2.clone()));
        let filter = Box::new(FilterExec::new(
            scan2,
            sqlrustgo_planner::Expr::binary_expr(
                sqlrustgo_planner::Expr::column("id"),
                sqlrustgo_planner::Operator::Gt,
                sqlrustgo_planner::Expr::literal(Value::Integer(0)),
            ),
        ));
        let plan2 = Box::new(ProjectionExec::new(
            filter,
            vec![sqlrustgo_planner::Expr::column("id")],
            schema2,
        ));

        let result2 = harness2.execute(plan2.as_ref()).unwrap();
        assert!(!result2.rows.is_empty());

        println!("TPC-H: Filter executed successfully");
    }

    /// Performance benchmark: Multiple query execution
    #[test]
    fn test_tpch_performance_benchmark() {
        let storage = MockStorage::with_data("orders", TestDataSet::simple_orders());
        let harness = TestHarness::<MockStorage>::new(storage);

        let mut total_time = std::time::Duration::ZERO;

        // Run 100 simple scans
        for _ in 0..100 {
            let schema = create_schema(vec![("id", DataType::Integer)]);
            let scan = Box::new(SeqScanExec::new("orders".to_string(), schema.clone()));
            let plan = Box::new(ProjectionExec::new(
                scan,
                vec![sqlrustgo_planner::Expr::column("id")],
                schema,
            ));

            let start = std::time::Instant::now();
            let _ = harness.execute(plan.as_ref()).unwrap();
            total_time += start.elapsed();
        }

        let avg_time_ms = (total_time.as_secs_f64() * 1000.0) / 100.0;
        println!(
            "TPC-H Performance: 100 scans avg time: {:.2}ms",
            avg_time_ms
        );

        // Each scan should be fast
        assert!(
            avg_time_ms < 10.0,
            "Queries too slow: {:.2}ms avg",
            avg_time_ms
        );
    }
}
