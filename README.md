# Mini Wallet
This is my playground for building around Ethereum using Rust.

**Goals**
- Interact with the blockchain using JSON-RPC in some capacity.
- Demonstrate hexogonal architecture in Rust.
- Demonstrate how I build software and write code.

**Features**
- track wallets given a name and address
- verifies wallet address format and checksum
- store balances to disk and refresh periodically
- list tracked wallets (name, address, balance)
- untrack wallets

**Breakdown**
```
core.rs   wallet and address rules. parses and checks address including checksum.
fs.rs     quick and dirty file system database.
infra.rs  defines wallet persistence and ethereum client interfaces.
main.rs   driver program. policy and dependency injection.
rpc.rs    lightweight Ethereum JSON-RPC client.
server.rs gRPC API and balance refresh loop.
wallet.rs business logic for tracking wallet balances.
```
