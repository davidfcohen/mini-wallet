use std::{collections::HashMap, error, fmt, io, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use tokio::{fs, sync::RwLock};

use crate::{
    core::{Address, Wallet},
    infra::{StoreError, WalletStore},
};

#[derive(Debug)]
pub struct FsError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file system database error")
    }
}

impl error::Error for FsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}

impl From<io::Error> for FsError {
    fn from(error: io::Error) -> Self {
        Self(error.into())
    }
}

impl From<DecodeError> for FsError {
    fn from(error: DecodeError) -> Self {
        Self(error.into())
    }
}

impl From<EncodeError> for FsError {
    fn from(error: EncodeError) -> Self {
        Self(error.into())
    }
}

#[derive(Debug, Clone)]
pub struct FsWalletStore {
    path: PathBuf,
    wallets: Arc<RwLock<HashMap<String, FsWallet>>>,
}

impl FsWalletStore {
    pub async fn open(path: impl AsRef<str>) -> Result<Self, FsError> {
        let path = PathBuf::from(path.as_ref());

        if !path.exists() {
            return Ok(Self {
                path,
                wallets: Arc::new(RwLock::new(HashMap::new())),
            });
        }

        let bytes = fs::read(&path).await?;
        let config = bincode::config::standard();
        let (wallets, _) = bincode::decode_from_slice(&bytes, config)?;

        Ok(Self {
            path,
            wallets: Arc::new(RwLock::new(wallets)),
        })
    }

    async fn write(&self) -> Result<(), FsError> {
        let wallet = self.wallets.read().await;

        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(&*wallet, config)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&self.path, bytes).await?;
        Ok(())
    }
}

impl From<FsError> for StoreError {
    fn from(error: FsError) -> Self {
        Self(error.into())
    }
}

#[async_trait]
impl WalletStore for FsWalletStore {
    async fn find(&self, name: &str) -> Result<Option<Wallet>, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let maybe_wallet = fs_wallets.get(name).map(fs_to_wallet);
        Ok(maybe_wallet)
    }

    async fn all(&self) -> Result<HashMap<String, Wallet>, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let wallets = fs_wallets
            .iter()
            .map(|(name, wallet)| (name.to_owned(), fs_to_wallet(wallet)))
            .collect();
        Ok(wallets)
    }

    async fn exists(&self, name: &str) -> Result<bool, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let found = fs_wallets.contains_key(name);
        Ok(found)
    }

    async fn save(&self, name: &str, wallet: &Wallet) -> Result<(), StoreError> {
        let mut fs_wallets = self.wallets.write().await;
        fs_wallets.insert(name.to_owned(), wallet_to_fs(wallet));
        drop(fs_wallets);

        self.write().await?;
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<(), StoreError> {
        let mut fs_wallets = self.wallets.write().await;
        fs_wallets.remove(name);
        drop(fs_wallets);

        self.write().await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Encode, Decode)]
struct FsWallet {
    address: [u8; 20],
    balance: u128,
}

fn fs_to_wallet(fs_wallet: &FsWallet) -> Wallet {
    let address = Address::new(fs_wallet.address);
    let mut wallet = Wallet::new(address);
    *wallet.balance_mut() = fs_wallet.balance;
    wallet
}

fn wallet_to_fs(wallet: &Wallet) -> FsWallet {
    FsWallet {
        address: *wallet.address().inner(),
        balance: wallet.balance(),
    }
}
