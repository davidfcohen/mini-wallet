use async_trait::async_trait;
use std::sync::Arc;

use crate::infra::WalletStore;

use super::{Result, Wallet};

#[async_trait]
pub trait List: Send + Sync + 'static {
    async fn execute(&self) -> Result<Vec<Wallet>>;
}

#[derive(Clone)]
pub struct ListExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
}

#[async_trait]
impl List for ListExecutor {
    async fn execute(&self) -> Result<Vec<Wallet>> {
        let wallets = self
            .wallet_store
            .load()
            .await?
            .into_iter()
            .map(|(name, wallet)| Wallet {
                name,
                address: wallet.address().to_string(),
            })
            .collect();

        Ok(wallets)
    }
}
