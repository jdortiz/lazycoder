//! Command line subcommands and options
//!
//! Options:
//! - level: Select the log level
//!
//! Subcommands:
//! - start: Initialize configuration, setting next position to 0
//! - next: Prints the next snippet to stdout and increments counter
//! - rewind: Decrements counter
//! - forward: Increments counter
//!
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(help_template = "{before-help}{name} {version}
{author-with-newline}{about-section}
{usage-heading} {usage}

{all-args}{after-help}")] // This is required to show the author
pub struct CliArgs {
    /// Verbosity level
    #[clap(short, long, value_enum, value_name = "LEVEL", default_value = "warn")]
    pub level: log::LevelFilter,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Use *FILENAME* to provide snippets
    #[command(visible_alias = "s")]
    Start {
        /// Path to snippet file
        filename: PathBuf,
    },
    /// Provide next snippet
    #[command(visible_alias = "n")]
    Next {},
    /// Rewind *n* snippet(s)
    #[command(visible_alias = "r")]
    Rewind {
        /// Set n (by default is 1)
        count: Option<usize>,
    },
    /// Forward *n* snippet(s)
    #[command(visible_alias = "f")]
    Forward {
        /// Set n (by default is 1)
        count: Option<usize>,
    },
}
