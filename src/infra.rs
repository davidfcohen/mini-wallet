use crate::core::Wallet;

#[derive(Clone)]
pub struct WalletRecord {
    name: String,
    wallet: Wallet,
}
