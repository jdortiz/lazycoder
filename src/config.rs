use crate::{lazy_coder_error::LazyCoderError, snippet_handler::SnippetHandler};
use directories::ProjectDirs;
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(not(test))]
use aux::config_dir;
#[cfg(not(test))]
use std::fs::read_to_string;
#[cfg(test)]
use tests::aux::{config_dir, read_to_string};

static FILE_NAME: &str = "lazycoder.toml";

/// LazyCoder configuration.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Config {
    file_path: String,
    position: usize,
}

impl Config {
    /// Creates a configuration with the provided filename and sets the snippet number to 0 so it can start from the
    /// beginning.
    ///
    /// # Arguments
    ///
    /// * `path` - path to the file with the snippets that will be stored in the configuration.
    pub fn new(path: &Path) -> Result<Config, LazyCoderError> {
        if let Ok(absolute_path) = fs::canonicalize(path) {
            debug!("{:?} does exist", absolute_path);
            let new_config = Config {
                file_path: absolute_path.to_str().unwrap().to_string(),
                position: 0,
            };
            new_config.save(true)?;
            Ok(new_config)
        } else {
            error!("{} doesn't exist", path.display());
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }

    /// Creates a configuration from the file if it exists.
    ///
    /// Configuration is stored in a file following the standards for each operating system.
    pub fn from_file() -> Result<Config, LazyCoderError> {
        if let Some(mut config_file) = config_dir() {
            config_file.push(FILE_NAME);
            debug!(
                "Reading configuration from file {}",
                config_file.as_path().display()
            );
            let toml_text = read_to_string(config_file)?;
            let cfg: Config = toml::from_str(&toml_text)?;
            // TODO: Check that the file_path is stil valid?
            Ok(cfg)
        } else {
            Err(LazyCoderError::ConfigDirError)
        }
    }

    pub fn next(&mut self) -> Result<String, LazyCoderError> {
        let path = PathBuf::from(self.file_path.clone());
        let snippet_hdlr: SnippetHandler = SnippetHandler::new(&path)?;
        let snippet = snippet_hdlr.get_snippet(self.position)?;
        self.position += 1;
        self.save(false)?;
        Ok(snippet)
    }

    pub fn peek(&mut self) -> Result<String, LazyCoderError> {
        let path = PathBuf::from(self.file_path.clone());
        let snippet_hdlr: SnippetHandler = SnippetHandler::new(&path)?;
        let snippet = snippet_hdlr.get_snippet(self.position)?;
        Ok(snippet)
    }

    pub fn forward(&mut self, count: usize) -> Result<(), LazyCoderError> {
        self.position += count;
        self.save(false)
    }

    pub fn rewind(&mut self, count: usize) -> Result<(), LazyCoderError> {
        if count <= self.position {
            self.position -= count;
            self.save(false)
        } else {
            Err(LazyCoderError::OperationOutOfRange)
        }
    }

    fn save(&self, create_dir: bool) -> Result<(), LazyCoderError> {
        let toml_text = toml::to_string(&self).expect("Failing to encode TOML");
        if let Some(project_dirs) = ProjectDirs::from("com", "mongodb", "lazycoder") {
            if create_dir {
                let config_dir = project_dirs.config_dir();
                if !config_dir.exists() {
                    fs::create_dir_all(config_dir)?;
                }
            }
            let mut config_file = project_dirs.config_dir().to_path_buf();
            config_file.push(FILE_NAME);
            debug!(
                "Writing configuration to file {}",
                config_file.as_path().display()
            );
            fs::write(config_file, toml_text)?;
            Ok(())
        } else {
            Err(LazyCoderError::ConfigDirError)
        }
    }
}

#[cfg(not(test))]
mod aux {
    use std::path::PathBuf;

    use directories::ProjectDirs;

    pub(crate) fn config_dir() -> Option<PathBuf> {
        Some(
            ProjectDirs::from("dev", "jorgeortiz", "lazycoder")?
                .config_dir()
                .to_path_buf(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, str::FromStr};

    use super::*;

    thread_local! {
    static CONFIG_DIR_ANSWER: Cell<Option<PathBuf>> = Cell::new(None);
    static READ_TO_STRING_ANSWER: Cell<Option<String>> = Cell::new(None);
    }

    #[test]
    fn config_from_file_fails_with_non_exiting_path_or_directory() {
        CONFIG_DIR_ANSWER.set(None);

        let sut = Config::from_file();

        assert!(matches!(sut, Err(LazyCoderError::ConfigDirError)));
    }

    #[test]
    fn config_from_non_exiting_file_fails() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from_str("Some path").unwrap()));
        READ_TO_STRING_ANSWER.set(None);

        let sut = Config::from_file();

        assert!(matches!(sut, Err(LazyCoderError::ConfigFileError(_))));
    }

    #[test]
    fn config_from_file_returns_valid_configuration() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from_str("Some path").unwrap()));
        READ_TO_STRING_ANSWER.set(Some(String::from(
            "file_path = \"/some/path/file.lazycoder\"\nposition = 2\n",
        )));

        let sut = Config::from_file();

        assert!(matches!(sut, Ok(config) if config == Config {
                file_path: String::from("/some/path/file.lazycoder"),
                position: 2,
            }
        ));
    }

    pub mod aux {
        use std::{
            io,
            path::{Path, PathBuf},
        };

        use super::{CONFIG_DIR_ANSWER, READ_TO_STRING_ANSWER};

        pub(crate) fn config_dir() -> Option<PathBuf> {
            CONFIG_DIR_ANSWER.take()
        }

        pub(crate) fn read_to_string<P: AsRef<Path>>(_path: P) -> std::io::Result<String> {
            match READ_TO_STRING_ANSWER.take() {
                Some(lines) => Ok(lines),
                None => Err(io::Error::other("Some error")),
            }
        }
    }
}
