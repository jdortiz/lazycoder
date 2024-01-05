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
use std::fmt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(help_template = "{before-help}{name} {version}
{author-with-newline}{about-section}
{usage-heading} {usage}

{all-args}{after-help}")] // This is required to show the author
pub struct CliArgs {
    /// Verbosity level
    #[clap(short, long, value_enum, default_value = "warn")]
    pub level: LogLevel,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::ValueEnum, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Use *FILENAME* to provide snippets
    Start {
        /// Path to snippet file
        filename: String,
    },
    /// Provide next snippet
    Next {},
    /// Rewind *n* snippet(s)
    Rewind {
        /// Set n (by default is 1)
        count: Option<usize>,
    },
    /// Forward *n* snippet(s)
    Forward {
        /// Set n (by default is 1)
        count: Option<usize>,
    },
}
