use std::{
    error,
    fmt::{self, Write},
    str::FromStr,
};

use hex::FromHexError;
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone)]
pub struct Wallet {
    address: Address,
    balance: Balance,
}

impl Wallet {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            balance: Balance::default(),
        }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn address_mut(&mut self) -> &mut Address {
        &mut self.address
    }

    pub fn balance(&self) -> Balance {
        self.balance
    }

    pub fn balance_mut(&mut self) -> &mut Balance {
        &mut self.balance
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Balance(u128);

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.wei())
    }
}

impl Balance {
    pub fn new(wei: u128) -> Self {
        Self(wei.into())
    }

    pub fn wei(&self) -> u128 {
        self.0
    }

    pub fn eth(&self) -> String {
        const ONE_ETH: u128 = 1_000_000_000_000_000_000;
        let wei = self.wei();
        let whole = wei / ONE_ETH;
        let fraction = wei % ONE_ETH;
        format!("{whole}.{fraction}")
    }
}

#[derive(Debug)]
pub struct AddrParseError {
    inner: InnerAddrParseError,
}

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            InnerAddrParseError::MissingPrefix => write!(f, "address missing prefix"),
            InnerAddrParseError::WrongLen => write!(f, "address is wrong length"),
            InnerAddrParseError::BadChecksum => write!(f, "address doesn't match checksum"),
            InnerAddrParseError::Decode(_) => write!(f, "couldn't decode address"),
        }
    }
}

impl error::Error for AddrParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.inner {
            InnerAddrParseError::Decode(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum InnerAddrParseError {
    MissingPrefix,
    WrongLen,
    BadChecksum,
    Decode(FromHexError),
}

impl From<InnerAddrParseError> for AddrParseError {
    fn from(error: InnerAddrParseError) -> Self {
        Self { inner: error }
    }
}

const ADDR_DECODE_SIZE: usize = 20;
const ADDR_ENCODE_SIZE: usize = ADDR_DECODE_SIZE * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address([u8; ADDR_DECODE_SIZE]);

impl Address {
    pub fn new(bytes: [u8; ADDR_DECODE_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn inner(&self) -> &[u8; ADDR_DECODE_SIZE] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut addr_encoded = [0u8; ADDR_ENCODE_SIZE];
        hex::encode_to_slice(self.inner(), &mut addr_encoded)
            .expect("20 bytes encodes to 40 bytes");
        make_addr_checksum(&mut addr_encoded);

        write!(f, "0x")?;
        for ch in addr_encoded {
            f.write_char(ch as char)?;
        }

        Ok(())
    }
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        let addr_encoded: &[u8; ADDR_ENCODE_SIZE] = addr
            .as_bytes()
            .strip_prefix(b"0x")
            .ok_or(InnerAddrParseError::MissingPrefix)?
            .try_into()
            .map_err(|_| InnerAddrParseError::WrongLen)?;

        let mut addr_decoded = [0; ADDR_DECODE_SIZE];
        hex::decode_to_slice(addr_encoded, &mut addr_decoded)
            .map_err(InnerAddrParseError::Decode)?;

        if !checksum_eq(addr_encoded) {
            Err(InnerAddrParseError::BadChecksum)?;
        }

        Ok(Self(addr_decoded))
    }
}

fn checksum_eq(addr: &[u8; ADDR_ENCODE_SIZE]) -> bool {
    let mut addr_checksum = *addr;
    make_addr_checksum(&mut addr_checksum);
    addr.eq(&addr_checksum)
}

fn make_addr_checksum(addr: &mut [u8; ADDR_ENCODE_SIZE]) {
    addr.make_ascii_lowercase();

    let mut addr_hash = [0u8; ADDR_DECODE_SIZE];
    let mut keccak = Keccak::v256();
    keccak.update(addr);
    keccak.finalize(&mut addr_hash);

    let addr_checksum = addr;
    let addr_hash_nibbles = addr_hash.iter().flat_map(|byte| [byte >> 4, byte & 0xf]);
    for (addr_checksum_ch, addr_hash_nibble) in addr_checksum
        .iter_mut()
        .zip(addr_hash_nibbles)
        .filter(|(ch, _)| ch.is_ascii_alphabetic())
    {
        if addr_hash_nibble >= 8 {
            addr_checksum_ch.make_ascii_uppercase();
        } else {
            addr_checksum_ch.make_ascii_lowercase();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addr_display_checksum() {
        let encoded = "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B";
        let decoded = Address::from_str(encoded).unwrap();
        assert_eq!(decoded.to_string(), encoded);

        let encoded = "0xB644Babc370f46f202DB5eaf2071A9Ee66fA1D5E";
        let decoded = Address::from_str(encoded).unwrap();
        assert_eq!(decoded.to_string(), encoded);

        let encoded = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let decoded = Address::from_str(encoded).unwrap();
        assert_eq!(decoded.to_string(), encoded);
    }

    #[test]
    fn addr_parse_success() {
        assert!(Address::from_str("0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B").is_ok());
        assert!(Address::from_str("0xF6369e1A96c7af1E2326826F5Dd84bFEf78d7D80").is_ok());
    }

    #[test]
    fn addr_parse_missing_prefix() {
        let error = Address::from_str("").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::MissingPrefix));

        let error = Address::from_str("Ab5801a7D398351b8bE11C439e05C5B3259aeC9B").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::MissingPrefix));

        let error = Address::from_str("F6369e1A96c7af1E2326826F5Dd84bFEf78d7D80").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::MissingPrefix));
    }

    #[test]
    fn addr_parse_wrong_len() {
        let error = Address::from_str("0x").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::WrongLen));

        let error = Address::from_str("0xAb5801a7D398351b8bE11C439e05C5B3259aeC9").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::WrongLen));

        let error = Address::from_str("0xF6369e1A96c7af1E2326826F5Dd84bFEf78d7D801").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::WrongLen));
    }

    #[test]
    fn addr_parse_bad_checksum() {
        let error = Address::from_str("0xaB5801A7d398351B8Be11c439E05c5b3259AEc9b").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::BadChecksum));

        let error = Address::from_str("0xab5801a7d398351b8be11c439e05c5b3259aec9b").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::BadChecksum));

        let error = Address::from_str("0xAB5801A7D398351B8BE11C439E05C5B3259AEC9B").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::BadChecksum));
    }

    #[test]
    fn addr_parse_decode_err() {
        let error = Address::from_str("0xABCDEFGHIJKLMNOPQRSTabcdefghijklmnopqrst").unwrap_err();
        assert!(matches!(error.inner, InnerAddrParseError::Decode(_)));
    }
}
