use clap::{Parser, Subcommand};

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
    Rewind { count: Option<u8> },
    Forward { count: Option<u8> },
}

fn main() {
    println!("LazyCoder");

    let value = Value::parse();

    match &value.command {
        Command::Start { filename } => {
            println!("Setting to work {}", filename);
        }
        Command::Next {} => {
            println!("Next");
        }
        Command::Forward { count } => {
            println!("Forward {}", count.unwrap_or(1));
        }
        Command::Rewind { count } => {
            println!("Rewind {}", count.unwrap_or(1));
        }
    }
}
