mod fs;
mod infra;
mod model;

mod wallet {
    use std::{error, fmt, result};

    use crate::{
        infra::{ChainError, StoreError},
        model::AddrParseError,
    };

    const NAME_MAX: usize = 30;

    pub type Result<T> = result::Result<T, WalletError>;

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
                WalletErrorKind::WalletChain => {
                    write!(f, "wallet blockchain error")
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
        WalletChain,
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

    impl From<ChainError> for WalletError {
        fn from(error: ChainError) -> Self {
            Self {
                kind: WalletErrorKind::WalletChain,
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
    }

    mod wallet_balance;
    mod wallet_list;
    mod wallet_track;
    mod wallet_untrack;
}
