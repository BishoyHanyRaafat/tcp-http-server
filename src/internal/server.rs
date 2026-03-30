use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use crate::internal::request::Request;
use crate::internal::response::Response;

pub struct Server {
    listener: Option<TcpListener>,
    closed: Arc<AtomicBool>,
}

impl Server {
    pub fn serve(port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        listener.set_nonblocking(true)?;

        let server = Server {
            listener: Some(listener),
            closed: Arc::new(AtomicBool::new(false)),
        };

        Ok(server)
    }

    pub fn close(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        self.listener = None
    }

    pub fn listen(&self) {
        let listener = self.listener.as_ref().unwrap();

        while !self.closed.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((conn, _addr)) => {
                    let closed = self.closed.clone();
                    thread::spawn(move || {
                        if !closed.load(Ordering::SeqCst) {
                            handle(conn);
                        }
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Sleep briefly to avoid tight spinning
                    thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => {
                    if !self.closed.load(Ordering::SeqCst) {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        }
    }
}

fn handle(mut conn: TcpStream) {
    // A valid HTTP response, same always
    let req = Request::from_reader(&mut conn).unwrap();
    // Add your request handling logic here
    println!("{}", req.line.request_target);
    let resp = match req.line.request_target.as_str() {
        "/" => Response::default(),
        _ => {
            let body = b"Not Found".to_vec();
            let mut header = crate::internal::headers::Headers::default();
            header.set("Content-Length", body.len().to_string().as_str());

            Response {
                response_line: crate::internal::response::ResponseLine {
                    http_version: req.line.http_version,
                    status_code: crate::internal::status_codes::StatusCode::NotFound,
                },
                headers: header,
                body: crate::internal::body::Body::from_bytes(body),
            }
        }
    };

    let _ = conn.write_all(resp.to_string().as_bytes());
    let _ = conn.flush();
}
