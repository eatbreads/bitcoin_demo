# Rust 区块链实现

这是一个用 Rust 实现的简单区块链系统，用于学习区块链的基本概念 。

## 功能特点

- 基本的区块链结构
- 工作量证明 (PoW) 机制
- 持久化存储
- 命令行界面
- 实现交易和 发送币


## 开发阶段


1. part2: Proof-of-Work      `工作量证明`            
   [commit eb822486](https://github.com/eatbreads/bitcoin_demo/tree/eb822486)
   - 实现工作量证明算法
   - 添加挖矿功能
   - 区块哈希计算

2. part3: Persistence and CLI `持久化和命令行`  
   [commit 8880dd2](https://github.com/eatbreads/bitcoin_demo/tree/8880dd2)
   - 使用 sled 实现数据持久化
   - 添加命令行交互界面
   - 实现区块链遍历功能
3. part3: Persistence and CLI `加入交易和发送币`  
   [commit 1876783](https://github.com/eatbreads/bitcoin_demo/tree/1876783)
   - 使用 sled 实现数据持久化
   - 添加命令行交互界面
   - 实现区块链遍历功能

4. part4: 钱包系统 `钱包系统`
   [commit abb226b](https://github.com/eatbreads/bitcoin_demo/tree/abb226b)
   - 实现钱包系统
   - 实现交易和发送币
   - 实现钱包余额查询
   - 实现钱包地址生成

   
5. part5: 交易改进 `交易系统优化`
   [commit 79f38cb](https://github.com/eatbreads/bitcoin_demo/tree/79f38cb0eebb4f6fc561e7fb0eecfcdbd875e1ef)
   - 改进交易验证机制
   - 优化 UTXO 管理
   - 增强交易安全性
   - 完善交易签名系统
6. part6: 分布式网络 `网络功能实现`
   [commit 0e356fd](https://github.com/eatbreads/bitcoin_demo/tree/0e356fd)
   - 实现节点间通信
   - 添加区块同步功能
   - 实现交易广播
   - 支持多节点挖矿

## 使用方法

### 构建项目

```bash
cargo build
```



## 命令行界面详细说明

### 钱包管理命令

1. 创建新钱包：
```bash
cargo run createwallet
blockchain.exe createwallet
```
创建一个新的钱包，返回钱包地址。

2. 列出所有钱包地址：
```bash
cargo run listaddresses
blockchain.exe listaddresses
```
显示当前系统中所有已创建的钱包地址。

3. 查询钱包余额：
```bash
cargo run getbalance <钱包地址>
blockchain.exe getbalance <钱包地址>
```
查询指定钱包地址的余额。

### 区块链管理命令

4. 创建新区块链：
```bash
cargo run createblockchain <钱包地址>
blockchain.exe createblockchain <钱包地址>
```
使用指定钱包地址创建新的区块链，该地址将获得创世区块奖励。

5. 打印区块链：
```bash
cargo run printchain
blockchain.exe printchain
```
打印整个区块链的详细信息，包括所有区块和交易。

6. 重建UTXO集合：
```bash
cargo run reindex
blockchain.exe reindex
```
重新构建UTXO（未花费交易输出）集合，用于数据修复或优化。

### 交易命令

7. 发送交易：
```bash
cargo run send <发送方地址> <接收方地址> <金额>
blockchain.exe send <发送方地址> <接收方地址> <金额>
```
在发送方和接收方之间创建交易。

8. 发送交易并立即挖矿：
```bash
cargo run send <发送方地址> <接收方地址> <金额> -m
blockchain.exe send <发送方地址> <接收方地址> <金额> -m
```
创建交易后立即进行挖矿，将交易打包到新区块中。

### 网络节点命令

9. 启动普通节点：
```bash
cargo run startnode <端口号>
blockchain.exe startnode <端口号>
```
启动一个普通的区块链节点，参与网络但不进行挖矿。

10. 启动挖矿节点：
```bash
cargo run startminer <端口号> <奖励接收地址>
blockchain.exe startminer <端口号> <奖励接收地址>
```
启动挖矿节点，自动处理交易并进行挖矿，挖矿奖励发送到指定地址。

## 网络功能

### 节点类型

**普通节点 (startnode)**
- 参与区块链网络同步
- 接收和转发交易
- 维护完整的区块链副本
- 不执行挖矿操作

**挖矿节点 (startminer)**
- 具备普通节点的所有功能
- 执行工作量证明挖矿
- 处理交易池中的待确认交易
- 获得挖矿奖励和交易手续费

### 网络通信

- 使用HTTP协议进行节点间通信
- 支持区块数据同步
- 实现交易广播机制
- 自动发现和连接网络节点

## 挖矿机制

### 工作量证明算法

- **挖矿难度**: 区块哈希前5位必须为0
- **哈希算法**: SHA-256
- **难度调整**: 固定难度（在实际应用中可动态调整）
- **Nonce机制**: 通过递增nonce值寻找有效哈希

### 挖矿奖励

- 每个成功挖出的区块包含coinbase交易
- 挖矿者获得固定的区块奖励
- 额外获得区块内所有交易的手续费

### 挖矿方式

1. **即时挖矿**: 使用`send`命令的`-m`参数
2. **节点挖矿**: 运行专门的挖矿节点
3. **自动挖矿**: 挖矿节点自动处理交易池

## 故障排除

### 常见错误及解决方案

**端口占用错误**
```
Error: Os { code: 10048, kind: AddrInUse }
```
解决方法：
- 检查端口占用：`netstat -ano | findstr :8080`
- 终止占用进程：`taskkill /PID <进程ID> /F`
- 使用其他端口启动节点

**钱包地址无效**
- 确保使用`createwallet`生成的有效地址
- 检查地址格式和长度

**余额不足**
- 使用`getbalance`检查发送方余额
- 确保账户有足够的币进行交易

**数据库问题**
- 删除`data`目录重新初始化
- 运行`reindex`命令重建数据

## 技术架构

### 核心模块

- **区块模块** (`block.rs`): 实现区块结构和工作量证明
- **区块链模块** (`blockchain.rs`): 管理区块链数据和验证逻辑  
- **交易模块** (`transaction.rs`): 实现UTXO模型和交易处理
- **钱包模块** (`wallets.rs`): 密钥管理和地址生成
- **网络模块** (`server.rs`): 节点通信和消息处理
- **UTXO模块** (`utxoset.rs`): 未花费输出管理
- **命令行模块** (`cli.rs`): 用户交互界面

### 数据持久化

- **存储引擎**: sled嵌入式数据库
- **数据目录**: `data/blockchain`
- **序列化**: 使用bincode进行数据序列化
- **键值存储**: 区块哈希作为键，区块数据作为值

### 密码学实现

- **数字签名**: Ed25519椭圆曲线算法
- **哈希函数**: SHA-256
- **地址生成**: 基于公钥哈希的Bitcoin地址格式
- **钱包安全**: 本地存储私钥，支持多钱包管理
```


