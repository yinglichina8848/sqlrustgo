# SQLRustGo MySQL Wire Protocol 修复指南（ChatGPT 建议）

> **目标**：修复 SQLRustGo 的 MySQL wire protocol 兼容性，使 sysbench 可连接

---

## 一、当前状态

系统已实现：
- TLS ClientHello detection
- rustls TLS handshake
- TlsStream wrapper
- handshake packet 修复
- 非 TLS packet 修复
- capability flags 部分支持

但仍然存在：
```
Packet sequence number wrong - got 1 expected 2
```

说明 MySQL connection phase state machine 不完整。

---

## 二、PHASE 1 — 打印完整 packet header trace

添加 debug instrumentation，记录 server→client / client→server 方向的完整 handshake 生命周期。

期望 trace：
```
S -> C handshake seq=0
C -> S auth seq=1
S -> C ok seq=2
```

如果不是，说明 sequence generator 错误。

---

## 三、PHASE 2 — 修复 sequence number 生命周期

MySQL protocol 要求每个 request-response exchange 的 sequence 从 0 开始递增。

connection phase 流程必须是：
```
server handshake packet seq=0
client auth packet seq=1
server OK packet seq=2
```

当前错误：服务器发送 OK packet seq=1，正确应该是 seq=2。

修复：`seq = incoming_seq + 1`，禁止 hardcode seq。

---

## 四、PHASE 3 — 修复 handshake capability flags

sysbench 使用 `mysql_native_password`，必须支持：

```
CLIENT_PROTOCOL_41
CLIENT_PLUGIN_AUTH
CLIENT_SECURE_CONNECTION
CLIENT_LONG_PASSWORD
CLIENT_LONG_FLAG
CLIENT_TRANSACTIONS
CLIENT_MULTI_RESULTS
```

---

## 五、PHASE 4 — 修复 auth plugin name

sysbench 默认 `mysql_native_password`，必须 handshake packet 中发送：
```
mysql_native_password\0
```

不是 `caching_sha2_password`。

---

## 六、PHASE 5 — 修复 auth response parsing

client auth packet layout：
```
4 capability flags
4 max packet size
1 charset
23 reserved
username
auth response
plugin name
```

当前实现疑似 incorrect length parsing（`length=7303026 seq=116` 明显 header decode 错误）。

---

## 七、PHASE 6 — 实现 minimal auth acceptor

当前阶段允许 root without password，实现 `fn accept_auth() -> bool { true }`，返回 OK packet。

---

## 八、PHASE 7 — 实现 connection state machine

```rust
enum ConnState {
    HandshakeSent,
    AuthReceived,
    Authenticated,
    Ready,
}
```

生命周期：start → send handshake → receive auth → send OK → READY

禁止跨状态 packet write。

---

## 九、PHASE 8 — 禁用 TLS 作为 first milestone

sysbench 默认 plaintext MySQL。临时关闭 TLS detection branch，只保留 plain MySQL protocol。

验证 mysql client / PyMySQL / sysbench 全部可连接后，再恢复 TLS。

---

## 十、PHASE 9 — 使用 mysql client 验证

测试顺序：
1. `mysql -h127.0.0.1 -P3306 -uroot`
2. `python pymysql_test.py`
3. `sysbench oltp_common.lua prepare`

顺序不能反。

---

## 十一、PHASE 10 — 添加 integration test

新增 `tests/mysql_handshake.rs`，测试 mock client handshake，验证 seq correctness / packet length / plugin correctness。

---

## 十二、SUCCESS CRITERIA

以下命令全部通过：
```
mysql connect
pymysql connect
sysbench prepare
sysbench run
```

---

## 十三、建议 roadmap

```
phase 1: mysql CLI connect
phase 2: pymysql connect
phase 3: sysbench prepare
phase 4: sysbench run
phase 5: OLTP transaction support
```

不要直接跳 phase 3。

---

**创建日期**: 2026-04-19
**来源**: ChatGPT MySQL Wire Protocol 修复建议
