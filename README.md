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
api.rs    quick and dirty gRPC API.
eth.rs    quick and dirty Ethereum JSON-RPC client.
main.rs   driver program. dependency injection.
```
