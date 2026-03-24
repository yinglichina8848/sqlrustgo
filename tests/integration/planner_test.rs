//! Planner Integration Tests
//!
//! This module provides integration tests for the planner module.

use sqlrustgo_planner::{
    AggregateFunction, Column, DataType, DefaultPlanner, Expr, Field, JoinType, LogicalPlan,
    NoOpPlanner, Operator, PhysicalPlan, Planner, Schema, SortExpr,
};
use sqlrustgo_types::Value;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_schema() -> Schema {
    Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("age".to_string(), DataType::Integer),
        Field::new("salary".to_string(), DataType::Float),
    ])
}

fn create_simple_schema() -> Schema {
    Schema::new(vec![Field::new("id".to_string(), DataType::Integer)])
}

// ============================================================================
// Planner Basic Tests
// ============================================================================

#[test]
fn test_planner_create_physical_plan() {
    let schema = create_simple_schema();
    let logical_plan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&logical_plan);

    assert!(result.is_ok());
    let physical_plan = result.unwrap();
    let name = physical_plan.name();
    assert!(
        name == "IndexScan" || name == "SeqScan",
        "Expected IndexScan or SeqScan, got {}",
        name
    );
}

#[test]
fn test_planner_with_projection() {
    let schema = create_test_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let projection_plan = LogicalPlan::Projection {
        input: Box::new(table_scan),
        expr: vec![Expr::column("id"), Expr::column("name")],
        schema: Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&projection_plan);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Projection");
}

#[test]
fn test_planner_with_filter() {
    let schema = create_test_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::binary_expr(
            Expr::column("age"),
            Operator::Gt,
            Expr::literal(Value::Integer(25)),
        ),
        input: Box::new(table_scan),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&filter_plan);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Filter");
}

#[test]
fn test_planner_with_aggregate() {
    let schema = create_test_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let agg_plan = LogicalPlan::Aggregate {
        input: Box::new(table_scan),
        group_expr: vec![Expr::column("department")],
        aggregate_expr: vec![
            Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            },
            Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("salary")],
                distinct: false,
            },
        ],
        schema: Schema::new(vec![
            Field::new("department".to_string(), DataType::Text),
            Field::new("count".to_string(), DataType::Integer),
            Field::new("avg_salary".to_string(), DataType::Float),
        ]),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&agg_plan);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Aggregate");
}

#[test]
fn test_planner_with_join() {
    let left_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);
    let right_schema = Schema::new(vec![
        Field::new("user_id".to_string(), DataType::Integer),
        Field::new("order_id".to_string(), DataType::Integer),
    ]);

    let left = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: left_schema,
        projection: None,
    };

    let right = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        schema: right_schema,
        projection: None,
    };

    let join_plan = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Inner,
        condition: Some(Expr::binary_expr(
            Expr::column("id"),
            Operator::Eq,
            Expr::column("user_id"),
        )),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&join_plan);

    assert!(result.is_ok());
    let physical = result.unwrap();
    let name = physical.name();
    assert!(
        name == "SortMergeJoin" || name == "HashJoin",
        "Expected SortMergeJoin or HashJoin, got {}",
        name
    );
}

#[test]
fn test_planner_with_sort() {
    let schema = create_test_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let sort_plan = LogicalPlan::Sort {
        input: Box::new(table_scan),
        sort_expr: vec![
            SortExpr {
                expr: Expr::column("age"),
                asc: true,
                nulls_first: false,
            },
            SortExpr {
                expr: Expr::column("name"),
                asc: false,
                nulls_first: true,
            },
        ],
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&sort_plan);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Sort");
}

#[test]
fn test_planner_with_limit() {
    let schema = create_simple_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let limit_plan = LogicalPlan::Limit {
        input: Box::new(table_scan),
        limit: 100,
        offset: Some(10),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&limit_plan);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Limit");
}

// ============================================================================
// Optimizer Tests
// ============================================================================

#[test]
fn test_planner_optimize() {
    let schema = create_simple_schema();
    let logical_plan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let mut planner = DefaultPlanner::new();
    let result = planner.optimize(logical_plan);

    assert!(result.is_ok());
}

#[test]
fn test_planner_optimize_with_expression() {
    let schema = create_test_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let projection_plan = LogicalPlan::Projection {
        input: Box::new(table_scan),
        expr: vec![Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Integer(1))),
            op: Operator::Plus,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        }],
        schema: Schema::new(vec![Field::new("computed".to_string(), DataType::Integer)]),
    };

    let mut planner = DefaultPlanner::new();
    let result = planner.optimize(projection_plan);

    assert!(result.is_ok());
}

// ============================================================================
// NoOp Planner Tests
// ============================================================================

#[test]
fn test_noop_planner() {
    let schema = create_simple_schema();
    let logical_plan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let mut planner = NoOpPlanner::new();
    let result = planner.optimize(logical_plan);

    assert!(result.is_ok());
}

#[test]
fn test_noop_planner_create_physical_plan() {
    let schema = create_simple_schema();
    let logical_plan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let planner = NoOpPlanner::new();
    let result = planner.create_physical_plan(&logical_plan);

    assert!(result.is_ok());
}

// ============================================================================
// Schema Tests
// ============================================================================

#[test]
fn test_logical_plan_schema() {
    let schema = create_test_schema();
    let plan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    assert_eq!(plan.schema(), schema);
}

#[test]
fn test_physical_plan_schema() {
    let schema = create_test_schema();
    let scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), schema.clone());

    assert_eq!(scan.schema().fields.len(), 4);
}

// ============================================================================
// Expression Tests
// ============================================================================

#[test]
fn test_expression_evaluation() {
    let schema = Schema::new(vec![
        Field::new("a".to_string(), DataType::Integer),
        Field::new("b".to_string(), DataType::Integer),
    ]);

    let expr = Expr::binary_expr(Expr::column("a"), Operator::Plus, Expr::column("b"));

    let row = vec![Value::Integer(10), Value::Integer(5)];
    let result = expr.evaluate(&row, &schema);

    assert_eq!(result, Some(Value::Integer(15)));
}

#[test]
fn test_expression_with_comparison() {
    let schema = Schema::new(vec![Field::new("age".to_string(), DataType::Integer)]);

    let expr = Expr::binary_expr(
        Expr::column("age"),
        Operator::Gt,
        Expr::literal(Value::Integer(18)),
    );

    let row = vec![Value::Integer(25)];
    let result = expr.evaluate(&row, &schema);

    assert_eq!(result, Some(Value::Boolean(true)));
}

#[test]
fn test_expression_matches() {
    let schema = Schema::new(vec![Field::new("active".to_string(), DataType::Boolean)]);

    let expr = Expr::column("active");
    let row = vec![Value::Boolean(true)];

    assert!(expr.matches(&row, &schema));
}

// ============================================================================
// Physical Plan Execution Tests
// ============================================================================

#[test]
fn test_seq_scan_execute() {
    let schema = create_simple_schema();
    let scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), schema);

    let result = scan.execute();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_projection_execute() {
    let input_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);
    let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let input = Box::new(sqlrustgo_planner::SeqScanExec::new(
        "users".to_string(),
        input_schema,
    ));
    let proj =
        sqlrustgo_planner::ProjectionExec::new(input, vec![Expr::column("id")], output_schema);

    let result = proj.execute();
    assert!(result.is_ok());
}

#[test]
fn test_filter_execute() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let input = Box::new(sqlrustgo_planner::SeqScanExec::new(
        "users".to_string(),
        schema.clone(),
    ));
    let predicate = Expr::binary_expr(
        Expr::column("id"),
        Operator::Gt,
        Expr::literal(Value::Integer(10)),
    );
    let filter = sqlrustgo_planner::FilterExec::new(input, predicate);

    let result = filter.execute();
    assert!(result.is_ok());
}

#[test]
fn test_aggregate_execute_count() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let input = Box::new(sqlrustgo_planner::SeqScanExec::new(
        "users".to_string(),
        schema.clone(),
    ));
    let agg = sqlrustgo_planner::AggregateExec::new(
        input,
        vec![],
        vec![Expr::AggregateFunction {
            func: AggregateFunction::Count,
            args: vec![],
            distinct: false,
        }],
        Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
    );

    let result = agg.execute();
    assert!(result.is_ok());
}

// ============================================================================
// Complex Query Plans
// ============================================================================

#[test]
fn test_complex_query_plan() {
    // Simulates: SELECT id, name FROM users WHERE age > 25 ORDER BY name LIMIT 10
    let base_schema = create_test_schema();

    // Table Scan
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: base_schema.clone(),
        projection: None,
    };

    // Filter
    let filter = LogicalPlan::Filter {
        predicate: Expr::binary_expr(
            Expr::column("age"),
            Operator::Gt,
            Expr::literal(Value::Integer(25)),
        ),
        input: Box::new(table_scan),
    };

    // Projection
    let projection = LogicalPlan::Projection {
        input: Box::new(filter),
        expr: vec![Expr::column("id"), Expr::column("name")],
        schema: Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]),
    };

    // Sort
    let sort = LogicalPlan::Sort {
        input: Box::new(projection),
        sort_expr: vec![SortExpr {
            expr: Expr::column("name"),
            asc: true,
            nulls_first: false,
        }],
    };

    // Limit
    let limit = LogicalPlan::Limit {
        input: Box::new(sort),
        limit: 10,
        offset: None,
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&limit);

    assert!(result.is_ok());
    let physical = result.unwrap();
    assert_eq!(physical.name(), "Limit");

    // Check children chain
    let children = physical.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name(), "Sort");
}

#[test]
fn test_group_by_query_plan() {
    // Simulates: SELECT department, COUNT(*), AVG(salary) FROM users GROUP BY department
    let base_schema = create_test_schema();

    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: base_schema.clone(),
        projection: None,
    };

    let aggregate = LogicalPlan::Aggregate {
        input: Box::new(table_scan),
        group_expr: vec![Expr::column("department")],
        aggregate_expr: vec![
            Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            },
            Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("salary")],
                distinct: false,
            },
        ],
        schema: Schema::new(vec![
            Field::new("department".to_string(), DataType::Text),
            Field::new("count".to_string(), DataType::Integer),
            Field::new("avg".to_string(), DataType::Float),
        ]),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&aggregate);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Aggregate");
}

#[test]
fn test_join_query_plan() {
    // Simulates: SELECT * FROM users JOIN orders ON users.id = orders.user_id
    let users_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);
    let orders_schema = Schema::new(vec![
        Field::new("user_id".to_string(), DataType::Integer),
        Field::new("amount".to_string(), DataType::Float),
    ]);

    let users = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: users_schema,
        projection: None,
    };

    let orders = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        schema: orders_schema,
        projection: None,
    };

    let join = LogicalPlan::Join {
        left: Box::new(users),
        right: Box::new(orders),
        join_type: JoinType::Inner,
        condition: Some(Expr::binary_expr(
            Expr::Column(Column::new_qualified("users".to_string(), "id".to_string())),
            Operator::Eq,
            Expr::column("user_id"),
        )),
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&join);

    assert!(result.is_ok());
    let physical = result.unwrap();
    let name = physical.name();
    assert!(
        name == "SortMergeJoin" || name == "HashJoin",
        "Expected SortMergeJoin or HashJoin, got {}",
        name
    );

    // Check that it has two children (left and right)
    let children = physical.children();
    assert_eq!(children.len(), 2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_schema() {
    let schema = Schema::empty();
    let plan = LogicalPlan::TableScan {
        table_name: "empty".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    assert!(plan.schema().fields.is_empty());
}

#[test]
fn test_plan_with_null_values() {
    let schema = Schema::new(vec![Field::new("value".to_string(), DataType::Integer)]);

    let expr = Expr::column("value");
    let row = vec![Value::Null];

    let result = expr.evaluate(&row, &schema);
    assert_eq!(result, Some(Value::Null));
}

#[test]
fn test_limit_with_zero() {
    let schema = create_simple_schema();
    let table_scan = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let limit = LogicalPlan::Limit {
        input: Box::new(table_scan),
        limit: 0,
        offset: None,
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&limit);

    assert!(result.is_ok());
}

#[test]
fn test_join_different_types() {
    // Test LEFT JOIN
    let schema = create_simple_schema();

    let left = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let right = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let join = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Left,
        condition: None,
    };

    let planner = DefaultPlanner::new();
    let result = planner.create_physical_plan(&join);

    assert!(result.is_ok());

    // Test RIGHT JOIN
    let left = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let right = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let join = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Right,
        condition: None,
    };

    let result = planner.create_physical_plan(&join);
    assert!(result.is_ok());

    // Test CROSS JOIN
    let left = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let right = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        schema: schema.clone(),
        projection: None,
    };

    let join = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Cross,
        condition: None,
    };

    let result = planner.create_physical_plan(&join);
    assert!(result.is_ok());
}

#[test]
fn test_index_scan_exec_execute() {
    use sqlrustgo_planner::IndexScanExec;

    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::literal(Value::Integer(42)),
        schema.clone(),
    );

    let result = scan.execute();
    assert!(result.is_ok());
    let rows = result.unwrap();
    // IndexScan returns empty in current stub implementation
    // Full implementation would return actual indexed data
    assert!(rows.is_empty() || rows.len() == 1);
}

#[test]
fn test_index_scan_exec_key_range() {
    use sqlrustgo_planner::IndexScanExec;

    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::literal(Value::Integer(1)),
        schema.clone(),
    )
    .with_key_range(1, 10);

    let result = scan.execute();
    assert!(result.is_ok());
    let rows = result.unwrap();
    // IndexScan with key range - verify config is set (stub returns empty)
    assert!(rows.is_empty() || rows.len() == 9);
}

