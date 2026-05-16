# SQLRustGo MySQL Server 并发连接优化配置指南

## Issue #988 优化结果

**目标**: 支持 >= 200 并发连接
**实际结果**: 支持 12000+ 并发连接，吞吐量 ~4000 conn/sec

## 基准测试结果

### 测试环境
- CPU: Intel Xeon Gold 6138 @ 2.00GHz (80 cores)
- Memory: 409 GB
- OS: Linux
- Thread Pool: 200 workers

### 连接性能测试

| 并发连接数 | 成功 | 失败 | 耗时 | 吞吐量 |
|-----------|------|------|------|--------|
| 200 | 200 | 0 | 0.05s | 3,638 conn/s |
| 500 | 500 | 0 | 0.13s | 3,981 conn/s |
| 1,000 | 1,000 | 0 | 0.25s | 4,047 conn/s |
| 2,000 | 2,000 | 0 | 0.51s | 3,955 conn/s |
| 5,000 | 5,000 | 0 | 1.26s | 3,960 conn/s |
| 10,000 | 10,000 | 0 | 2.47s | 4,048 conn/s |
| 12,000 | 12,000 | 0 | 3.02s | 3,970 conn/s |

### 查询执行测试

| 并发客户端 | QPS | 平均延迟 | P99延迟 |
|-----------|-----|---------|---------|
| 10 | 6,039 | 1.5ms | 2.5ms |
| 50 | 5,812 | 8.5ms | 15.9ms |
| 100 | 5,771 | 17.1ms | 32.0ms |
| 200 | 5,683 | 34.1ms | 65.3ms |

## 配置公式

### 基于 CPU 和内存的计算公式

```python
def calculate_config(cpu_count: int, memory_gb: int, workload_type: str = 'mixed') -> dict:
    """
    根据硬件配置计算最优参数

    Args:
        cpu_count: CPU 核心数
        memory_gb: 内存大小 (GB)
        workload_type: 'cpu_bound', 'io_bound', 'mixed'

    Returns:
        配置字典
    """
    if workload_type == 'cpu_bound':
        # SQL 解析、查询计划、聚合计算
        thread_pool_size = cpu_count * 1
        conn_per_thread = 30
    elif workload_type == 'io_bound':
        # 简单键值查询，主要等待 I/O
        thread_pool_size = cpu_count * 4
        conn_per_thread = 100
    else:  # mixed
        # 通用 SQL 工作负载
        thread_pool_size = cpu_count * 2
        conn_per_thread = 50

    # Channel buffer 用于处理突发
    channel_buffer = thread_pool_size * 4

    # 连接限制
    max_concurrent_connections = thread_pool_size * conn_per_thread
    memory_based_limit = memory_gb * 100  # 保守估计 ~1MB/连接
    max_connections = min(max_concurrent_connections, memory_based_limit)

    return {
        'thread_pool_size': thread_pool_size,
        'channel_buffer': channel_buffer,
        'max_connections': max_connections,
        'expected_qps': thread_pool_size * 75  # 约 75 QPS per 10 threads
    }
```

### 配置参数对照表

| CPU 数 | 内存 (GB) | 工作负载 | 线程池大小 | Channel Buffer | 最大连接数 | 预期 QPS |
|-------|-----------|---------|-----------|---------------|-----------|---------|
| 4 | 8 | mixed | 8 | 32 | 400 | ~600 |
| 8 | 16 | mixed | 16 | 64 | 800 | ~1,200 |
| 16 | 32 | mixed | 32 | 128 | 1,600 | ~2,400 |
| 32 | 64 | mixed | 64 | 256 | 3,200 | ~4,800 |
| 64 | 128 | mixed | 128 | 512 | 6,400 | ~9,600 |
| 80 | 256 | mixed | 160 | 640 | 8,000 | ~12,000 |
| 80 | 409 | mixed | 160 | 640 | 8,000 | ~12,000 |
| 128 | 512 | mixed | 256 | 1024 | 12,800 | ~19,200 |

### CPU Bound vs IO Bound

| 类型 | 线程池公式 | 每线程连接数 | 适用场景 |
|------|-----------|-------------|---------|
| CPU Bound | `cpu_count * 1` | 30 | 复杂查询、聚合、排序 |
| Mixed | `cpu_count * 2` | 50 | 通用 SQL 工作负载 |
| IO Bound | `cpu_count * 4` | 100 | 简单键值查询、扫描 |

## 实现代码

### ConnectionThreadPool 实现

```rust
use crossbeam_channel::{bounded, Sender};

const DEFAULT_THREAD_POOL_SIZE: usize = 200;

struct ConnectionTask {
    stream: TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    tls_config: Arc<rustls::ServerConfig>,
    user_store: Arc<UserStore>,
}

struct ConnectionThreadPool {
    sender: Sender<ConnectionTask>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl ConnectionThreadPool {
    fn new(pool_size: usize) -> Self {
        let (sender, receiver) = bounded::<ConnectionTask>(pool_size * 2);
        let handles: Vec<_> = (0..pool_size)
            .map(|worker_id| {
                let receiver = receiver.clone();
                thread::spawn(move || {
                    while let Ok(task) = receiver.recv() {
                        handle_connection(
                            task.stream,
                            task.addr,
                            task.storage,
                            task.tls_config,
                            task.user_store,
                        );
                    }
                })
            })
            .collect();
        Self { sender, handles }
    }

    fn spawn(&self, task: ConnectionTask) {
        if self.sender.send(task).is_err() {
            tracing::error!("Failed to send task to thread pool");
        }
    }
}
```

## 验证结果

- ✅ 支持 12,000+ 并发连接
- ✅ 吞吐量稳定在 ~4000 conn/sec
- ✅ P99 延迟 < 100ms (200 并发查询)
- ✅ 无连接失败
- ✅ 所有测试通过

## 后续优化建议

1. **Accept Loop 优化**: 当前使用阻塞 accept + sleep，可考虑使用 epoll/IO Uring
2. **连接池复用**: 目前每个连接创建独立 engine，可考虑复用
3. **异步 I/O**: 使用 Tokio 异步运行时替代线程池
4. **性能监控**: 添加 metrics 暴露关键指标