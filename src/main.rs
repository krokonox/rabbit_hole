use std::path::{Path, PathBuf};

extern crate chrono;
extern crate clap;
extern crate rprompt;
extern crate std;
extern crate colored;

use chrono::{Date, Local, NaiveDate};
use clap::{App, Arg};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::fs::{self, File};

use serde::de::DeserializeOwned;
use std::error::Error;
use std::io::BufReader;
use colored::*;

mod utility;

struct HabitClt {
    habit_list: Vec<Habit>,
    entries: Vec<Entry>,
    entries_file: PathBuf,
    habits_file: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
struct Habit {
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    habit: Habit,
    #[serde(serialize_with = "utility::helper_functions::serialize_date")]
    #[serde(deserialize_with = "utility::helper_functions::deserialize_date")]
    date: NaiveDate,
    value: String,
} 

fn main() {
    let matches = App::new("My Program")
        .arg(Arg::with_name("command").multiple(true))
        .get_matches();

    let mut habit_clt = HabitClt::new();

    if let Some(command) = matches.values_of("command") {
        let words: Vec<_> = command.collect();
        process_command(&mut habit_clt, words);
    }
}

fn process_command(habit_clt: &mut HabitClt, words: Vec<&str>) {
    if let Some(first_word) = words.first() {
        match *first_word {
            "new" => process_new_command(habit_clt, words),
            "habits" => process_habits_command(habit_clt),
            "list" => process_list_command(habit_clt),
            "log" => process_log_command(habit_clt),
            "delete" => process_delete_command(habit_clt, words),
            "delete all" => process_delete_all_command(habit_clt),
            _ => println!("Unknown command"),
        }
    }
}

fn process_new_command(habit_clt: &mut HabitClt, words: Vec<&str>) {
    let habit_name = words[1..].join(" ");
    habit_clt.add_habit(habit_name);
    let habit_list = habit_clt.habit_list.clone();
    let habits_file = habit_clt.habits_file.clone();
    habit_clt.save_to_file(habit_list, &habits_file);
}

fn process_habits_command(habit_clt: &mut HabitClt) {
    for (i, habit) in habit_clt.habit_list.iter().enumerate() {
        let color = utility::helper_functions::get_random_color();
        println!("{}. {}", i + 1, habit.name.color(color));
    }
}

fn process_list_command(habit_clt: &mut HabitClt) {
    for entry in &habit_clt.entries {
        println!("{}: {}, {}", entry.habit.name, entry.date, entry.value.yellow());
    }
}

fn process_log_command(habit_clt: &mut HabitClt) {
    habit_clt.add_entry();
    let entries = habit_clt.entries.clone();
    let entries_file = habit_clt.entries_file.clone();
    habit_clt.save_to_file(entries, &entries_file);
}

fn process_delete_command(habit_clt: &mut HabitClt, words: Vec<&str>) {
    let habit_name = words[1..].join(" ");
    habit_clt.delete_habit(&habit_name);
    let habit_list = habit_clt.habit_list.clone();
    let habits_file = habit_clt.habits_file.clone();
    habit_clt.save_to_file(habit_list, &habits_file);
    let entries = habit_clt.entries.clone();
    let entries_file = habit_clt.entries_file.clone();
    habit_clt.save_to_file(entries, &entries_file);
}

fn process_delete_all_command(habit_clt: &mut HabitClt) {
    habit_clt.habit_list.clear();
    let habit_list = habit_clt.habit_list.clone();
    let habits_file = habit_clt.habits_file.clone();
    habit_clt.save_to_file(habit_list, &habits_file);
}

impl HabitClt {
    fn new() -> HabitClt {
        let entries_dir = Path::new("entries");
        if !entries_dir.is_dir() {
            fs::create_dir(entries_dir).unwrap();
        }

        let mut entries_file = entries_dir.join("entries.json");
        let mut habits_file = entries_dir.join("habits.json");

        if !habits_file.is_file() {
            fs::File::create(&habits_file).unwrap();
        }

        if !entries_file.is_file() {
            fs::File::create(&entries_file).unwrap();
        }

        let habit_list: Vec<Habit> = match Self::load_from_file(&habits_file) {
            Ok(habits) => habits,
            Err(_) => Vec::new(),
        };
        let entries: Vec<Entry> = match Self::load_from_file(&entries_file) {
            Ok(entries) => entries,
            Err(_) => Vec::new(),
        };

        HabitClt {
            habit_list,
            entries,
            entries_file,
            habits_file,
        }
    } 
    
    fn add_entry(&mut self) {
        let habit_name = rprompt::prompt_reply_stdout("Enter habit name: ").unwrap();
        let habit_value = rprompt::prompt_reply_stdout("Enter habit value: ").unwrap();
    
        if !self.contains_habit(&habit_name) {
            let add_habit = rprompt::prompt_reply_stdout("Habit not found. Do you want to add a new one? (yes/no): ").unwrap();
            if add_habit.to_lowercase() == "yes" {
                self.add_habit(habit_name.clone());
                let habit_list = self.habit_list.clone();
                let habits_file = self.habits_file.clone();
                self.save_to_file(habit_list, &habits_file);
            }
        }

        let entry = Entry {
            habit: Habit {
                name: habit_name,
            },
            date: Local::today().naive_local(),
            value: habit_value,
        };
        self.entries.push(entry);
    }
    
    fn add_habit(&mut self, habit_name: String) {
        if self.habit_list.iter().any(|habit| habit.name == habit_name) {
            println!("Habit already in the list!");
            return;
        }
    
        let habit = Habit {
            name: habit_name,
        };
        self.habit_list.push(habit);
    }       
    
    fn delete_habit(&mut self, habit_name: &str) {
        self.habit_list.retain(|habit| habit.name != habit_name);    
        self.entries.retain(|entry| entry.habit.name != habit_name);
    }
    
    // ************** File operations **************

    fn save_to_file<T: Serialize>(&mut self, data: T, file_path: &Path) {
        let json_data = serde_json::to_string(&data).unwrap();
        fs::write(file_path, json_data).unwrap();
    }

    fn load_from_file<T: DeserializeOwned>(file_path: &Path) -> Result<T, Box<dyn Error>> {
        if let Ok(metadata) = fs::metadata(file_path) {
            if metadata.len() == 0 {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "The file is empty.",
                )));
            }
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }

    fn contains_habit(&self, name: &str) -> bool {
        self.habit_list.iter().any(|habit| habit.name == name)
    }
}
