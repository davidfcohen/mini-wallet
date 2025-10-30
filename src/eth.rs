use std::{error, fmt, time::Duration};

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::infra::{ClientError, WalletClient};

#[derive(Debug)]
pub struct EthError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for EthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ethereum client error")
    }
}

impl error::Error for EthError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}

impl From<reqwest::Error> for EthError {
    fn from(error: reqwest::Error) -> Self {
        Self(error.into())
    }
}

#[derive(Debug, Clone)]
pub struct EthWalletClient {
    client: Client,
    url: String,
}

impl EthWalletClient {
    pub fn new(url: impl Into<String>) -> Result<Self, EthError> {
        Ok(Self {
            client: Client::builder().timeout(Duration::from_secs(30)).build()?,
            url: url.into(),
        })
    }
}

#[async_trait]
impl WalletClient for EthWalletClient {
    async fn balance(&self, address: &str) -> Result<f64, ClientError> {
        let response = self
            .client
            .post(&self.url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": [address, "latest"],
                "id": 1,
            }))
            .send()
            .await
            .map_err(|e| ClientError(e.into()))?;

        let body: serde_json::Value = response.json().await.map_err(|e| ClientError(e.into()))?;
        let balance = body["result"]
            .as_str()
            .and_then(|s| s.strip_prefix("0x"))
            .ok_or(ClientError("missing result field".into()))?;

        let wei = hex::decode(balance)
            .map_err(|e| ClientError(e.into()))?
            .iter()
            .fold(0u128, |acc, &byte| acc * 256 + byte as u128);

        let eth = wei as f64 / 1e18;
        Ok(eth)
    }
}
