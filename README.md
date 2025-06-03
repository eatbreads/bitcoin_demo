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
### 基本命令使用

1. 创建钱包：
```bash
cargo run createwallet
```
输出示例：`创建钱包地址: 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa`

2. 列出所有钱包地址：
```bash
cargo run listaddresses
```

3. 创建区块链（需要指定接收创世区块奖励的钱包地址）：
```bash
cargo run createblockchain <钱包地址>
```

4. 查看钱包余额：
```bash
cargo run getbalance <钱包地址>
```

5. 转账交易：
```bash
cargo run send <发送方地址> <接收方地址> <金额>
```
添加 `-m` 参数可以立即由发送方进行挖矿：
```bash
cargo run send <发送方地址> <接收方地址> <金额> -m
```

6. 打印整个区块链：
```bash
cargo run printchain
```

7. 重建 UTXO 集合：
```bash
cargo run reindex
```

8. 启动节点服务器：
```bash
cargo run startnode <端口号>
```

9. 启动挖矿节点：
```bash
cargo run startminer <端口号> <奖励接收地址>
```

### 使用示例

1. 创建两个钱包并记录地址：
```bash
cargo run createwallet
cargo run createwallet
```

2. 使用第一个地址创建区块链：
```bash
cargo run createblockchain <第一个钱包地址>
```

3. 查看两个钱包的余额：
```bash
cargo run getbalance <第一个钱包地址>
cargo run getbalance <第二个钱包地址>
```

4. 从第一个钱包向第二个钱包转账：
```bash
cargo run send <第一个钱包地址> <第二个钱包地址> 10 -m
```

5. 再次查看余额确认交易：
```bash
cargo run getbalance <第一个钱包地址>
cargo run getbalance <第二个钱包地址>
```

## 项目结构

- `src/block.rs`: 区块结构实现
- `src/blockchain.rs`: 区块链核心功能
- `src/cli.rs`: 命令行界面
- `src/main.rs`: 程序入口
- `src/transactions.rs`: 交易相关
## 数据存储

区块链数据存储在 `data/blockchain` 目录下，使用 sled 嵌入式数据库。

## 注意事项

- 确保运行环境已安装 Rust 和 Cargo
- 所有数据会持久化保存
