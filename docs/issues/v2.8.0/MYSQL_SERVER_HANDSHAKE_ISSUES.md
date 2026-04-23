# MySQL Wire Protocol 握手问题诊断报告

> **创建日期**: 2026-04-23
> **问题**: SQLRustGo MySQL Server 无法与标准 MySQL 客户端完成握手
> **目标**: 请求 DeepSeek 协助诊断根本原因

---

## 1. 问题概述

SQLRustGo MySQL Server (`crates/mysql-server`) 在与标准 MySQL 客户端（mysql CLI、sysbench）握手时失败。

**错误信息**:
```
ERROR 2012 (HY000): Error in server handshake
```

**服务器日志**:
```
Auth packet read error (continuing anyway): IO error: failed to fill whole buffer
Sending OK packet with seq=1
Connection error: IO error: Broken pipe (os error 32)
```

---

## 2. 已尝试的修复

### 2.1 握手包格式修复

对比 MySQL 8.0 真实握手包，修复了以下问题：

| 字段 | 修复前 | 修复后 |
|------|--------|--------|
| capability_lower | 0x0000 (错误) | 0x123b (正确) |
| auth_plugin_len | 21 | 8 |
| auth_plugin_name | mysql_native_password | mysql_native_password |

**当前握手包十六进制**:
```
0a 53 51 4c 52 75 73 74 47 6f 2d 32 2e 38 2e 30 00 01 00 00 00 
bf a9 cd b5 21 2d ce c4 00 00 12 2c 02 00 3b 00 08 00 00 00 00 
00 00 00 00 00 00 6d 79 73 71 6c 5f 6e 61 74 69 76 65
```

### 2.2 认证响应读取修复

- 使用更短的超时时间 (100ms) 读取握手响应
- 握手响应读取失败时继续处理（容错）

### 2.3 OK 响应包

当前 OK 包格式：
- `0x00` (OK packet marker)
- `affected_rows` (length-encoded integer)
- `last_insert_id` (length-encoded integer)
- `server_status` (2 bytes)
- `warning_count` (2 bytes)

---

## 3. 握手流程分析

### 3.1 成功捕获的握手响应

通过 Python 脚本手动测试，成功捕获到握手响应：

```
Auth packet: len=46, seq=1, payload: 
[f7, a0, 00, 00, 00, 00, 00, 01, 2c, 00, 00, 00, 00, 00, 00, 00, 
00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 
73, 62, 75, 73, 65, 72, 00, 06, 73, 62, 70, 61, 73, 73]
```

解析：
- capability: 0xa0f7
- max_packet: 0x01000000
- charset: 0x2c
- username: "sbuser"
- password: "sbpass" (length=6)

### 3.2 问题点

握手响应被成功读取后，服务器发送 OK 响应时出现 `Broken pipe` 错误，表明：
1. 客户端在收到 OK 响应前断开了连接
2. 或者 OK 响应格式不正确导致客户端断开

---

## 4. MySQL 8.0 真实握手包参考

```
Length: 91, Seq: 0

1. protocol_version: 10 (0x0a)
2. server_version: 8.0.45-0ubuntu0.24.04.1
3. connection_id: 494
4. auth_part1: 2e3f57406f2b5245
5. capability_lower: ffff (65535)
   - PROTOCOL_41: True
   - SECURE_CONNECTION: True (bit 12)
6. charset: 255 (0xff)
7. server_status: 0002
8. capability_upper: dfff (57343)
9. auth_plugin_data_length: 21
10. reserved: 00000000000000000000
11. auth_plugin_name: caching_sha2_password
```

---

## 5. 关键代码位置

- **握手包生成**: `crates/mysql-server/src/lib.rs:make_handshake_packet()`
- **握手响应读取**: `crates/mysql-server/src/lib.rs:handle_connection()`
- **OK 包生成**: `crates/mysql-server/src/lib.rs:make_ok_packet()`

---

## 6. 需要 DeepSeek 协助的问题

1. **握手响应读取失败的根本原因**
   - 为什么 `failed to fill whole buffer` 错误持续出现？
   - MySQL 客户端的握手响应格式是什么？

2. **OK 响应包格式**
   - 当前 OK 包格式是否正确？
   - 是否需要添加额外字段（如 auth_plugin）？

3. **认证流程**
   - 是否需要实现完整的 `caching_sha2_password` 认证？
   - 或者可以简化为什么都不验证的模式？

---

## 7. 测试命令

```bash
# 启动服务器
cargo build --release -p sqlrustgo-mysql-server
./target/release/sqlrustgo-mysql-server --port 3307

# 测试连接
mysql -u sbuser -psbpass -h 127.0.0.1 -P 3307 -e "SELECT 1"
```

---

## 8. 相关文件

- `crates/mysql-server/src/lib.rs` - 主要实现
- `crates/mysql-server/src/main.rs` - 入口点
- `sysbench_results/SYSBENCH_COMPARISON_REPORT.md` - 性能测试报告

---

*本报告由 SQLRustGo 团队生成*
