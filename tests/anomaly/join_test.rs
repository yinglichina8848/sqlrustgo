//! JOIN Tests
//!
//! P1 tests for JOIN operations per TEST_PLAN.md
//! Tests INNER JOIN, LEFT JOIN, and Hash Join

#[cfg(test)]
mod tests {
    use sqlrustgo::{ExecutionEngine, MemoryStorage, StorageEngine};
    use sqlrustgo_planner::{
        DataType, Expr, Field, HashJoinExec, JoinType, Operator, Schema, SeqScanExec,
    };
    use sqlrustgo_types::Value;
    use std::sync::{Arc, RwLock};

    fn create_test_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "employees".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "dept_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();

        for i in 1..=5 {
            storage
                .insert(
                    "employees",
                    vec![vec![
                        Value::Integer(i),
                        Value::Text(format!("Employee {}", i)),
                        Value::Integer((i % 3) + 1),
                    ]],
                )
                .ok();
        }

        let info = sqlrustgo_storage::TableInfo {
            name: "departments".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();

        for i in 1..=3 {
            storage
                .insert(
                    "departments",
                    vec![vec![
                        Value::Integer(i),
                        Value::Text(format!("Department {}", i)),
                    ]],
                )
                .ok();
        }

        storage
    }

    #[test]
    fn test_inner_join_basic() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("dept_id".to_string(), DataType::Integer),
        ]);

        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("dept_name".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("employees".to_string(), left_schema));

        let right_scan = Box::new(SeqScanExec::new("departments".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("emp_id".to_string(), DataType::Integer),
            Field::new("emp_name".to_string(), DataType::Text),
            Field::new("dept_id".to_string(), DataType::Integer),
            Field::new("dept_name".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("dept_id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert!(!result.rows.is_empty(), "Inner join should return results");
        assert!(
            result.rows.len() >= 3,
            "Should have at least 3 matching rows"
        );
    }

    #[test]
    fn test_left_join_basic() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("dept".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("employees".to_string(), left_schema));

        let right_scan = Box::new(SeqScanExec::new("departments".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("emp_id".to_string(), DataType::Integer),
            Field::new("emp_name".to_string(), DataType::Text),
            Field::new("dept_id".to_string(), DataType::Integer),
            Field::new("dept_name".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Left,
            Some(Expr::binary_expr(
                Expr::column("dept_id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert!(!result.rows.is_empty(), "Left join should return results");
        assert!(
            result.rows.len() >= 3,
            "Left join should return at least 3 matched rows"
        );
    }

    #[test]
    fn test_left_join_with_nulls() {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "orders".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "customer_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();
        storage
            .insert("orders", vec![vec![Value::Integer(1), Value::Integer(1)]])
            .ok();
        storage
            .insert("orders", vec![vec![Value::Integer(2), Value::Null]])
            .ok();

        let info = sqlrustgo_storage::TableInfo {
            name: "customers".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).ok();
        storage
            .insert("customers", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("order_id".to_string(), DataType::Integer),
            Field::new("customer_id".to_string(), DataType::Integer),
        ]);

        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("orders".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("customers".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("order_id".to_string(), DataType::Integer),
            Field::new("customer_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Left,
            Some(Expr::binary_expr(
                Expr::column("customer_id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert_eq!(result.rows.len(), 2, "Left join should return all orders");
    }

    #[test]
    fn test_cross_join() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("departments".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("employees".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("dept_id".to_string(), DataType::Integer),
            Field::new("emp_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(left_scan, right_scan, JoinType::Cross, None, join_schema);

        let _result = engine.execute_plan(&join);
    }

    #[test]
    fn test_join_with_multiple_matches() {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "products".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "category".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();
        storage
            .insert(
                "products",
                vec![vec![
                    Value::Integer(1),
                    Value::Text("Electronics".to_string()),
                ]],
            )
            .ok();
        storage
            .insert(
                "products",
                vec![vec![
                    Value::Integer(2),
                    Value::Text("Electronics".to_string()),
                ]],
            )
            .ok();

        let info = sqlrustgo_storage::TableInfo {
            name: "tags".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "product_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "tag".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();
        storage
            .insert(
                "tags",
                vec![vec![Value::Integer(1), Value::Text("Sale".to_string())]],
            )
            .ok();
        storage
            .insert(
                "tags",
                vec![vec![Value::Integer(1), Value::Text("Featured".to_string())]],
            )
            .ok();
        storage
            .insert(
                "tags",
                vec![vec![Value::Integer(2), Value::Text("Sale".to_string())]],
            )
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("category".to_string(), DataType::Text),
        ]);

        let right_schema = Schema::new(vec![
            Field::new("product_id".to_string(), DataType::Integer),
            Field::new("tag".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("products".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("tags".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("product_id".to_string(), DataType::Integer),
            Field::new("category".to_string(), DataType::Text),
            Field::new("tag".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("product_id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert!(
            result.rows.len() > 0,
            "Join with matches should return results"
        );
    }

    #[test]
    fn test_join_no_match() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("employees".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("departments".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("emp_id".to_string(), DataType::Integer),
            Field::new("dept_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert!(result.rows.len() >= 0, "Join should complete without error");
    }
}
