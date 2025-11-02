use std::{any::type_name, fmt, str::FromStr, sync::Arc};

use async_trait::async_trait;

use super::{Result, WalletError, WalletErrorKind};
use crate::{
    core::{Address, Wallet},
    infra::{WalletClient, WalletStore},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Track: Send + Sync + 'static {
    async fn execute(&self, name: &str, address: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct TrackExecutor {
    pub wallet_store: Arc<dyn WalletStore>,
    pub wallet_client: Arc<dyn WalletClient>,
}

impl fmt::Debug for TrackExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
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
        let mut wallet = Wallet::new(address);

        let balance = self.wallet_client.balance(&address).await?;
        *wallet.balance_mut() = balance;

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        infra::{MockWalletClient, MockWalletStore},
        wallet::{NAME_MAX, Track, TrackExecutor, WalletErrorKind},
    };

    const ADDR: &str = "0xB644Babc370f46f202DB5eaf2071A9Ee66fA1D5E";

    #[tokio::test]
    async fn wallet_track_success() {
        let mut wallet_store = MockWalletStore::new();
        wallet_store.expect_exists().returning(|_| Ok(false));
        wallet_store.expect_save().returning(|_, _| Ok(()));

        let mut wallet_client = MockWalletClient::new();
        wallet_client.expect_balance().returning(|_| Ok(0));

        let track = TrackExecutor {
            wallet_store: Arc::new(wallet_store),
            wallet_client: Arc::new(wallet_client),
        };

        assert!(track.execute("David's Wallet", ADDR).await.is_ok())
    }

    #[tokio::test]
    async fn wallet_track_name_empty() {
        let track = TrackExecutor {
            wallet_store: Arc::new(MockWalletStore::new()),
            wallet_client: Arc::new(MockWalletClient::new()),
        };

        let error = track.execute("", ADDR).await.unwrap_err();
        assert_eq!(error.kind(), WalletErrorKind::NameEmpty);
    }

    #[tokio::test]
    async fn wallet_track_name_too_long() {
        let track = TrackExecutor {
            wallet_store: Arc::new(MockWalletStore::new()),
            wallet_client: Arc::new(MockWalletClient::new()),
        };

        let error = track
            .execute(&"s".repeat(NAME_MAX + 1), ADDR)
            .await
            .unwrap_err();
        assert_eq!(error.kind(), WalletErrorKind::NameTooLong);
    }

    #[tokio::test]
    async fn wallet_track_name_conflict() {
        let mut wallet_store = MockWalletStore::new();
        wallet_store.expect_exists().returning(|_| Ok(true));

        let track = TrackExecutor {
            wallet_store: Arc::new(wallet_store),
            wallet_client: Arc::new(MockWalletClient::new()),
        };

        let error = track.execute("David's Wallet", ADDR).await.unwrap_err();
        assert_eq!(error.kind(), WalletErrorKind::NameConflict);
    }

    #[tokio::test]
    async fn wallet_track_parse_address() {
        let mut wallet_store = MockWalletStore::new();
        wallet_store.expect_exists().returning(|_| Ok(false));

        let track = TrackExecutor {
            wallet_store: Arc::new(wallet_store),
            wallet_client: Arc::new(MockWalletClient::new()),
        };

        let error = track
            .execute("David's Wallet", "not an address")
            .await
            .unwrap_err();
        assert_eq!(error.kind(), WalletErrorKind::WalletAddrParse);
    }
}
