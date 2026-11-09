//! Lazycoder - A simple snippet generator for espanso
//!
//! `lazycoder start </filepath/demo.lazycoder>`
//! - works with only one demo at a time
//! - save file name
//! - save initial next position: 0
//! - config file location depends on OS. saved in ~/.lazycoder
//!
//! `lazycoder next`
//! - reads from config file
//! - reads next snippet
//! - increments counter to next snippet
//!
//! `lazycoder peek`
//! - reads from config file
//! - reads next snippet
//!
//! `lazycoder rewind [number]`
//! - decrements counter (number times)
//! - returns nothing
//!
//! `lazycoder forward [number]`
//! - increments counter (number times)
//! - returns nothing
//!
//! `lazycoder list-sections`
//! - prints the sections with their numbers, indented by level
//!
//! `lazycoder next-section [number]`
//! - moves to the first snippet of the section that is number sections ahead
//! - returns nothing
//!
//! `lazycoder prev-section [number]`
//! - moves to the first snippet of the current section, or of the section number sections behind
//! - returns nothing
//!
//! `lazycoder goto <section>`
//! - moves to the first snippet of the section with that number (e.g. 2.1)
//! - returns nothing
//!
mod cli_args;
mod config;
mod document_parser;
mod file_system;
mod lazy_coder_error;
mod navigation;
mod org_parser;
mod outline;
mod snippet_handler;
mod snippet_index;

use std::{fmt::Write, path::Path};

use clap::Parser;
#[cfg_attr(test, double)]
use config::Config;
use eyre::{Result, WrapErr, eyre};
use log::{debug, error, info};
#[cfg(test)]
use mockall_double::double;

use crate::{
    cli_args::{CliArgs, Command},
    lazy_coder_error::LazyCoderError,
    outline::Outline,
};

fn main() -> Result<()> {
    let cli = CliArgs::parse();
    if let Some(level) = cli.level {
        env_logger::builder().filter(None, level).init();
    } else {
        env_logger::init();
    }

    match cli.command {
        Command::Start { filename } => start(&filename)?,
        Command::Next => next()?,
        Command::Peek => peek()?,
        Command::Forward { count } => forward(count)?,
        Command::Rewind { count } => rewind(count)?,
        Command::ListSections => list_sections()?,
        Command::NextSection { count } => next_section(count)?,
        Command::PrevSection { count } => prev_section(count)?,
        Command::Goto { section } => goto(&section)?,
    }
    Ok(())
}

/// Restart the configuration for the given path.
fn start(filename: &Path) -> Result<()> {
    info!("Setting to work {}", filename.display());

    Config::new(filename)
        .map(|_| {
            debug!("Configuration successfully created.");
        })
        .map_err(|err| {
            error!("Failed to create configuration: {err}.");
            eyre!("Failed to create configuration: {err}")
        })
}

/// Print next snippet and advance.
fn next() -> Result<()> {
    info!("Next");
    peek_or_next(true)
}

/// Print next snippet and don't advance.
fn peek() -> Result<()> {
    info!("peek");
    peek_or_next(false)
}

/// Print next snippet and optionally advance.
fn peek_or_next(advance: bool) -> Result<()> {
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    let result = if advance { cfg.next() } else { cfg.peek() };
    result
        .map(|snippet| {
            print!("{snippet}");
        })
        .map_err(|err| {
            error!(
                "Failed to obtain {} snippet: {}.",
                if advance { "next" } else { "current" },
                err
            );
            eyre!(
                "Failed to obtain {} snippet: {}.",
                if advance { "next" } else { "current" },
                err
            )
        })
}

/// Increases the counter by the number provided in the argument. It returns a result of the operation.
fn forward(count: usize) -> Result<()> {
    info!("Forward {count}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.forward(count).map_err(|err| {
        error!("Failed to forward: {err}.");
        eyre!("Failed to forward: {err}.")
    })
}

/// Decreases the counter by the number provided in the argument. It returns a result of the operation.
fn rewind(count: usize) -> Result<()> {
    info!("Rewind {count}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.rewind(count).map_err(|err| {
        error!("Failed to rewind: {err}.");
        eyre!("Failed to rewind: {err}.")
    })
}

/// Moves to the first snippet of the section that is `count` sections forward.
fn next_section(count: usize) -> Result<()> {
    info!("Next section {count}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.next_section(count).map_err(|err| {
        error!("Failed to move to next section: {err}.");
        eyre!("Failed to move to next section: {err}.")
    })
}

/// Moves to the first snippet of the section that is `count` sections backward.
fn prev_section(count: usize) -> Result<()> {
    info!("Previous section {count}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.prev_section(count).map_err(|err| {
        error!("Failed to move to previous section: {err}.");
        eyre!("Failed to move to previous section: {err}.")
    })
}

/// Moves to the first snippet of the section with the given number.
fn goto(section: &str) -> Result<()> {
    info!("Go to section {section}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.goto(section).map_err(|err| {
        error!("Failed to go to section {section}: {err}.");
        eyre!("Failed to go to section {section}: {err}.")
    })
}

/// Prints the sections of the snippet file.
fn list_sections() -> Result<()> {
    info!("List sections");
    let cfg = Config::from_file().wrap_err("Failed to read config file")?;
    match cfg.outline() {
        Ok(outline) if !outline.sections.is_empty() => {
            print!("{}", format_sections(&outline, cfg.position()));
            Ok(())
        }
        result => {
            let err = result.err().unwrap_or(LazyCoderError::NoSections);
            error!("Failed to list sections: {err}.");
            Err(eyre!("Failed to list sections: {err}."))
        }
    }
}

/// One line per section with its number, title, and snippets, indented by level. The snippets are shown as
/// `[f:<first snippet>/o:<own snippets>/t:<total snippets, including subsections>]`. The section of the snippet at
/// `position` is marked with `>`.
fn format_sections(outline: &Outline, position: usize) -> String {
    let current = outline.section_of(position);
    outline
        .sections
        .iter()
        .enumerate()
        .fold(String::new(), |mut text, (index, section)| {
            let marker = if current == Some(index) { ">" } else { " " };
            let indent = "  ".repeat(usize::from(section.level.saturating_sub(1)));
            let _ = writeln!(
                text,
                "{marker} {indent}{} {} [f:{}/o:{}/t:{}]",
                section.number_string(),
                section.title,
                section.first_snippet,
                outline.own_snippets(index).len(),
                outline.total_snippets(index).len()
            );
            text
        })
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use crate::{
        config::MockConfig,
        outline::{Section, SnippetLocation},
    };

    use super::*;

    static MTX: Mutex<()> = Mutex::new(());

    #[test]
    fn start_creates_config_and_reports_ok() {
        let _mtx = MTX.lock();
        let path = PathBuf::from("/some/confid/file");
        let context = MockConfig::new_context();
        context.expect().returning(|_| {
            let config_mock = MockConfig::default();
            Ok(config_mock)
        });

        assert!(start(&path).is_ok(), "Unexpected result");
    }

    #[test]
    fn start_returns_error_if_config_cannot_be_created() {
        let _mtx = MTX.lock();
        let path = PathBuf::from("/some/confid/file");
        let context = MockConfig::new_context();
        context
            .expect()
            .returning(|_| Err(LazyCoderError::ConfigDirError));

        assert!(start(&path).is_err(), "Unexpected result");
    }

    #[test]
    fn next_uses_config_and_reports_ok() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_next()
                .returning(|| Ok(String::from("This is an snippet")));
            Ok(config_mock)
        });

        assert!(next().is_ok(), "Unexpected result");
    }

    #[test]
    fn next_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(next().is_err(), "Unexpected result");
    }

    #[test]
    fn next_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_next()
                .returning(|| Err(LazyCoderError::RunOutOfSnippets));
            Ok(config_mock)
        });

        assert!(next().is_err(), "Unexpected result");
    }

    #[test]
    fn peek_uses_config_and_reports_ok() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_peek()
                .returning(|| Ok(String::from("This is an snippet")));
            Ok(config_mock)
        });

        assert!(peek().is_ok(), "Unexpected result");
    }

    #[test]
    fn peek_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(peek().is_err(), "Unexpected result");
    }

    #[test]
    fn peek_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_peek()
                .returning(|| Err(LazyCoderError::RunOutOfSnippets));
            Ok(config_mock)
        });

        assert!(peek().is_err(), "Unexpected result");
    }

    #[test]
    fn forward_uses_config_and_reports_ok() {
        const FORWARD_NUM: usize = 5;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_forward()
                .with(predicate::eq(FORWARD_NUM))
                .returning(|_| Ok(()));
            Ok(config_mock)
        });

        assert!(forward(FORWARD_NUM).is_ok(), "Unexpected result");
    }

    #[test]
    fn forward_returns_error_if_no_config() {
        const FORWARD_NUM: usize = 5;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(forward(FORWARD_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn forward_returns_error_if_config_operation_fails() {
        const FORWARD_NUM: usize = 5;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_forward()
                .with(predicate::eq(FORWARD_NUM))
                .returning(|_| Err(LazyCoderError::OperationOutOfRange));
            Ok(config_mock)
        });

        assert!(forward(FORWARD_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn rewind_uses_config_and_reports_ok() {
        const REWIND_NUM: usize = 3;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_rewind()
                .with(predicate::eq(REWIND_NUM))
                .returning(|_| Ok(()));
            Ok(config_mock)
        });

        assert!(rewind(REWIND_NUM).is_ok(), "Unexpected result");
    }

    #[test]
    fn rewind_returns_error_if_no_config() {
        const REWIND_NUM: usize = 5;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(rewind(REWIND_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn rewind_returns_error_if_config_operation_fails() {
        const REWIND_NUM: usize = 5;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_rewind()
                .with(predicate::eq(REWIND_NUM))
                .returning(|_| Err(LazyCoderError::OperationOutOfRange));
            Ok(config_mock)
        });

        assert!(rewind(REWIND_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn next_section_uses_config_and_reports_ok() {
        const COUNT: usize = 2;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_next_section()
                .with(predicate::eq(COUNT))
                .once()
                .returning(|_| Ok(()));
            Ok(config_mock)
        });

        assert!(next_section(COUNT).is_ok(), "Unexpected result");
    }

    #[test]
    fn next_section_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(next_section(1).is_err(), "Unexpected result");
    }

    #[test]
    fn next_section_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_next_section()
                .returning(|_| Err(LazyCoderError::NoSections));
            Ok(config_mock)
        });

        assert!(next_section(1).is_err(), "Unexpected result");
    }

    #[test]
    fn prev_section_uses_config_and_reports_ok() {
        const COUNT: usize = 2;
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_prev_section()
                .with(predicate::eq(COUNT))
                .once()
                .returning(|_| Ok(()));
            Ok(config_mock)
        });

        assert!(prev_section(COUNT).is_ok(), "Unexpected result");
    }

    #[test]
    fn prev_section_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(prev_section(1).is_err(), "Unexpected result");
    }

    #[test]
    fn prev_section_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_prev_section()
                .returning(|_| Err(LazyCoderError::OperationOutOfRange));
            Ok(config_mock)
        });

        assert!(prev_section(1).is_err(), "Unexpected result");
    }

    #[test]
    fn goto_uses_config_and_reports_ok() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_goto()
                .withf(|section| section == "2.1")
                .once()
                .returning(|_| Ok(()));
            Ok(config_mock)
        });

        assert!(goto("2.1").is_ok(), "Unexpected result");
    }

    #[test]
    fn goto_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(goto("2.1").is_err(), "Unexpected result");
    }

    #[test]
    fn goto_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_goto()
                .returning(|_| Err(LazyCoderError::SectionNotFound));
            Ok(config_mock)
        });

        assert!(goto("2.1").is_err(), "Unexpected result");
    }

    #[test]
    fn list_sections_uses_config_and_reports_ok() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock.expect_outline().once().returning(|| {
                Ok(Outline {
                    sections: vec![Section::new(1, "Intro", &[1], 0)],
                    snippets: vec![SnippetLocation::new(1, 2, 0)],
                })
            });
            config_mock.expect_position().return_const(0_usize);
            Ok(config_mock)
        });

        assert!(list_sections().is_ok(), "Unexpected result");
    }

    #[test]
    fn list_sections_returns_error_if_no_config() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(list_sections().is_err(), "Unexpected result");
    }

    #[test]
    fn list_sections_returns_error_if_config_operation_fails() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_outline()
                .returning(|| Err(LazyCoderError::MalformedDocument { line: 2 }));
            Ok(config_mock)
        });

        assert!(list_sections().is_err(), "Unexpected result");
    }

    #[test]
    fn list_sections_returns_error_if_file_has_no_sections() {
        let _mtx = MTX.lock();
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_outline()
                .returning(|| Ok(Outline::default()));
            Ok(config_mock)
        });

        assert!(list_sections().is_err(), "Unexpected result");
    }

    #[test]
    fn sections_are_formatted_indented_by_level_with_snippet_counts_and_marker() {
        assert_eq!(
            format_sections(&outline_with_subsections(), 3),
            "  1 Intro [f:0/o:2/t:5]\n    1.1 Setup [f:2/o:1/t:3]\n>     1.1.1 Details [f:3/o:2/t:2]\n  2 Code [f:5/o:1/t:1]\n"
        );
    }

    #[test]
    fn sections_are_formatted_without_marker_when_all_snippets_were_provided() {
        assert_eq!(
            format_sections(&outline_with_subsections(), 6),
            "  1 Intro [f:0/o:2/t:5]\n    1.1 Setup [f:2/o:1/t:3]\n      1.1.1 Details [f:3/o:2/t:2]\n  2 Code [f:5/o:1/t:1]\n"
        );
    }

    #[test]
    fn no_sections_are_formatted_as_empty_text() {
        assert_eq!(format_sections(&Outline::default(), 0), "");
    }

    /// Outline with 6 snippets and nested sections.
    fn outline_with_subsections() -> Outline {
        Outline {
            sections: vec![
                Section::new(1, "Intro", &[1], 0),
                Section::new(2, "Setup", &[1, 1], 2),
                Section::new(3, "Details", &[1, 1, 1], 3),
                Section::new(1, "Code", &[2], 5),
            ],
            snippets: (0..6)
                .map(|line| SnippetLocation::new(line, line + 1, 0))
                .collect(),
        }
    }
}
