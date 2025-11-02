use async_trait::async_trait;
use std::{any::type_name, fmt, sync::Arc};

use crate::infra::WalletStore;

use super::{Result, Wallet};

#[cfg_attr(test, mockall::automock)]
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
                balance: wallet.balance() as f64 / 1e18,
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr, sync::Arc};

    use crate::{
        core::{Address, Wallet},
        infra::MockWalletStore,
        wallet::{List, ListExecutor},
    };

    #[tokio::test]
    async fn wallet_list_success() {
        let mut wallet_store = MockWalletStore::new();
        wallet_store.expect_all().returning(|| {
            let mut wallets = HashMap::new();

            let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
            let address = Address::from_str(address).unwrap();
            let mut wallet = Wallet::new(address);
            *wallet.balance_mut() = 3756447340569860785;
            wallets.insert("Vitalik's Wallet".to_string(), wallet);

            let address = "0xB644Babc370f46f202DB5eaf2071A9Ee66fA1D5E";
            let address = Address::from_str(address).unwrap();
            let wallet = Wallet::new(address);
            wallets.insert("David's Wallet".to_string(), wallet);

            let address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
            let address = Address::from_str(address).unwrap();
            let mut wallet = Wallet::new(address);
            *wallet.balance_mut() = 2203446400537254477610554;
            wallets.insert("Wrapped Ether".to_string(), wallet);

            Ok(wallets)
        });

        let list = ListExecutor {
            wallet_store: Arc::new(wallet_store),
        };

        let wallets = list.execute().await.unwrap();
        assert_eq!(wallets[0].name, "David's Wallet");
        assert_eq!(wallets[1].name, "Vitalik's Wallet");
        assert_eq!(wallets[2].name, "Wrapped Ether");

        const T: f64 = 1e-9;
        assert!((wallets[0].balance - 0.0).abs() < T);
        assert!((wallets[1].balance - 3.756447340569860785).abs() < T);
        assert!((wallets[2].balance - 2203446.400537254477610554).abs() < T);
    }
}
