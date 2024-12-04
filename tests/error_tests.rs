use naked_rust_api::{Db, Todo, handle_connection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_invalid_request_line() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8094").expect("Failed to bind to port 8094");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8094").expect("Failed to connect to server");
    let request = "INVALID / HTTP/1.1\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("405 Method Not Allowed"));
}

#[test]
fn test_create_todo_without_title() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8095").expect("Failed to bind to port 8095");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8095").expect("Failed to connect to server");
    let request_body = r#"{}"#;
    let request = format!(
        "POST /todos HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
        request_body.len(),
        request_body
    );
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("400 Bad Request"));
    assert!(response.contains("Title is required."));
}

#[test]
fn test_update_nonexistent_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new())); // Empty database

    let listener = TcpListener::bind("127.0.0.1:8096").expect("Failed to bind to port 8096");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8096").expect("Failed to connect to server");
    let request_body = r#"{"title":"Nonexistent Todo","completed":true}"#;
    let request = format!(
        "PUT /todos/999 HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
        request_body.len(),
        request_body
    );
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("404 Not Found"));
    assert!(response.contains("Todo not found."));
}

#[test]
fn test_delete_nonexistent_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new())); // Empty database

    let listener = TcpListener::bind("127.0.0.1:8097").expect("Failed to bind to port 8097");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8097").expect("Failed to connect to server");
    let request = "DELETE /todos/999 HTTP/1.1\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("404 Not Found"));
    assert!(response.contains("Todo not found."));
}

#[test]
fn test_create_todo_invalid_json() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8098").expect("Failed to bind to port 8098");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8098").expect("Failed to connect to server");
    let request_body = r#"{"title":123}"#; // title is not a string
    let request = format!(
        "POST /todos HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
        request_body.len(),
        request_body
    );
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("400 Bad Request"));
    assert!(response.contains("Title is required."));
}

#[test]
fn test_update_todo_invalid_json() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let todo = Todo {
        id: 5,
        title: "Valid Todo".to_string(),
        completed: false,
    };
    db.lock().unwrap().insert(todo.id.to_string(), todo);

    let listener = TcpListener::bind("127.0.0.1:8099").expect("Failed to bind to port 8099");
    let db_clone = Arc::clone(&db);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let db = Arc::clone(&db_clone);
                    handle_connection(stream, db);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:8099").expect("Failed to connect to server");
    let request_body = r#"{"title":"Updated Todo","completed":"yes"}"#; // completed is not a bool
    let request = format!(
        "PUT /todos/5 HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
        request_body.len(),
        request_body
    );
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("400 Bad Request"));
    assert!(response.contains("Invalid JSON format."));
}
