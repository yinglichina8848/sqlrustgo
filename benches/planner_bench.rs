use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::planner::{
    AggregateFunction, Column, DataType, Expr, Field, JoinType, LogicalPlan, Operator, Schema,
};

fn bench_logical_plan_simple(c: &mut Criterion) {
    use sqlrustgo::planner::{DataType, Field, LogicalPlan, Schema};

    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    c.bench_function("logical_plan_table_scan", |b| {
        b.iter(|| LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: schema.clone(),
        });
    });
}

fn bench_logical_plan_filter(c: &mut Criterion) {
    use sqlrustgo::planner::{Column, DataType, Expr, Field, LogicalPlan, Schema};

    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("age".to_string(), DataType::Integer),
    ]);

    let input = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        projection: None,
        filters: vec![],
        limit: None,
        schema: schema.clone(),
    };

    let predicate = Expr::BinaryExpr {
        left: Box::new(Expr::Column(Column::new("age".to_string()))),
        op: Operator::Gt,
        right: Box::new(Expr::Literal(sqlrustgo::Value::Integer(18))),
    };

    c.bench_function("logical_plan_filter", |b| {
        b.iter(|| LogicalPlan::Filter {
            input: Box::new(input.clone()),
            predicate: predicate.clone(),
        });
    });
}

fn bench_logical_plan_projection(c: &mut Criterion) {
    use sqlrustgo::planner::{Column, DataType, Expr, Field, LogicalPlan, Schema};

    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("age".to_string(), DataType::Integer),
    ]);

    let output_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    let input = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        projection: None,
        filters: vec![],
        limit: None,
        schema: schema.clone(),
    };

    let exprs = vec![
        Expr::Column(Column::new("id".to_string())),
        Expr::Column(Column::new("name".to_string())),
    ];

    c.bench_function("logical_plan_projection", |b| {
        b.iter(|| LogicalPlan::Projection {
            input: Box::new(input.clone()),
            expr: exprs.clone(),
            schema: output_schema.clone(),
        });
    });
}

fn bench_logical_plan_join(c: &mut Criterion) {
    use sqlrustgo::planner::{Column, DataType, Expr, Field, JoinType, LogicalPlan, Schema};

    let left_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    let right_schema = Schema::new(vec![
        Field::new("user_id".to_string(), DataType::Integer),
        Field::new("order_id".to_string(), DataType::Integer),
    ]);

    let join_schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("order_id".to_string(), DataType::Integer),
    ]);

    let left = LogicalPlan::TableScan {
        table_name: "users".to_string(),
        projection: None,
        filters: vec![],
        limit: None,
        schema: left_schema.clone(),
    };

    let right = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        projection: None,
        filters: vec![],
        limit: None,
        schema: right_schema.clone(),
    };

    let on = vec![(
        Expr::Column(Column::new("id".to_string())),
        Expr::Column(Column::new("user_id".to_string())),
    )];

    c.bench_function("logical_plan_join", |b| {
        b.iter(|| LogicalPlan::Join {
            left: Box::new(left.clone()),
            right: Box::new(right.clone()),
            join_type: JoinType::Inner,
            on: on.clone(),
            filter: None,
            schema: join_schema.clone(),
        });
    });
}

fn bench_logical_plan_aggregate(c: &mut Criterion) {
    use sqlrustgo::planner::{
        AggregateFunction, Column, DataType, Expr, Field, LogicalPlan, Schema,
    };

    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("category".to_string(), DataType::Text),
        Field::new("amount".to_string(), DataType::Integer),
    ]);

    let output_schema = Schema::new(vec![
        Field::new("category".to_string(), DataType::Text),
        Field::new("count".to_string(), DataType::Integer),
    ]);

    let input = LogicalPlan::TableScan {
        table_name: "orders".to_string(),
        projection: None,
        filters: vec![],
        limit: None,
        schema: schema.clone(),
    };

    let group_expr = vec![Expr::Column(Column::new("category".to_string()))];
    let aggr_expr = vec![Expr::AggregateFunction {
        func: AggregateFunction::Count,
        args: vec![Expr::Column(Column::new("amount".to_string()))],
        distinct: false,
    }];

    c.bench_function("logical_plan_aggregate", |b| {
        b.iter(|| LogicalPlan::Aggregate {
            input: Box::new(input.clone()),
            group_expr: group_expr.clone(),
            aggr_expr: aggr_expr.clone(),
            schema: output_schema.clone(),
        });
    });
}

fn bench_schema_creation(c: &mut Criterion) {
    c.bench_function("schema_creation", |b| {
        b.iter(|| {
            let fields: Vec<_> = (0..20)
                .map(|i| Field::new(format!("field_{}", i), DataType::Integer))
                .collect();
            Schema::new(fields)
        });
    });
}

fn bench_expr_parsing(c: &mut Criterion) {
    use sqlrustgo::planner::{Column, Expr, Operator};

    c.bench_function("expr_binary", |b| {
        b.iter(|| Expr::BinaryExpr {
            left: Box::new(Expr::Column(Column::new("a".to_string()))),
            op: Operator::Plus,
            right: Box::new(Expr::Column(Column::new("b".to_string()))),
        });
    });

    c.bench_function("expr_nested", |b| {
        b.iter(|| Expr::BinaryExpr {
            left: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("a".to_string()))),
                op: Operator::Plus,
                right: Box::new(Expr::Column(Column::new("b".to_string()))),
            }),
            op: Operator::And,
            right: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("c".to_string()))),
                op: Operator::Gt,
                right: Box::new(Expr::Literal(sqlrustgo::Value::Integer(100))),
            }),
        });
    });
}

criterion_group!(
    benches,
    bench_logical_plan_simple,
    bench_logical_plan_filter,
    bench_logical_plan_projection,
    bench_logical_plan_join,
    bench_logical_plan_aggregate,
    bench_schema_creation,
    bench_expr_parsing
);
criterion_main!(benches);
