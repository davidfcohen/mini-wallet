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

#[derive(Debug)]
enum ErrorKind {
    ExpectedPrefix,
    Decode,
}

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::ExpectedPrefix => write!(f, "expected address prefix"),
            ErrorKind::Decode => write!(f, "couldn't decode address"),
        }
    }
}

impl error::Error for AddrParseError {}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits = s.strip_prefix("0x").ok_or(AddrParseError {
            kind: ErrorKind::ExpectedPrefix,
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
