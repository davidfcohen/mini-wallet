use secp256k1::{PublicKey, Secp256k1, SecretKey, rand};
use tiny_keccak::{Hasher, Keccak};

const SECRET_KEY_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Wallet {
    secret_key: SecretKey,
}

impl Wallet {
    pub fn new(secret_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        Self { secret_key }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut rng);
        Self { secret_key }
    }

    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    pub fn public_key(&self) -> PublicKey {
        let secp = Secp256k1::new();
        PublicKey::from_secret_key(&secp, &self.secret_key)
    }

    pub fn address(&self) -> String {
        let public_key = &self.public_key().serialize_uncompressed()[1..];

        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(public_key);
        hasher.finalize(&mut hash);

        format!("0x{}", hex::encode(&hash[12..]))
    }
}
