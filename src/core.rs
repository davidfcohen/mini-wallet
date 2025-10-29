use std::{error, fmt, str::FromStr};

#[derive(Debug, Clone)]
pub struct Wallet {
    address: Address,
}

impl Wallet {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

const ADDR_SIZE: usize = 20;

#[derive(Debug, Clone)]
pub struct Address([u8; ADDR_SIZE]);

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    source: Option<Box<dyn error::Error + Send + Sync + 'static>>,
}

#[derive(Debug)]
enum ParseErrorKind {
    MissingPrefix,
    ParseHash,
    BadChecksum,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::MissingPrefix => write!(f, "missing address prefix"),
            ParseErrorKind::ParseHash => write!(f, "couldn't parse address hash"),
            ParseErrorKind::BadChecksum => write!(f, "address checksum failed"),
        }
    }
}

impl error::Error for ParseError {}

impl FromStr for Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coordinates = s.strip_prefix("0x").ok_or(ParseError {
            kind: ParseErrorKind::MissingPrefix,
            source: None,
        })?;

        let mut hash = [0; ADDR_SIZE];
        hex::decode_to_slice(coordinates, &mut hash).map_err(|e| ParseError {
            kind: ParseErrorKind::ParseHash,
            source: Some(e.into()),
        })?;

        Ok(Self(hash))
    }
}
