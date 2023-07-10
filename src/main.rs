use std::env;
use std::fs::{OpenOptions, File};
use fs2::FileExt;
use std::io::{self, Write, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;
use serde::{Serialize, Deserialize};

const TASK_FILE: &str = "tasks.json";

#[derive(Clone)]
#[derive(Serialize)]
#[derive(Deserialize)]
struct Task {
    id: u32,
    description: String,
    time_spent: u32,
    done: bool,
    archived: bool,
}

impl Task {
    fn new(id: u32, description: String) -> Task {
        Task {
            id,
            description,
            time_spent: 0,
            done: false,
            archived: false,
        }
    }
}

fn main() {
    let mut file = open_file();
    let mut tasks = read_tasks(&mut file);

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        list_tasks(&tasks, false);
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "--pomodoro" | "-p" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            let task_id = match args[2].parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    println!("Invalid task ID.");
                    return;
                }
            };
            start_pomodoro(task_id, &mut tasks);
        }
        "--list" | "-l" => list_tasks(&tasks, false),
        "--list-archived" => list_tasks(&tasks, true),
        "--check" | "-c" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => check_task(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        }
        "--uncheck" | "-u" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => uncheck_task(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        }
        "--archive" | "-a" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => archive_task(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        }
        "--unarchive" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => unarchive_task(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        }
        "--help" | "-h" => {
            println!("Usage: task [command] [arguments]");
            println!("Commands:");
            println!("  [no command]                List all tasks");
            println!("  [no command] [description]  Add a new task with the specified description");
            println!("  -p, --pomodoro [task ID]    Start a pomodoro for the specified task");
            println!("  -l, --list                  List all tasks");
            println!("  --list-archived             List all archived tasks");
            println!("  -c, --check [task ID]       Check the specified task");
            println!("  -u, --uncheck [task ID]     Uncheck the specified task");
            println!("  -a, --archive [task ID]     Archive the specified task");
            println!("  --unarchive [task ID]       Unarchive the specified task");
            println!("  -h, --help                  Display this help message");
            
        }
        _ => {
            // Assume the user is adding a new task
            let description = args[1..].join(" ");
            add_task(description, &mut tasks);
            list_tasks(&tasks, false);
        }
    }

    write_tasks(&tasks, &mut file);
    drop(file);
}

fn start_pomodoro(task_id: u32, tasks: &mut Vec<Task>) {
    // Start timer for 25 minutes (1500 seconds)
    let timer = 1500;

    // Update task time spent
    match tasks.iter_mut().find(|task| task.id == task_id) {
        Some(t) => {
            t.time_spent += timer;
            println!("Pomodoro started for task {}.", task_id);
        },
        None => {
            println!("Task {} not found.", task_id);
            return;
        }
    };
}

fn list_tasks(tasks: &[Task], list_archived: bool) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    for task in tasks {
        if task.archived != list_archived {
            continue;
        }
        let status = if task.done { "x" } else { " " };
        let task_str = format!("{:0>3} [{}]: {} (Σ{} min)", task.id, status, task.description, task.time_spent / 60);
        println!("{}", task_str);
    }
}

fn check_task(task_id: u32, tasks: &mut Vec<Task>) {
    let task = tasks.iter_mut().find(|task| task.id == task_id);
    match task {
        Some(t) => {
            t.done = true;
            println!("Task {} checked.", t.id);
        }
        None => {
            println!("Task {} not found.", task_id);
        }
    }
}

fn uncheck_task(task_id: u32, tasks: &mut Vec<Task>) {
    let task = tasks.iter_mut().find(|task| task.id == task_id);
    match task {
        Some(t) => {
            t.done = false;
            println!("Task {} unchecked.", t.id);
        }
        None => {
            println!("Task {} not found.", task_id);
        }
    }
}

fn archive_task(task_id: u32, tasks: &mut Vec<Task>) {
    let task = tasks.iter_mut().find(|task| task.id == task_id);
    match task {
        Some(t) => {
            t.archived = true;
            println!("Task {} moved to archive.", t.id);
        }
        None => {
            println!("Task {} not found.", task_id);
        }
    }
}

fn unarchive_task(task_id: u32, tasks: &mut Vec<Task>) {
    let task = tasks.iter_mut().find(|task| task.id == task_id);
    match task {
        Some(t) => {
            t.archived = false;
            println!("Task {} moved out of archive.", t.id);
        }
        None => {
            println!("Task {} not found.", task_id);
        }
    }
}

fn add_task(description: String, tasks: &mut Vec<Task>) {
    let next_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
    let task = Task::new(next_id, description);
    tasks.push(task);
    println!("Task {} added.", next_id);
}

fn open_file() -> File {
    let path = Path::new(TASK_FILE);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .expect("Failed to open task file.");
    
    file.lock_exclusive()
        .expect("Failed to lock task file.");

    file
}

fn read_tasks(file: &mut File) -> Vec<Task> {
    if file.metadata().unwrap().len() == 0 {
        return Vec::new();
    }
    let reader = io::BufReader::new(file);
    let tasks: Vec<Task> = serde_json::from_reader(reader).expect("Failed to parse task file.");
    tasks
}

fn write_tasks(tasks: &[Task], file: &mut File) {
    file.set_len(0).expect("Failed to truncate task file.");
    file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of task file.");
    let serialized_tasks = serde_json::to_string_pretty(tasks).expect("Failed to serialize tasks.");
    let mut writer = io::BufWriter::new(file);
    writer
        .write_all(serialized_tasks.as_bytes())
        .expect("Failed to write tasks.");
}
