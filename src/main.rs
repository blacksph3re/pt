use std::env;
use std::fs::{OpenOptions, File};
use fs2::FileExt;
use std::io::{self, Write, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use notify_rust::{Notification, Timeout};
use rodio::{Decoder, OutputStream, Sink};

const TASK_FILE: &str = "tasks.json";
const POMODORO_DURATION: Duration = Duration::from_secs(25 * 60);

struct NotificationContent {
    title: String,
    body: String,
}

#[derive(Clone)]
#[derive(Serialize)]
#[derive(Deserialize)]
struct Pomodoro {
    start_time: SystemTime,
    end_time: Option<SystemTime>,
}

#[derive(Clone)]
#[derive(Serialize)]
#[derive(Deserialize)]
struct Task {
    id: u32,
    description: String,
    done: bool,
    archived: bool,
    pomodoros: Vec<Pomodoro>,
}

impl Task {
    fn new(id: u32, description: String) -> Task {
        Task {
            id,
            description,
            done: false,
            archived: false,
            pomodoros: Vec::new(),
        }
    }

    fn time_spent(&self) -> Duration {
        let mut time = Duration::new(0, 0);
        for pomodoro in &self.pomodoros {
            match pomodoro.end_time {
                Some(end_time) => time += end_time.duration_since(pomodoro.start_time).unwrap(),
                None => time += pomodoro.start_time.elapsed().unwrap(),
            }
        }
        time
    }

    fn pomodoro_time_remaining(&self) -> Option<Duration> {
        match self.pomodoros.last() {
            Some(pomodoro) => {
                match pomodoro.end_time {
                    Some(_end_time) => None,
                    None => Some(POMODORO_DURATION - pomodoro.start_time.elapsed().unwrap()),
                }
            },
            None => None,
        }
    }

    fn pomodoro_active(&self) -> bool {
        match self.pomodoros.last() {
            Some(pomodoro) => {
                match pomodoro.end_time {
                    Some(_end_time) => false,
                    None => true,
                }
            },
            None => false,
        }
    }
}

fn main() {
    let mut file = open_file();
    let mut tasks = read_tasks(&mut file);
    let mut notifications: Vec<NotificationContent> = Vec::new();

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
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => start_pomodoro(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        },
        "--finish-pomodoro" | "-f" => {
            if args.len() < 3 {
                println!("No task ID specified.");
                return;
            }
            for arg in args.iter().skip(2) {
                match arg.parse::<u32>() {
                    Ok(id) => finish_pomodoro(id, &mut tasks),
                    Err(_) => {
                        println!("Invalid task ID {}.", arg);
                        return;
                    }
                }
            };
            list_tasks(&tasks, false);
        },
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
        "--notify" => {
            compute_notifications(&mut tasks, &mut notifications);
        }
        "--help" | "-h" => {
            println!("Usage: task [command] [arguments]");
            println!("Commands:");
            println!("  [no command]                List all tasks");
            println!("  [no command] [description]  Add a new task with the specified description");
            println!("  -p, --pomodoro [task ID]    Start a pomodoro for the specified task");
            println!("  -f, --finish-pomodoro [task ID] Finish the pomodoro for the specified task");
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

    display_notifications(notifications);
}

fn start_pomodoro(task_id: u32, tasks: &mut Vec<Task>) {
    // Update task time spent
    match tasks.iter_mut().find(|task| task.id == task_id) {
        Some(t) => {
            if t.pomodoro_active() {
                println!("Pomodoro already active for task {}.", task_id);
                return;
            }

            t.pomodoros.push(Pomodoro {
                start_time: SystemTime::now(),
                end_time: None,
            });
            println!("Pomodoro started for task {}.", task_id);
        },
        None => {
            println!("Task {} not found.", task_id);
            return;
        }
    };
}

fn finish_pomodoro(task_id: u32, tasks: &mut Vec<Task>) {
    // Update task time spent
    match tasks.iter_mut().find(|task| task.id == task_id) {
        Some(t) => {
            match t.pomodoros.last_mut() {
                Some(p) => {
                    match p.end_time {
                        Some(_) => {
                            println!("No pomodoro active for task {}.", task_id);
                        },
                        None => {
                            p.end_time = Some(SystemTime::now());
                            println!("Pomodoro finished for task {}.", task_id);
                        },
                    }
                },
                None => {
                    println!("No pomodoros found for task {}.", task_id);
                }
            }
        },
        None => {
            println!("Task {} not found.", task_id);
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
        let time = match task.pomodoro_time_remaining() {
            None => format!("Σ{} min", task.time_spent().as_secs() / 60),
            Some(t) => format!("{}m {:0>2}s", t.as_secs() / 60, t.as_secs() % 60),
        };
        let task_str = format!("{:0>3} [{}]: {} ({})", task.id, status, task.description, time);
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

fn compute_notifications(tasks: &mut Vec<Task>, notifications: &mut Vec<NotificationContent>) {
    for task in tasks {
        match task.pomodoro_time_remaining() {
            Some(t) => {
                if t.as_secs() <= 0 {
                    task.pomodoros.last_mut().unwrap().end_time = Some(task.pomodoros.last().unwrap().start_time + POMODORO_DURATION);
                    notifications.push(NotificationContent {
                        title: format!("Pomodoro finished for task {}.", task.id),
                        body: task.description.clone(),
                    });
                }
            },
            None => {},
        }
    }
}

fn display_notifications(notifications: Vec<NotificationContent>) {
    for notification in &notifications {
        println!("{}: {}", notification.title, notification.body);
        match Notification::new()
            .summary(&notification.title)
            .body(&notification.body)
            .timeout(Timeout::Milliseconds(6000)) //milliseconds
            .show() {
                Ok(_) => {},
                Err(e) => println!("Failed to display notification: {}", e),
            }
    }
    if !notifications.is_empty() {
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = io::BufReader::new(File::open("alarm.mp3").unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // Play the sound directly on the device
        sink.append(source);

        // The sound plays in a separate thread. This call will block the current thread until the sink
        // has finished playing all its queued sounds.
        sink.sleep_until_end();
    }
    
}