mod cmd;
mod internal;
// use internal::Request;
// use std::net::TcpListener;
use std::io;

use crate::cmd::http_server;

fn main() -> io::Result<()> {
    // let stream = TcpListener::bind("127.0.0.1:42069")?;
    // for connection in stream.incoming() {
    //     let mut stream = connection?;
    //     let req = Request::from_reader(&mut stream).unwrap();
    //     println!("Request line:");
    //     println!("- Method: {}", req.line.method);
    //     println!("- Target: {}", req.line.request_target);
    //     println!("- Version: {}", req.line.http_version);
    //
    //     println!("Headers:");
    //     for (name, value) in req.headers {
    //         println!("- {}: {}", name, value);
    //     }
    //
    //     println!("Body:");
    //     println!("{}", req.body);
    // }
    http_server::main();

    Ok(())
}
