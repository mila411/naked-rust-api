use naked_rust_api::{Db, ThreadPool, handle_connection};
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");
    let pool = ThreadPool::new(4); // Specify the size of the thread pool

    println!("Server is running at http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db = Arc::clone(&db);
                pool.execute(move || {
                    handle_connection(stream, db);
                });
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
            }
        }
    }
}
