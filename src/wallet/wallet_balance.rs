use async_trait::async_trait;
use std::sync::Arc;

use crate::infra::{WalletClient, WalletStore};

use super::{Result, WalletError, WalletErrorKind};

#[async_trait]
pub trait Balance: Send + Sync + 'static {
    async fn execute(&self, name: &str) -> Result<f64>;
}

#[derive(Clone)]
pub struct BalanceExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
    pub wallet_client: Arc<dyn WalletClient>,
}

#[async_trait]
impl Balance for BalanceExecutor {
    async fn execute(&self, name: &str) -> Result<f64> {
        let Some(wallet) = self.wallet_store.find(name).await? else {
            return Err(WalletError {
                kind: WalletErrorKind::NotFound,
                source: None,
            });
        };

        let address = wallet.address().to_string();
        let balance = self.wallet_client.balance(&address).await?;
        Ok(balance)
    }
}
