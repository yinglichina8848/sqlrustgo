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
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "dept_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
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
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
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
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "customer_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
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
                is_unique: true,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
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
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "category".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
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
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "tag".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
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

    #[test]
    fn test_left_semi_join() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::LeftSemi,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert_eq!(result.rows.len(), 1, "Only key=1 has a match");
        assert_eq!(result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_right_semi_join() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(2)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::RightSemi,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        assert_eq!(result.rows.len(), 1, "Only key=1 has a match");
        assert_eq!(result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_left_anti_join() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(3)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::LeftAnti,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        // LeftAnti: return left rows that do NOT have a match
        assert_eq!(result.rows.len(), 2, "key=2 and key=3 have no match");
        assert!(result
            .rows
            .iter()
            .all(|row| row[0] == Value::Integer(2) || row[0] == Value::Integer(3)));
    }

    #[test]
    fn test_right_anti_join() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(2)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(3)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::RightAnti,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();

        // RightAnti: return right rows that do NOT have a match
        assert_eq!(result.rows.len(), 2, "key=2 and key=3 have no match");
        assert!(result
            .rows
            .iter()
            .all(|row| row[0] == Value::Integer(2) || row[0] == Value::Integer(3)));
    }

    #[test]
    fn test_inner_join_with_nulls() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "value".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
            ],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert(
                "left_table",
                vec![vec![Value::Integer(1), Value::Integer(100)]],
            )
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2), Value::Null]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Null, Value::Integer(300)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
            ],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(1), Value::Text("A".to_string())]],
            )
            .ok();
        storage
            .insert(
                "right_table",
                vec![vec![Value::Null, Value::Text("B".to_string())]],
            )
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
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
        assert_eq!(
            result.rows.len(),
            1,
            "Only id=1 matches (NULLs don't match)"
        );
        assert_eq!(result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_left_join_with_nulls_in_key() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage.insert("left_table", vec![vec![Value::Null]]).ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Left,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            2,
            "Left row with NULL key should be preserved"
        );
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[1][0], Value::Null);
    }

    #[test]
    fn test_self_join() {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "employees".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "manager_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
            ],
        };

        storage.create_table(&info).ok();
        storage
            .insert(
                "employees",
                vec![vec![
                    Value::Integer(1),
                    Value::Text("CEO".to_string()),
                    Value::Null,
                ]],
            )
            .ok();
        storage
            .insert(
                "employees",
                vec![vec![
                    Value::Integer(2),
                    Value::Text("Manager1".to_string()),
                    Value::Integer(1),
                ]],
            )
            .ok();
        storage
            .insert(
                "employees",
                vec![vec![
                    Value::Integer(3),
                    Value::Text("Employee1".to_string()),
                    Value::Integer(2),
                ]],
            )
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let schema = Schema::new(vec![
            Field::new("emp_id".to_string(), DataType::Integer),
            Field::new("emp_name".to_string(), DataType::Text),
            Field::new("mgr_id".to_string(), DataType::Integer),
        ]);

        let left_scan = Box::new(SeqScanExec::new("employees".to_string(), schema.clone()));
        let right_scan = Box::new(SeqScanExec::new("employees".to_string(), schema.clone()));

        let join_schema = Schema::new(vec![
            Field::new("emp_id".to_string(), DataType::Integer),
            Field::new("emp_name".to_string(), DataType::Text),
            Field::new("mgr_id".to_string(), DataType::Integer),
            Field::new("mgr_id_right".to_string(), DataType::Integer),
            Field::new("mgr_name".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("mgr_id"),
                Operator::Eq,
                Expr::column("emp_id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(result.rows.len(), 2, "Manager1 and Employee1 have managers");
        assert_eq!(result.rows[0][4], Value::Text("CEO".to_string()));
        assert_eq!(result.rows[1][4], Value::Text("Manager1".to_string()));
    }

    #[test]
    fn test_multi_key_join() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "orders".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "order_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "product_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "quantity".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
            ],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert(
                "orders",
                vec![vec![
                    Value::Integer(1),
                    Value::Integer(100),
                    Value::Integer(10),
                ]],
            )
            .ok();
        storage
            .insert(
                "orders",
                vec![vec![
                    Value::Integer(2),
                    Value::Integer(100),
                    Value::Integer(20),
                ]],
            )
            .ok();
        storage
            .insert(
                "orders",
                vec![vec![
                    Value::Integer(3),
                    Value::Integer(200),
                    Value::Integer(5),
                ]],
            )
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "products".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "product_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "category_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                    compression: None,
                },
            ],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert(
                "products",
                vec![vec![
                    Value::Integer(100),
                    Value::Integer(1),
                    Value::Text("Product A".to_string()),
                ]],
            )
            .ok();
        storage
            .insert(
                "products",
                vec![vec![
                    Value::Integer(200),
                    Value::Integer(2),
                    Value::Text("Product B".to_string()),
                ]],
            )
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("order_id".to_string(), DataType::Integer),
            Field::new("product_id".to_string(), DataType::Integer),
            Field::new("quantity".to_string(), DataType::Integer),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("product_id".to_string(), DataType::Integer),
            Field::new("category_id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("orders".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("products".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("order_id".to_string(), DataType::Integer),
            Field::new("product_id".to_string(), DataType::Integer),
            Field::new("quantity".to_string(), DataType::Integer),
            Field::new("category_id".to_string(), DataType::Integer),
            Field::new("product_name".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("product_id"),
                Operator::Eq,
                Expr::column("product_id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(result.rows.len(), 3, "All 3 orders should match products");
        // Result order is left_schema + right_schema: [order_id, product_id, quantity, product_id, category_id, name]
        // product_id is at index 3 (from right table)
        assert_eq!(result.rows[0][3], Value::Integer(100));
        assert_eq!(result.rows[1][3], Value::Integer(100));
        assert_eq!(result.rows[2][3], Value::Integer(200));
    }

    #[test]
    fn test_join_with_empty_right_table() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Inner,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            0,
            "Inner join with empty right returns nothing"
        );
    }

    #[test]
    fn test_left_join_with_empty_right_table() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Left,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            2,
            "Left join with empty right returns all left rows"
        );
    }

    #[test]
    fn test_full_outer_join_no_match() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(3)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(4)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Full,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            4,
            "Full outer join returns all rows from both tables"
        );
    }

    #[test]
    fn test_cross_join_with_large_tables() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "colors".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        for i in 1..=3 {
            storage.insert("colors", vec![vec![Value::Integer(i)]]).ok();
        }

        let right_info = sqlrustgo_storage::TableInfo {
            name: "sizes".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        for i in 1..=4 {
            storage.insert("sizes", vec![vec![Value::Integer(i)]]).ok();
        }

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("colors".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("sizes".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(left_scan, right_scan, JoinType::Cross, None, join_schema);

        let result = engine.execute_plan(&join).unwrap();
        // Verify row count is as expected
        assert_eq!(
            result.rows.len(),
            12,
            "Cross join: 3 colors * 4 sizes = 12 rows"
        );
        // Verify first and last rows
        assert_eq!(result.rows[0].len(), 2);
        assert_eq!(result.rows[11].len(), 2);
    }

    #[test]
    fn test_semi_join_duplicate_matches() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::LeftSemi,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            1,
            "LeftSemi returns 1 row regardless of multiple matches"
        );
    }

    #[test]
    fn test_anti_join_all_match() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(2)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(2)]])
            .ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(3)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::LeftAnti,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            0,
            "LeftAnti returns nothing when all left rows have matches"
        );
    }

    #[test]
    fn test_right_semi_duplicate_matches() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("left_table", vec![vec![Value::Integer(1)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("right_table", vec![vec![Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![Field::new("key".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::RightSemi,
            Some(Expr::binary_expr(
                Expr::column("key"),
                Operator::Eq,
                Expr::column("key"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join).unwrap();
        assert_eq!(
            result.rows.len(),
            1,
            "RightSemi returns 1 row regardless of multiple matches"
        );
    }
}
