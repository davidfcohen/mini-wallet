use std::{collections::HashMap, error, fmt};

use async_trait::async_trait;

use crate::core::Wallet;

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
    async fn all(&self) -> Result<HashMap<String, Wallet>, StoreError>;
    async fn exists(&self, name: &str) -> Result<bool, StoreError>;
    async fn save(&self, name: &str, wallet: &Wallet) -> Result<(), StoreError>;
    async fn delete(&self, name: &str) -> Result<(), StoreError>;
}

#[derive(Debug)]
pub struct ClientError(pub Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blockchain error")
    }
}

impl error::Error for ClientError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[async_trait]
pub trait WalletClient: Send + Sync + 'static {
    async fn balance(&self, address: &str) -> Result<f64, ClientError>;
}
