mod wallet_list;
mod wallet_refresh;
mod wallet_track;
mod wallet_untrack;

use std::{error, fmt, result};

use chrono::{DateTime, Utc};

use crate::{
    core::AddrParseError,
    infra::{ClientError, StoreError},
};

const NAME_MAX: usize = 30;

pub type Result<T> = result::Result<T, WalletError>;

pub use wallet_list::{List, ListExecutor};
pub use wallet_refresh::{Refresh, RefreshExecutor};
pub use wallet_track::{Track, TrackExecutor};
pub use wallet_untrack::{Untrack, UntrackExecutor};

#[derive(Debug)]
pub struct WalletError {
    kind: WalletErrorKind,
    source: Option<Box<dyn error::Error + Send + Sync + 'static>>,
}

impl WalletError {
    pub fn kind(&self) -> WalletErrorKind {
        self.kind
    }
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            WalletErrorKind::NotFound => {
                write!(f, "wallet not found")
            }
            WalletErrorKind::NameConflict => {
                write!(f, "name conflict")
            }
            WalletErrorKind::NameEmpty => {
                write!(f, "wallet name is empty")
            }
            WalletErrorKind::NameTooLong => {
                write!(f, "wallet name exeeds {NAME_MAX} characters")
            }
            WalletErrorKind::WalletStore => {
                write!(f, "wallet store error")
            }
            WalletErrorKind::WalletClient => {
                write!(f, "wallet client error")
            }
            WalletErrorKind::WalletAddrParse => {
                write!(f, "couldn't parse wallet address")
            }
        }
    }
}

impl error::Error for WalletError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref().map(|e| e as _)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WalletErrorKind {
    NotFound,
    NameConflict,
    NameEmpty,
    NameTooLong,
    WalletStore,
    WalletClient,
    WalletAddrParse,
}

impl From<StoreError> for WalletError {
    fn from(error: StoreError) -> Self {
        Self {
            kind: WalletErrorKind::WalletStore,
            source: Some(error.0),
        }
    }
}

impl From<ClientError> for WalletError {
    fn from(error: ClientError) -> Self {
        Self {
            kind: WalletErrorKind::WalletClient,
            source: Some(error.0),
        }
    }
}

impl From<AddrParseError> for WalletError {
    fn from(error: AddrParseError) -> Self {
        Self {
            kind: WalletErrorKind::WalletAddrParse,
            source: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: String,
    pub address: String,
    pub balance: String,
    pub last_update: DateTime<Utc>,
}
