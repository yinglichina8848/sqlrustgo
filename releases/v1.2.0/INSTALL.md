# SQLRustGo v1.2.0 安装与测试指南

## 版本信息

- **版本**: v1.2.0
- **发布日期**: 2026-03-13
- **平台**: macOS ARM64 (Apple Silicon)

## 系统要求

- **操作系统**: macOS 10.15+ / Linux / Windows (WSL)
- **架构**: x86_64 或 ARM64
- **依赖**: 无 (静态链接二进制)

## 安装方式

### 方式一：直接安装预编译二进制

```bash
# 下载并解压
curl -L -o sqlrustgo-v1.2.0-darwin-arm64.tar.gz \
  https://github.com/minzuuniversity/sqlrustgo/releases/download/v1.2.0/sqlrustgo-v1.2.0-darwin-arm64.tar.gz

tar -xzf sqlrustgo-v1.2.0-darwin-arm64.tar.gz
cd v1.2.0

# 安装
sudo cp sqlrustgo /usr/local/bin/
sudo chmod +x /usr/local/bin/sqlrustgo

# 验证
sqlrustgo --version
```

### 方式二：使用安装脚本

```bash
# 方式 A：二进制安装
curl -L -o install.sh \
  https://raw.githubusercontent.com/minzuuniversity/sqlrustgo/v1.2.0/releases/v1.2.0/install.sh
chmod +x install.sh
./install.sh --binary

# 方式 B：源码编译安装
./install.sh --source

# 方式 C：Cargo 安装
./install.sh --cargo
```

### 方式三：Cargo 安装

```bash
cargo install sqlrustgo --locked
```

### 方式四：Docker

```bash
docker run -it minzuuniversity/sqlrustgo:v1.2.0
```

## 验证安装

```bash
# 检查版本
sqlrustgo --version

# 输出应为:
# SQLRustGo Database System initialized
# SQLRustGo v1.2.0
```

## 快速开始

### 启动交互式 REPL

```bash
sqlrustgo
```

### 执行 SQL 文件

```bash
sqlrustgo < schema.sql
```

### 快速命令参考

| 命令 | 说明 |
|------|------|
| `sqlrustgo` | 启动 REPL |
| `sqlrustgo --version` | 显示版本 |
| `sqlrustgo --help` | 显示帮助 |

## 测试验证

### 基本功能测试

```bash
# 创建测试数据库
echo "CREATE TABLE test (id INTEGER, name TEXT)" | sqlrustgo

# 插入数据
echo "INSERT INTO test VALUES (1, 'hello')" | sqlrustgo

# 查询数据
echo "SELECT * FROM test" | sqlrustgo

# 预期输出应包含: 1 | hello
```

### 运行内置测试

```bash
# 编译测试
cargo test --workspace

# 覆盖率测试
cargo tarpaulin --workspace --all-features
```

### 性能基准测试

```bash
# 运行基准测试
cargo bench
```

## 已知限制

- 仅支持单节点部署
- 事务支持为预览版
- 网络复制功能开发中

## 获取帮助

- 文档: https://github.com/minzuuniversity/sqlrustgo/docs
- Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- 讨论: https://github.com/minzuuniversity/sqlrustgo/discussions

## 卸载

```bash
sudo rm /usr/local/bin/sqlrustgo
rm -rf ~/.sqlrustgo
```
