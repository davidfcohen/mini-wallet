use std::sync::Arc;

use mini_wallet::{eth::EthWalletClient, fs::FsWalletStore};

#[derive(Debug, Clone)]
struct Dependencies {
    wallet_store: Arc<FsWalletStore>,
    wallet_client: Arc<EthWalletClient>,
}

#[tokio::main]
async fn main() {}

fn build_dependencies() -> Dependencies {
    todo!()
}
