use std::{collections::HashMap, error, fmt};

use async_trait::async_trait;

use crate::model::Wallet;

#[derive(Debug)]
pub struct StoreError(pub Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "store error")
    }
}

impl error::Error for StoreError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[async_trait]
pub trait WalletStore: Send + Sync + 'static {
    async fn find(&self, name: &str) -> Result<Option<Wallet>, StoreError>;
    async fn load(&self) -> Result<HashMap<String, Wallet>, StoreError>;
    async fn exists(&self, name: &str) -> Result<bool, StoreError>;
    async fn save(&self, name: &str, wallet: &Wallet) -> Result<(), StoreError>;
    async fn delete(&self, name: &str) -> Result<(), StoreError>;
}

#[derive(Debug)]
pub struct ChainError(pub Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blockchain error")
    }
}

impl error::Error for ChainError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[async_trait]
pub trait WalletChain: Send + Sync + 'static {
    async fn balance(&self, address: &str) -> Result<f64, ChainError>;
}
