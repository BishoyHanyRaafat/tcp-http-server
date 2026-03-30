use crate::internal::{body::Body, headers::Headers};

use super::shared::{RequestErrorEnum, Result, SEPARATOR};
use std::{
    cmp::min,
    fmt::{self, Display, Formatter},
    io::Read,
};

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
    Option,
    Head,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let method_str = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Option => "OPTION",
            Method::Head => "HEAD",
        };
        write!(f, "{}", method_str)
    }
}

impl Method {
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            "OPTION" => Ok(Self::Option),
            "Head" => Ok(Self::Head),
            _ => Err(RequestErrorEnum::MethodNotFound),
        }
    }

    pub fn from_bytes(s: &[u8]) -> Result<Self> {
        match s {
            b"GET" => Ok(Self::Get),
            b"POST" => Ok(Self::Post),
            b"PATCH" => Ok(Self::Patch),
            b"DELETE" => Ok(Self::Delete),
            b"OPTION" => Ok(Self::Option),
            b"Head" => Ok(Self::Head),
            _ => Err(RequestErrorEnum::MethodNotFound),
        }
    }
}

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

#[derive(Debug)]
pub struct RequestLine {
    pub http_version: HttpVersion,
    pub request_target: String,
    pub method: Method,
}

impl RequestLine {
    pub fn parse_request_line(request: &[u8]) -> Result<(Self, usize)> {
        let idx = request
            .windows(SEPARATOR.len())
            .position(|w| w == SEPARATOR)
            .ok_or(RequestErrorEnum::IncompleteRequest)?;

        let request_line = &request[..idx];
        let consumed = idx + SEPARATOR.len();

        let mut parts = request_line.split(|b| *b == b' ');
        let method_bytes = parts.next().ok_or(RequestErrorEnum::ParseError)?;
        let target_bytes = parts.next().ok_or(RequestErrorEnum::ParseError)?;
        let version_bytes = parts.next().ok_or(RequestErrorEnum::ParseError)?;

        if parts.next().is_some() {
            return Err(RequestErrorEnum::ParseError);
        }

        let method = Method::from_bytes(method_bytes)?;
        let request_target = std::str::from_utf8(target_bytes)
            .map_err(|_| RequestErrorEnum::ParseError)?
            .to_string();

        let http_version = HttpVersion::from_bytes(version_bytes)?;

        if http_version.major != "1" || http_version.minor != "1" {
            return Err(RequestErrorEnum::IncompatibleVersion);
        }

        Ok((
            RequestLine {
                http_version,
                method,
                request_target,
            },
            consumed,
        ))
    }
}

#[derive(Debug)]
pub struct Request {
    pub line: RequestLine,
    pub headers: Headers,
    pub body: Body,
}

#[derive(Debug)]
enum ParseState {
    RequestLine,
    RequestHeaders,
    Body,
    Done,
}

impl Request {
    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + Send + 'static,
    {
        let mut buffer = Vec::new();
        let mut request_line: Option<RequestLine> = None;
        let mut request_headers: Option<Headers> = None;
        let mut request_body: Body = Body::new();
        let mut state = ParseState::RequestLine;
        let mut read_buf = [0u8; 1024];

        match reader.read(&mut read_buf) {
            Ok(0) => return Err(RequestErrorEnum::NoRequestToRead),
            Ok(n) => buffer.extend_from_slice(&read_buf[..n]),
            Err(_) => return Err(RequestErrorEnum::ParseError),
        }
        loop {
            // Try to advance the state machine first
            match Self::state_machine(
                &state,
                &mut buffer,
                &mut request_line,
                &mut request_headers,
                &mut request_body,
            ) {
                Ok(next_state) => {
                    state = next_state;
                    if matches!(state, ParseState::Done) {
                        return Ok(Request {
                            line: request_line
                                .take()
                                .ok_or(RequestErrorEnum::UnexpectedStateError)?,
                            headers: request_headers
                                .take()
                                .ok_or(RequestErrorEnum::UnexpectedStateError)?,
                            body: request_body,
                        });
                    }
                }
                Err(RequestErrorEnum::IncompleteRequest) => {
                    // Read more data into the buffer
                    match reader.read(&mut read_buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => buffer.extend_from_slice(&read_buf[..n]),
                        Err(_) => return Err(RequestErrorEnum::ParseError),
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(RequestErrorEnum::ParseError)
    }

    fn state_machine(
        state: &ParseState,
        buffer: &mut Vec<u8>,
        request_line: &mut Option<RequestLine>,
        request_headers: &mut Option<Headers>,
        request_body: &mut Body,
    ) -> Result<ParseState> {
        match state {
            ParseState::RequestLine => match RequestLine::parse_request_line(buffer) {
                Ok((rl, consumed)) => {
                    buffer.drain(..consumed);
                    *request_line = Some(rl);
                    Ok(ParseState::RequestHeaders)
                }
                Err(e) => Err(e),
            },
            ParseState::RequestHeaders => match Headers::new_from_data(buffer) {
                Ok((headers, consumed)) => {
                    buffer.drain(..consumed + 2); // Extra CTRL removal
                    *request_headers = Some(headers);
                    Ok(ParseState::Body)
                }
                Err(e) => Err(e),
            },
            ParseState::Body => {
                let len = request_headers
                    .as_ref()
                    .ok_or(RequestErrorEnum::UnexpectedStateError)?
                    .get("Content-Length");
                match len {
                    Some("0") => Ok(ParseState::Done),
                    None => Ok(ParseState::Done),
                    Some(len_str) => {
                        let len_int = len_str
                            .parse::<usize>()
                            .map_err(|_| RequestErrorEnum::ParseError)?;

                        let body_len = min(len_int - request_body.len(), buffer.len());
                        request_body.extend(&buffer.drain(..body_len).collect());
                        if request_body.len() == len_int {
                            Ok(ParseState::Done)
                        } else {
                            Err(RequestErrorEnum::IncompleteRequest)
                        }
                    }
                }
            }
            ParseState::Done => Ok(ParseState::Done),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_good_get_root() {
        let input = "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let mut c = Cursor::new(input);
        let r = Request::from_reader(&mut c).expect("should parse successfully");
        assert_eq!(r.line.method.to_string(), "GET");
        assert_eq!(r.line.request_target, "/");
        assert_eq!(r.line.http_version.major, "1");
        assert_eq!(r.line.http_version.minor, "1");
    }

    #[test]
    fn test_good_get_path() {
        let input = "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let mut c = Cursor::new(input);
        let r = Request::from_reader(&mut c).expect("should parse successfully");
        assert_eq!(r.line.method.to_string(), "GET");
        assert_eq!(r.line.request_target, "/coffee");
        assert_eq!(r.line.http_version.major, "1");
        assert_eq!(r.line.http_version.minor, "1");
    }

    #[test]
    fn test_invalid_request_line() {
        let input = "/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let mut c = Cursor::new(input);
        let result = Request::from_reader(&mut c);
        assert!(result.is_err());
    }

    pub struct ChunkReader {
        data: Vec<u8>,
        num_bytes_per_read: usize,
        pos: usize,
    }

    impl ChunkReader {
        pub fn new(data: &str, num_bytes_per_read: usize) -> Self {
            Self {
                data: data.as_bytes().to_vec(),
                num_bytes_per_read,
                pos: 0,
            }
        }
    }

    impl Read for ChunkReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pos >= self.data.len() {
                return Ok(0); // EOF
            }

            // maximum allowed by config
            let max = self.num_bytes_per_read;

            // maximum allowed by caller's buffer
            let limit = buf.len().min(max);

            // maximum allowed by remaining data
            let remaining = self.data.len() - self.pos;
            let n = limit.min(remaining);

            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;

            Ok(n)
        }
    }
    #[test]
    fn test_good_get_root_chunk() {
        let mut reader = ChunkReader::new(
            "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            3,
        );

        let req = Request::from_reader(&mut reader).expect("should parse successfully");

        assert_eq!(req.line.method.to_string(), "GET");
        assert_eq!(req.line.request_target, "/");
        assert_eq!(req.line.http_version.major, "1");
        assert_eq!(req.line.http_version.minor, "1");
    }

    #[test]
    fn test_good_get_path_chunk() {
        let mut reader = ChunkReader::new(
            "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            1,
        );

        let req = Request::from_reader(&mut reader).expect("should parse successfully");

        assert_eq!(req.line.method.to_string(), "GET");
        assert_eq!(req.line.request_target, "/coffee");
        assert_eq!(req.line.http_version.major, "1");
        assert_eq!(req.line.http_version.minor, "1");
    }

    #[test]
    fn test_request_body_with_context_length() {
        let data = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 13\r\n\r\nhello world!\n"
            .to_vec();
        let mut reader = ChunkReader {
            data,
            pos: 0,
            num_bytes_per_read: 3,
        };

        let r = Request::from_reader(&mut reader).expect("should parse successfully");
        assert_eq!(r.body.as_bytes(), b"hello world!\n");
    }

    #[test]
    fn test_request_no_context_length() {
        let data2 = b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 20\r\n\r\npartial content"
            .to_vec();
        let mut reader2 = ChunkReader {
            data: data2,
            pos: 0,
            num_bytes_per_read: 3,
        };

        let r2 = Request::from_reader(&mut reader2);
        assert!(
            r2.is_err(),
            "should fail because body is shorter than Content-Length"
        );
    }
}
