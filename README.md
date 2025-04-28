# Rust 区块链实现

这是一个用 Rust 实现的简单区块链系统，用于学习区块链的基本概念。

## 功能特点

- 基本的区块链结构
- 工作量证明 (PoW) 机制
- 持久化存储
- 命令行界面
- 实现交易和发送币


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

## 使用方法

### 构建项目

```bash
cargo build
```

### 运行命令

1. 创建区块链,并且创世块奖励给alice：
```bash
cargo run createblockchain Alice
```

2. 打印区块链：
```bash
cargo run printchain
```

3. 查看帮助：
```bash
cargo run help
```

4. 查询余额
```bash
cargo run getbalance Alice
```

5. 转账
```bash
cargo run send Alice Bob 10
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
