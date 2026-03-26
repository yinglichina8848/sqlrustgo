//! RIGHT JOIN / FULL OUTER JOIN Tests
//!
//! P2 tests for Query Engine JOIN operations per TEST_PLAN.md
//! Tests RIGHT JOIN and FULL OUTER JOIN functionality

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

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "value".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&left_info).ok();

        storage
            .insert(
                "left_table",
                vec![vec![Value::Integer(1), Value::Text("A".to_string())]],
            )
            .ok();
        storage
            .insert(
                "left_table",
                vec![vec![Value::Integer(2), Value::Text("B".to_string())]],
            )
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&right_info).ok();

        storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(2), Value::Text("Right2".to_string())]],
            )
            .ok();
        storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(3), Value::Text("Right3".to_string())]],
            )
            .ok();

        storage
    }

    #[test]
    fn test_right_join_basic() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("left_id".to_string(), DataType::Integer),
            Field::new("left_value".to_string(), DataType::Text),
            Field::new("right_id".to_string(), DataType::Integer),
            Field::new("right_name".to_string(), DataType::Text),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Right,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join);

        assert!(result.is_ok(), "Right join should execute without error");
    }

    #[test]
    fn test_full_outer_join_basic() {
        let storage = create_test_storage();
        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("left_id".to_string(), DataType::Integer),
            Field::new("right_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Full,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join);

        assert!(
            result.is_ok(),
            "Full outer join should execute without error"
        );
    }

    #[test]
    fn test_right_join_no_match() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "left".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage.insert("left", vec![vec![Value::Integer(100)]]).ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "right".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            }],
        };

        storage.create_table(&right_info).ok();
        storage.insert("right", vec![vec![Value::Integer(1)]]).ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(SeqScanExec::new("left".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("right".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("l_id".to_string(), DataType::Integer),
            Field::new("r_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Right,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join);

        assert!(result.is_ok());
    }

    #[test]
    fn test_right_join_with_multiple_matches() {
        let mut storage = MemoryStorage::new();

        let left_info = sqlrustgo_storage::TableInfo {
            name: "categories".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            }],
        };

        storage.create_table(&left_info).ok();
        storage
            .insert("categories", vec![vec![Value::Integer(1)]])
            .ok();

        let right_info = sqlrustgo_storage::TableInfo {
            name: "products".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "category_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&right_info).ok();
        storage
            .insert("products", vec![vec![Value::Integer(1), Value::Integer(1)]])
            .ok();
        storage
            .insert("products", vec![vec![Value::Integer(2), Value::Integer(1)]])
            .ok();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("category_id".to_string(), DataType::Integer),
        ]);

        let left_scan = Box::new(SeqScanExec::new("categories".to_string(), left_schema));
        let right_scan = Box::new(SeqScanExec::new("products".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("cat_id".to_string(), DataType::Integer),
            Field::new("prod_id".to_string(), DataType::Integer),
            Field::new("prod_cat".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(
            left_scan,
            right_scan,
            JoinType::Right,
            Some(Expr::binary_expr(
                Expr::column("id"),
                Operator::Eq,
                Expr::column("category_id"),
            )),
            join_schema,
        );

        let result = engine.execute_plan(&join);

        assert!(result.is_ok());
    }
}
