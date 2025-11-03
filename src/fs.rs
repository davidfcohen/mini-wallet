use std::{collections::HashMap, error, fmt, io, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use chrono::DateTime;
use tokio::{fs, sync::RwLock};
use tracing::{debug, info, instrument};

use crate::{
    core::{Address, Wallet},
    infra::{StoreError, WalletRecord, WalletStore},
};

#[derive(Debug)]
pub struct FsError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file system store error")
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
    #[instrument(fields(path = %path.as_ref()))]
    pub async fn open(path: impl AsRef<str>) -> Result<Self, FsError> {
        let path_str = path.as_ref();
        let path = PathBuf::from(path_str);

        let store = if !path.exists() {
            let wallets = Arc::new(RwLock::new(HashMap::new()));
            let store = Self { path, wallets };
            store.write().await?;
            info!("created wallet store");
            store
        } else {
            let bytes = fs::read(&path).await?;
            let config = bincode::config::standard();
            let (wallets, _) = bincode::decode_from_slice(&bytes, config)?;
            let wallets = Arc::new(RwLock::new(wallets));
            info!("opened wallet store");
            Self { path, wallets }
        };

        Ok(store)
    }

    #[instrument(skip(self), fields(path = %self.path.to_string_lossy()))]
    async fn write(&self) -> Result<(), FsError> {
        let wallet = self.wallets.read().await;

        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(&*wallet, config)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let len = bytes.len();
        fs::write(&self.path, bytes).await?;
        debug!("wrote {} bytes to wallet store", len);
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
    async fn find(&self, name: &str) -> Result<Option<WalletRecord>, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let maybe_record = fs_wallets.get(name).map(fs_to_record);
        Ok(maybe_record)
    }

    async fn all(&self) -> Result<HashMap<String, WalletRecord>, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let wallets = fs_wallets
            .iter()
            .map(|(name, record)| (name.to_owned(), fs_to_record(record)))
            .collect();
        Ok(wallets)
    }

    async fn exists(&self, name: &str) -> Result<bool, StoreError> {
        let fs_wallets = self.wallets.read().await;
        let found = fs_wallets.contains_key(name);
        Ok(found)
    }

    async fn save(&self, name: &str, record: &WalletRecord) -> Result<(), StoreError> {
        let mut fs_wallets = self.wallets.write().await;
        fs_wallets.insert(name.to_owned(), record_to_fs(record));
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
    last_update: i64,
}

fn fs_to_record(fs: &FsWallet) -> WalletRecord {
    let address = Address::new(fs.address);
    let mut wallet = Wallet::new(address);
    *wallet.balance_mut() = fs.balance;
    WalletRecord {
        wallet,
        last_update: DateTime::from_timestamp(fs.last_update, 0).unwrap_or_default(),
    }
}

fn record_to_fs(record: &WalletRecord) -> FsWallet {
    FsWallet {
        address: *record.wallet.address().inner(),
        balance: record.wallet.balance(),
        last_update: record.last_update.timestamp(),
    }
}
