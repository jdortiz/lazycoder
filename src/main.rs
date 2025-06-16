//! Lazycoder - A simple snippet generator for expanso
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
mod cli_args;
mod config;
mod lazy_coder_error;
mod snippet_handler;

use clap::Parser;
use log::{debug, error, info};
use mockall_double::double;
use std::{env, path::Path};
use eyre::{eyre, Result, WrapErr};

use cli_args::{CliArgs, Command};
#[double]
use config::Config;

fn main() -> Result<()> {
    let cli = CliArgs::parse();
    unsafe {
        env::set_var("RUST_LOG", cli.level.to_string());
    }
    env_logger::init();

    match cli.command {
        Command::Start { filename } => {
            start(&filename)?
        }
        Command::Next {} => {
            next()?
        }
        Command::Peek {} => {
            peek()?
        }
        Command::Forward { count } => {
            let count = count.unwrap_or(1);
            forward(count)?
        }
        Command::Rewind { count } => {
            let count = count.unwrap_or(1);
            rewind(count)?
        }
    }
    Ok(())
}


/// Restart the configuration for the given path.
fn start(filename: &Path) -> Result<()> {
    info!("Setting to work {}", filename.display());

    Config::new(filename)
        .map(|_| {
            debug!("Configuration successfully created.");
            ()
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
            ()
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
                err)
        })
}

/// Increases the counter by the number provided in the argument. It returns a result of the operation.
fn forward(count: usize) -> Result<()> {
    info!("Forward {count}");
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.forward(count)
        .map(|_| { () })
        .map_err(|err| {
            error!("Failed to foward: {err}.");
            eyre!("Failed to foward: {err}.")
        })
}

/// Decreases the counter by the number provided in the argument. It returns a result of the operation.
fn rewind(count: usize) -> Result<()> {
    info!("Rewind {}", count);
    let mut cfg = Config::from_file().wrap_err("Failed to read config file")?;
    cfg.rewind(count)
        .map(|_| { () })
        .map_err(|err| {
            error!("Failed to rewind: {err}.");
            eyre!("Failed to foward: {err}.")
        })
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use std::path::PathBuf;

    use crate::config::MockConfig;
    use crate::lazy_coder_error::LazyCoderError;

    use super::*;

    #[test]
    fn start_creates_config_and_reports_ok() {
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
        let path = PathBuf::from("/some/confid/file");
        let context = MockConfig::new_context();
        context
            .expect()
            .returning(|_| Err(LazyCoderError::ConfigDirError));

        assert!(start(&path).is_err(), "Unexpected result");
    }

    #[test]
    fn next_uses_config_and_reports_ok() {
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
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(next().is_err(), "Unexpected result");
    }

    #[test]
    fn next_returns_error_if_config_operation_fails() {
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
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(peek().is_err(), "Unexpected result");
    }

    #[test]
    fn peek_returns_error_if_config_operation_fails() {
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
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(forward(FORWARD_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn forward_returns_error_if_config_operation_fails() {
        const FORWARD_NUM: usize = 5;
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
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert!(rewind(REWIND_NUM).is_err(), "Unexpected result");
    }

    #[test]
    fn rewind_returns_error_if_config_operation_fails() {
        const REWIND_NUM: usize = 5;
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
}
