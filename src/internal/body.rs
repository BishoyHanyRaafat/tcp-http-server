use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body(Vec<u8>);

#[allow(dead_code)]
impl Body {
    pub fn new() -> Self {
        Body(Vec::new())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Body(bytes)
    }

    pub fn extend(&mut self, bytes: &Vec<u8>) {
        self.0.extend(bytes)
    }

    pub fn append(&mut self, bytes: &mut Vec<u8>) {
        self.0.append(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.as_bytes()))
    }
}
