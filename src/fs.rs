use std::{collections::HashMap, error, fmt, io, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use tokio::{fs, sync::RwLock};

use crate::{
    infra::{InfraError, WalletStore},
    model::{Address, Wallet},
};

#[derive(Debug)]
pub struct FsError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file system error")
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

#[derive(Debug, Clone, Encode, Decode)]
pub struct FsWallet {
    address: [u8; 20],
}

impl FsWalletStore {
    pub async fn new(path: impl AsRef<str>) -> Result<Self, FsError> {
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

    async fn persist(&self) -> Result<(), FsError> {
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

#[async_trait]
impl WalletStore for FsWalletStore {
    async fn find(&self, name: &str) -> Result<Option<Wallet>, InfraError> {
        let fs_wallets = self.wallets.read().await;
        let maybe_wallet = fs_wallets.get(name).map(fs_to_wallet);
        Ok(maybe_wallet)
    }
    async fn load(&self) -> Result<HashMap<String, Wallet>, InfraError> {
        let fs_wallets = self.wallets.read().await;
        let wallets = fs_wallets
            .iter()
            .map(|(name, wallet)| (name.to_owned(), fs_to_wallet(wallet)))
            .collect();
        Ok(wallets)
    }

    async fn exists(&self, name: &str) -> Result<bool, InfraError> {
        let fs_wallets = self.wallets.read().await;
        let found = fs_wallets.contains_key(name);
        Ok(found)
    }

    async fn save(&self, name: &str, wallet: &Wallet) -> Result<(), InfraError> {
        let mut fs_wallets = self.wallets.write().await;
        fs_wallets.insert(name.to_owned(), wallet_to_fs(wallet));
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<(), InfraError> {
        let mut fs_wallets = self.wallets.write().await;
        fs_wallets.remove(name);
        drop(fs_wallets);

        self.persist().await.map_err(|e| InfraError(e.into()))?;
        Ok(())
    }
}

fn fs_to_wallet(fs_wallet: &FsWallet) -> Wallet {
    let address = Address::new(fs_wallet.address);
    Wallet::new(address)
}

fn wallet_to_fs(wallet: &Wallet) -> FsWallet {
    let address = wallet.address().inner().to_owned();
    FsWallet { address }
}
