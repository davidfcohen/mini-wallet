# Mini Wallet
This is my playground for building around Ethereum using Rust.

**Breakdown**
```
core.rs   wallet and address rules. parses address and checks correctness including checksum.
infra.rs  defines wallet persistence and ethereum client interfaces.
wallet.rs business logic used to track wallet balances.
fs.rs     quick and dirty file system database.
api.rs    quick and dirty gRPC API.
main.rs   driver program. dependency injection.
```
