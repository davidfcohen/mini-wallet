use std::{error, fmt, time::Duration};

use async_trait::async_trait;
use hex::FromHexError;
use reqwest::{Client, Error as ReqwestError};
use serde_json::json;
use tracing::{debug, instrument};

use crate::{
    core::Address,
    infra::{ClientError, WalletClient},
};

#[derive(Debug)]
pub struct RpcError(Box<dyn error::Error + Send + Sync + 'static>);

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC client error")
    }
}

impl error::Error for RpcError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}

impl From<ReqwestError> for RpcError {
    fn from(error: ReqwestError) -> Self {
        Self(error.into())
    }
}

impl From<FromHexError> for RpcError {
    fn from(error: FromHexError) -> Self {
        Self(error.into())
    }
}

#[derive(Debug, Clone)]
pub struct RpcWalletClient {
    client: Client,
    url: String,
}

impl RpcWalletClient {
    pub fn new(url: impl Into<String>) -> Result<Self, RpcError> {
        Ok(Self {
            client: Client::builder().timeout(Duration::from_secs(30)).build()?,
            url: url.into(),
        })
    }
}

#[async_trait]
impl WalletClient for RpcWalletClient {
    #[instrument(skip(self), fields(address = %address.to_string()))]
    async fn balance(&self, address: &Address) -> Result<u128, ClientError> {
        let address = address.to_string();

        debug!("calling wallet balance rpc");
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
            .map_err(RpcError::from)?;

        let body: serde_json::Value = response.json().await.map_err(RpcError::from)?;
        let balance = body["result"]
            .as_str()
            .and_then(|s| s.strip_prefix("0x"))
            .ok_or(RpcError("missing result field".into()))?;

        let wei = extract_wei(balance)?;
        debug!(wei = %wei, hex = %balance, "got wallet balance");

        Ok(wei)
    }
}

impl From<RpcError> for ClientError {
    fn from(error: RpcError) -> Self {
        ClientError(error.into())
    }
}

fn extract_wei(balance: &str) -> Result<u128, RpcError> {
    let balance = if balance.len().is_multiple_of(2) {
        balance.to_string()
    } else {
        format!("0{balance}")
    };

    let wei = hex::decode(&balance)
        .map_err(RpcError::from)?
        .iter()
        .fold(0, |acc, &byte| acc * 256 + byte as u128);

    Ok(wei)
}
