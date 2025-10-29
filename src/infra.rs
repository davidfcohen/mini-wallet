use uuid::Uuid;

use crate::core::Wallet;

#[derive(Clone)]
pub struct WalletRecord {
    id: Uuid,
    name: String,
    wallet: Wallet,
}
