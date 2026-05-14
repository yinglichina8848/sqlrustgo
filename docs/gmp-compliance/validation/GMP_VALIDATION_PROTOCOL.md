# GMP 计算机化系统验证协议

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 1. 验证范围与目标

### 1.1 适用范围

本文档适用于 SQLRustGo 作为 GMP 计算机化系统的验证。GMP 计算机化系统包括：
- 数据库核心引擎 (SQL 解析、执行、存储)
- 事务管理系统 (MVCC、WAL、恢复)
- 审计系统 (Hash Chain、数字签名)
- 观测系统 (指标、日志、追踪)

### 1.2 验证目标

| 目标 | 说明 |
|------|------|
| 安全性 | 系统不会被未授权访问或篡改 |
| 可靠性 | 系统在规定条件下正常运行 |
| 可审计性 | 所有操作可追溯、可验证 |
| 可恢复性 | 崩溃后可恢复到一致状态 |

### 1.3 验证阶段

| 阶段 | 名称 | 目的 |
|------|------|------|
| IQ | 安装验证 | 确认系统按规范安装 |
| OQ | 操作验证 | 确认功能按规范运行 |
| PQ | 性能验证 | 确认系统在生产负载下达标 |

---

## 2. IQ - 安装验证

### 2.1 IQ 测试矩阵

| 测试 ID | 测试项 | 验收标准 | 方法 | 结果 |
|---------|--------|----------|------|------|
| IQ-001 | 二进制完整性 | SHA-256 与发布签名一致 | 验证签名 | 待测 |
| IQ-002 | 依赖完整性 | 所有 .so 文件存在 | ldd 检查 | 待测 |
| IQ-003 | 配置验证 | config.toml 语法正确 | 加载测试 | 待测 |
| IQ-004 | 目录权限 | 数据目录 700, 审计目录 700 | ls -la | 待测 |
| IQ-005 | TLS 证书 | 证书未过期, 格式正确 | openssl verify | 待测 |

### 2.2 IQ-001: 二进制完整性验证

```bash
#!/bin/bash
# IQ-001: 验证二进制完整性

BINARY="/usr/local/bin/sqlrustgo"
SIGNATURE="/var/lib/sqlrustgo/release.sig"
PUBLIC_KEY="/etc/sqlrustgo/keys/release-public.pem"

# 1. 计算 SHA-256
ACTUAL_HASH=$(sha256sum $BINARY | cut -d' ' -f1)

# 2. 读取预期哈希 (从签名文件)
EXPECTED_HASH=$(openssl rsautl -verify \
    -pubin -inkey $PUBLIC_KEY \
    -in $SIGNATURE 2>/dev/null)

# 3. 比较
if [ "$ACTUAL_HASH" = "$EXPECTED_HASH" ]; then
    echo "IQ-001: PASS"
    exit 0
else
    echo "IQ-001: FAIL - Binary tampered"
    exit 1
fi
```

**验收标准**: 脚本返回 "PASS"

### 2.3 IQ-002: 依赖完整性验证

```bash
#!/bin/bash
# IQ-002: 验证动态链接库

BINARY="/usr/local/bin/sqlrustgo"
MISSING=$(ldd $BINARY 2>&1 | grep "not found")

if [ -z "$MISSING" ]; then
    echo "IQ-002: PASS - All dependencies satisfied"
    exit 0
else
    echo "IQ-002: FAIL - Missing dependencies:"
    echo "$MISSING"
    exit 1
fi
```

**验收标准**: 无 "not found" 输出

### 2.4 IQ-003: 配置验证

```bash
#!/bin/bash
# IQ-003: 配置语法验证

CONFIG="/etc/sqlrustgo/sqlrustgo.toml"

# 使用 --dry-run 验证配置
sqlrustgo --dry-run --config $CONFIG 2>&1
RESULT=$?

if [ $RESULT -eq 0 ]; then
    echo "IQ-003: PASS - Configuration valid"
    exit 0
else
    echo "IQ-003: FAIL - Configuration error"
    exit 1
fi
```

**验收标准**: 命令返回 0

### 2.5 IQ-004: 目录权限验证

```bash
#!/bin/bash
# IQ-004: 验证目录权限

DATA_DIR="/var/lib/sqlrustgo/data"
AUDIT_DIR="/var/lib/sqlrustgo/audit"
KEY_DIR="/etc/sqlrustgo/keys"

# 验证数据目录
DATA_PERMS=$(stat -c "%a" $DATA_DIR)
if [ "$DATA_PERMS" = "700" ]; then
    echo "IQ-004a: PASS - Data dir permissions 700"
else
    echo "IQ-004a: FAIL - Data dir permissions $DATA_PERMS"
    exit 1
fi

# 验证审计目录
AUDIT_PERMS=$(stat -c "%a" $AUDIT_DIR)
if [ "$AUDIT_PERMS" = "700" ]; then
    echo "IQ-004b: PASS - Audit dir permissions 700"
else
    echo "IQ-004b: FAIL - Audit dir permissions $AUDIT_PERMS"
    exit 1
fi
```

**验收标准**: 所有关键目录 700

### 2.6 IQ-005: TLS 证书验证

```bash
#!/bin/bash
# IQ-005: TLS 证书验证

CERT="/etc/sqlrustgo/tls/server.crt"

# 1. 验证证书格式
openssl x509 -in $CERT -text -noout > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "IQ-005: FAIL - Invalid certificate format"
    exit 1
fi

# 2. 验证未过期
EXPIRY=$(openssl x509 -in $CERT -noout -enddate | cut -d= -f2)
EXPIRY_EPOCH=$(date -d "$EXPIRY" +%s)
NOW_EPOCH=$(date +%s)

if [ $EXPIRY_EPOCH -gt $NOW_EPOCH ]; then
    echo "IQ-005: PASS - Certificate valid until $EXPIRY"
else
    echo "IQ-005: FAIL - Certificate expired"
    exit 1
fi
```

**验收标准**: 证书格式正确且未过期

---

## 3. OQ - 操作验证

### 3.1 OQ 测试矩阵

| 测试 ID | 测试项 | 验收标准 | 方法 | 结果 |
|---------|--------|----------|------|------|
| OQ-001 | 用户认证 | 正确密码登录成功，错误密码拒绝 | 功能测试 | 待测 |
| OQ-002 | RBAC 权限 | Admin 可执行所有操作，ReadOnly 只能 SELECT | 功能测试 | 待测 |
| OQ-003 | 审计事件记录 | 执行 DDL 后查询 audit_log 有记录 | 功能测试 | 待测 |
| OQ-004 | 哈希链验证 | 篡改后 verify_chain 返回 false | 故障注入 | 待测 |
| OQ-005 | 签名验证 | Ed25519 签名验证正确工作 | 功能测试 | 待测 |
| OQ-006 | TLS 连接 | TLS 1.2+ 连接成功，TLS 1.0 拒绝 | 功能测试 | 待测 |
| OQ-007 | WAL 恢复 | 模拟崩溃后数据一致 | 故障注入 | 待测 |
| OQ-008 | 会话超时 | 30 分钟无活动会话失效 | 计时测试 | 待测 |

### 3.2 OQ-001: 用户认证测试

```sql
-- OQ-001: 认证测试

-- 步骤 1: 创建用户
CREATE USER alice WITH PASSWORD 'Str0ng!Pass';
GRANT SELECT ON mydb.* TO alice;

-- 步骤 2: 正确认证
-- 使用 alice 凭证连接
-- 预期: 登录成功

-- 步骤 3: 错误密码
-- 使用 alice + 错误密码连接
-- 预期: 拒绝访问

-- 步骤 4: 不存在用户
-- 使用 bob + 任意密码连接
-- 预期: 拒绝访问
```

**验收标准**: 正确登录 1 次，错误登录 2 次拒绝

### 3.3 OQ-002: RBAC 权限测试

```sql
-- OQ-002: 权限测试

-- 准备
CREATE TABLE secret_data (id INT, content TEXT);
INSERT INTO secret_data VALUES (1, 'sensitive');

-- Admin 用户测试
-- 预期: SELECT, INSERT, UPDATE, DELETE, DROP 全部成功

-- ReadOnly 用户测试
-- 预期: SELECT 成功, INSERT 拒绝, DELETE 拒绝

-- AuditViewer 测试
-- 预期: SELECT ON audit_log 成功, SELECT ON secret_data 拒绝
```

**验收标准**: 每种角色操作结果符合矩阵

### 3.4 OQ-003: 审计事件记录测试

```sql
-- OQ-003: 审计记录测试

-- 步骤 1: 执行 DDL
CREATE TABLE audit_test (id INT, name TEXT);

-- 步骤 2: 验证审计记录
SELECT * FROM audit_log 
WHERE object_name = 'audit_test' 
ORDER BY timestamp DESC 
LIMIT 1;

-- 预期: 找到 CREATE TABLE 记录

-- 步骤 3: 执行 DML
INSERT INTO audit_test VALUES (1, 'test');

-- 步骤 4: 验证
SELECT * FROM audit_log 
WHERE object_name = 'audit_test' 
AND action = 'INSERT';

-- 预期: 找到 INSERT 记录
```

**验收标准**: DDL 和 DML 都有审计记录

### 3.5 OQ-004: 哈希链验证测试

```rust
// OQ-004: 哈希链验证测试

#[test]
fn test_hash_chain_tamper_detection() {
    let mut audit_store = AuditStore::new();
    let mut verifier = AuditVerifier::new();
    
    // 1. 插入事件
    let event = AuditEvent::new("user1", "SELECT", "test_db");
    verifier.verify_on_insert(&event).unwrap();
    
    // 2. 篡改事件内容
    let mut tampered = event.clone();
    tampered.actor = "hacker".to_string();
    
    // 3. 验证应失败
    let result = verifier.verify_on_insert(&tampered);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        VerificationError::ChainBroken { .. }
    ));
}
```

**验收标准**: 篡改检测延迟 < 1ms

### 3.6 OQ-005: 签名验证测试

```rust
// OQ-005: 签名验证测试

#[test]
fn test_signature_verify() {
    let manager = SignatureManager::new();
    let data = b"test data for signature";
    
    // 1. 签名
    let signature = manager.sign(data);
    
    // 2. 验证正确签名
    assert!(manager.verify(data, &signature));
    
    // 3. 篡改后验证应失败
    let tampered = b"tampered data";
    assert!(!manager.verify(tampered, &signature));
    
    // 4. 错误签名应失败
    let wrong_sig = manager.sign(b"other data");
    assert!(!manager.verify(data, &wrong_sig));
}
```

**验收标准**: 签名和验证正常工作

### 3.7 OQ-006: TLS 连接测试

```bash
#!/bin/bash
# OQ-006: TLS 版本测试

# 测试 TLS 1.2 连接 (应成功)
openssl s_client -connect localhost:5432 \
    -tls1_2 \
    -cert /etc/sqlrustgo/tls/client.crt \
    -key /etc/sqlrustgo/tls/client.key \
    </dev/null 2>/dev/null | grep "Protocol.*TLS"
# 预期: Protocol : TLSv1.2

# 测试 TLS 1.0 连接 (应失败)
openssl s_client -connect localhost:5432 \
    -tls1 \
    </dev/null 2>/dev/null | grep -i "alert"
# 预期: 收到 alert 或连接拒绝
```

**验收标准**: TLS 1.2+ 成功，TLS 1.0/1.1 拒绝

### 3.8 OQ-007: WAL 恢复测试

```bash
#!/bin/bash
# OQ-007: WAL 恢复测试

DB_PATH="/tmp/oq_test_db"

# 1. 初始化数据库
sqlrustgo --init --db-path $DB_PATH

# 2. 执行事务
sqlrustgo -e "CREATE TABLE recover_test (id INT)" --db-path $DB_PATH
sqlrustgo -e "INSERT INTO recover_test VALUES (1)" --db-path $DB_PATH
sqlrustgo -e "COMMIT" --db-path $DB_PATH

# 3. 模拟崩溃 (注入故障)
sqlrustgo --inject-crash=AfterWalWrite --db-path $DB_PATH

# 4. 重启恢复
sqlrustgo --db-path $DB_PATH 2>&1 | grep "Recovery"

# 5. 验证数据一致性
RESULT=$(sqlrustgo -e "SELECT COUNT(*) FROM recover_test" --db-path $DB_PATH)
if [ "$RESULT" = "1" ]; then
    echo "OQ-007: PASS"
else
    echo "OQ-007: FAIL"
    exit 1
fi
```

**验收标准**: 恢复后数据一致

### 3.9 OQ-008: 会话超时测试

```bash
#!/bin/bash
# OQ-008: 会话超时测试

# 1. 登录获取 session
SESSION=$(sqlrustgo-cli auth --user alice --pass 'Str0ng!Pass')
echo "Session: $SESSION"

# 2. 等待 29 分钟 (小于超时)
sleep 1740  # 29 分钟

# 3. 查询应成功
RESULT=$(sqlrustgo-cli query --session $SESSION "SELECT 1")
if [ "$RESULT" = "1" ]; then
    echo "OQ-008a: PASS - Session still valid at 29min"
else
    echo "OQ-008a: FAIL - Session expired too early"
    exit 1
fi

# 4. 等待 2 分钟 (总 31 分钟)
sleep 120

# 5. 查询应失败
RESULT=$(sqlrustgo-cli query --session $SESSION "SELECT 1" 2>&1)
if echo "$RESULT" | grep -q "Session expired"; then
    echo "OQ-008b: PASS - Session expired at 31min"
else
    echo "OQ-008b: FAIL - Session did not expire"
    exit 1
fi
```

**验收标准**: 30 分钟精确超时

---

## 4. PQ - 性能验证

### 4.1 PQ 测试矩阵

| 测试 ID | 测试项 | 验收标准 | 方法 | 结果 |
|---------|--------|----------|------|------|
| PQ-001 | 吞吐量 | >= 10,000 TPS | TPC-C 测试 | 待测 |
| PQ-002 | 审计验证延迟 | p99 < 10ms | 1000 次验证 | 待测 |
| PQ-003 | 恢复时间 | RTO < 5 分钟 (1GB WAL) | 故障注入 | 待测 |
| PQ-004 | 内存稳定性 | 72h 内存增长 < 5% | 长稳测试 | 待测 |
| PQ-005 | 并发用户 | 100 并发无死锁 | 压力测试 | 待测 |

### 4.2 PQ-001: 吞吐量测试

```bash
#!/bin/bash
# PQ-001: TPC-C 吞吐量测试

DURATION=300  # 5 分钟
WAREHOUSES=10

# 运行 TPC-C
pgbench -c 10 -j 4 -T $DURATION \
    -h localhost -p 5432 -U sqlrustgo -d tpcc \
    --report-latencies

# 提取 TPS
TPS=$(grep "tps =" /tmp/pgbench.log | tail -1 | awk '{print $3}')

if (( $(echo "$TPS >= 10000" | bc -l) )); then
    echo "PQ-001: PASS - TPS=$TPS >= 10000"
else
    echo "PQ-001: FAIL - TPS=$TPS < 10000"
    exit 1
fi
```

**验收标准**: TPS >= 10,000

### 4.3 PQ-002: 审计验证延迟测试

```rust
// PQ-002: 审计验证延迟测试

#[test]
fn test_audit_verify_latency() {
    let verifier = AuditVerifier::new();
    let mut times = Vec::new();
    
    // 生成 1000 个测试事件
    for i in 0..1000 {
        let event = AuditEvent::new("user", "SELECT", "db");
        let start = Instant::now();
        verifier.verify_on_insert(&event).unwrap();
        times.push(start.elapsed().as_micros());
    }
    
    // 计算 p99
    times.sort();
    let p99_idx = (times.len() as f64 * 0.99) as usize;
    let p99_latency_us = times[p99_idx];
    
    assert!(p99_latency_us < 10000); // < 10ms
    println!("PQ-002: p99 latency = {}us", p99_latency_us);
}
```

**验收标准**: p99 < 10ms

### 4.4 PQ-003: 恢复时间测试

```bash
#!/bin/bash
# PQ-003: RTO 测试

# 1. 生成 1GB WAL
dd if=/dev/urandom of=/tmp/1gb_wal bs=1M count=1024
cp /tmp/1gb_wal /var/lib/sqlrustgo/wal/wal.log

# 2. 触发崩溃
sqlrustgo --force-crash

# 3. 计时恢复
START=$(date +%s)
sqlrustgo --recover
END=$(date +%s)

RTO=$((END - START))

if [ $RTO -lt 300 ]; then
    echo "PQ-003: PASS - RTO=$RTO seconds < 300"
else
    echo "PQ-003: FAIL - RTO=$RTO seconds >= 300"
    exit 1
fi
```

**验收标准**: RTO < 5 分钟

### 4.5 PQ-004: 内存稳定性测试

```bash
#!/bin/bash
# PQ-004: 内存稳定性测试

# 启动 72h 测试
sqlrustgo --run-stability-test \
    --duration 259200 \  # 72h
    --report-interval 3600 \
    --memory-threshold 1.05 \
    > /tmp/stability.log 2>&1 &

PID=$!

# 监控
for i in {1..72}; do
    MEM=$(ps -p $PID -o rss=)
    echo "Hour $i: RSS=${MEM}KB"
    sleep 3600
done

# 验证
wait $PID
RESULT=$?

if [ $RESULT -eq 0 ]; then
    echo "PQ-004: PASS - Memory stable for 72h"
else
    echo "PQ-004: FAIL - Memory growth exceeded 5%"
    exit 1
fi
```

**验收标准**: 72h 内存增长 < 5%

### 4.6 PQ-005: 并发用户测试

```bash
#!/bin/bash
# PQ-005: 并发测试

# 启动 100 个并发用户
for i in {1..100}; do
    (
        for j in {1..100}; do
            sqlrustgo -e "SELECT $i, $j" > /dev/null 2>&1
        done
    ) &
done

# 等待完成
wait

# 检查死锁
DEADLOCKS=$(sqlrustgo-cli show-deadlocks 2>/dev/null)

if [ -z "$DEADLOCKS" ]; then
    echo "PQ-005: PASS - 100 concurrent users, 0 deadlocks"
else
    echo "PQ-005: FAIL - Deadlocks detected: $DEADLOCKS"
    exit 1
fi
```

**验收标准**: 100 并发 0 死锁

---

## 5. 偏差处理

### 5.1 偏差分类

| 严重性 | 定义 | 处理时间 |
|--------|------|----------|
| Critical | 影响数据完整性或安全 | 24h 内 |
| Major | 影响功能但有缓解措施 | 7 天内 |
| Minor | 不影响功能或安全 | 30 天内 |

### 5.2 偏差报告模板

```markdown
## 偏差报告

**偏差 ID**: DEV-YYYY-XXX
**报告日期**: YYYY-MM-DD
**严重性**: Critical / Major / Minor
**测试阶段**: IQ / OQ / PQ

### 偏差描述
[详细描述偏差情况]

### 预期结果
[验证协议中的验收标准]

### 实际结果
[实际观察到的结果]

### 影响评估
[对 GMP 合规性的影响]

### 根本原因分析
[5 Why 分析]

### 纠正措施
[立即采取的措施]

### 预防措施
[防止再次发生的措施]

### 审批
- 偏差责任人: ____________
- QA 审批: ____________
- 系统负责人: ____________
```

---

## 6. 验证总结

### 6.1 验证完成标准

| 阶段 | 通过标准 |
|------|----------|
| IQ | 所有测试 PASS |
| OQ | 所有测试 PASS |
| PQ | 所有测试 PASS + 偏差已关闭 |

### 6.2 验证报告

```markdown
## 验证总结报告

**系统**: SQLRustGo v3.1.0
**验证周期**: YYYY-MM-DD 至 YYYY-MM-DD
**验证结论**: [APPROVED / CONDITIONALLY APPROVED / REJECTED]

### IQ 结果
- 测试数: X
- 通过: X
- 失败: X
- 偏差数: X

### OQ 结果
- 测试数: X
- 通过: X
- 失败: X
- 偏差数: X

### PQ 结果
- 测试数: X
- 通过: X
- 失败: X
- 偏差数: X

### 最终结论
[详细说明]
```

---

## 7. 附录

### A. 验证工具清单

| 工具 | 版本 | 用途 |
|------|------|------|
| sqlrustgo | 3.1.0 | 被测系统 |
| openssl | 1.1+ | TLS 验证 |
| bc | any | 数学计算 |
| pgbench | any | 性能测试 |

### B. 参考标准

- FDA 21 CFR Part 11
- EU Annex 11
- GAMP 5
- ISPE GAMP Good Practice Guide: IT Infrastructure