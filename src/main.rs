use std::fs::File;
use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};


#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
    priority: String,
    due_date: Option<DateTime<Utc>>,
}
#[derive(Parser)]
#[command(name = "Todo CLI")]
#[command(about = "Ein einfaches To-Do-Listen-Programm in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        description: String,
        #[arg(short, long, default_value = "Mittel")]
        priority: String,
        #[arg(short, long)]
        due: Option<String>,
    },
    List,
    Complete {
        id: u32,
    },

    Delete {
        id:u32,
    },
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
let cli = Cli::parse();
let mut tasks = load_tasks()?;

match cli.command {
    Commands::Add { description , priority, due} => {
        let next_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;
        let due_date = if let Some(due_str) = due {
            match NaiveDateTime::parse_from_str(&due_str, "%d.%m.%Y %H:%M") {
                Ok(datetime) => {
                    Some(Utc.from_utc_datetime(&datetime))
                },
                Err(e) => {
                    eprintln!("Ungültiges Datumformat: {}", e);
                    eprintln!("Bitte das Datum im Format DD.MM.YYYY HH:MM angeben.");
                    return Ok(());
                }
            }
        } else {
            None
        };

        let new_task = Task {
            id: next_id,
            description,
            completed: false,
            priority,
            due_date,
        };
        println!("Aufgabe hinzugefügt: [{}] {}", next_id, new_task.description);
        tasks.push(new_task);
        save_tasks(&tasks)?;
    },
    Commands::List => {
        if tasks.is_empty() {
            println!("Keine Aufgaben vorhanden.");
        } else {
            let now = Utc::now();
            for task in &tasks {
                let status = if task.completed { "Erledigt" } else { "Offen" };
                let due_date_str = match task.due_date {
                    Some(ref due_date) => {
                        let overdue = !task.completed && due_date < &now;
                        if overdue {
                            format!("{} (Überfällig!)", due_date.format("%d.%m.%Y %H:%M"))
                        } else {
                            format!("{}", due_date.format("%d-%m-%Y"))
                        }
                    },
                    None => "Kein Fälligkeitsdatum".to_string(),
                };
                println!(
                    "[{}] {} - Status: [{}] Priorität: [{}] - Fällig am: {}",
                    task.id,
                    task.description,
                    status,
                    task.priority,
                    due_date_str,
                );
            }
        }
    },
    Commands::Complete { id } => {
       if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
        task.completed = true;
        println!("Aufgabe als erledigt markiert: [{}] {}", id, task.description);
        save_tasks(&tasks)?;
       } else {
        println!("Keine Aufgabe mit ID {} gefunden.", id);
       }
    },
    Commands::Delete { id } => {
        let len_before = tasks.len();
        tasks.retain(|task| task.id != id);
        if tasks.len() < len_before {
            save_tasks(&tasks)?;
            println!("Aufgabe gelöscht: ID {}", id);
        } else {
            println!("Keine Aufgabe mit ID {} gefunden", id);
        }
    },
}
Ok(())
}

fn load_tasks() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let mut content = String::new();

    match File::open("tasks.json") {
        Ok(mut file) => {
            file.read_to_string(&mut content)?;
            if content.trim().is_empty() {
                Ok(Vec::new())
            } else {
                let tasks: Vec<Task> = serde_json::from_str(&content)?;
                Ok(tasks)
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            Ok(Vec::new())
        },
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(tasks)?;
    let mut file = File::create("tasks.json")?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}
