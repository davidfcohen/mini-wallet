use std::{error, fmt, str::FromStr};

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

const ADDR_SIZE: usize = 20;

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
        write!(f, "0x")?;
        self.0
            .iter()
            .try_for_each(|byte| write!(f, "{:02x}", byte))?;
        Ok(())
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
    Decode,
}

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::MissingPrefix => write!(f, "missing address prefix"),
            ErrorKind::Decode => write!(f, "couldn't decode address"),
        }
    }
}

impl error::Error for AddrParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref().map(|e| e as _)
    }
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits = s.strip_prefix("0x").ok_or(AddrParseError {
            kind: ErrorKind::MissingPrefix,
            source: None,
        })?;

        let mut address = [0; ADDR_SIZE];
        hex::decode_to_slice(digits, &mut address).map_err(|e| AddrParseError {
            kind: ErrorKind::Decode,
            source: Some(e.into()),
        })?;

        Ok(Self(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_parse() {
        const ADDR: &str = "0xf6369E1A96c7aF1e2326826f5dD84BfEf78d7d80";
        assert!(Address::from_str(ADDR).is_ok())
    }

    #[test]
    fn address_parse_expected_prefix() {
        const ADDR: &str = "f6369E1A96c7aF1e2326826f5dD84BfEf78d7d80";
        let AddrParseError { kind, .. } = Address::from_str(ADDR).unwrap_err();
        assert_eq!(kind, ErrorKind::MissingPrefix)
    }

    #[test]
    fn address_parse_invalid_len() {
        const ADDR: &str = "0xf6369E1A96c7aF1e2326";
        let AddrParseError { kind, .. } = Address::from_str(ADDR).unwrap_err();
        assert_eq!(kind, ErrorKind::Decode)
    }
}
