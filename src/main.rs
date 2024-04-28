use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use serde::{Deserialize, Serialize};
use std::{io, path::Path};
use tide::prelude::*;
use tide::{Request, Response};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use serde_json::json;
use reqwest::Client;
use reqwest::Error;

static RECEIVE_URL: Lazy<String> = Lazy::new(|| String::from("http://127.0.0.1:9999/givemesecret"));
static TARGET_URL: Lazy<String> = Lazy::new(|| String::from("http://127.0.0.1:8000/sendSecretToMe"));
static USER_DATA: Lazy<Arc<Mutex<Option<Secret>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));


#[derive(Debug, Deserialize)]
struct AppRequest {
    receive_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Secret {
    username: String,
    password: String,
}


#[derive(Debug, Deserialize, Serialize)]
struct Task {
    pub id: i32,
    pub title: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tasks {
    pub tasks: Vec<Task>,
}




#[tokio::main]
async fn main() {
    let mut input = String::new();
    tokio::spawn(async {
        let mut app = tide::new();
        app.at("/givemesecret").post(handle_login);
        app.listen("127.0.0.1:9999").await.unwrap();
    });

    loop {
        println!("Enter command (add, get, list, delete, modify, exit)");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();
        let file_path = Path::new("tasks.json");
        let commands = ["add", "get", "list", "delete","modify"];
        if commands.contains(&command) {
            loop{
                let receive_url = &*RECEIVE_URL;
                let target_url = &*TARGET_URL;
                let json_body = json!({"receive_url": receive_url});
                if let Err(e) = post_request(target_url, json_body).await {
                    eprintln!("Failed to post request: {:?}", e);
                    sleep(Duration::from_secs(1));
                }else{
                    if(check_Secret()){break;}
                }
            }
            loop{
                if(check_login()){break;}
                else{
                    println!("Login failed, please try again.");
                };
            }
        }

        match command {
            "add" => add_task(file_path),
            "get" => get_task(file_path),
            "list" => list_task(file_path),
            "delete" => delete_task(file_path),
            "modify" => modify_task(file_path),
            "exit" => {
                println!("Exiting the todo-list app.");
                tokio::signal::ctrl_c().await.unwrap();
                break;
            }
            _ => println!("Unknown command. Available commands: add, get, list, delete, modify，exit"),
        }
        input.clear();
    }
}




impl Task {
    fn update(&mut self, new_title: String, new_status: String) {
        println!("Updating task with id:{},new_title:{},new_status:{}", self.id, new_title,new_status);
        let done = if new_status.eq_ignore_ascii_case("Finished"){
            true
        } else{
            false
        };
        self.done = done;
        self.title = new_title.parse().unwrap();
    }
}

async fn handle_login(mut req: Request<()>) -> tide::Result<Response> {
    let user_data: Secret = req.body_json().await?;
    // println!("Received secret: username: {}, password: {}", user_data.username, user_data.password);
    let mut data_lock = USER_DATA.lock().unwrap();
    *data_lock = Some(user_data);
    Ok(Response::new(200))
}

async fn post_request<T>(url: &String, body: T) -> Result<(), Error>
    where
        T: serde::Serialize,
{
    let client = Client::new();
    let json_body = serde_json::to_string(&body).expect("Serialization failed");
    let response = client
        .post(url)
        .body(reqwest::Body::from(json_body))
        .send()
        .await?;
    if response.status().is_success() {
        // println!("Status: {}", response.status());
        // println!("Headers: {:#?}", response.headers());
        let body = response.text().await?;
        // println!("Body: {}", body);
        Ok(())
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}


pub fn check_Secret() -> bool {
    let data_lock = USER_DATA.lock().unwrap();
    let user_data_option = data_lock.as_ref();
    if let Some(user_data) = user_data_option {
        // println!("User data is not null: {:?}", user_data);
        return true;
    } else {
        // println!("User data is null");
        return false;
    }
}

pub fn check_login() -> bool {
    println!("Please input username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Cannot read username");
    println!("Please input password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Cannot read password");

    let data_lock = USER_DATA.lock().unwrap();
    let user_data_option = data_lock.as_ref();
    if let Some(user_data) = user_data_option {
        if user_data.username.eq(&username.trim().to_string()) && user_data.password.eq(&password.trim().to_string()) {
            return true;
        }else{
            return false;
        }
    } else {
        println!("User data is null");
        return false;
    }
}

pub fn save_task(tasks: &Vec<Task>, file_path: &Path) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, tasks)?;
    Ok(())
}

pub fn load_tasks(file_path: &Path) -> std::io::Result<Vec<Task>> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Ok(vec![]),
    };

    let reader = BufReader::new(file);
    let tasks = match serde_json::from_reader(reader) {
        Ok(tasks) => tasks,
        Err(_) => vec![],
    };
    Ok(tasks)
}


pub fn add_task(file_path: &Path) {
    let tasks = load_tasks(file_path).expect("Cannot load tasks");
    let mut next_id = tasks.iter().map(|task| task.id).max().unwrap_or(0);
    let mut task_list: Vec<Task> = tasks;
    loop {
        // Read task
        println!("Please input the task's title.");
        let mut task_title = String::new();
        io::stdin().read_line(&mut task_title).expect("Cannot read task's title");
        println!("Please input the task's status.Finished or unfinished.");
        let mut task_status = String::new();
        io::stdin().read_line(&mut task_status).expect("Cannot read task's status");
        let mut task_done = false;
        if task_status.trim().eq_ignore_ascii_case("Finished") { task_done = true }

        next_id = next_id + 1;
        let task = Task {
            id: next_id,
            title: task_title.trim().to_string(),
            done: task_done,
        };
        task_list.push(task);

        println!("Continue add Task? Yes or No");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq_ignore_ascii_case("No") {
            break;
        }
    }

    save_task(&task_list, file_path).unwrap();
}

pub fn list_task(file_path: &Path) {
    let tasks = load_tasks(file_path).expect("Failed to load tasks");
    if tasks.is_empty() {
        println!("No tasks found");
        return;
    }
    for task in tasks {
        let done = if task.done {
            "YES"
        } else {
            "NO"
        };
        println!(
            "id: {}, title: {}, done: {}",
            task.id, task.title, done
        );
    }
}

pub fn get_task(file_path: &Path) {
    let mut tasks = load_tasks(file_path).expect("Failed to load tasks");
    println!("Enter the ID of the task you want to get:");
    let mut task_id = String::new();
    io::stdin()
        .read_line(&mut task_id)
        .expect("Failed to read input");
    let task_id: i32 = task_id.trim().parse().expect("Enter valid id");

    let found_task = tasks.iter().find(|task| task.id == task_id);

    match found_task {
        Some(task) => {
            let done = if task.done {
                "YES"
            } else {
                "NO"
            };
            println!(
                "id: {}, title: {}, done: {}",
                task.id, task.title, done
            );
        },
        None => println!("No task found with id {}", task_id),
    }
}


pub fn modify_task(file_path: &Path) {
    let mut tasks = load_tasks(file_path).expect("Failed to load tasks");
    println!("Enter the ID of the task you want to modify:");
    let mut task_id = String::new();
    io::stdin()
        .read_line(&mut task_id)
        .expect("Failed to read input");
    let task_id: i32 = task_id.trim().parse().expect("Enter valid id");

    println!("Enter the new title for the task :");
    let mut new_title = String::new();
    io::stdin()
        .read_line(&mut new_title)
        .expect("Failed to read input");
    let new_title: String = new_title.trim().parse().expect("Enter valid new title");

    println!("Enter the task's status.Finished or unfinished:");
    let mut new_status = String::new();
    io::stdin()
        .read_line(&mut new_status)
        .expect("Failed to read input");
    let new_status: String = new_status.trim().parse().expect("Enter valid new status");

    if let Some(position) = tasks.iter().position(|task| task.id == task_id) {
        tasks[position].update(new_title,new_status);
    } else {
        println!("No task found with id {}", task_id);
    }

    save_task(&tasks, file_path).expect("Failed to save");
}

pub fn delete_task(file_path: &Path) {
    let mut tasks = load_tasks(file_path).expect("Failed to load tasks");

    println!("Enter the ID of the task you want to delete:");
    let mut task_id = String::new();
    io::stdin()
        .read_line(&mut task_id)
        .expect("Failed to read input");
    let task_id: i32 = task_id.trim().parse().expect("Enter valid id");

    let pos = tasks
        .iter()
        .position(|task| task.id == task_id)
        .expect("not found");

    tasks.remove(pos);
    save_task(&tasks, file_path).expect("Failed to save");
}
