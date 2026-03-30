use crate::internal::{
    body::Body, headers::Headers, request::HttpVersion, status_codes::StatusCode,
};
use std::fmt::{Display, Formatter};

pub struct ResponseLine {
    pub http_version: HttpVersion,
    pub status_code: StatusCode,
}

impl Default for ResponseLine {
    fn default() -> Self {
        ResponseLine {
            http_version: HttpVersion::default(),
            status_code: StatusCode::Ok,
        }
    }
}

pub struct Response {
    pub response_line: ResponseLine,
    pub headers: Headers,
    pub body: Body,
}

impl Default for Response {
    fn default() -> Self {
        Response {
            response_line: ResponseLine::default(),
            headers: Headers::default(),
            body: Body::new(),
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP/{} {}\r\n",
            self.response_line.http_version, self.response_line.status_code
        )?;
        for (name, value) in &self.headers {
            write!(f, "{}: {}\r\n", name, value)?;
        }
        write!(f, "\r\n")?;
        write!(f, "{}", self.body)
    }
}
