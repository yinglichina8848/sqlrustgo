# SQLRustGo 版本演化计划

> **版本**: v9.0
> **更新日期**: 2026-03-29
> **战略定位**: 教学数据库产品（Teaching DBMS）
> **核心原则**: 替代 MySQL 教学，差异化超越

---

## 一、战略定位（关键转向）

### 从：数据库内核工程
### 到：教学数据库产品（Teaching DBMS）

```
💡 核心洞察：

❌ 不要试图完全兼容 MySQL（工程量爆炸）
✅ 正确策略：兼容"教材"，而不是兼容"数据库"

🎯 最终目标：

v1.7 = 可替代 MySQL（教学 + 上机）← 合并 v1.8 + v1.9
v2.0 = 明显优于 MySQL（教学体验 + 分析能力）
```

---

## 二、版本语义化与技术范式

| 主版本 | 技术范式 | 成熟度等级 | 核心特征 | 对标系统 |
|--------|----------|------------|----------|----------|
| 1.x | 单机基础数据库 | L3 | 火山模型、SQL-92、MVCC、B+Tree | early PostgreSQL |
| 2.x | 高性能分析引擎 | L4 | 向量化、列存、CBO、并行执行 | DuckDB |

**演进路径**：1.x（教学可用）→ 2.x（超越 MySQL）

---

## 三、版本路线（教学 DBMS 战略）

### 战略总览

```
v1.6 ✅      v1.7 (合并版)                    v2.0
  Benchmark   SQL+可观测+MySQL兼容+稳定+教学   高性能分析
  修复        ⭐⭐⭐ 全面替代 MySQL 教学         超越MySQL
```

### 详细版本规划

| 版本 | 代号 | 核心目标 | 关键功能 |
|------|------|----------|----------|
| **v1.7** | **MySQL 教学替代版** | 可替代 MySQL | UNION/VIEW, EXPLAIN ANALYZE, FOREIGN KEY, AUTO_INCREMENT, CLI, WAL, ≥85%覆盖 |
| **v2.0** | **高性能分析** | 超越 MySQL | 向量化, 列存, 可视化执行 |

### v1.7 Epics (合并版)

| Epic | 名称 | 来源版本 |
|------|------|----------|
| Epic-01 | SQL 补完 | 原 v1.7 |
| Epic-02 | 可观测性 | 原 v1.7 |
| Epic-03 | Benchmark 完善 | 原 v1.7 |
| Epic-04 | 错误系统 | 原 v1.7 |
| Epic-05 | 约束与外键 | 原 v1.8 |
| Epic-06 | MySQL 兼容语法 | 原 v1.8 |
| Epic-07 | CLI 工具完善 | 原 v1.8 |
| Epic-08 | 稳定性强化 | 原 v1.9 |
| Epic-09 | 覆盖率提升 | 原 v1.9 |
| Epic-10 | 教学支持 | 原 v1.9 |

---

## 四、v1.7 核心版本详解（MySQL 教学替代版）

### 🔥 一站式替代 MySQL 教学

#### Epic-01~04: SQL + 可观测性（原 v1.7）

| 功能 | 说明 |
|------|------|
| UNION, UNION ALL, INTERSECT, EXCEPT | 集合运算 |
| VIEW | 视图支持 |
| EXPLAIN, EXPLAIN ANALYZE | 执行计划可视化 |
| MySQL 风格错误 | Unknown column, Table not found, Duplicate key |

#### Epic-05: 约束与外键（原 v1.8）

```sql
FOREIGN KEY (user_id) REFERENCES users(id)
```

#### Epic-06: MySQL 兼容语法（原 v1.8）

```sql
-- AUTO_INCREMENT
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255)
);

-- LIMIT offset, count
SELECT * FROM orders LIMIT 10, 20;

-- SHOW / DESCRIBE
SHOW TABLES;
DESCRIBE orders;

-- 常用函数
NOW(), COUNT(), DATE_FORMAT()
```

#### Epic-07: CLI 工具完善（原 v1.8）

```bash
sqlrustgo

支持：
.tables
.schema orders
.indexes orders
```

#### Epic-08: 稳定性强化（原 v1.9）

- WAL 恢复强化
- Crash 安全
- 长时间运行测试

#### Epic-09: 覆盖率提升（原 v1.9）

- ≥ 85%

#### Epic-10: 教学支持（原 v1.9）

```bash
SQLRUSTGO_TEACHING_MODE=1
```

效果：
- 禁用优化器（便于教学）
- 强制 EXPLAIN 输出
- 更详细日志

教学资源：
- 12 个标准实验
- MySQL → SQLRustGo 对照表
- Lab 文档

---

## 五、v2.0 核心版本详解（高性能分析）

### 🔥 超越 MySQL

#### 1️⃣ 向量化执行（核心革命）

```rust
// DataChunk 格式
struct DataChunk {
    columns: Vec<ColumnArray>,
    num_rows: usize,
}

// SIMD 加速
impl AggregateFunction for Sum<i32> {
    fn sum_batch(&self, chunk: &DataChunk) -> ScalarValue {
        // SIMD 加速聚合
    }
}
```

#### 2️⃣ 列存（分析能力）

- Columnar storage
- Projection pushdown
- Parquet 支持

#### 3️⃣ 教学增强（差异化）

> MySQL 做不到的：

- 可视化执行 pipeline
- 算子级 profiling
- vectorized trace

---

## 六、可观测性演进

| 版本 | 新增可观测能力 |
|------|----------------|
| v1.7 | **EXPLAIN ANALYZE**（核心亮点）, 算子级耗时, 教学模式 |

---

## 七、成熟度演进

```
L1 (Toy)   →   L2 (Query Engine)   →   L3 (Mini DBMS)   →   L4 (Analytical DB)
                   v1.7                    v2.0
               能替代MySQL教学          超越MySQL
```

---

## 八、风险与应对

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 试图完全兼容 MySQL | 极高 | ❌ 禁止！只兼容"教材" |
| MySQL 协议实现难度 | 高 | 先做 CLI，协议可选 |
| 教学文档工作量 | 中 | 参考现有 MySQL 教材 |
| 向量化开发难度 | 高 | v2.0 预留充足时间 |

---

## 九、版本号与分支策略

| 策略 | 说明 |
|------|------|
| 开发分支 | develop/v1.7.0, develop/v2.1.0 |
| 发布分支 | release/v1.7.0, release/v2.1.0 |
| 主分支 | main 始终指向最新稳定版 |

---

## 十、一句顶级结论

```
💥 你现在不是在做数据库
👉 而是在做"下一代数据库教学平台"
```

---

## 十一、关联文档

| 文档 | 说明 |
|------|------|
| `docs/plans/2026-03-21-v170-release-plan.md` | v1.7.0 详细计划 |
| `docs/releases/v1.6.1/RELEASE_GATE_CHECKLIST.md` | v1.6.1 门禁参考 |
| [AI员工1.0自研开发讨论稿](./AI_EMPLOYEE_V1_DEVELOPMENT_DISCUSSION.md) | AI员工GMP审核系统详细规划 |
| [企业AI员工系统架构](./ENTERPRISE_AI_AGENT_SYSTEM.md) | 系统架构设计 |

---

## 十二、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-12 | 初始版本 |
| 2.0 | 2026-03-13 | v1.x/v2.x/v3.x 重构 |
| 3.0 | 2026-03-13 | 工程优化版 |
| 4.0 | 2026-03-18 | 整合 v1.x 版本 |
| 5.0 | 2026-03-21 | SQL-92 路线图 |
| 6.0 | 2026-03-21 | 教学 DBMS 战略定位 |
| 7.0 | 2026-03-21 | v1.7 合并 v1.8+v1.9，v2.0 独立 |
| **8.0** | **2026-03-29** | **添加 v2.1.0-v2.5.0 AI员工GMP审核系统规划** |
| **9.0** | **2026-03-29** | **v2.1整合运维+故障+MySQL兼容，新增OpenClaw API功能** |

---

**文档状态**: 定稿
**制定日期**: 2026-03-21
**最后更新**: 2026-03-29
**制定人**: yinglichina8848
**战略定位**: 教学数据库产品（Teaching DBMS）

---

## v2.1 - 省心线+MySQL兼容版 (2026年4月)

> **版本合并**: 原v2.1(运维) + 原v2.2(故障) + 原v3.0(MySQL兼容) → 整合为单一版本
> **战略调整**: 三个月完成，省心线+MySQL 5.6兼容+OpenClaw API

**目标**: 一站式达成省心线 + MySQL 5.6 兼容 + OpenClaw API

### 交付物

#### A. 省心线功能
- Prometheus 监控端点
- Grafana Dashboard
- 慢查询日志
- 一键备份/恢复 CLI
- 自动故障检测 (< 5s)
- 自动主从切换 (< 30s)
- 读写分离代理
- 运维 Web 面板
- SHOW PROCESSLIST / KILL 命令
- 性能基准: 3000+ QPS

#### B. MySQL 5.6 兼容功能
- 触发器 (行级/语句级)
- 存储过程 + 存储函数
- 分区表 (RANGE/KEY/HASH)
- 全文索引 + 中文分词
- Prepared Statements + 缓存
- JSON 数据类型 + 函数
- AUTO_INCREMENT 完善
- UPSERT (ON DUPLICATE KEY UPDATE)
- SHOW / DESCRIBE 命令完善
- MySQL 兼容度: ≥80%

#### C. OpenClaw API 功能 (新增)
- OpenClaw Agent 直连 SQLRustGo
- SQL 执行结果结构化返回
- Schema 信息查询 API
- 事务控制 API (BEGIN/COMMIT/ROLLBACK)
- 健康检查 API
- 指标暴露 API (Prometheus格式)

### Issue 列表 (共51个)

#### 省心线 Issue (12个)
- #1013 Prometheus 监控端点
- #1014 慢查询日志系统
- #1015 健康检查端点
- #1016 mysqldump 兼容导入
- #1017 物理备份 CLI
- #1018 增量备份工具
- #1019 备份恢复验证
- #1025 自动故障检测
- #1026 自动主从切换
- #1029 读写分离代理
- #1035 SHOW PROCESSLIST
- #1036 KILL 命令

#### MySQL 兼容 Issue (26个)
- #1038 触发器语法解析
- #1039 触发器执行引擎
- #1040 行级/语句级触发器
- #1041 存储过程基础
- #1042 存储函数
- #1043 分区表语法解析
- #1044 分区表物理存储
- #1045 分区裁剪优化
- #1046 分区表 DDL
- #1047 全文索引语法
- #1048 倒排索引实现
- #1049 MATCH 查询
- #1050 中文分词
- #1051 PREPARE 语句解析
- #1052 EXECUTE 执行
- #1053 PreparedStatement 缓存
- #1060 JSON 数据类型
- #1061 JSON 函数
- #1124 AUTO_INCREMENT 执行逻辑
- #1125 UPSERT 执行逻辑
- #1126 SHOW TABLES 完善
- #1127 DESCRIBE 命令完善
- #1128 LIMIT offset,count 完善
- #1129 常用函数补充 (DATE_FORMAT等)
- #1130 外键 DELETE/UPDATE 动作完善

#### OpenClaw API Issue (5个)
- #1141 OpenClaw 连接器 - SQL 执行接口
- #1142 OpenClaw 连接器 - Schema 查询接口
- #1143 OpenClaw 连接器 - 事务控制接口
- #1144 OpenClaw 连接器 - 健康检查接口
- #1145 OpenClaw 连接器 - Prometheus 指标接口

#### 优化 Issue (8个)
- #1121 配置热更新
- #1122 版本升级脚本
- #1123 日志轮转
- #1131 查询缓存优化
- #1132 连接池
- #1032 运维 Web 面板
- #1033 SQL 执行界面
- #1034 备份管理界面

### 时间规划

```
2026-04-01 ── 2026-04-07   Phase 1: 触发器 + 监控基础
2026-04-08 ── 2026-04-15   Phase 2: 存储过程 + 分区表
2026-04-16 ── 2026-04-22   Phase 3: 全文索引 + JSON + Prepared
2026-04-23 ── 2026-04-30   Phase 4: OpenClaw API + 收尾
```

### OpenClaw API 设计

```rust
// OpenClaw SQLRustGo 连接器接口

#[async_trait]
pub trait SqlRustGoConnector {
    // SQL执行
    async fn execute(&self, sql: &str) -> Result<ExecuteResult>;
    async fn query(&self, sql: &str) -> Result<QueryResult>;

    // Schema操作
    async fn get_tables(&self) -> Result<Vec<TableInfo>>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnInfo>>;
    async fn get_indexes(&self, table: &str) -> Result<Vec<IndexInfo>>;

    // 事务控制
    async fn begin(&self) -> Result<TransactionId>;
    async fn commit(&self, tx_id: TransactionId) -> Result<()>;
    async fn rollback(&self, tx_id: TransactionId) -> Result<()>;

    // 健康检查
    async fn health_check(&self) -> Result<HealthStatus>;

    // 指标
    async fn metrics(&self) -> Result<PrometheusMetrics>;
}
```

### 状态
📋 规划完成 (2026-03-29)

---

## 省心线达成时间表

| 版本 | 目标月份 | 省心程度 | MySQL兼容度 | 成本估算 |
|------|----------|----------|-------------|----------|
| v2.0 | 2026-03 | ⭐⭐⭐ 功能可用 | 60% | ¥100-600 |
| **v2.1** | **2026-04** | **⭐⭐⭐⭐⭐ 省心线+MySQL兼容+OpenClaw API** | **≥80%** | **+¥200-600** |

**说明**: v2.1 整合原v2.1(运维)+v2.2(故障)+v3.0(MySQL兼容)，三个月完成三大目标

---

## v2.1.0 - AI基础版 (2026年4月)

**目标**: 完成核心架构重构，奠定AI向量化基础

### SQLRustGo 任务
- 核心业务逻辑层重构（含数据访问层）
- 性能优化：TPS 提升 30%
- 安全修复：OWASP ZAP 渗透测试
- 兼容性验证：旧接口过渡期 1个月

### AI员工 GMP审核系统任务
- 向量化接口开发：GPU向量化接口、CUDA集成
- OCR文本抽取：PDF/Word/Excel解析
- Embedding模型集成：bge-large本地部署
- 分词模块：jieba/哈工大词表
- ETL批量加载：COPY命令、批量事务

### 交付物
- SQLRustGo 核心重构完成
- 向量化基础模块
- 文档预处理模块

### AI员工里程碑
- [ ] 向量化接口完成
- [ ] OCR文本抽取完成
- [ ] Embedding生成能力验证

**资源**：安全团队 2人·周

---

## v2.2.0 - AI检索版 (2026年5月)

**目标**: 扩展系统能力，支持AI检索集成

### SQLRustGo 任务
- 多语言框架：中/英/日
- 第三方 API 集成：扩展生态
- 权限管理系统升级：RBAC

### AI员工 GMP审核系统任务
- 向量索引开发：HNSW/FAISS集成
- 全文索引：倒排索引/Bitmap
- OpenClaw集成：Agent框架接入
- Qwen3-14B部署：本地大模型推理
- Milvus部署：向量数据库
- Elasticsearch部署：全文检索引擎

### 交付物
- 多语言支持（中文/英文/日文）
- RBAC权限管理
- 向量检索能力
- 全文检索能力
- OpenClaw Agent总控
- Qwen3-14B推理服务

### AI员工里程碑
- [ ] ≈80%审核准确率原型
- [ ] 向量索引HNSW/FAISS集成
- [ ] OpenClaw Agent框架接入

**依赖**：v2.1.0 完成

---

## v2.3.0 - AI审核核心版 (2026年6月)

**目标**: 引入智能化能力，GMP审核核心功能上线

### SQLRustGo 任务
- 智能助手对话引擎：核心交互能力
- 机器学习模型引入：推荐/分类场景
- 数据可视化增强：图表/报表

### AI员工 GMP审核系统任务
- 领域Embedding微调：LoRA微调GMP语料
- 知识图谱构建：Neo4j条款关联
- 规则引擎开发：Drools/自定义规则
- 审计系统：查询日志/报表生成
- 大模型精炼分析：Qwen3-14B条款匹配
- 混合检索开发：向量+全文+规则融合
- 置信度排序：低置信人工复核

### 交付物
- 智能助手对话引擎
- GMP审核核心能力
- 领域微调Embedding
- 知识图谱
- 规则引擎
- 审计报表系统

### AI员工里程碑
- [ ] ≈90%审核准确率
- [ ] 领域微调Embedding完成
- [ ] 知识图谱Neo4j上线

**依赖**：v2.2.0 完成

---

## v2.4.0 - AI精炼优化版 (2026年7月)

**目标**: 扩展用户触达，平台完善，精炼优化

### SQLRustGo 任务
- 移动端适配：iOS / Android
- 实时协作功能：多人同时操作
- 安全合规审计：等保 / GDPR

### AI员工 GMP审核系统任务
- RAG增强检索：检索增强生成
- 性能优化-内存：LRU/LFU缓存
- 性能优化-GPU：CUDA Stream调度
- 性能优化-并发：40核心并行
- 高精度OCR：PaddleOCR/Nougat
- 药品术语库：专业词库构建
- 结果融合算法：向量+规则+大模型融合

### 交付物
- 移动端适配
- 实时协作
- 安全合规
- GMP审核增强
- 性能大幅提升

### AI员工里程碑
- [ ] ≈92-95%审核准确率
- [ ] RAG增强完成
- [ ] 性能优化完成

**依赖**：v2.3.0 完成

---

## v2.5.0 - AI交付版 (2026年8月)

**目标**: 优化交付，准备商业化，95%+准确率验证

### SQLRustGo 任务
- 全面性能优化：速度/资源占用
- 用户体验提升：UI/UX 改进
- 商业化部署准备：许可证/计费系统

### AI员工 GMP审核系统任务
- 准确率验证：95%+目标达成
- 压力测试：GB级文档性能
- 系统集成测试：端到端验证
- 用户手册：API文档/用户手册
- 运维监控：Prometheus/Grafana
- 知识图谱完善：实体关系补充
- 规则库完善：新增GMP条款

### 交付物
- SQLRustGo 2.5.0
- 全面性能优化
- UI/UX改进
- 商业化部署包
- AI员工1.0
- 95%+准确率
- GB级文档支持

### AI员工里程碑
- [ ] 95%+审核准确率验证
- [ ] GB级文档压力测试通过
- [ ] AI员工1.0正式发布

**依赖**：v2.4.0 完成

---

## AI员工GMP审核系统 - 版本对照表

| 版本 | 时间 | SQLRustGo任务 | AI员工GMP审核任务 | 准确率目标 |
|------|------|---------------|------------------|------------|
| **v2.1.0** | 4月 | 核心重构、性能优化、安全修复 | 向量化接口、OCR抽取、Embedding集成 | - |
| **v2.2.0** | 5月 | 多语言、API集成、RBAC | 向量索引、ES全文检索、OpenClaw、Qwen3-14B | ≈80% |
| **v2.3.0** | 6月 | 智能助手、ML模型、可视化 | 微调Embedding，知识图谱、规则引擎、审计 | ≈90% |
| **v2.4.0** | 7月 | 移动端、协作、安全合规 | RAG增强、性能优化、高精度OCR | ≈92-95% |
| **v2.5.0** | 8月 | 性能优化、UX、商业化 | 95%+验证、压力测试、交付 | ≥95% |

---

## 硬件需求（AI员工系统）

| 版本 | GPU | 内存 | CPU | 存储 |
|------|-----|------|-----|------|
| v2.1.0 | RTX 5080 | 416GB | 40核 | 2TB NVMe |
| v2.2.0 | RTX 5080 + M6000 | 512GB | 40核 | 4TB NVMe |
| v2.3.0 | RTX 5080 + M6000 | 512GB | 40核 | 8TB NVMe |
| v2.4.0 | RTX 5080 + M6000 | 512GB | 40核 | 8TB NVMe |
| v2.5.0 | RTX 5080 + M6000 | 512GB | 40核 | 8TB NVMe |

**结论**：现有硬件完全满足需求，无需额外投入

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [AI员工1.0自研开发讨论稿](./AI_EMPLOYEE_V1_DEVELOPMENT_DISCUSSION.md) | AI员工GMP审核系统详细规划 |
| [企业AI员工系统架构](./ENTERPRISE_AI_AGENT_SYSTEM.md) | 系统架构设计 |

---

*文档更新: 2026-03-29*