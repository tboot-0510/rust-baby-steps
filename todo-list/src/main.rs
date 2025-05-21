pub use rust_projects::{help, Todo};
use std::env;

fn split_args(args: &[String]) -> Vec<String> {
    let joined = args.join(" ");
    joined
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn main() {
    let todo = Todo::new();

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("run todo");
        return;
    }

    let command = &args[2];
    match command.as_ref() {
        "list" => todo.list(),
        "done" => todo.done(&args[3..]),
        "edit" => todo.edit(&args[3..]),
        "rm" => todo.remove(&args[3..]),
        "raw" => todo.raw(&args[3..]),
        "reset" => todo.reset(),
        "sort" => todo.sort(),
        "add" => todo.add(split_args(&args[3..])),
        "help" | "--help" | "-h" | _ => help(),
    }
}
