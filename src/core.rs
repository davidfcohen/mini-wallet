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
    ExpectedPrefix,
    Decode,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::ExpectedPrefix => write!(f, "expected address prefix"),
            ParseErrorKind::Decode => write!(f, "couldn't decode address"),
        }
    }
}

impl error::Error for ParseError {}

impl FromStr for Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nibbles = s.strip_prefix("0x").ok_or(ParseError {
            kind: ParseErrorKind::ExpectedPrefix,
            source: None,
        })?;

        let mut address = [0; ADDR_SIZE];
        hex::decode_to_slice(nibbles, &mut address).map_err(|e| ParseError {
            kind: ParseErrorKind::Decode,
            source: Some(e.into()),
        })?;

        Ok(Self(address))
    }
}
