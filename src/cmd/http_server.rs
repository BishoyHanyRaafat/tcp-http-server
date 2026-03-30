use crate::internal::server::Server;
use std::process;

const PORT: u16 = 42069;

pub fn main() {
    let server = match Server::serve(PORT) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error starting server: {}", e);
            process::exit(1);
        }
    };

    println!("Server started on port {}", PORT);
    server.listen();
}
