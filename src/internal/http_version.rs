use super::shared::{RequestErrorEnum, Result};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct HttpVersion {
    pub major: String,
    pub minor: String,
}

impl Default for HttpVersion {
    fn default() -> Self {
        Self {
            major: "1".into(),
            minor: "1".into(),
        }
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl HttpVersion {
    pub fn from_bytes(s: &[u8]) -> Result<Self> {
        if !s.starts_with(b"HTTP/") {
            return Err(RequestErrorEnum::HttpVersionParseError);
        }
        let rest = &s[5..];

        let dot_pos = rest
            .iter()
            .position(|&b| b == b'.')
            .ok_or(RequestErrorEnum::HttpVersionParseError)?;

        let major = &rest[..dot_pos];
        let minor = &rest[dot_pos + 1..];

        if major.is_empty() || minor.is_empty() {
            return Err(RequestErrorEnum::HttpVersionParseError);
        }

        // Convert to &str without allocating — safe because digits are ASCII.
        let major_str =
            std::str::from_utf8(major).map_err(|_| RequestErrorEnum::HttpVersionParseError)?;

        let minor_str =
            std::str::from_utf8(minor).map_err(|_| RequestErrorEnum::HttpVersionParseError)?;

        Ok(Self {
            major: major_str.to_string(),
            minor: minor_str.to_string(),
        })
    }
}
