use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlrustgo_rag::{Document, InvertedIndex, RAGPipeline};

fn bench_inverted_index_search(c: &mut Criterion) {
    let mut index = InvertedIndex::new();

    for i in 0..10000 {
        index.add_document(
            i,
            &format!(
                "Document {} with content about testing and search functionality",
                i
            ),
        );
    }

    c.benchmark_group("inverted_index_search")
        .significance_level(0.01)
        .sample_size(100)
        .bench_function("search_10k_docs", |b| {
            b.iter(|| {
                let _ = index.search(black_box("testing search"));
            });
        });
}

fn bench_rag_pipeline(c: &mut Criterion) {
    let mut pipeline = RAGPipeline::new();

    for i in 0..1000 {
        pipeline.add_document(Document::new(
            i,
            format!("Document {} about machine learning and AI", i),
        ));
    }

    c.benchmark_group("rag_pipeline")
        .significance_level(0.01)
        .sample_size(50)
        .bench_function("retrieve_top_10", |b| {
            b.iter(|| {
                let _ = pipeline.retrieve(black_box("machine learning AI"), 10);
            });
        });
}

criterion_group!(benches, bench_inverted_index_search, bench_rag_pipeline);
criterion_main!(benches);
