use naked_rust_api::{Db, Todo, handle_connection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_create_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8089").expect("Failed to bind to port 8089");
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

    let mut stream = TcpStream::connect("127.0.0.1:8089").expect("Failed to connect to server");
    let request_body = r#"{"title":"Learn Rust"}"#;
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

    assert!(response.contains("201 Created"));
    assert!(response.contains("\"title\":\"Learn Rust\""));
}

#[test]
fn test_get_todos() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let todo = Todo {
        id: 1,
        title: "Learn Rust".to_string(),
        completed: false,
    };
    db.lock().unwrap().insert(todo.id.to_string(), todo);

    let listener = TcpListener::bind("127.0.0.1:8090").expect("Failed to bind to port 8090");
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

    let mut stream = TcpStream::connect("127.0.0.1:8090").expect("Failed to connect to server");
    let request = "GET /todos HTTP/1.1\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("200 OK"));
    assert!(response.contains("\"title\":\"Learn Rust\""));
}

#[test]
fn test_get_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let todo = Todo {
        id: 2,
        title: "Write Tests".to_string(),
        completed: false,
    };
    db.lock().unwrap().insert(todo.id.to_string(), todo);

    let listener = TcpListener::bind("127.0.0.1:8091").expect("Failed to bind to port 8091");
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

    let mut stream = TcpStream::connect("127.0.0.1:8091").expect("Failed to connect to server");
    let request = "GET /todos/2 HTTP/1.1\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("200 OK"));
    assert!(response.contains("\"title\":\"Write Tests\""));
}

#[test]
fn test_update_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let todo = Todo {
        id: 3,
        title: "Initial Title".to_string(),
        completed: false,
    };
    db.lock().unwrap().insert(todo.id.to_string(), todo);

    let listener = TcpListener::bind("127.0.0.1:8092").expect("Failed to bind to port 8092");
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

    let mut stream = TcpStream::connect("127.0.0.1:8092").expect("Failed to connect to server");
    let request_body = r#"{"title":"Updated Title","completed":true}"#;
    let request = format!(
        "PUT /todos/3 HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
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

    assert!(response.contains("200 OK"));
    assert!(response.contains("\"title\":\"Updated Title\""));
    assert!(response.contains("\"completed\":true"));
}

#[test]
fn test_delete_todo() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let todo = Todo {
        id: 4,
        title: "To be deleted".to_string(),
        completed: false,
    };
    db.lock().unwrap().insert(todo.id.to_string(), todo);

    let listener = TcpListener::bind("127.0.0.1:8093").expect("Failed to bind to port 8093");
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

    let mut stream = TcpStream::connect("127.0.0.1:8093").expect("Failed to connect to server");
    let request = "DELETE /todos/4 HTTP/1.1\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let response = String::from_utf8_lossy(&buffer[..]);

    assert!(response.contains("200 OK"));
    assert!(response.contains("Todo has been deleted."));
}
