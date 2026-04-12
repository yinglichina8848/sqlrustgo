# SQLRustGo Benchmark Runner 参考实现

> **说明**: 此代码为参考骨架，需根据实际项目结构调整

---

## src/main.rs

```rust
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tracing::{info, instrument};

mod analysis;

#[derive(Debug, Serialize, Deserialize)]
struct Metrics {
    tps: f64,
    latency_ms: HashMap<String, f64>,
    lock: LockMetrics,
    wal: WalMetrics,
    executor: ExecutorMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct LockMetrics {
    wait_ratio: f64,
    conflicts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalMetrics {
    flush_ratio: f64,
    bytes_written: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecutorMetrics {
    cpu_ratio: f64,
    time_ms: f64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Benchmark Runner...");

    let metrics_sqlrustgo_embedded = run_embedded().await;
    let metrics_sqlrustgo_tcp = run_tcp("127.0.0.1:4000").await;
    let metrics_postgres = run_postgres().await;

    // 输出 JSON
    serde_json::to_writer_pretty(
        std::fs::File::create("metrics_sqlrustgo_embedded.json").unwrap(),
        &metrics_sqlrustgo_embedded,
    ).unwrap();

    // 分析
    analysis::analyze(&metrics_sqlrustgo_embedded);
    analysis::analyze(&metrics_sqlrustgo_tcp);
    analysis::analyze(&metrics_postgres);
}

#[instrument]
async fn run_embedded() -> Metrics {
    // 内嵌执行器调用
    todo!()
}

#[instrument]
async fn run_tcp(addr: &str) -> Metrics {
    let mut stream = TcpStream::connect(addr).await?;
    // TCP 查询
    todo!()
}

#[instrument]
async fn run_postgres() -> Metrics {
    // PostgreSQL 调用
    todo!()
}
```

---

## src/analysis.rs

```rust
use crate::Metrics;

pub fn analyze(metrics: &Metrics) {
    println!("===== Benchmark Analysis =====");
    println!("TPS: {:.2}", metrics.tps);

    if metrics.lock.wait_ratio > 0.2 {
        println!("⚠️ Lock contention detected");
    }
    if metrics.wal.flush_ratio > 0.3 {
        println!("⚠️ WAL bottleneck detected");
    }
    if metrics.latency_ms.get("p99").unwrap_or(&0.0) > &5.0 {
        println!("⚠️ High P99 latency");
    }
}
```

---

## 使用说明

### Flamegraph

```bash
cargo flamegraph --bin benchmark
```

### Tracing

```bash
RUST_LOG=trace cargo run --bin benchmark
```

---

*参考代码，需集成到实际项目*
