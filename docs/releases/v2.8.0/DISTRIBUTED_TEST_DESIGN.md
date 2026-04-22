# v2.8.0 分布式测试设计

> 版本: `v2.8.0`
> 目标: 验证分布式复制、故障转移、负载均衡功能
> 最后更新: 2026-04-23

---

## 一、测试架构

### 1.1 集群拓扑

```
┌─────────────────────────────────────────────────────────────┐
│                      Load Balancer                           │
│                  (sqlrustgo-lb :8080)                       │
└────────────────┬─────────────────────────────┬──────────────┘
                 │                             │
         ┌───────▼───────┐             ┌───────▼───────┐
         │  Master Node  │             │  Slave Node 1 │
         │ :3306 (write) │◄── replicate ──►│ :3306 (read) │
         └───────────────┘             └───────────────┘
                 │
         ┌───────▼───────┐
         │  Slave Node 2 │
         │ :3306 (read)  │
         └───────────────┘
```

### 1.2 测试环境

| 组件 | 数量 | 配置 |
|------|------|------|
| Load Balancer | 1 | 2 CPU, 4GB RAM |
| Master | 1 | 2 CPU, 4GB RAM |
| Slave | 2 | 2 CPU, 4GB RAM |

---

## 二、主从复制测试 (T-24)

### 2.1 GTID 复制测试

```bash
# 测试脚本: tests/distributed/test_gtid_replication.sh

# 1. 启动集群
docker-compose -f docker/distributed.yml up -d

# 2. 创建测试数据
mysql -h master -e "CREATE TABLE t (id INT PRIMARY KEY, v INT);"
mysql -h master -e "INSERT INTO t VALUES (1, 100), (2, 200);"

# 3. 等待复制
sleep 5

# 4. 验证数据一致性
mysql -h slave1 -e "SELECT * FROM t;" | sort
mysql -h slave2 -e "SELECT * FROM t;" | sort

# 5. 验证 GTID
mysql -h master -e "SHOW MASTER STATUS\G"
mysql -h slave1 -e "SHOW SLAVE STATUS\G"
```

### 2.2 数据一致性测试

| 测试用例 | 操作 | 预期结果 | 验证方式 |
|----------|------|----------|----------|
| 基础复制 | INSERT 1000 rows | 所有从节点数据一致 | COUNT(*) |
| 批量复制 | LOAD DATA | 所有从节点数据一致 | CHECKSUM TABLE |
| 更新复制 | UPDATE 500 rows | 所有从节点数据一致 | CHECKSUM TABLE |
| 删除复制 | DELETE 100 rows | 所有从节点数据一致 | COUNT(*) |
| 事务复制 | BEGIN/COMMIT | 原子性复制 | 断电测试 |

### 2.3 复制延迟测试

```bash
# 测试脚本: tests/distributed/test_replication_latency.sh

# 测试方法: 记录主节点执行时间戳，从节点应用时间戳，差值为延迟

# 测试用例:
# 1. 单行 INSERT 延迟 < 100ms
# 2. 批量 INSERT (100 rows) 延迟 < 500ms
# 3. 事务提交延迟 < 200ms
```

### 2.4 半同步复制测试

```bash
# 测试脚本: tests/distributed/test_semisync.sh

# 1. 启用半同步
mysql -h master -e "SET GLOBAL rpl_semi_sync_master_enabled = 1;"
mysql -h slave1 -e "SET GLOBAL rpl_semi_sync_slave_enabled = 1;"

# 2. 测试半同步行为
# - 从节点 ACK 超时后自动降级为异步
# - 网络恢复后自动恢复半同步

# 3. 验证指标
# - rpl_semi_sync_master_clients
# - rpl_semi_sync_master_no_times
# - rpl_semi_sync_master_no_tx
```

---

## 三、故障转移测试 (T-25)

### 3.1 主节点宕机测试

```bash
# 测试脚本: tests/distributed/test_failover.sh

# 1. 正常状态验证
curl http://lb:8080/health  # expected: 200
mysql -h master -e "SELECT 1;"  # expected: success

# 2. 故障注入 (kill master)
docker kill sqlrustgo_master
# 或: kill -9 $(pgrep -f sqlrustgo-master)

# 3. 等待故障检测
# 预期: 5 秒内检测到主节点不可达

# 4. 等待故障转移
# 预期: 30 秒内完成故障转移

# 5. 验证新主节点
curl http://lb:8080/health  # expected: 200 (新主节点)
mysql -h new_master -e "SELECT 1;"  # expected: success

# 6. 验证数据不丢失
mysql -h new_master -e "SELECT COUNT(*) FROM t;"  # expected: 与故障前一致
```

### 3.2 故障转移时间测试

| 测试用例 | 目标时间 | 超时时间 |
|----------|----------|----------|
| 故障检测 | < 5s | 10s |
| 故障转移 | < 30s | 60s |
| 写恢复 | < 5s | 10s |
| 总恢复时间 | < 40s | 70s |

### 3.3 自动切换条件测试

| 条件 | 触发切换 | 不切换 |
|------|----------|--------|
| 节点宕机 (kill -9) | ✅ | - |
| 节点僵死 (OOM) | ✅ | - |
| 网络闪断 (< 5s) | - | ✅ |
| 网络中断 (> 30s) | ✅ | - |
| 手动触发 | ✅ | - |

### 3.4 恢复测试

```bash
# 测试脚本: tests/distributed/test_recovery.sh

# 1. 原主节点恢复
docker start sqlrustgo_master_old

# 2. 验证数据追平
# 预期: 原主节点作为从节点，追平新主节点数据

# 3. 验证可重新选主
# 在新主节点故障后，原主节点可被选为新主节点
```

---

## 四、负载均衡测试 (T-26)

### 4.1 轮询策略测试

```bash
# 测试脚本: tests/distributed/test_round_robin.sh

# 1. 发送 100 个 SELECT 请求
for i in $(seq 1 100); do
  mysql -h lb -e "SELECT @@server_id;" &
done
wait

# 2. 统计分布
# 预期: 每个节点约 33-34 次

# 3. 验证无重复连接
# netstat -an | grep lb:3306 | wc -l
```

### 4.2 最少连接策略测试

```bash
# 测试脚本: tests/distributed/test_least_connections.sh

# 1. 打开长连接占用 slave1
mysql -h slave1 -e "SELECT SLEEP(100);" &

# 2. 发送 10 个请求
for i in $(seq 1 10); do
  curl http://lb:8080/query?sql="SELECT 1" &
done
wait

# 3. 验证请求分发
# 预期: slave1 不在新请求分发列表中，直到连接释放
```

### 4.3 健康检查测试

```bash
# 测试脚本: tests/distributed/test_health_check.sh

# 1. 标记节点不健康
curl -X POST http://lb:8080/admin/down?node=slave1

# 2. 发送请求
# 预期: 请求不会分发到 slave1

# 3. 恢复节点
curl -X POST http://lb:8080/admin/up?node=slave1

# 4. 验证恢复
# 预期: 请求重新分发到 slave1
```

### 4.4 负载均衡算法测试

| 算法 | 测试用例 | 预期结果 |
|------|----------|----------|
| Round-Robin | 100 请求均匀分发 | 每节点 33-34 次 |
| Least-Connections | 长连接占用 | 空闲节点优先 |
| Random | 100 请求分布 | 统计均匀 |
| IP-Hash | 同一 IP 来源 | 会话保持 |

### 4.5 负载测试

```bash
# 测试脚本: tests/distributed/test_load.sh

# 工具: wrk, mysqlslap

# 1. 基准测试 (单节点)
wrk -t4 -c100 -d30s http://single:8080/query?sql="SELECT * FROM t WHERE id=1"

# 2. 负载均衡测试
wrk -t4 -c100 -d30s http://lb:8080/query?sql="SELECT * FROM t WHERE id=1"

# 3. 对比 QPS
# 预期: 负载均衡 QPS > 单节点 QPS * 节点数 * 0.8
```

---

## 五、读写分离测试 (T-27)

### 5.1 路由规则测试

```bash
# 测试脚本: tests/distributed/test_rw_split.sh

# 1. 验证写操作路由到主节点
mysql -h lb -e "INSERT INTO t VALUES (999, 999);"
# 验证: 只有 master 有这条记录

# 2. 验证读操作路由到从节点
mysql -h lb -e "SELECT * FROM t WHERE id=999;"
# 验证: 可从任意节点读取

# 3. 验证事务路由
mysql -h lb -e "BEGIN; SELECT * FROM t; INSERT INTO t VALUES (998, 998); COMMIT;"
# 验证: 整个事务在主节点执行
```

### 5.2 读写分离规则

| SQL 类型 | 路由目标 | 匹配规则 |
|----------|----------|----------|
| INSERT | Master | 匹配 INSERT |
| UPDATE | Master | 匹配 UPDATE |
| DELETE | Master | 匹配 DELETE |
| SELECT (显式) | 可配置 | 显式指定 |
| SELECT (隐式) | 轮询 | 默认行为 |
| BEGIN | Master | 事务开始 |
| COMMIT | Master | 事务提交 |

### 5.3 会话一致性测试

```bash
# 测试脚本: tests/distributed/test_session_consistency.sh

# 1. 写后读一致性
mysql -h lb -e "INSERT INTO t VALUES (100, 100);"
mysql -h lb -e "SELECT * FROM t WHERE id=100;"
# 预期: 读取到刚插入的数据 (主节点读)

# 2. 配置从节点延迟后验证
# 场景: 写后立即读可能读到旧数据 (如果读从节点)
```

---

## 六、性能测试

### 6.1 基准测试

```bash
# 测试脚本: tests/distributed/test_performance.sh

# TPC-C 基准 (使用 go-tpc)
go-tpc workload -h master:3306 -D testdb -T 16

# 对比指标:
# - 单节点: tpmC
# - 集群: tpmC
# - 扩展效率: 集群 tpmC / (单节点 tpmC * 节点数)
```

### 6.2 故障转移影响

```bash
# 测试脚本: tests/distributed/test_failover_impact.sh

# 1. 基准 QPS
wrk -t4 -c100 -d60s http://lb:8080/query

# 2. 故障注入
docker kill master

# 3. 故障转移期间 QPS
wrk -t4 -c100 -d60s http://lb:8080/query

# 4. 验证
# 预期: 故障转移期间 QPS > 0
# 预期: 故障转移后 QPS 恢复到基准的 80%
```

---

## 七、测试工具清单

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| mysql (client) | SQL 执行 | apt install mysql-client |
| mysqldiff | 数据一致性对比 | apt install mysql-utilities |
| wrk | HTTP 压测 | apt install wrk |
| go-tpc | TPC-C 基准 | go install github.com/pingcap/go-tpc@latest |
| docker-compose | 集群管理 | apt install docker-compose |
| netstat/ss | 连接监控 | 内置 |
| tcpdump | 抓包分析 | apt install tcpdump |

---

## 八、测试执行流程

### 8.1 每日构建测试

```bash
# 在 CI 中自动执行
cd tests/distributed
./run_daily_tests.sh
```

### 8.2 发布前测试

```bash
# 在 RC/GA 门禁前执行
cd tests/distributed
./run_full_tests.sh --release
```

### 8.3 测试结果记录

| 测试 | 日期 | 结果 | 负责人 |
|------|------|------|--------|
| GTID 复制 | YYYY-MM-DD | PASS/FAIL | |
| 故障转移 | YYYY-MM-DD | PASS/FAIL | |
| 负载均衡 | YYYY-MM-DD | PASS/FAIL | |
| 读写分离 | YYYY-MM-DD | PASS/FAIL | |
| 性能测试 | YYYY-MM-DD | PASS/FAIL | |

---

*本文档由 SQLRustGo Team 维护*
*最后更新: 2026-04-23*