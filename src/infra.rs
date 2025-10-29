use std::{collections::HashMap, error, fmt, result};

use async_trait::async_trait;

use crate::model::Wallet;

type Result<T> = result::Result<T, InfraError>;

#[derive(Debug)]
pub struct InfraError(pub Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "infrastructure error")
    }
}

impl error::Error for InfraError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[async_trait]
pub trait WalletStore: Send + Sync + 'static {
    async fn find(&self, name: &str) -> Result<Option<Wallet>>;
    async fn load(&self) -> Result<HashMap<String, Wallet>>;
    async fn exists(&self, name: &str) -> Result<bool>;
    async fn save(&self, name: &str, wallet: &Wallet) -> Result<()>;
    async fn delete(&self, name: &str) -> Result<()>;
}

#[async_trait]
pub trait WalletChain: Send + Sync + 'static {
    async fn balance(&self, address: &str) -> Result<f64>;
}
