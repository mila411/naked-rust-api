use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

#[derive(Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
}

pub type Db = Arc<Mutex<HashMap<String, Todo>>>;

#[derive(Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    completed: Option<bool>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Thread pool size must be at least 1.");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        println!("Sending job to thread pool.");
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv().unwrap()
                };
                println!("Worker {} received a job. Executing.", id);
                job();
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Validation functions
fn validate_todo_title(title: &str) -> Result<(), &'static str> {
    if title.trim().is_empty() {
        Err("Title cannot be empty.")
    } else {
        Ok(())
    }
}

fn validate_todo_completed(completed: &Option<bool>) -> Result<(), &'static str> {
    if completed.is_some() {
        Ok(())
    } else {
        Err("The 'completed' field must be of type bool.")
    }
}

// Function to log errors to a file
fn log_error(message: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("error.log")
        .expect("Failed to open error.log file.");
    let timestamp = Utc::now().to_rfc3339();
    writeln!(file, "[{}] {}", timestamp, message).expect("Failed to write to error log.");
}

pub fn process_request(request: &str, db: Db) -> (&'static str, String) {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            let error = "Invalid request line.";
            log_error(error);
            return ("400 Bad Request", error.to_string());
        }
        let method = parts[0];
        let path = parts[1];
        let version = parts[2];

        if version != "HTTP/1.1" && version != "HTTP/1.0" && version != "HTTP/2.0" {
            let error = "HTTP version is not supported.";
            log_error(error);
            return ("505 HTTP Version Not Supported", error.to_string());
        }

        let mut headers = HashMap::new();
        let mut content_length = None;
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                if key.eq_ignore_ascii_case("Content-Length") {
                    content_length = Some(value.parse().unwrap_or(0));
                }
                headers.insert(key.to_string(), value.to_string());
            } else {
                let error = "Invalid header format.";
                log_error(error);
                return ("400 Bad Request", error.to_string());
            }
        }

        let body: String = lines.collect::<Vec<&str>>().join("\n");
        let body = &body[..content_length.unwrap_or(0)];

        match method {
            "GET" => {
                if path == "/todos" {
                    return process_request_get_todos(db);
                } else if path.starts_with("/todos/") {
                    if let Some(id_str) = path.strip_prefix("/todos/") {
                        if let Ok(id) = id_str.parse::<usize>() {
                            return get_todo(id, db);
                        }
                    }
                    let error = "Invalid ID.";
                    log_error(error);
                    return ("400 Bad Request", error.to_string());
                }
                let error = "Endpoint not found.";
                log_error(error);
                return ("404 Not Found", error.to_string());
            }
            "POST" => {
                if path == "/todos" {
                    match serde_json::from_str::<Value>(body) {
                        Ok(json) => {
                            if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                                if let Err(e) = validate_todo_title(title) {
                                    log_error(e);
                                    return ("400 Bad Request", e.to_string());
                                }
                                let title = title.to_string();
                                return create_todo(title, db);
                            } else {
                                let error = "Title is required.";
                                log_error(error);
                                return ("400 Bad Request", error.to_string());
                            }
                        }
                        Err(_) => {
                            let error = "Invalid JSON format.";
                            log_error(error);
                            return ("400 Bad Request", error.to_string());
                        }
                    }
                }
                let error = "Endpoint not found.";
                log_error(error);
                return ("404 Not Found", error.to_string());
            }
            "PUT" => {
                if path.starts_with("/todos/") {
                    if let Some(id_str) = path.strip_prefix("/todos/") {
                        if let Ok(id) = id_str.parse::<usize>() {
                            match serde_json::from_str::<UpdateTodoRequest>(body) {
                                Ok(update_req) => {
                                    if let Some(ref title) = update_req.title {
                                        if let Err(e) = validate_todo_title(title) {
                                            log_error(e);
                                            return ("400 Bad Request", e.to_string());
                                        }
                                    }
                                    if let Err(e) = validate_todo_completed(&update_req.completed) {
                                        log_error(e);
                                        return ("400 Bad Request", e.to_string());
                                    }
                                    return update_todo(
                                        id,
                                        update_req.title,
                                        update_req.completed,
                                        db,
                                    );
                                }
                                Err(e) => {
                                    let error = "JSON deserialization error occurred.";
                                    log_error(&format!("Error details: {}", e));
                                    return ("400 Bad Request", error.to_string());
                                }
                            }
                        }
                    }
                    let error = "Invalid ID.";
                    log_error(error);
                    return ("400 Bad Request", error.to_string());
                }
                let error = "Endpoint not found.";
                log_error(error);
                return ("404 Not Found", error.to_string());
            }
            "DELETE" => {
                if path.starts_with("/todos/") {
                    if let Some(id_str) = path.strip_prefix("/todos/") {
                        if let Ok(id) = id_str.parse::<usize>() {
                            return delete_todo(id, db);
                        }
                    }
                    let error = "Invalid ID.";
                    log_error(error);
                    return ("400 Bad Request", error.to_string());
                }
                let error = "Endpoint not found.";
                log_error(error);
                return ("404 Not Found", error.to_string());
            }
            _ => {
                let error = "Method is not allowed.";
                log_error(error);
                return ("405 Method Not Allowed", error.to_string());
            }
        }
    }
    ("400 Bad Request", "Invalid request.".to_string())
}

fn process_request_get_todos(db: Db) -> (&'static str, String) {
    let db = db.lock().unwrap();
    let todos: Vec<&Todo> = db.values().collect();
    let body = serde_json::to_string(&todos).unwrap();
    ("200 OK", body)
}

pub fn get_todo(id: usize, db: Db) -> (&'static str, String) {
    let db = db.lock().unwrap();
    if let Some(todo) = db.get(&id.to_string()) {
        let body = serde_json::to_string(todo).unwrap();
        ("200 OK", body)
    } else {
        let error = "Todo not found.";
        log_error(error);
        ("404 Not Found", error.to_string())
    }
}

pub fn create_todo(title: String, db: Db) -> (&'static str, String) {
    let mut db = db.lock().unwrap();
    let id = db.len() + 1;
    let todo = Todo {
        id,
        title,
        completed: false,
    };
    db.insert(id.to_string(), todo.clone());
    let body = serde_json::to_string(&todo).unwrap();
    ("201 Created", body)
}

pub fn update_todo(
    id: usize,
    title: Option<String>,
    completed: Option<bool>,
    db: Db,
) -> (&'static str, String) {
    let mut db = db.lock().unwrap();
    if let Some(todo) = db.get_mut(&id.to_string()) {
        if let Some(t) = title {
            todo.title = t;
        }
        if let Some(c) = completed {
            todo.completed = c;
        }
        let body = serde_json::to_string(todo).unwrap();
        ("200 OK", body)
    } else {
        let error = "Todo not found.";
        log_error(error);
        ("404 Not Found", error.to_string())
    }
}

pub fn delete_todo(id: usize, db: Db) -> (&'static str, String) {
    let mut db = db.lock().unwrap();
    if db.remove(&id.to_string()).is_some() {
        ("200 OK", "Todo has been deleted.".to_string())
    } else {
        let error = "Todo not found.";
        log_error(error);
        ("404 Not Found", error.to_string())
    }
}

pub fn handle_connection(mut stream: TcpStream, db: Db) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
            let (status, body) = process_request(&request, db);

            let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            let content_length = body.len();
            let response = format!(
                "HTTP/1.1 {}\r\n\
                Date: {}\r\n\
                Content-Type: application/json; charset=UTF-8\r\n\
                Content-Length: {}\r\n\
                Connection: close\r\n\
                \r\n\
                {}",
                status, date, content_length, body
            );

            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to write to stream: {}", e);
                log_error(&format!("Stream write error: {}", e));
            }
        }
        Err(e) => {
            let error = "Failed to read from stream.";
            log_error(&format!("Read error details: {}", e));
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
