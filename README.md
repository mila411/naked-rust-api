# Rust Todo API

This is an experimental project that I created because I wanted to try making an API using only pure Rust, without using external crates like Axum or Actix, like Golang.

## Description

A simple RESTful API server built with Rust for managing Todo items. This API supports creating, reading, updating, and deleting (CRUD) operations, with robust data validation and comprehensive error logging.

## Features

- **Create Todos**: Add new Todo items with titles.
- **Read Todos**: Retrieve all Todos or a specific Todo by ID.
- **Update Todos**: Modify existing Todos' titles and completion status.
- **Delete Todos**: Remove Todos by ID.
- **Data Validation**: Ensures that titles are not empty and the `completed` field is a boolean.
- **Error Logging**: Logs detailed error messages with timestamps to `error.log`.
- **Thread Pool**: Efficiently handles multiple incoming connections using a thread pool.

## Technologies Used

- **Rust**: Programming language used for building the API.
- **Serde**: Serialization and deserialization framework.
- **TCP Networking**: Utilizes `TcpListener` and `TcpStream` for handling HTTP requests.
- **Threading**: Implements a thread pool for managing concurrent connections.
- **Chrono**: Handles date and time for logging purposes.

## Installation

### Prerequisites

- **Rust**: Ensure Rust is installed. If not, install via [rustup](https://rustup.rs/).

### Clone the Repository

```bash
git clone https://github.com/yourusername/rust-todo-api.git
cd rust-todo-api
```

### Build the Project

`cargo build --release`

## Usage

### Running the Server

`cargo run --release`

The server will start and listen on [http://127.0.0.1:8080](http://127.0.0.1:8080)

### API Endpoints

- **URL:** `/todos`
- **Method**: `GET`
- **Response:** JSON array of Todo items.

### Get a Specific Todo

- **URL:** `/todos/{id}`
- **Method:** `GET`
- **Response:** JSON object of the Todo item.

### Create a Todo

- **URL:** `/todos`
- **Method:** `POST`
- **Headers:**
  - Content-Type: application/json
  - Content-Length: {length}
- **Body:**

```json
{
  "title": "Learn Rust"
}
```

- **Response:** JSON object of the created Todo item.

### Update a Todo

- **URL:** `/todos/{id}`
- **Method:** `PUT`
- **Headers:**
  - `Content-Type: application/json`
  - `Content-Length: {length}`
- **Body:**

```json
{
  "title": "Learn Advanced Rust",
  "completed": true
}
```

- **Response:** JSON object of the updated Todo item.

### Delete a Todo

- **URL:** `/todos/{id}`
- **Method:** `DELETE`
- **Response:** Message indicating successful deletion.

## Testing

Running Tests
Execute the test suite using the following command:

`cargo test`

## Error Handling and Logging

All errors are logged to error.log with timestamps for easy troubleshooting. The API provides detailed error messages to clients, ensuring clarity on what went wrong.
