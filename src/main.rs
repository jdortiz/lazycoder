mod config;
mod lazy_coder_error;
mod snippet_handler;

use clap::{Parser, Subcommand};
use config::Config;
use std::process::exit;

// lazycoder start /filepath/demo.md
// - works with only one demo at a time
// - save initial next position: 0
// - file name
// - config. saved in ~/.lazycoder
// lazycoder next
// - reads from config file
// - reads next snippet
// - incs pointer to next snippet
// lazycoder rewind [number]
// - decs pointer (number times)
// - returns nothing
// lazycoder forward [number]
// - inc pointer (number times)
// - returns nothing
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
struct Value {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start { filename: String },
    Next {},
    Rewind { count: Option<usize> },
    Forward { count: Option<usize> },
}

fn main() {
    let value = Value::parse();

    match &value.command {
        Command::Start { filename } => {
            start(filename);
        }
        Command::Next {} => {
            next();
        }
        Command::Forward { count } => {
            let count = count.unwrap_or(1);
            forward(count);
        }
        Command::Rewind { count } => {
            let count = count.unwrap_or(1);
            rewind(count);
        }
    }
}

fn start(filename: &str) {
    println!("Setting to work {}", filename);
    match Config::new(filename) {
        Ok(_) => {
            eprintln!("Configuration successfully created");
            exit(0);
        }
        Err(err) => {
            eprintln!("Failed to create configuration: {}", err);
            exit(1);
        }
    }
}

fn next() {
    eprintln!("Next");
    match Config::read() {
        Ok(mut cfg) => {
            match cfg.next() {
                Ok(snippet) => {
                    print!("{snippet}");
                    exit(0);
                }
                Err(err) => {
                    eprintln!("Failed to obtain next snippet: {}", err);
                    exit(1);
                }
            };
        }
        Err(err) => {
            eprintln!("Failed to obtain next snippet: {}", err);
            exit(1);
        }
    };
}

fn forward(count: usize) {
    eprintln!("Forward {}", count);
    match Config::read() {
        Ok(mut cfg) => {
            if let Err(err) = cfg.forward(count) {
                eprintln!("Failed to obtain next snippet: {}", err);
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(err) => {
            eprintln!("Failed to foward: {}", err);
            exit(1);
        }
    };
}

fn rewind(count: usize) {
    eprintln!("Rewind {}", count);
    match Config::read() {
        Ok(mut cfg) => {
            if let Err(err) = cfg.rewind(count) {
                eprintln!("Failed to obtain next snippet: {}", err);
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(err) => {
            eprintln!("Failed to rewind: {}", err);
            exit(1);
        }
    };
}
