use async_trait::async_trait;
use std::{any::type_name, fmt, sync::Arc};

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

impl fmt::Debug for ListExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[async_trait]
impl List for ListExecutor {
    async fn execute(&self) -> Result<Vec<Wallet>> {
        let mut wallets: Vec<Wallet> = self
            .wallet_store
            .all()
            .await?
            .into_iter()
            .map(|(name, wallet)| Wallet {
                name,
                address: wallet.address().to_string(),
            })
            .collect();

        wallets.sort_by(|a, b| {
            let a = a.name.to_lowercase();
            let b = b.name.to_lowercase();
            a.cmp(&b)
        });
        Ok(wallets)
    }
}
