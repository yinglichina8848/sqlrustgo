# MySQL Wire Protocol (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> MySQL 兼容协议: 握手、认证、命令处理、结果发送

## 1. MySQL 协议架构

### 1.1 核心常量

```rust
const SERVER_VERSION: &str = "8.0.33-SQLRustGo";
const AUTH_PLUGIN: &str = "mysql_native_password";
const SCRAMBLE_LENGTH: usize = 20;
```

### 1.2 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [lib.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/mysql-server/src/lib.rs) | 2591 | MySQL 协议完整实现 |
| [main.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/mysql-server/src/main.rs) | 58 | 可执行入口 |
| [http_server.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/mysql-server/src/http_server.rs) | - | HTTP 监控端点 |
| [monitoring.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/mysql-server/src/monitoring.rs) | - | 监控指标 |

## 2. 连接建立链路

### 2.1 握手时序图

```
MySQL Client (mysql-cli / sysbench / mysql crate)
    │
    ▼ TCP connect (port 3306)
┌──────────────────────────────────────────────────────────┐
│ Phase 1: TLS 握手 (可选)                                 │
│  ├── Server: 发送 TLS 支持                               │
│  └── Client: 选择是否启用 TLS (rustls 自签名证书)        │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────┐
│ Phase 2: MySQL 握手                                      │
│  ├── Server → Client: HandshakePacket                    │
│  │   ├── Protocol Version: 10                            │
│  │   ├── Server Version: "8.0.33-SQLRustGo"             │
│  │   ├── Connection ID: 1                                │
│  │   ├── Auth Plugin: "mysql_native_password"            │
│  │   └── Scramble: 20 bytes random                       │
│  │                                                        │
│  ├── Client → Server: HandshakeResponse                  │
│  │   ├── Username                                         │
│  │   ├── Auth Response: SHA1(password) XOR SHA1(SHA1(SHA1(password)) + scramble) │
│  │   ├── Database (初始选择)                              │
│  │   └── Client Capabilities                             │
│  │                                                        │
│  └── Server: verify_mysql_native_password()               │
│      ├── 计算期望的 auth_response                         │
│      └── 比较客户端发送的 auth_response                   │
└──────────────────────┬───────────────────────────────────┘
                       │
              ┌────────┴────────┐
              │  认证结果        │
              └──┬──────────┬───┘
              OK │          │ FAIL
                 ▼          ▼
          ┌──────────┐ ┌──────────┐
          │OK Packet │ │ERR Packet│
          │继续命令  │ │关闭连接  │
          └──────────┘ └──────────┘
```

### 2.2 认证算法

```
mysql_native_password 认证:

  1. Server 生成 scramble (20 bytes random)
  2. Client 计算:
     SHA1(password) = hash1
     SHA1(SHA1(password)) = hash2
     SHA1(hash2 + scramble) = hash3
     auth_response = hash1 XOR hash3
  3. Server 验证:
     stored_hash2 = 存储的 SHA1(SHA1(password))
     SHA1(stored_hash2 + scramble) = expected_hash3
     received_hash1 = auth_response XOR expected_hash3
     SHA1(received_hash1) == stored_hash2? → 认证成功
```

## 3. 命令处理链路

### 3.1 命令循环时序图

```
┌──────────────────────────────────────────────────────────┐
│ Command Loop                                              │
│                                                           │
│  loop {                                                   │
│    Client → Server: Packet { sequence_id, payload }       │
│    ├── COM_QUERY (0x03):                                  │
│    │   ├── split_sql_statements(sql)                      │
│    │   ├── replace_placeholders(sql, params)              │
│    │   ├── engine.execute(sql)                            │
│    │   └── send_result_set(rows) ⚠️ 逐行 flush          │
│    │                                                      │
│    ├── COM_STMT_PREPARE (0x16):                           │
│    │   ├── parse(sql) → Statement                         │
│    │   ├── 计算参数数和列数                                │
│    │   └── 返回 statement_id                              │
│    │                                                      │
│    ├── COM_STMT_EXECUTE (0x17):                           │
│    │   ├── parse_stmt_execute_params()                    │
│    │   ├── replace_placeholders()                         │
│    │   └── execute + send_result                          │
│    │                                                      │
│    ├── COM_PING (0x0E) → OK                               │
│    ├── COM_INIT_DB (0x02) → switch database               │
│    └── COM_QUIT (0x01) → close connection                │
│  }                                                        │
└──────────────────────────────────────────────────────────┘
```

### 3.2 结果发送活动图

```
    ┌──────────────────────────────┐
    │    SQL 执行完成               │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │  判断结果类型                 │
    └──┬──────────────────────┬───┘
  SELECT                    DML/DDL
       │                        │
       ▼                        ▼
┌──────────────────┐    ┌──────────────┐
│ Column Definition │    │ OK Packet    │
│ Packet (每列一个) │    │ affected_rows│
└────────┬─────────┘    │ last_insert_id│
         │              └──────────────┘
         ▼
┌──────────────────┐
│ Row Data Packet  │
│ (每行一个)       │
│ ⚠️ 每行 flush!  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ EOF Packet       │
└──────────────────┘
```

## 4. 性能瓶颈分析

### 4.1 当前性能

```
Sysbench point_select: 1,688 QPS
直接执行 (无协议): 183,918 QPS
性能差距: 109x
```

### 4.2 瓶颈链路

```
┌────────────┐    ┌──────────────┐    ┌───────────────┐    ┌────────────┐
│ TCP recv   │───▶│ is_select()  │───▶│ extract_table │───▶│ execute()  │
│            │    │ uppercase()  │    │ _name() 2x    │    │            │
└────────────┘    └──────────────┘    └───────────────┘    └─────┬──────┘
                                                                  │
                    ┌──────────────┐    ┌───────────────┐         │
                    │ replace_     │◀───│ send_result_  │◀────────┘
                    │ placeholders │    │ set() N flush │
                    └──────────────┘    └───────────────┘
```

### 4.3 5 个瓶颈点详解

| # | 瓶颈 | 影响 | 修复 |
|---|------|------|------|
| 1 | **Packet::write_to() 强制 flush** | 100行 = 100+次网络往返 | 批量发送，最后 flush |
| 2 | **is_select() 每次创建新 String** | 堆分配开销 | 栈分配或缓存 |
| 3 | **extract_table_name() 两次 uppercase** | 双倍分配 | 缓存 uppercase 结果 |
| 4 | **send_result_set() 逐行 flush** | 极低效 | 批量写入 BufWriter |
| 5 | **replace_placeholders() 字符串分配** | 每次执行分配 | 预编译参数化查询 |

### 4.4 优化路线图

```
Phase 1 (v3.1.1, 1周):
  ├── 移除 Packet::write_to() 中的强制 flush
  ├── send_result_set() 批量写入
  └── 预期: 1,688 → 5,000+ QPS

Phase 2 (v3.2.0, 2周):
  ├── 缓存 uppercase 表名
  ├── 栈分配 is_select()
  └── 预期: 5,000 → 15,000+ QPS

Phase 3 (v3.2.0, 2周):
  ├── 预编译 Prepared Statement
  ├── 零拷贝结果发送
  └── 预期: 15,000 → 30,000+ QPS
```

## 5. 与其他模块的依赖

```
mysql-server
  ├── 依赖: MemoryExecutionEngine (⚠️ 紧耦合)
  ├── 依赖: rustls (TLS)
  ├── 被依赖: Sysbench 基准测试
  └── 被依赖: mysql 客户端连接
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充握手时序图、5个瓶颈点详解 |
