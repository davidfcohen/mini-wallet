use std::sync::Arc;

use async_trait::async_trait;

use super::{Result, WalletError, WalletErrorKind};
use crate::infra::WalletStore;

#[async_trait]
pub trait Untrack: Send + Sync + 'static {
    async fn execute(&self, name: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct UntrackExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
}

#[async_trait]
impl Untrack for UntrackExecutor {
    async fn execute(&self, name: &str) -> Result<()> {
        if !self.wallet_store.exists(name).await? {
            return Err(WalletError {
                kind: WalletErrorKind::NotFound,
                source: None,
            });
        }

        self.wallet_store.delete(name).await?;
        Ok(())
    }
}
