# SQLRustGo 网络层增强开发计划

> 版本：v1.0
> 日期：2026-03-02
> 目标：实现完整的 Client-Server 架构，支持跨系统通信

---

## 一、背景与目标

### 1.1 当前状态

| 组件 | 状态 | 说明 |
|------|------|------|
| MySQL 协议 | ✅ 已实现 | HandshakeV10, OK/Error Packet, Result Set |
| 同步服务器 | ⚠️ 基础 | `start_server_sync()` 单连接处理 |
| 客户端连接 | ⚠️ 基础 | `execute_query_on_server()` 简单查询 |
| 独立可执行程序 | ❌ 缺失 | 无 `bin/server.rs` 和 `bin/client.rs` |
| 异步处理 | ❌ 缺失 | 无异步服务器实现 |
| 连接池 | ❌ 缺失 | 无多连接管理 |

### 1.2 目标

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          目标架构                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────┐                      ┌──────────────────┐           │
│   │  sqlrustgo-client │◄──── TCP/MySQL ────►│  sqlrustgo-server │           │
│   │  (独立可执行程序)  │     Protocol        │  (独立可执行程序)  │           │
│   └──────────────────┘                      └──────────────────┘           │
│           │                                          │                      │
│           │                                          │                      │
│           ▼                                          ▼                      │
│   ┌──────────────────┐                      ┌──────────────────┐           │
│   │ • 命令行解析      │                      │ • 存储引擎        │           │
│   │ • 交互式 REPL    │                      │ • 执行引擎        │           │
│   │ • 结果格式化     │                      │ • 会话管理        │           │
│   │ • 脚本执行       │                      │ • 连接池          │           │
│   └──────────────────┘                      └──────────────────┘           │
│                                                                              │
│   支持场景:                                                                  │
│   ├── 本地通信: localhost:3306                                              │
│   ├── 局域网: 192.168.x.x:3306                                              │
│   └── 跨网络: 通过防火墙/NAT                                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、开发阶段规划

### Phase 1: 基础 Client-Server (1周)

**目标**: 实现可工作的独立客户端和服务器程序

#### 2.1.1 Server 可执行程序

**文件**: `src/bin/server.rs`

```rust
//! SQLRustGo 独立服务器程序
//! 
//! Usage:
//!   sqlrustgo-server [OPTIONS]
//! 
//! Options:
//!   -a, --addr <ADDR>        监听地址 [default: 0.0.0.0:3306]
//!   -d, --data-dir <DIR>     数据目录 [default: ./data]
//!   -c, --config <FILE>      配置文件路径
//!   -v, --verbose            详细日志
//!   -h, --help               帮助信息

use sqlrustgo::{StorageEngine, start_server_async};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = parse_args();
    
    // 初始化日志
    init_logging(args.verbose);
    
    // 加载配置
    let config = load_config(&args)?;
    
    // 初始化存储引擎
    let storage = StorageEngine::new(&config.data_dir)?;
    
    // 启动服务器
    println!("SQLRustGo Server starting on {}", args.addr);
    start_server_async(&args.addr, storage).await?;
    
    Ok(())
}
```

**功能清单**:

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 命令行参数解析 | addr, data-dir, config, verbose | P0 |
| 配置文件加载 | TOML 格式配置文件 | P1 |
| 存储引擎初始化 | 数据目录创建和初始化 | P0 |
| 信号处理 | SIGINT/SIGTERM 优雅关闭 | P0 |
| 日志输出 | 访问日志、错误日志 | P1 |

#### 2.1.2 Client 可执行程序

**文件**: `src/bin/client.rs`

```rust
//! SQLRustGo 独立客户端程序
//! 
//! Usage:
//!   sqlrustgo-client [OPTIONS] [QUERY]
//! 
//! Options:
//!   -H, --host <HOST>        服务器地址 [default: 127.0.0.1]
//!   -P, --port <PORT>        服务器端口 [default: 3306]
//!   -u, --user <USER>        用户名 [default: root]
//!   -p, --password <PASS>    密码
//!   -D, --database <DB>      数据库名
//!   -i, --interactive        交互模式
//!   -f, --file <FILE>        执行SQL文件
//!   -h, --help               帮助信息

use sqlrustgo::SqlRustGoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_args();
    
    // 连接服务器
    let mut client = SqlRustGoClient::connect(&args.host, args.port).await?;
    
    if args.interactive {
        // 交互模式
        run_interactive_mode(&mut client).await?;
    } else if let Some(file) = &args.file {
        // 执行文件
        execute_file(&mut client, file).await?;
    } else if let Some(query) = &args.query {
        // 执行单条查询
        let result = client.execute(query).await?;
        println!("{}", result);
    }
    
    Ok(())
}
```

**功能清单**:

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 命令行参数解析 | host, port, user, password, database | P0 |
| 单次查询执行 | 执行一条SQL并输出结果 | P0 |
| 交互模式 (REPL) | 持续读取和执行SQL | P1 |
| SQL文件执行 | 执行文件中的多条SQL | P1 |
| 结果格式化 | 表格/JSON/CSV格式输出 | P1 |

#### 2.1.3 服务器-执行器集成

**文件**: `src/network/server.rs`

```rust
//! 异步服务器实现

use crate::{ExecutionEngine, StorageEngine, SqlError};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// 服务器配置
pub struct ServerConfig {
    pub addr: String,
    pub max_connections: usize,
    pub timeout_ms: u64,
}

/// 异步服务器
pub struct AsyncServer {
    config: ServerConfig,
    storage: Arc<StorageEngine>,
    executor: Arc<ExecutionEngine>,
    connections: Arc<AtomicUsize>,
}

impl AsyncServer {
    pub async fn start(&self) -> Result<(), SqlError> {
        let listener = TcpListener::bind(&self.config.addr).await?;
        
        loop {
            let (stream, addr) = listener.accept().await?;
            
            // 检查连接数限制
            if self.connections.load(Ordering::Relaxed) >= self.config.max_connections {
                continue;
            }
            
            // 处理连接
            let server = self.clone();
            tokio::spawn(async move {
                server.handle_connection(stream).await;
            });
        }
    }
    
    async fn handle_connection(&self, stream: TcpStream) {
        // MySQL 握手
        // 读取查询
        // 执行查询
        // 返回结果
    }
}
```

---

### Phase 2: 功能完善 (1-2周)

#### 2.2.1 异步服务器改造

**目标**: 使用 Tokio 实现高性能异步服务器

```rust
/// 异步连接处理
pub struct ConnectionHandler {
    stream: TcpStream,
    executor: Arc<dyn ExecutionEngine>,
    session: Session,
}

impl ConnectionHandler {
    pub async fn handle(&mut self) -> Result<(), SqlError> {
        // 发送握手包
        self.send_handshake().await?;
        
        // 读取认证包
        self.read_auth().await?;
        
        // 命令循环
        loop {
            let command = self.read_command().await?;
            
            match command {
                Command::Query(sql) => {
                    let result = self.executor.execute(&sql)?;
                    self.send_result(result).await?;
                }
                Command::Quit => break,
                Command::Ping => {
                    self.send_ok("PONG").await?;
                }
                _ => {
                    self.send_error(1047, "Command not supported").await?;
                }
            }
        }
        
        Ok(())
    }
}
```

#### 2.2.2 连接池实现

```rust
/// 连接池配置
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

/// 连接池
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Vec<Arc<Connection>>,
    available: mpsc::Sender<Arc<Connection>>,
}

impl ConnectionPool {
    pub async fn get(&self) -> Result<PooledConnection, SqlError> {
        // 从池中获取连接
    }
    
    pub async fn put(&self, conn: Arc<Connection>) {
        // 归还连接到池
    }
}
```

#### 2.2.3 会话管理

```rust
/// 会话状态
pub struct Session {
    pub id: u32,
    pub user: String,
    pub database: Option<String>,
    pub created_at: Instant,
    pub last_activity: Instant,
}

/// 会话管理器
pub struct SessionManager {
    sessions: HashMap<u32, Session>,
    next_id: AtomicU32,
}

impl SessionManager {
    pub fn create(&mut self, user: String) -> Session {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let session = Session::new(id, user);
        self.sessions.insert(id, session.clone());
        session
    }
    
    pub fn get(&self, id: u32) -> Option<&Session> {
        self.sessions.get(&id)
    }
    
    pub fn remove(&mut self, id: u32) {
        self.sessions.remove(&id);
    }
}
```

---

### Phase 3: 生产就绪 (2-3周)

#### 2.3.1 认证机制

```rust
/// 认证配置
pub struct AuthConfig {
    pub enabled: bool,
    pub users: HashMap<String, UserInfo>,
}

/// 用户信息
pub struct UserInfo {
    pub username: String,
    pub password_hash: String,
    pub databases: Vec<String>,
    pub permissions: PermissionSet,
}

/// 认证处理器
pub struct Authenticator {
    config: AuthConfig,
}

impl Authenticator {
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        // 验证用户名密码
        // 检查权限
        // 创建会话
    }
}
```

#### 2.3.2 SSL/TLS 支持

```rust
/// TLS 配置
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

/// TLS 接受器
pub struct TlsAcceptor {
    acceptor: SslAcceptor,
}

impl TlsAcceptor {
    pub async fn accept(&self, stream: TcpStream) -> Result<SslStream<TcpStream>, SslError> {
        // SSL 握手
    }
}
```

#### 2.3.3 配置文件

```toml
# config/server.toml

[server]
addr = "0.0.0.0:3306"
max_connections = 100
timeout_ms = 30000

[storage]
data_dir = "./data"
page_size = 8192
buffer_pool_size = 1000

[auth]
enabled = true
default_user = "root"

[log]
level = "info"
access_log = "./logs/access.log"
error_log = "./logs/error.log"

[tls]
enabled = false
cert_path = "./certs/server.crt"
key_path = "./certs/server.key"
```

---

## 三、文件结构规划

```
src/
├── bin/
│   ├── server.rs           # 独立服务器程序
│   └── client.rs           # 独立客户端程序
│
├── network/
│   ├── mod.rs              # 模块导出
│   ├── server.rs           # 异步服务器实现
│   ├── client.rs           # 客户端实现
│   ├── connection.rs       # 连接处理
│   ├── session.rs          # 会话管理
│   ├── pool.rs             # 连接池
│   ├── auth.rs             # 认证模块
│   ├── tls.rs              # TLS 支持
│   └── protocol/
│       ├── mod.rs          # 协议模块
│       ├── handshake.rs    # 握手协议
│       ├── packet.rs       # 数据包
│       └── result_set.rs   # 结果集
│
└── config/
    ├── mod.rs              # 配置模块
    └── server.rs           # 服务器配置
```

---

## 四、任务分解

### 4.1 Phase 1 任务 (1周)

| ID | 任务 | 负责人 | 预估时间 | 依赖 |
|----|------|--------|----------|------|
| N-001 | 创建 `src/bin/server.rs` 基础框架 | AI-CLI | 2h | - |
| N-002 | 实现命令行参数解析 | AI-CLI | 2h | N-001 |
| N-003 | 集成存储引擎初始化 | AI-CLI | 2h | N-001 |
| N-004 | 实现信号处理和优雅关闭 | AI-CLI | 2h | N-001 |
| N-005 | 创建 `src/bin/client.rs` 基础框架 | AI-CLI | 2h | - |
| N-006 | 实现客户端命令行参数 | AI-CLI | 2h | N-005 |
| N-007 | 实现单次查询执行 | AI-CLI | 3h | N-005 |
| N-008 | 服务器-执行器集成 | AI-CLI | 4h | N-001 |
| N-009 | 结果集序列化完善 | AI-CLI | 3h | N-008 |
| N-010 | 本地通信测试 | AI-CLI | 2h | N-009 |

### 4.2 Phase 2 任务 (1-2周)

| ID | 任务 | 负责人 | 预估时间 | 依赖 |
|----|------|--------|----------|------|
| N-011 | 异步服务器实现 | AI-CLI | 4h | N-010 |
| N-012 | 连接池实现 | AI-CLI | 4h | N-011 |
| N-013 | 会话管理实现 | AI-CLI | 3h | N-011 |
| N-014 | 交互模式 (REPL) | AI-CLI | 3h | N-007 |
| N-015 | SQL文件执行 | AI-CLI | 2h | N-014 |
| N-016 | 结果格式化 (表格/JSON/CSV) | AI-CLI | 3h | N-014 |
| N-017 | 配置文件支持 | AI-CLI | 2h | N-001 |
| N-018 | 多连接测试 | AI-CLI | 2h | N-012 |

### 4.3 Phase 3 任务 (2-3周)

| ID | 任务 | 负责人 | 预估时间 | 依赖 |
|----|------|--------|----------|------|
| N-019 | 认证机制实现 | AI-CLI | 4h | N-013 |
| N-020 | 用户权限管理 | AI-CLI | 3h | N-019 |
| N-021 | SSL/TLS 支持 | AI-CLI | 4h | N-011 |
| N-022 | 连接超时和重连 | AI-CLI | 2h | N-012 |
| N-023 | 访问日志和查询日志 | AI-CLI | 2h | N-011 |
| N-024 | 性能测试和优化 | AI-CLI | 4h | N-023 |
| N-025 | 文档编写 | AI-CLI | 3h | N-024 |

---

## 五、测试计划

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_server_startup() { /* ... */ }
    
    #[test]
    fn test_client_connect() { /* ... */ }
    
    #[test]
    fn test_query_execution() { /* ... */ }
    
    #[test]
    fn test_result_serialization() { /* ... */ }
}
```

### 5.2 集成测试

```rust
#[tokio::test]
async fn test_client_server_communication() {
    // 启动服务器
    let server = start_test_server().await;
    
    // 客户端连接
    let mut client = SqlRustGoClient::connect("127.0.0.1", server.port).await?;
    
    // 执行查询
    let result = client.execute("SELECT 1").await?;
    assert!(result.contains("1"));
    
    // 关闭
    server.shutdown().await;
}
```

### 5.3 性能测试

```rust
#[bench]
fn bench_query_throughput(b: &mut Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut client = rt.block_on(SqlRustGoClient::connect("127.0.0.1", 3306)).unwrap();
    
    b.iter(|| {
        rt.block_on(client.execute("SELECT 1")).unwrap()
    });
}
```

---

## 六、验收标准

### 6.1 Phase 1 验收

- [ ] `sqlrustgo-server` 可独立启动
- [ ] `sqlrustgo-client` 可连接服务器
- [ ] 支持基本 SQL 查询执行
- [ ] 结果正确返回和显示
- [ ] 本地通信测试通过

### 6.2 Phase 2 验收

- [ ] 异步服务器稳定运行
- [ ] 支持多客户端同时连接
- [ ] 交互模式正常工作
- [ ] 配置文件正确加载

### 6.3 Phase 3 验收

- [ ] 认证机制正常工作
- [ ] SSL/TLS 连接成功
- [ ] 性能达标 (1000+ QPS)
- [ ] 文档完整

---

## 七、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 异步改造复杂 | 高 | 分阶段实施，先同步后异步 |
| 性能不达标 | 中 | 基准测试，逐步优化 |
| 协议兼容性 | 中 | 参考 MySQL 协议文档 |
| 资源泄漏 | 高 | 完善测试，使用 RAII |

---

## 八、AI-CLI 协同开发指南

### 8.1 任务分配原则

```
AI-CLI 角色:
├── 实现者: 根据设计文档编写代码
├── 测试者: 编写单元测试和集成测试
├── 审查者: 代码审查和优化建议
└── 文档者: 编写和更新文档
```

### 8.2 开发流程

```
1. 创建任务分支
   git checkout -b feature/network-enhancement

2. AI-CLI 实现任务
   - 按照 TASK_ID 顺序执行
   - 每个任务完成后提交

3. 测试验证
   - 运行单元测试
   - 运行集成测试
   - 性能基准测试

4. 代码审查
   - AI-CLI 自我审查
   - 生成审查报告

5. 合并主分支
   - 创建 PR
   - 审核通过后合并
```

### 8.3 提交规范

```
feat(network): add async server implementation

- Implement AsyncServer with Tokio
- Add connection handling
- Support multiple concurrent connections

Task: N-011
```

---

*本文档由 TRAE (GLM-5.0) 创建*
