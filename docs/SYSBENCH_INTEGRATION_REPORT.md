# SQLRustGo MySQL Server TLS/兼容性支持工作报告

## 1. 概述

本工作报告记录了 SQLRustGo MySQL Server 的 TLS 支持和 sysbench 兼容性改进工作。

**目标**：让 sysbench 能够连接到 SQLRustGo MySQL Server 并运行 OLTP 兼容性测试。

## 2. 已完成的工作

### 2.1 TLS 支持实现

#### 2.1.1 TLS ClientHello 检测
- 实现了正确检测 TLS ClientHello 的逻辑（检查前2字节是否为 `0x16 0x03`）
- 位置：`crates/mysql-server/src/lib.rs` 第 710-711 行

```rust
let is_tls = n >= 2 && initial_header[0] == 0x16 && initial_header[1] == 0x03;
```

#### 2.1.2 TlsStream 类型实现
- 创建了 `TlsStream` 结构体来包装 rustls `ServerConnection`
- 实现了 `Read` 和 `Write` trait
- 位置：`crates/mysql-server/src/lib.rs` 第 80-128 行

```rust
struct TlsStream<'a> {
    conn: &'a mut rustls::ServerConnection,
    stream: &'a mut TcpStream,
}

impl<'a> Read for TlsStream<'a> { ... }
impl<'a> Write for TlsStream<'a> { ... }
```

#### 2.1.3 TLS 握手处理
- 使用 `rustls::ServerConnection::complete_io()` 完成 TLS 握手
- 握手完成后通过 `TlsStream` 处理 MySQL 协议数据

### 2.2 MySQL 协议修复

#### 2.2.1 Handshake Packet 格式修正
- 修复了 auth-plugin-data-length 字段（第369行）
- 原代码错误地将 `seed.len()` 作为 auth-plugin-data-length
- 修正后使用 `0x00`（表示 20 字节的 nonce）
- 位置：`crates/mysql-server/src/lib.rs` 第 369 行

#### 2.2.2 Non-TLS 路径 Packet 读取修复
- 修复了 `initial_header` 读取后的数据重复读取问题
- 直接使用已读取的 5 字节构造 auth_packet
- 位置：`crates/mysql-server/src/lib.rs` 第 853-884 行

### 2.3 依赖项更新

在 `crates/mysql-server/Cargo.toml` 中添加了：
```toml
rustls = "0.23"
rcgen = "0.13"
```

## 3. 当前卡住的问题

### 3.1 PyMySQL 连接 Sequence 错误

**问题描述**：
```
Error: Packet sequence number wrong - got 1 expected 2
```

**分析**：
- 服务器发送 OK packet 时使用 seq=1
- PyMySQL 客户端期望 seq=2
- 这表明客户端认为这是第二个数据包（seq=1），但实际应该是第三个（seq=2）

**可能原因**：
1. Handshake packet 的 sequence number 处理可能有问题
2. 客户端可能在读取 handshake 后期望特定的响应 sequence

**服务器日志**：
```
WRITE handshake packet: len=73, seq=0
WRITE complete
Received initial packet: length=99, seq=1
Sending OK packet with seq=1
```

### 3.2 Raw Socket 测试显示数据问题

**测试发现**：
- 当 Python socket 直接发送时，服务器报告 `Received initial packet: length=7303026, seq=116`
- 这表明字节序解析或数据长度解析存在问题

## 4. 建议的开发改进

### 4.1 协议解析改进

1. **严格验证 packet header**
   - packet length 应该是合理值（0-16MB）
   - sequence number 应该是连续的

2. **添加更多调试日志**
   - 记录完整的 packet header
   - 记录 payload 的前20字节

3. **实现完整的 MySQL 协议状态机**
   - HANDshake → WAIT_AUTH → AUTHENTICATED → READY

### 4.2 Handshake 改进

1. **正确设置 capability flags**
   - 当前可能缺少 SSL capability 或其他必要标志

2. **实现完整的 auth 流程**
   - 当前简化了 auth 验证，直接接受所有连接

3. **处理更多的 auth 方法**
   - `mysql_native_password`
   - `caching_sha2_password`
   - `sha256_password`

### 4.3 TLS 改进

1. **改进 TlsStream 的错误处理**
2. **支持 TLS 客户端证书认证**
3. **添加 TLS 配置的可配置性**

## 5. 测试计划

### 5.1 单元测试

1. **Packet 序列化/反序列化测试**
2. **Handshake packet 生成测试**
3. **TLS ClientHello 检测测试**

### 5.2 集成测试

1. **PyMySQL 连接测试**
   ```python
   conn = pymysql.connect(host='127.0.0.1', port=3306, user='root')
   cursor.execute("SELECT 1")
   ```

2. **MariaDB Connector/C 测试**
   ```bash
   mariadb-connect --host=127.0.0.1 --port=3306 --user=root
   ```

3. **sysbench 连接测试**
   ```bash
   sysbench --db-driver=mysql --mysql-host=127.0.0.1 --mysql-port=3306 \
            /usr/local/share/sysbench/oltp_common.lua prepare
   ```

### 5.3 协议兼容性测试

1. **MySQL 5.7 兼容性测试**
2. **MySQL 8.0 兼容性测试**
3. **MariaDB 10.x 兼容性测试**

## 6. 相关文件

- `crates/mysql-server/src/lib.rs` - 主要实现
- `crates/mysql-server/Cargo.toml` - 依赖配置
- `crates/mysql-server/src/main.rs` - 服务器入口

## 7. 参考资料

- [MySQL Wire Protocol](https://dev.mysql.com/doc/internals/en/connection-phase.html)
- [rustls Documentation](https://docs.rs/rustls/latest/rustls/)
- [PyMySQL Source](https://github.com/PyMySQL/PyMySQL)
- [MariaDB Connector/C](https://mariadb.com/kb/en/mariadb-connector-c/)

## 8. 下一步行动

1. 调试 PyMySQL sequence 错误问题
2. 实现正确的 handshake 响应处理
3. 测试 sysbench 连接
4. 提交完整的 PR