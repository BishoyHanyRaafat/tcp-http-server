use std::collections::HashMap;

use super::shared::{CRLF, RequestErrorEnum, Result};

#[derive(Debug)]
pub struct Headers(HashMap<String, String>);

// Consuming iteration
impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Borrowed iteration
impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Mutable borrowed iteration
impl<'a> IntoIterator for &'a mut Headers {
    type Item = (&'a String, &'a mut String);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Default for Headers {
    fn default() -> Self {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Length".to_string(), "0".to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        Headers(headers)
    }
}

impl Headers {
    pub fn new() -> Self {
        Headers(HashMap::new())
    }

    pub fn new_from_data(data: &[u8]) -> Result<(Self, usize)> {
        let mut headers = Headers::new();
        let n = headers.parse(data)?;
        Ok((headers, n))
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<usize> {
        if let Some(pos) = data.windows(4).position(|w| w == CRLF).map(|p| p + 2) {
            let header_bytes = &data[..pos];
            let lines = str::from_utf8(header_bytes).map_err(|_| RequestErrorEnum::ParseError)?;
            for line in lines.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                };
                let v: Vec<&str> = line.splitn(2, ':').collect();
                if v[0].ends_with(" ") || v[0].is_empty() {
                    return Err(RequestErrorEnum::ParseError);
                }
                let k = v[0];
                if !Self::is_valid_token(k) {
                    return Err(RequestErrorEnum::MalFormedHeader);
                }

                self.set(k, v[1].trim());
            }
            Ok(pos)
        } else {
            Err(RequestErrorEnum::IncompleteRequest)
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(&key.to_lowercase()).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(v) = self.0.get_mut(&key.to_lowercase()) {
            *v = format!("{}, {}", v, value);
        } else {
            self.0.insert(key.to_lowercase(), value.to_string());
        }
    }

    fn is_valid_token(token: &str) -> bool {
        for c in token.chars() {
            if !c.is_ascii_alphanumeric()
                && c != '!'
                && c != '#'
                && c != '$'
                && c != '%'
                && c != '&'
                && c != '\''
                && c != '*'
                && c != '+'
                && c != '-'
                && c != '.'
                && c != '^'
                && c != '_'
                && c != '`'
                && c != '|'
                && c != '~'
            {
                return false;
            }
        }
        true
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(&key.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_single_header() {
        let mut headers = Headers::new();
        let data = b"Host: localhost:42069\r\n\r\n";

        let n = headers.parse(data).expect("should parse");
        assert_eq!(headers.get("Host").unwrap(), "localhost:42069");
        assert_eq!(n, 23);
    }

    #[test]
    fn test_invalid_spacing_header() {
        let mut headers = Headers::new();
        let data = b"       Host : localhost:42069       \r\n\r\n";

        let result = headers.parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_single_header_with_white_space() {
        let mut headers = Headers::new();
        let data = b"Host: localhost:333      \r\n\r\n";

        let n = headers.parse(data).expect("Shold parser");
        assert_eq!(headers.get("Host").unwrap(), "localhost:333");
        assert_eq!(n, 27);
    }

    #[test]
    fn test_valid_multi_headers() {
        let mut headers = Headers::new();
        let data = b"Host: localhost:333      \r\nContent-Length: xyz\r\nName: Bishoy\r\n\r\n";

        let n = headers.parse(data).expect("Shold parser");
        assert_eq!(headers.get("Host").unwrap(), "localhost:333");
        assert_eq!(headers.get("Content-Length").unwrap(), "xyz");
        assert_eq!(headers.get("Name").unwrap(), "Bishoy");
        assert_eq!(n, 62);
    }

    #[test]
    fn test_valid_not_done_headers() {
        let mut headers = Headers::new();
        let data = b"Host: localhost:322 \r\n Host123: 231";

        let n = headers.parse(data);
        assert!(n.is_err());
        assert!(matches!(
            n.err().unwrap(),
            RequestErrorEnum::IncompleteRequest
        ));
    }

    #[test]
    fn test_invalid_header_token() {
        let mut headers = Headers::new();

        let data = b"Ho@st: localhost:42069\r\n\r\n";
        let result = headers.parse(data);
        assert!(result.is_err());

        let data = b"Invalid-\xA9Token: localhost:12345\r\n\r\n";
        let result = headers.parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_same_headers() {
        let mut headers = Headers::new();
        let data = b"Cookie: cookie1=value1\r\nCookie: cookie2=value2\r\n\r\n";

        let n = headers.parse(data).expect("should parse");
        assert_eq!(
            headers.get("Cookie").unwrap(),
            "cookie1=value1, cookie2=value2"
        );
        assert_eq!(n, 48);
    }
}
