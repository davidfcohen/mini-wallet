# Mini Wallet
This is my playground for building around Ethereum using Rust.

**Goals**
- Interact with the blockchain using JSON-RPC in some capacity.
- Demonstrate hexogonal architecture in Rust.
- Demonstrate how I build software and write code.

**Breakdown**
```
core.rs   wallet and address rules. parses and checks address including checksum.
infra.rs  defines wallet persistence and ethereum client interfaces.
wallet.rs business logic used to track wallet balances.
fs.rs     quick and dirty file system database.
api.rs    gRPC API used to interface with the application.
rpc.rs    lightweight Ethereum JSON-RPC client using websockets.
main.rs   driver program. policy and dependency injection.
```
