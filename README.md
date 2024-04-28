# Rust Todo List Application

A simple command-line todo list application written in Rust. This application allows users to add, get, list, delete, and modify tasks, and it uses `serde` for JSON serialization and deserialization, and `tide` for handling HTTP requests.

## Dependencies

- `tide`: An asynchronous web framework for building asynchronous web applications.
- `serde`: A framework for serializing and deserializing Rust data structures efficiently and generically.
- `reqwest`: An asynchronous web client similar to `requests` in Python.
- `once_cell`: A single assignment cell for Rust.

## Build and Run

To build and run the application, follow these steps:

1. Install Rust from [the official site](https://www.rust-lang.org/tools/install).
2. Clone this repository:
   git clone https://github.com/112Cookie/todo-list-app-rust.git

3. Navigate to the project directory:
   cd your-repo-name

4. Start the application using `cargo`:
   cargo run


## Usage

The application runs on two endpoints:

- `http://127.0.0.1:9999/givemesecret`: This endpoint is used to handle the login process.
- `http://127.0.0.1:8000/sendSecretToMe`: This endpoint is used for sending secrets.

Use the following commands to interact with the todo list:

- `add`: Add a new task.
- `get`: Get a specific task by ID.
- `list`: List all tasks.
- `delete`: Delete a task by ID.
- `modify`: Modify a task by ID.
- `exit`: Exit the application.

### Commands

- To add a task, enter `add` and follow the prompts.
- To get a task, enter `get` and provide the task ID.
- To list all tasks, enter `list`.
- To delete a task, enter `delete` and provide the task ID.
- To modify a task, enter `modify` and follow the prompts.
- To exit the application, enter `exit`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
