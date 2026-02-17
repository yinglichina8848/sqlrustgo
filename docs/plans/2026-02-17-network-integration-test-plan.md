# Network 模块集成测试计划

## 目标

提升 network 模块覆盖率（48.3% → 目标80%）

## 问题分析

当前 network 模块中以下函数无法通过单元测试覆盖，需要真实 TCP 连接：

| 函数 | 行号 | 问题 |
|------|------|------|
| `NetworkHandler::handle()` | 411 | 需要 TcpStream |
| `NetworkHandler::send_greeting()` | 450 | 需要 TcpStream.write_all |
| `NetworkHandler::read_packet()` | 465 | 需要 TcpStream.read |
| `NetworkHandler::execute_query()` | 502 | 需要 TcpStream |
| `NetworkHandler::send_ok()` | 523 | 需要 TcpStream |
| `NetworkHandler::send_error()` | 529 | 需要 TcpStream |
| `NetworkHandler::send_packet()` | 535 | 需要 TcpStream |
| `NetworkHandler::send_select_response()` | 550 | 需要 TcpStream |
| `start_server_sync()` | 594 | 需要 TcpListener.bind |
| `connect()` | 628 | 需要 TcpStream::connect |

## 解决方案

### 方案 1: 使用 Mock TcpStream（推荐）

创建一个 Mock TcpStream 来模拟网络行为：

```rust
use std::io::{Read, Write};
use std::net::TcpStream;

struct MockTcpStream {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // 从 read_buffer 读取
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 写入 write_buffer
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
```

### 方案 2: 集成测试

在 `tests/` 目录下创建集成测试，实际启动服务器和客户端。

## 实现计划

### Phase 1: Mock TcpStream 实现

1. 创建 `tests/network/mocks.rs`
   - 实现 `MockTcpStream`
   - 实现 `MockTcpListener`
   - 支持预设响应

2. 创建 `tests/network/handler_tests.rs`
   - 测试 `send_greeting()`
   - 测试 `read_packet()` 
   - 测试 `execute_query()` 各种 SQL 命令

3. 创建 `tests/network/integration_tests.rs`
   - 测试完整连接流程
   - 测试 SELECT/INSERT/UPDATE/DELETE

### Phase 2: 服务器集成测试

1. 创建 `tests/network/server_tests.rs`
   - 测试 `start_server_sync()`
   - 测试多客户端连接
   - 测试并发查询

### Phase 3: 客户端集成测试

1. 创建 `tests/network/client_tests.rs`
   - 测试 `connect()`
   - 测试 `execute_query_on_server()`

## 预期覆盖率提升

| 测试类型 | 目标覆盖率 |
|----------|-----------|
| Mock 单元测试 | +15% |
| 集成测试 | +10% |
| 边界测试 | +5% |
| **总计** | **78%** |

## 任务分配

| 任务 | 开发者 | 预计时间 |
|------|--------|----------|
| Mock TcpStream 实现 | OpenClaw | 2h |
| Handler 单元测试 | OpenClaw | 2h |
| 集成测试 | 待分配 | 3h |

## 验收标准

- [ ] network 模块覆盖率 ≥ 78%
- [ ] cargo test --all-features 通过
- [ ] 文档更新

---

*创建时间: 2026-02-17*
*状态: 待批准*
