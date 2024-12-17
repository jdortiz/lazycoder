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
use std::{env, path::Path, process::exit};

use cli_args::{CliArgs, Command};
#[double]
use config::Config;

fn main() {
    let cli = CliArgs::parse();
    unsafe {
        env::set_var("RUST_LOG", cli.level.to_string());
    }
    env_logger::init();

    match cli.command {
        Command::Start { filename } => {
            exit(start(&filename));
        }
        Command::Next {} => {
            exit(next());
        }
        Command::Peek {} => {
            exit(peek());
        }
        Command::Forward { count } => {
            let count = count.unwrap_or(1);
            exit(forward(count));
        }
        Command::Rewind { count } => {
            let count = count.unwrap_or(1);
            exit(rewind(count));
        }
    }
}

fn start(filename: &Path) -> i32 {
    info!("Setting to work {}", filename.display());
    match Config::new(filename) {
        Ok(_) => {
            debug!("Configuration successfully created.");
            0
        }
        Err(err) => {
            error!("Failed to create configuration: {err}.");
            1
        }
    }
}

fn next() -> i32 {
    info!("Next");
    peek_or_next(true)
}

fn peek() -> i32 {
    info!("peek");
    peek_or_next(false)
}

fn peek_or_next(advance: bool) -> i32 {
    match Config::from_file() {
        Ok(mut cfg) => {
            let result = if advance { cfg.next() } else { cfg.peek() };
            match result {
                Ok(snippet) => {
                    print!("{snippet}");
                    0
                }
                Err(err) => {
                    error!(
                        "Failed to obtain {} snippet: {}.",
                        if advance { "next" } else { "current" },
                        err
                    );
                    1
                }
            }
        }
        Err(err) => {
            error!("Failed to read config file: {err}.");
            1
        }
    }
}

/// Increases the counter by the number provided in the argument. It returns the exit code to use.
fn forward(count: usize) -> i32 {
    info!("Forward {count}");
    match Config::from_file() {
        Ok(mut cfg) => {
            if let Err(err) = cfg.forward(count) {
                error!("Failed to foward: {err}.");
                1
            } else {
                0
            }
        }
        Err(err) => {
            error!("Failed to foward: {err}.");
            1
        }
    }
}

/// Decreases the counter by the number provided in the argument. It returns the exit code to use.
fn rewind(count: usize) -> i32 {
    info!("Rewind {}", count);
    match Config::from_file() {
        Ok(mut cfg) => {
            if let Err(err) = cfg.rewind(count) {
                error!("Failed to rewind: {err}.");
                1
            } else {
                0
            }
        }
        Err(err) => {
            error!("Failed to rewind: {err}.");
            1
        }
    }
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

        assert_eq!(start(&path), 0, "Unexpected exit code");
    }

    #[test]
    fn start_exits_with_error_if_config_cannot_be_created() {
        let path = PathBuf::from("/some/confid/file");
        let context = MockConfig::new_context();
        context
            .expect()
            .returning(|_| Err(LazyCoderError::ConfigDirError));

        assert_eq!(start(&path), 1, "Unexpected exit code");
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

        assert_eq!(next(), 0, "Unexpected exit code");
    }

    #[test]
    fn next_exits_with_error_if_no_config() {
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert_eq!(next(), 1, "Unexpected exit code");
    }

    #[test]
    fn next_exits_with_error_if_config_operation_fails() {
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_next()
                .returning(|| Err(LazyCoderError::RunOutOfSnippets));
            Ok(config_mock)
        });

        assert_eq!(next(), 1, "Unexpected exit code");
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

        assert_eq!(peek(), 0, "Unexpected exit code");
    }

    #[test]
    fn peek_exits_with_error_if_no_config() {
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert_eq!(peek(), 1, "Unexpected exit code");
    }

    #[test]
    fn peek_exits_with_error_if_config_operation_fails() {
        let context = MockConfig::from_file_context();
        context.expect().returning(|| {
            let mut config_mock = MockConfig::default();
            config_mock
                .expect_peek()
                .returning(|| Err(LazyCoderError::RunOutOfSnippets));
            Ok(config_mock)
        });

        assert_eq!(peek(), 1, "Unexpected exit code");
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

        assert_eq!(forward(FORWARD_NUM), 0, "Unexpected exit code");
    }

    #[test]
    fn forward_exits_with_error_if_no_config() {
        const FORWARD_NUM: usize = 5;
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert_eq!(forward(FORWARD_NUM), 1, "Unexpected exit code");
    }

    #[test]
    fn forward_exits_with_error_if_config_operation_fails() {
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

        assert_eq!(forward(FORWARD_NUM), 1, "Unexpected exit code");
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

        assert_eq!(rewind(REWIND_NUM), 0, "Unexpected exit code");
    }

    #[test]
    fn rewind_exits_with_error_if_no_config() {
        const REWIND_NUM: usize = 5;
        let context = MockConfig::from_file_context();
        context
            .expect()
            .returning(|| Err(LazyCoderError::ConfigDirError));

        assert_eq!(rewind(REWIND_NUM), 1, "Unexpected exit code");
    }

    #[test]
    fn rewind_exits_with_error_if_config_operation_fails() {
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

        assert_eq!(rewind(REWIND_NUM), 1, "Unexpected exit code");
    }
}
