use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;

use super::{Result, WalletError, WalletErrorKind};
use crate::{
    infra::WalletStore,
    model::{Address, Wallet},
};

#[async_trait]
pub trait Track: Send + Sync + 'static {
    async fn execute(&self, name: &str, address: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct TrackExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
}

#[async_trait]
impl Track for TrackExecutor {
    async fn execute(&self, name: &str, address: &str) -> Result<()> {
        validate_name(name)?;

        if self.wallet_store.exists(name).await? {
            return Err(WalletError {
                kind: WalletErrorKind::NameConflict,
                source: None,
            });
        }

        let address = Address::from_str(address)?;
        let wallet = Wallet::new(address);

        self.wallet_store.save(name, &wallet).await?;
        Ok(())
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        Err(WalletError {
            kind: WalletErrorKind::NameEmpty,
            source: None,
        })
    } else if name.chars().count() > 30 {
        Err(WalletError {
            kind: WalletErrorKind::NameTooLong,
            source: None,
        })
    } else {
        Ok(())
    }
}
