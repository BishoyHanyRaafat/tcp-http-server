pub const SEPARATOR: &[u8] = b"\r\n";
pub const CRLF: &[u8] = b"\r\n\r\n";

pub type Result<T> = std::result::Result<T, RequestErrorEnum>;

#[derive(Debug)]
pub enum RequestErrorEnum {
    /// Parsing the request failed
    ParseError,
    /// the method not in the methods enum check [Method]
    MethodNotFound,
    /// Request incomplete
    IncompleteRequest,
    /// Parsing HTTP version failed
    HttpVersionParseError,
    /// HTTP version is not 1.1
    IncompatibleVersion,
    /// Found a malformed header in the request
    MalFormedHeader,
    /// Unexpected state encountered during request parsing
    UnexpectedStateError,
    /// Request does not contain any data to read
    NoRequestToRead,
}
