# Mini Wallet
This is my playground for building around Ethereum using Rust.

**Breakdown**
```
core.rs   wallet model. parses address and checks correctness including checksum.
infra.rs  defines persistence and ethereum client functions.
wallet.rs business logic used to track wallet balances.
fs.rs     quick and dirty file system database.
api.rs    quick and dirty gRPC API.
main.rs   driver program. dependency injection.
```
