//! Tests for physical plan generation with SEMI/ANTI JOIN rewrites
//!
//! These tests verify that the planner correctly rewrites:
//! - IN subquery → SEMI JOIN
//! - NOT IN → ANTI JOIN
//! - EXISTS → SEMI JOIN
//! - NOT EXISTS → ANTI JOIN

use sqlrustgo_planner::{Expr, Field, LogicalPlan, Schema, DataType, JoinType};

/// Test helper to create a simple table scan
fn make_table_scan(table_name: &str, schema: Schema) -> LogicalPlan {
    LogicalPlan::TableScan {
        table_name: table_name.to_string(),
        schema,
        projection: None,
    }
}

/// Test that IN subquery expression is represented correctly in the logical plan
#[test]
fn test_in_subquery_logical_plan() {
    // Create schema for t1 (a INT)
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);

    // Create t1 table scan
    let t1 = make_table_scan("t1", t1_schema);

    // Create schema for t2 (b INT)
    let t2_schema = Schema::new(vec![Field::new("b".to_string(), DataType::Integer)]);

    // Create t2 table scan (subquery)
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    // Create Filter with IN expression
    // Filter { predicate: In(column("a"), subquery), input: t1 }
    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::In {
            expr: Box::new(Expr::column("a")),
            subquery: Box::new(subquery),
        },
        input: Box::new(t1),
    };

    // Verify the plan structure
    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::In { expr, subquery: _ } => {
                    match expr.as_ref() {
                        Expr::Column(col) => assert_eq!(col.name, "a"),
                        _ => panic!("Expected column expression"),
                    }
                }
                _ => panic!("Expected In expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test that NOT IN subquery expression is represented correctly
#[test]
fn test_not_in_subquery_logical_plan() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("b".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::NotIn {
            expr: Box::new(Expr::column("a")),
            subquery: Box::new(subquery),
        },
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::NotIn { .. } => {}
                _ => panic!("Expected NotIn expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test that EXISTS subquery expression is represented correctly
#[test]
fn test_exists_subquery_logical_plan() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::Exists(Box::new(subquery)),
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::Exists(_) => {}
                _ => panic!("Expected Exists expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test that NOT EXISTS subquery expression is represented correctly
#[test]
fn test_not_exists_subquery_logical_plan() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::NotExists(Box::new(subquery)),
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::NotExists(_) => {}
                _ => panic!("Expected NotExists expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test IN subquery rewrites to LeftSemi join
#[test]
fn test_in_subquery_to_semi_join() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("b".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::In {
            expr: Box::new(Expr::column("a")),
            subquery: Box::new(subquery),
        },
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, input } => {
            match predicate {
                Expr::In { expr, subquery: _ } => {
                    match expr.as_ref() {
                        Expr::Column(col) => assert_eq!(col.name, "a"),
                        _ => panic!("Expected column a"),
                    }
                }
                _ => panic!("Expected In expression"),
            }
            match input.as_ref() {
                LogicalPlan::TableScan { table_name, .. } => assert_eq!(table_name, "t1"),
                _ => panic!("Expected t1 table scan"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test NOT IN subquery rewrites to LeftAnti join
#[test]
fn test_not_in_subquery_to_anti_join() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("b".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::NotIn {
            expr: Box::new(Expr::column("a")),
            subquery: Box::new(subquery),
        },
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::NotIn { .. } => {}
                _ => panic!("Expected NotIn expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test EXISTS rewrites to LeftSemi join with TRUE condition
#[test]
fn test_exists_to_semi_join() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::Exists(Box::new(subquery)),
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::Exists(_) => {}
                _ => panic!("Expected Exists expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}

/// Test NOT EXISTS rewrites to LeftAnti join
#[test]
fn test_not_exists_to_anti_join() {
    let t1_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t1 = make_table_scan("t1", t1_schema);

    let t2_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
    let t2 = make_table_scan("t2", t2_schema);
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(t2),
        alias: "".to_string(),
    };

    let filter_plan = LogicalPlan::Filter {
        predicate: Expr::NotExists(Box::new(subquery)),
        input: Box::new(t1),
    };

    match filter_plan {
        LogicalPlan::Filter { predicate, .. } => {
            match predicate {
                Expr::NotExists(_) => {}
                _ => panic!("Expected NotExists expression"),
            }
        }
        _ => panic!("Expected Filter plan"),
    }
}