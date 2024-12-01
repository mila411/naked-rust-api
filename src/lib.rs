use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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
        assert!(
            size > 0,
            "The size of the thread pool must be greater than 1."
        );

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
        println!("The job is sent to the thread pool.");
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
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} has received the job. Running.", id);
                job();
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn validate_todo_title(title: &str) -> Result<(), &'static str> {
    if title.trim().is_empty() {
        Err("The title cannot be left blank.")
    } else {
        Ok(())
    }
}

fn validate_todo_completed(completed: &Option<bool>) -> Result<(), &'static str> {
    if let Some(_) = completed {
        Ok(())
    } else {
        Err("The completed field must be of type bool.")
    }
}

pub fn process_request(request: &str, db: Db) -> (&'static str, String) {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            return (
                "400 Bad Request",
                "The request line is invalid.".to_string(),
            );
        }
        let method = parts[0];
        let path = parts[1];
        let version = parts[2];

        if version != "HTTP/1.1" && version != "HTTP/1.0" && version != "HTTP/2.0" {
            return (
                "505 HTTP Version Not Supported",
                "HTTP version not supported".to_string(),
            );
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
                return (
                    "400 Bad Request",
                    "The header format is invalid.".to_string(),
                );
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
                    return ("400 Bad Request", "Invalid ID".to_string());
                }
                return ("404 Not Found", "No endpoint found".to_string());
            }
            "POST" => {
                if path == "/todos" {
                    match serde_json::from_str::<Value>(body) {
                        Ok(json) => {
                            if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                                if let Err(e) = validate_todo_title(title) {
                                    return ("400 Bad Request", e.to_string());
                                }
                                let title = title.to_string();
                                return create_todo(title, db);
                            } else {
                                return ("400 Bad Request", "A title is required.".to_string());
                            }
                        }
                        Err(_) => {
                            return ("400 Bad Request", "JSON format is invalid.".to_string());
                        }
                    }
                }
                return ("404 Not Found", "No endpoint found".to_string());
            }
            "PUT" => {
                if path.starts_with("/todos/") {
                    if let Some(id_str) = path.strip_prefix("/todos/") {
                        if let Ok(id) = id_str.parse::<usize>() {
                            match serde_json::from_str::<UpdateTodoRequest>(body) {
                                Ok(update_req) => {
                                    if let Some(ref title) = update_req.title {
                                        if let Err(e) = validate_todo_title(title) {
                                            return ("400 Bad Request", e.to_string());
                                        }
                                    }
                                    if let Err(e) = validate_todo_completed(&update_req.completed) {
                                        return ("400 Bad Request", e.to_string());
                                    }
                                    return update_todo(
                                        id,
                                        update_req.title,
                                        update_req.completed,
                                        db,
                                    );
                                }
                                Err(_) => {
                                    return (
                                        "400 Bad Request",
                                        "JSON format is invalid.".to_string(),
                                    );
                                }
                            }
                        }
                    }
                    return ("400 Bad Request", "Invalid ID".to_string());
                }
                return ("404 Not Found", "No endpoint found".to_string());
            }
            "DELETE" => {
                if path.starts_with("/todos/") {
                    if let Some(id_str) = path.strip_prefix("/todos/") {
                        if let Ok(id) = id_str.parse::<usize>() {
                            return delete_todo(id, db);
                        }
                    }
                    return ("400 Bad Request", "Invalid ID".to_string());
                }
                return ("404 Not Found", "No endpoint found".to_string());
            }
            _ => {
                return (
                    "405 Method Not Allowed",
                    "The method is not allowed.".to_string(),
                );
            }
        }
    }
    ("400 Bad Request", "The request is invalid.".to_string())
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
        ("404 Not Found", "I can't find Todo.".to_string())
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
        ("404 Not Found", "can't find Todo.".to_string())
    }
}

pub fn delete_todo(id: usize, db: Db) -> (&'static str, String) {
    let mut db = db.lock().unwrap();
    if db.remove(&id.to_string()).is_some() {
        ("200 OK", "Todo has been deleted.".to_string())
    } else {
        ("404 Not Found", "can't find Todo.".to_string())
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
                eprintln!("Failed to write to the stream.: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
