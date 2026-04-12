use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::MemoryStorage;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn bench_tcp_connect(c: &mut Criterion) {
    let addr = "127.0.0.1:19999";

    let _listener = TcpListener::bind(addr).unwrap();

    thread::spawn(move || {
        for stream in TcpListener::bind(addr).unwrap().incoming() {
            let _ = stream;
        }
    });

    std::thread::sleep(Duration::from_millis(100));

    c.bench_function("tcp_connect", |b| {
        b.iter(|| {
            std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(1))
        });
    });
}

fn bench_tcp_send_receive(c: &mut Criterion) {
    let addr = "127.0.0.1:19998";

    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();

    let _ = listener.accept();

    let mut stream = std::net::TcpStream::connect(addr).unwrap();
    stream.set_nonblocking(false).unwrap();

    let data = b"SELECT * FROM users WHERE id = 1";

    c.bench_function("tcp_send_receive", |b| {
        b.iter(|| {
            stream.write_all(data).unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf).unwrap();
        });
    });
}

fn bench_query_latency(c: &mut Criterion) {
    let mut engine = sqlrustgo::ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(sqlrustgo::parse("CREATE TABLE latency_test (id INTEGER, value TEXT)").unwrap())
        .unwrap();

    for i in 0..100 {
        engine
            .execute(
                sqlrustgo::parse(&format!(
                    "INSERT INTO latency_test VALUES ({}, 'value{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    c.bench_function("query_latency_single", |b| {
        b.iter(|| {
            engine
                .execute(sqlrustgo::parse("SELECT * FROM latency_test WHERE id = 50").unwrap())
                .unwrap()
        });
    });
}

fn bench_query_throughput(c: &mut Criterion) {
    let mut engine = sqlrustgo::ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(sqlrustgo::parse("CREATE TABLE throughput_test (id INTEGER)").unwrap())
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(
                sqlrustgo::parse(&format!("INSERT INTO throughput_test VALUES ({})", i)).unwrap(),
            )
            .unwrap();
    }

    c.bench_function("query_throughput", |b| {
        b.iter(|| {
            for _ in 0..100 {
                engine
                    .execute(sqlrustgo::parse("SELECT * FROM throughput_test").unwrap())
                    .unwrap();
            }
        });
    });
}

fn bench_concurrent_queries(c: &mut Criterion) {
    let mut engine = sqlrustgo::ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(sqlrustgo::parse("CREATE TABLE concurrent_test (id INTEGER)").unwrap())
        .unwrap();

    for i in 0..100 {
        engine
            .execute(
                sqlrustgo::parse(&format!("INSERT INTO concurrent_test VALUES ({})", i)).unwrap(),
            )
            .unwrap();
    }

    c.bench_function("concurrent_queries", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    thread::spawn(|| {
                        let mut eng =
                            sqlrustgo::ExecutionEngine::new(Arc::new(MemoryStorage::new()));
                        for _ in 0..10 {
                            let _ = eng.execute(
                                sqlrustgo::parse("SELECT * FROM concurrent_test").unwrap(),
                            );
                        }
                    })
                })
                .collect();
            for h in handles {
                let _ = h.join();
            }
        });
    });
}

criterion_group!(
    benches,
    bench_tcp_connect,
    bench_tcp_send_receive,
    bench_query_latency,
    bench_query_throughput,
    bench_concurrent_queries
);
criterion_main!(benches);
