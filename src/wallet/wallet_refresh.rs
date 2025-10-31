use std::{any::type_name, fmt, sync::Arc};

use async_trait::async_trait;
use futures::future::try_join_all;

use crate::{
    core::Wallet,
    infra::{WalletClient, WalletStore},
};

use super::Result;

#[async_trait]
pub trait Refresh: Send + Sync + 'static {
    async fn execute(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct RefreshExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
    pub wallet_client: Arc<dyn WalletClient>,
}

impl fmt::Debug for RefreshExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[async_trait]
impl Refresh for RefreshExecutor {
    async fn execute(&self) -> Result<()> {
        let wallets = self.wallet_store.all().await?;

        let futures: Vec<_> = wallets
            .iter()
            .map(|(name, wallet)| self.refresh_wallet(name, wallet))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }
}

impl RefreshExecutor {
    async fn refresh_wallet(&self, name: &str, wallet: &Wallet) -> Result<()> {
        let balance = self.wallet_client.balance(wallet.address()).await?;

        let mut wallet = wallet.clone();
        *wallet.balance_mut() = balance;

        self.wallet_store.save(name, &wallet).await?;
        Ok(())
    }
}
