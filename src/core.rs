use std::{
    error,
    fmt::{self, Write},
    str::FromStr,
};

use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone)]
pub struct Wallet {
    address: Address,
}

impl Wallet {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }
}

#[derive(Debug)]
pub struct AddrParseError {
    kind: ErrorKind,
    source: Option<Box<dyn error::Error + Send + Sync + 'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ErrorKind {
    MissingPrefix,
    WrongLen,
    BadChecksum,
    Decode,
}

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::MissingPrefix => write!(f, "address missing prefix"),
            ErrorKind::WrongLen => write!(f, "address is wrong length"),
            ErrorKind::BadChecksum => write!(f, "address doesn't match checksum"),
            ErrorKind::Decode => write!(f, "couldn't decode address"),
        }
    }
}

const ADDR_SIZE: usize = 20;
const ADDR_LEN: usize = ADDR_SIZE * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address([u8; ADDR_SIZE]);

impl Address {
    pub fn new(bytes: [u8; ADDR_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn inner(&self) -> &[u8; ADDR_SIZE] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut addr_encoded = [0u8; ADDR_LEN];
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

impl error::Error for AddrParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref().map(|e| e as _)
    }
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        let addr_encoded = addr.as_bytes().strip_prefix(b"0x").ok_or(AddrParseError {
            kind: ErrorKind::MissingPrefix,
            source: None,
        })?;

        let addr_encoded: &[u8; ADDR_LEN] =
            addr_encoded.try_into().map_err(|_| AddrParseError {
                kind: ErrorKind::WrongLen,
                source: None,
            })?;

        if !checksum_eq(addr_encoded) {
            return Err(AddrParseError {
                kind: ErrorKind::BadChecksum,
                source: None,
            });
        }

        let mut addr_decoded = [0; ADDR_SIZE];
        hex::decode_to_slice(addr_encoded, &mut addr_decoded).map_err(|e| AddrParseError {
            kind: ErrorKind::Decode,
            source: Some(e.into()),
        })?;

        Ok(Self(addr_decoded))
    }
}

fn checksum_eq(addr: &[u8; ADDR_LEN]) -> bool {
    let mut addr_checksum = *addr;
    make_addr_checksum(&mut addr_checksum);
    addr.eq(&addr_checksum)
}

fn make_addr_checksum(addr_lower: &mut [u8; ADDR_LEN]) {
    addr_lower.make_ascii_lowercase();

    let mut addr_hash = [0u8; ADDR_SIZE];
    let mut keccak = Keccak::v256();
    keccak.update(addr_lower);
    keccak.finalize(&mut addr_hash);
    let addr_hash_nibbles = addr_hash.iter().flat_map(|byte| [byte >> 4, byte & 0xf]);

    let addr_checksum = addr_lower;
    for (addr_checksum_ch, addr_hash_nibble) in addr_checksum
        .iter_mut()
        .filter(|ch| ch.is_ascii_alphabetic())
        .zip(addr_hash_nibbles)
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
    fn addr_parse_ok() {
        const ADDR: &str = "0xf6369E1A96c7aF1e2326826f5dD84BfEf78d7d80";
        assert!(Address::from_str(ADDR).is_ok())
    }

    #[test]
    fn addr_parse_missing_prefix() {
        const ADDR: &str = "f6369E1A96c7aF1e2326826f5dD84BfEf78d7d80";
        let AddrParseError { kind, .. } = Address::from_str(ADDR).unwrap_err();
        assert_eq!(kind, ErrorKind::MissingPrefix)
    }

    #[test]
    fn addr_bad_checksum() {}
}
