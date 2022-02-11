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
    #[clap(short, long, parse(from_occurrences), help = "Verbosity level")]
    verbose: u8,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Use <FILENAME> to provide snippets")]
    Start {
        #[clap(help = "Path to snippet file")]
        filename: String,
    },
    #[clap(about = "Provide next snippet")]
    Next {},
    #[clap(about = "Rewind [n] snippet(s)")]
    Rewind {
        #[clap(help = "n")]
        count: Option<usize>,
    },
    #[clap(about = "Forward [n] snippet(s)")]
    Forward {
        #[clap(help = "n")]
        count: Option<usize>,
    },
}

fn main() {
    let value = Value::parse();

    match &value.command {
        Command::Start { filename } => {
            start(filename, value.verbose);
        }
        Command::Next {} => {
            next(value.verbose);
        }
        Command::Forward { count } => {
            let count = count.unwrap_or(1);
            forward(count, value.verbose);
        }
        Command::Rewind { count } => {
            let count = count.unwrap_or(1);
            rewind(count, value.verbose);
        }
    }
}

fn start(filename: &str, verbose_level: u8) {
    if verbose_level > 0 {
        println!("Setting to work {}", filename);
    }
    match Config::new(filename, verbose_level) {
        Ok(_) => {
            if verbose_level > 0 {
                eprintln!("Configuration successfully created.");
            }
            exit(0);
        }
        Err(err) => {
            eprintln!("Failed to create configuration: {err}.");
            exit(1);
        }
    }
}

fn next(verbose_level: u8) {
    if verbose_level > 0 {
        eprintln!("Next");
    }
    match Config::read(verbose_level) {
        Ok(mut cfg) => {
            match cfg.next(verbose_level) {
                Ok(snippet) => {
                    print!("{snippet}");
                    exit(0);
                }
                Err(err) => {
                    eprintln!("Failed to obtain next snippet: {err}.");
                    exit(1);
                }
            };
        }
        Err(err) => {
            eprintln!("Failed to obtain next snippet: {err}.");
            exit(1);
        }
    };
}

fn forward(count: usize, verbose_level: u8) {
    if verbose_level > 0 {
        eprintln!("Forward {count}");
    }
    match Config::read(verbose_level) {
        Ok(mut cfg) => {
            if let Err(err) = cfg.forward(count, verbose_level) {
                eprintln!("Failed to foward: {err}.");
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(err) => {
            eprintln!("Failed to foward: {err}.");
            exit(1);
        }
    };
}

fn rewind(count: usize, verbose_level: u8) {
    if verbose_level > 0 {
        eprintln!("Rewind {}", count);
    }
    match Config::read(verbose_level) {
        Ok(mut cfg) => {
            if let Err(err) = cfg.rewind(count, verbose_level) {
                eprintln!("Failed to rewind: {err}.");
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(err) => {
            eprintln!("Failed to rewind: {err}.");
            exit(1);
        }
    };
}
