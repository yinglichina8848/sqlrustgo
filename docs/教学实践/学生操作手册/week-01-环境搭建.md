# 第1周：开发环境搭建

> 实验时间：2学时
> 实验类型：验证性

---

## 一、实验目标

- [ ] 掌握Rust开发环境的安装和配置
- [ ] 掌握Git版本控制工具的配置
- [ ] 掌握TRAE IDE的安装和使用
- [ ] 能够克隆并运行SQLRustGo项目

---

## 二、实验环境

| 项目 | 要求 |
|------|------|
| 操作系统 | macOS / Linux / Windows 10+ |
| 网络环境 | 能够访问GitHub |
| 硬件要求 | 8GB+ 内存，20GB+ 硬盘空间 |

---

## 三、操作步骤

### 步骤1：安装Rust工具链（20分钟）

#### 1.1 下载并运行安装脚本

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

当提示选择安装选项时，输入 `1` 选择默认安装。

#### 1.2 配置环境变量

```bash
source $HOME/.cargo/env
```

#### 1.3 验证安装

```bash
rustc --version
cargo --version
```

**预期输出**:
```
rustc 1.xx.x (xxxxxxx 202x-xx-xx)
cargo 1.xx.x (xxxxxxx 202x-xx-xx)
```

#### 1.4 安装额外组件

```bash
rustup component add clippy rustfmt
```

#### ✅ 检查点1：截图保存
- 截图 `rustc --version` 输出
- 截图 `cargo --version` 输出

---

### 步骤2：配置Git和SSH（20分钟）

#### 2.1 配置Git用户信息

```bash
git config --global user.name "你的姓名"
git config --global user.email "你的邮箱@xxx.com"
```

#### 2.2 生成SSH密钥

```bash
ssh-keygen -t ed25519 -C "你的邮箱@xxx.com"
```

按三次回车使用默认设置。

#### 2.3 查看公钥

```bash
cat ~/.ssh/id_ed25519.pub
```

#### 2.4 添加SSH密钥到GitHub

1. 复制公钥内容
2. 访问 GitHub → Settings → SSH and GPG keys
3. 点击 "New SSH key"
4. 粘贴公钥内容，保存

#### 2.5 验证SSH连接

```bash
ssh -T git@github.com
```

**预期输出**:
```
Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

#### ✅ 检查点2：截图保存
- 截图 `git config --list` 输出
- 截图 SSH 连接成功信息

---

### 步骤3：安装TRAE IDE（15分钟）

#### 3.1 下载TRAE IDE

访问 TRAE 官网下载对应平台的安装包。

#### 3.2 安装并启动

- macOS: 拖拽到 Applications 文件夹
- Windows: 运行安装程序
- Linux: 解压并运行

#### 3.3 登录账号

启动 TRAE IDE，使用账号登录。

#### 3.4 配置Rust插件

1. 打开设置 (Settings)
2. 搜索 Rust
3. 确认 rust-analyzer 插件已安装

#### ✅ 检查点3：截图保存
- 截图 TRAE IDE 主界面

---

### 步骤4：克隆项目并运行（25分钟）

#### 4.1 克隆项目

```bash
git clone git@github.com:minzuuniversity/sqlrustgo.git
```

#### 4.2 进入项目目录

```bash
cd sqlrustgo
```

#### 4.3 构建项目

```bash
cargo build
```

**预期输出**:
```
   Compiling sqlrustgo v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in xx.xx s
```

#### 4.4 运行测试

```bash
cargo test
```

**预期输出**:
```
running xx tests
...
test result: ok. xx passed; 0 failed; 0 ignored
```

#### 4.5 运行代码检查

```bash
cargo clippy
```

#### 4.6 格式化代码

```bash
cargo fmt
```

#### ✅ 检查点4：截图保存
- 截图 `cargo build` 成功输出
- 截图 `cargo test` 通过输出

---

## 四、实验报告

### 4.1 报告内容

请将以下截图整理到实验报告中：

1. **环境信息截图**
   - Rust版本截图
   - Git配置截图
   - TRAE IDE界面截图

2. **项目运行截图**
   - cargo build 成功截图
   - cargo test 通过截图

3. **问题记录**
   - 遇到的问题
   - 解决方法

### 4.2 提交方式

```bash
# 创建实验报告分支
git checkout -b experiment/week-01-你的学号

# 创建报告文件
mkdir -p reports/week-01
# 将截图和报告放入该目录

# 提交
git add reports/week-01/
git commit -m "experiment: submit week-01 report"
git push origin experiment/week-01-你的学号
```

---

## 五、常见问题

### Q1: Rust安装失败

**解决方法**: 检查网络连接，或使用国内镜像源。

```bash
export RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup
```

### Q2: SSH连接失败

**解决方法**: 
1. 确认SSH密钥已添加到GitHub
2. 检查网络连接
3. 尝试使用HTTPS方式克隆

### Q3: cargo build失败

**解决方法**:
1. 检查Rust版本是否最新
2. 运行 `rustup update` 更新
3. 删除 `Cargo.lock` 后重试

---

## 六、评分标准

| 检查项 | 分值 |
|--------|------|
| Rust环境安装成功 | 20分 |
| Git和SSH配置正确 | 20分 |
| TRAE IDE安装成功 | 20分 |
| 项目构建成功 | 20分 |
| 测试全部通过 | 10分 |
| 实验报告完整 | 10分 |

---

*最后更新: 2026-03-05*
