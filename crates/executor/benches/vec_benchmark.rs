use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo_planner::{DataType, Expr, Field, Operator, Schema};
use sqlrustgo_types::Value;
use sqlrustgo_executor::vec_table_scan::VecTableScanExecutor;
use sqlrustgo_executor::vectorization::{DataChunk, ColumnArray};

fn create_test_chunk(num_rows: usize) -> DataChunk {
    let mut chunk = DataChunk::new(num_rows).with_schema(vec![
        "id".to_string(),
        "value".to_string(),
    ]);

    let ids: Vec<i64> = (0..num_rows as i64).collect();
    let values: Vec<i64> = (0..num_rows as i64).map(|i| i * 2).collect();

    chunk.add_column(ColumnArray::Int64(ids));
    chunk.add_column(ColumnArray::Int64(values));

    chunk
}

fn create_test_schema() -> Schema {
    Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("value".to_string(), DataType::Integer),
    ])
}

fn bench_vec_table_scan_row_iteration(c: &mut Criterion) {
    let chunk = create_test_chunk(100_000);
    let schema = create_test_schema();
    let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 1024);

    c.bench_function("vec_table_scan_row_iteration_100k", |b| {
        b.iter(|| {
            executor.open().unwrap();
            let mut count = 0;
            while let Ok(Some(_)) = executor.next() {
                black_box(count += 1);
            }
            executor.close().unwrap();
        });
    });
}

fn bench_vec_table_scan_batch_iteration(c: &mut Criterion) {
    let chunk = create_test_chunk(100_000);
    let schema = create_test_schema();
    let mut executor = VecTableScanExecutor::from_chunk(chunk, schema, 1024);

    c.bench_function("vec_table_scan_batch_iteration_100k", |b| {
        b.iter(|| {
            executor.open().unwrap();
            let mut count = 0;
            while let Ok(Some(_)) = executor.next_batch() {
                black_box(count += 1);
            }
            executor.close().unwrap();
        });
    });
}

fn bench_filter_volcano_model(c: &mut Criterion) {
    use sqlrustgo_executor::filter::FilterVolcanoExecutor;
    use sqlrustgo_executor::executor::VolcanoExecutor;

    let chunk = create_test_chunk(100_000);
    let schema = create_test_schema();
    let mut scan_executor = VecTableScanExecutor::from_chunk(chunk, schema, 1024);

    let predicate = Expr::BinaryExpr {
        left: Box::new(Expr::Column("value".to_string())),
        op: Operator::Gt,
        right: Box::new(Expr::Literal(Value::Integer(100000))),
    };

    let mut filter_executor = FilterVolcanoExecutor::new(
        Box::new(scan_executor),
        predicate,
        schema.clone(),
        schema,
    );

    c.bench_function("filter_volcano_100k", |b| {
        b.iter(|| {
            filter_executor.open().unwrap();
            let mut count = 0;
            while let Ok(Some(_)) = filter_executor.next() {
                black_box(count += 1);
            }
            filter_executor.close().unwrap();
        });
    });
}

fn bench_data_chunk_filter(c: &mut Criterion) {
    use sqlrustgo_executor::vectorization::vectorized_filter::apply_filter;

    let chunk = create_test_chunk(100_000);
    let predicate_array = ColumnArray::Int64(
        (0..100_000i64).map(|i| if i > 100000 { 1 } else { 0 }).collect()
    );

    c.bench_function("data_chunk_filter_100k", |b| {
        b.iter(|| {
            let filtered = apply_filter(black_box(&chunk), black_box(&predicate_array));
            black_box(filtered.num_rows());
        });
    });
}

fn bench_simd_like_aggregation(c: &mut Criterion) {
    use sqlrustgo_executor::vectorization::simd_agg::sum_i64;

    let values: Vec<i64> = (0..1_000_000i64).collect();

    c.bench_function("simd_like_sum_1m", |b| {
        b.iter(|| {
            let result = sum_i64(black_box(&values));
            black_box(result);
        });
    });
}

fn bench_scalar_aggregation(c: &mut Criterion) {
    let values: Vec<i64> = (0..1_000_000i64).collect();

    c.bench_function("scalar_sum_1m", |b| {
        b.iter(|| {
            let result: i64 = values.iter().sum();
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_vec_table_scan_row_iteration,
    bench_vec_table_scan_batch_iteration,
    bench_filter_volcano_model,
    bench_data_chunk_filter,
    bench_simd_like_aggregation,
    bench_scalar_aggregation
);
criterion_main!(benches);
