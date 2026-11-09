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
//! - list-sections: Prints the sections with their numbers
//! - next-section: Moves to the first snippet of a following section
//! - prev-section: Moves to the first snippet of the current or a preceding section
//! - goto: Moves to the first snippet of a section given by its number
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
    #[arg(short, long, value_enum, value_name = "LEVEL")]
    pub level: Option<log::LevelFilter>,

    #[command(subcommand)]
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
    Next,
    /// Provide next snippet without moving forward
    #[command(visible_alias = "p")]
    Peek,
    /// Rewind *n* snippet(s)
    #[command(visible_alias = "r")]
    Rewind {
        /// Set n (by default is 1)
        #[arg(default_value_t = 1)]
        count: usize,
    },
    /// Forward *n* snippet(s)
    #[command(visible_alias = "f")]
    Forward {
        /// Set n (by default is 1)
        #[arg(default_value_t = 1)]
        count: usize,
    },
    /// List the sections with their numbers
    #[command(visible_alias = "l")]
    ListSections,
    /// Move to the first snippet of the *n*-th next section
    #[command(visible_alias = "ns")]
    NextSection {
        /// Set n (by default is 1)
        #[arg(default_value_t = 1)]
        count: usize,
    },
    /// Move to the first snippet of the *n*-th previous section (the current one first, if past its start)
    #[command(visible_alias = "ps")]
    PrevSection {
        /// Set n (by default is 1)
        #[arg(default_value_t = 1)]
        count: usize,
    },
    /// Move to the first snippet of section *SECTION*
    #[command(visible_alias = "g")]
    Goto {
        /// Section number, e.g. 2.1
        section: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINARY: &str = "lazycoder";

    #[test]
    fn cli_passes_verification() {
        use clap::CommandFactory;

        CliArgs::command().debug_assert();
    }

    #[test]
    fn list_sections() {
        assert!(matches!(parse(&["list-sections"]), Command::ListSections));
    }

    #[test]
    fn list_sections_alias() {
        assert!(matches!(parse(&["l"]), Command::ListSections));
    }

    #[test]
    fn next_section_produces_command_with_count() {
        assert!(matches!(
            parse(&["next-section", "3"]),
            Command::NextSection { count: 3 }
        ));
    }

    #[test]
    fn next_section_alias_defaults_to_one() {
        assert!(matches!(parse(&["ns"]), Command::NextSection { count: 1 }));
    }

    #[test]
    fn prev_section_produces_command_with_count() {
        assert!(matches!(
            parse(&["prev-section", "2"]),
            Command::PrevSection { count: 2 }
        ));
    }

    #[test]
    fn prev_section_alias_defaults_to_one() {
        assert!(matches!(parse(&["ps"]), Command::PrevSection { count: 1 }));
    }

    #[test]
    fn section_count_must_be_usize_convertible() {
        assert!(try_parse(&["next-section", "two"]).is_err());
        assert!(try_parse(&["next-section", "0.3"]).is_err());
        assert!(try_parse(&["prev-section", "four"]).is_err());
        assert!(try_parse(&["prev-section", "-1"]).is_err());
    }

    #[test]
    fn goto_produces_command_with_section_number() {
        assert!(matches!(
            parse(&["goto", "2.1"]),
            Command::Goto { section } if section == "2.1"
        ));
    }

    #[test]
    fn goto_alias_produces_command_with_section_number() {
        assert!(matches!(
            parse(&["g", "3"]),
            Command::Goto { section } if section == "3"
        ));
    }

    #[test]
    fn goto_requires_section_number() {
        assert!(try_parse(&["goto"]).is_err());
    }

    /// Parse the arguments after the binary name.
    fn try_parse(args: &[&str]) -> Result<CliArgs, clap::Error> {
        CliArgs::try_parse_from(std::iter::once(BINARY).chain(args.iter().copied()))
    }

    /// Subcommand obtained from the arguments after the binary name.
    fn parse(args: &[&str]) -> Command {
        try_parse(args).expect("Failed to parse arguments").command
    }
}
