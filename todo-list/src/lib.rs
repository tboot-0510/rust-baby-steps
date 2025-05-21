use colored::*;
use std::env;
use std::io::prelude::Read;
use std::io::{BufWriter, Write};
use std::process;

mod file_utils {
    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, BufWriter};

    pub fn open_file_and_append(path: &str) -> BufWriter<File> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("Couldn't open the todofile");

        BufWriter::new(file)
    }

    pub fn create_file(path: &str) -> BufReader<File> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .expect("Failed to open file");

        BufReader::new(file)
    }

    pub fn open_file_and_truncate(path: &str) -> BufWriter<File> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .expect("Couldn't open the todofile");

        BufWriter::new(file)
    }
}

use file_utils::{create_file, open_file_and_append, open_file_and_truncate};

pub struct Todo {
    pub todo: Vec<String>,
    pub path: String,
}
pub struct Entry {
    pub checked: bool,
    pub element: String,
}

impl Entry {
    pub fn new(element: String, checked: bool) -> Self {
        Entry { element, checked }
    }

    pub fn read_line(line: &str) -> Self {
        let (checked, element) = line.split_at(4);
        Entry {
            checked: checked == "[*] ",
            element: element.to_string(),
        }
    }

    pub fn show_line(&self, number: usize) -> String {
        let element = if self.checked {
            self.element.strikethrough().to_string()
        } else {
            self.element.clone()
        };

        format!("{} {} \n", number, element)
    }

    pub fn save(self) -> String {
        let symbol = if self.checked { "[*] " } else { "[ ] " };
        format!("{}{}\n", symbol, self.element)
    }

    pub fn replace(self, new_element: &str) -> Self {
        Entry {
            checked: self.checked,
            element: new_element.to_string(),
        }
    }
}

impl Todo {
    pub fn new() -> Self {
        let home = env::var("HOME").unwrap();
        println!("home {:?}", home);

        // Create a file in the current working directory
        let current_dir = env::current_dir().unwrap();
        let current_file_path = format!("{}/data/example.txt", current_dir.display());

        let mut buf_reader = create_file(&current_file_path);

        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        let todo = contents.lines().map(str::to_string).collect();

        Todo {
            todo,
            path: current_file_path,
        }
    }

    pub fn list(&self) {
        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());

        let list_todo: Vec<String> = self
            .todo
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let entry = Entry::read_line(&line);
                entry.show_line(idx + 1)
            })
            .collect();

        let _ = writer.write(list_todo.join("").as_bytes());
    }

    pub fn done(self, args: &[String]) {
        if args.is_empty() || args.len() != 1 {
            println!("args can not be empty");
            process::exit(1);
        }

        let line_number: usize = self.verify_number(args);

        let mut buffer = open_file_and_truncate(&self.path);

        let update_todo: Vec<String> = self
            .todo
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let mut entry = Entry::read_line(line);
                if (idx + 1) == line_number {
                    entry.checked = !entry.checked;
                    return entry.save();
                };
                entry.save()
            })
            .collect();

        let _ = buffer.write_all(update_todo.join("").as_bytes());
    }

    pub fn add(self, args: Vec<String>) {
        if args.is_empty() {
            println!("args can not be empty");
            process::exit(1);
        }

        let mut buffer = open_file_and_append(&self.path);

        for arg in args {
            let entry = Entry::new(arg, false);
            let _ = buffer.write_all(entry.save().as_bytes());
        }

        self.list()
    }

    pub fn edit(&self, args: &[String]) {
        if args.is_empty() || args.len() != 2 {
            println!("args can not be empty");
            process::exit(1);
        }

        let line_number: usize = self.verify_number(args);

        let mut buffer = open_file_and_truncate(&self.path);

        let edit_todo: Vec<String> = self
            .todo
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let entry = Entry::read_line(line);
                if (idx + 1) == line_number {
                    return entry.replace(&args[1]).save();
                };
                entry.save()
            })
            .collect();

        let _ = buffer.write_all(edit_todo.join("").as_bytes());
    }

    pub fn remove(&self, args: &[String]) {
        if args.is_empty() || args.len() != 1 {
            println!("args can not be empty");
            process::exit(1);
        }

        let line_number: usize = self.verify_number(args);

        let mut buffer = open_file_and_truncate(&self.path);

        let removed_todo: Vec<String> = self
            .todo
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let entry = Entry::read_line(line);
                if (idx + 1) == line_number {
                    return None;
                };
                Some(entry.save())
            })
            .collect();

        let _ = buffer.write_all(removed_todo.join("").as_bytes());
    }

    pub fn reset(self) {
        let mut buffer = open_file_and_truncate(&self.path);

        let _ = buffer.write_all(String::new().as_bytes());
    }

    pub fn sort(self) {
        let stdout = std::io::stdout();
        let mut buffer = BufWriter::new(stdout.lock());

        let mut entries: Vec<Entry> = self
            .todo
            .iter()
            .map(|line| Entry::read_line(line))
            .collect();

        entries.sort_by(|a, b| a.checked.cmp(&b.checked).then(a.element.cmp(&b.element)));

        let sorted_lines: Vec<String> = entries.into_iter().map(|entry| entry.save()).collect();

        let _ = buffer.write_all(sorted_lines.join("").as_bytes());
    }

    pub fn raw(self, args: &[String]) {
        if args.is_empty() || args.len() > 1 {
            print!("args not valid");
            process::exit(1);
        }

        let listed_by_value: Vec<String> = match args[0].as_str() {
            "done" => self
                .todo
                .iter()
                .filter(|line| Entry::read_line(line).checked)
                .map(|line| line.to_string())
                .collect(),
            "todo" => self
                .todo
                .iter()
                .filter(|line| !Entry::read_line(line).checked)
                .map(|line| line.to_string())
                .collect(),
            _ => {
                println!("raw value must be either 'done' or 'todo'");
                process::exit(1);
            }
        };

        let stdout = std::io::stdout();
        let mut buffer = BufWriter::new(stdout.lock());

        let _ = buffer.write_all(listed_by_value.join("\n").as_bytes());
    }

    fn verify_number(&self, args: &[String]) -> usize {
        let line_number: usize = args[0].parse().unwrap_or_else(|_| {
            println!("1st element must be a valid number");
            process::exit(1);
        });

        if line_number == 0 || line_number > self.todo.len() {
            println!("1st element number is out of bound");
            process::exit(1);
        }

        line_number
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
    - add [TASK/s]
        adds new task/s
        Example: todo add \"buy carrots\"
    - edit [INDEX] [EDITED TASK/s]
        edits an existing task/s
        Example: todo edit 1 banana
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX]
        removes a task
        Example: todo rm 4
    - reset
        deletes all tasks
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort
    - raw [todo/done]
        prints nothing but done/incompleted tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{:?}", TODO_HELP);
}
