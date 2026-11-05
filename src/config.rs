use std::path::Path;

use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{file_system::FileSystem, lazy_coder_error::LazyCoderError};

#[cfg(not(test))]
use aux::{config_dir, file_system, get_snippet_provider};
#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use tests::aux::{config_dir, file_system, get_snippet_provider};

static FILE_NAME: &str = "lazycoder.toml";

/// `LazyCoder` configuration.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Config {
    file_path: String,
    position: usize,
}

#[cfg_attr(test, automock)]
impl Config {
    /// Creates a configuration with the provided filename and sets the snippet number to 0 so it can start from the
    /// beginning.
    ///
    /// # Arguments
    ///
    /// * `path` - path to the file with the snippets that will be stored in the configuration.
    pub fn new(path: &Path) -> Result<Self, LazyCoderError> {
        let fs = file_system();
        if let Ok(absolute_path) = fs.canonicalize(path) {
            debug!("{} does exist", absolute_path.display());
            let new_config = Config {
                file_path: absolute_path
                    .to_str()
                    .ok_or(LazyCoderError::ConfigDirError)?
                    .to_string(),
                position: 0,
            };
            new_config.save(fs.as_ref(), true)?;
            Ok(new_config)
        } else {
            error!("{} doesn't exist", path.display());
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }

    /// Creates a configuration from the file if it exists.
    ///
    /// Configuration is stored in a file following the standards for each operating system.
    pub fn from_file() -> Result<Self, LazyCoderError> {
        if let Some(mut config_file) = config_dir() {
            config_file.push(FILE_NAME);
            debug!(
                "Reading configuration from file {}",
                config_file.as_path().display()
            );
            let toml_text = file_system().read_to_string(&config_file)?;
            let cfg: Config = toml::from_str(&toml_text)?;
            // TODO: Check that the file_path is stil valid?
            Ok(cfg)
        } else {
            Err(LazyCoderError::ConfigDirError)
        }
    }

    /// Read snippet from the file in the configuration, increment position, and update config file.
    pub fn next(&mut self) -> Result<String, LazyCoderError> {
        let path = Path::new(&self.file_path);
        let snippet_prov = get_snippet_provider(path)?;
        let snippet = snippet_prov.get_snippet(self.position)?;
        self.position += 1;
        self.save(file_system().as_ref(), false)?;
        Ok(snippet)
    }

    /// Read snippet from the file in the configuration without updating the config file.
    pub fn peek(&self) -> Result<String, LazyCoderError> {
        let path = Path::new(&self.file_path);
        let snippet_prov = get_snippet_provider(path)?;
        let snippet = snippet_prov.get_snippet(self.position)?;
        Ok(snippet)
    }

    /// Change the configuration file to point to a snippet that is `count` forward.
    /// Out-of-range positions are accepted silently, but peek/next will work fine.
    pub fn forward(&mut self, count: usize) -> Result<(), LazyCoderError> {
        self.position += count;
        self.save(file_system().as_ref(), false)
    }

    /// Change the configuration file to point to a snippet that is `count` backward.
    pub fn rewind(&mut self, count: usize) -> Result<(), LazyCoderError> {
        if count <= self.position {
            self.position -= count;
            self.save(file_system().as_ref(), false)
        } else {
            Err(LazyCoderError::OperationOutOfRange)
        }
    }

    /// Saves this configuration to the standard path and file.
    fn save(&self, fs: &dyn FileSystem, create_dir: bool) -> Result<(), LazyCoderError> {
        let toml_text = toml::to_string(&self).expect("Failing to encode TOML");
        if let Some(mut config_path) = config_dir() {
            if !fs.exists(&config_path) {
                if create_dir {
                    fs.create_dir_all(&config_path)?;
                } else {
                    return Err(LazyCoderError::ConfigDirError);
                }
            }
            config_path.push(FILE_NAME);
            debug!(
                "Writing configuration to file {}",
                config_path.as_path().display()
            );
            fs.write(&config_path, &toml_text)?;
            Ok(())
        } else {
            Err(LazyCoderError::ConfigDirError)
        }
    }
}

#[cfg(not(test))]
mod aux {
    use std::path::{Path, PathBuf};

    use directories::ProjectDirs;

    use crate::{
        file_system::{DiskFileSystem, FileSystem},
        lazy_coder_error::LazyCoderError,
        snippet_handler::{SnippetProvider, snippet_provider},
    };

    pub(crate) fn config_dir() -> Option<PathBuf> {
        Some(
            ProjectDirs::from("dev", "jorgeortiz", "lazycoder")?
                .config_dir()
                .to_path_buf(),
        )
    }

    pub fn file_system() -> Box<dyn FileSystem> {
        Box::new(DiskFileSystem)
    }

    pub fn get_snippet_provider(path: &Path) -> Result<Box<dyn SnippetProvider>, LazyCoderError> {
        snippet_provider(path, Box::new(DiskFileSystem))
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, io, path::PathBuf, str::FromStr};

    use mockall::predicate;

    use crate::{
        file_system::MockFileSystem,
        snippet_handler::{MockSnippetProvider, SnippetProvider},
    };

    use super::*;

    const CONFIG_FILE: &str = "/some/config/path/lazycoder.toml";

    thread_local! {
        static CONFIG_DIR_ANSWER: Cell<Option<PathBuf>> = const { Cell::new(None) };
        static FILE_SYSTEM_ANSWER: Cell<Option<Box<dyn FileSystem>>> = const { Cell::new(None) };
        static SNIPPET_PROVIDER_ANSWER: Cell<Option<Box<dyn SnippetProvider>>> = const { Cell::new(None) };
    }

    /// File system where the configuration directory exists and the configuration file is written once with
    /// `contents`.
    fn file_system_expecting_config(contents: &'static str) -> MockFileSystem {
        let mut fs = MockFileSystem::new();
        fs.expect_exists().return_const(true);
        fs.expect_create_dir_all().never();
        fs.expect_write()
            .withf(move |path, written| path == Path::new(CONFIG_FILE) && written == contents)
            .once()
            .returning(|_, _| Ok(()));
        fs
    }

    #[test]
    fn config_new_from_non_existing_path_fails() {
        let mut fs = MockFileSystem::new();
        fs.expect_canonicalize()
            .returning(|_| Err(io::Error::other("Some error")));
        fs.expect_write().never();
        FILE_SYSTEM_ANSWER.set(Some(Box::new(fs)));

        let sut = Config::new(Path::new(""));

        assert!(matches!(sut, Err(LazyCoderError::SnippetFileNotFound)));
    }

    #[test]
    fn config_new_from_existing_path_is_created() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let mut fs = file_system_expecting_config("file_path = \"/tmp\"\nposition = 0\n");
        fs.expect_canonicalize()
            .withf(|path| path == Path::new("/tmp"))
            .returning(|_| Ok(PathBuf::from("/tmp")));
        FILE_SYSTEM_ANSWER.set(Some(Box::new(fs)));

        let sut = Config::new(Path::new("/tmp"));

        assert!(
            matches!(sut, Ok(ref cfg) if cfg.file_path == "/tmp" && cfg.position == 0),
            "Wrong config: {sut:?}"
        );
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
        let mut fs = MockFileSystem::new();
        fs.expect_read_to_string()
            .returning(|_| Err(io::Error::other("Some error")));
        FILE_SYSTEM_ANSWER.set(Some(Box::new(fs)));

        let sut = Config::from_file();

        assert!(matches!(sut, Err(LazyCoderError::ConfigFileError(_))));
    }

    #[test]
    fn config_from_file_returns_valid_configuration() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let mut fs = MockFileSystem::new();
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(CONFIG_FILE))
            .returning(|_| {
                Ok(String::from(
                    "file_path = \"/some/path/file.lazycoder\"\nposition = 2\n",
                ))
            });
        FILE_SYSTEM_ANSWER.set(Some(Box::new(fs)));

        let sut = Config::from_file();

        assert!(matches!(sut, Ok(config) if config == Config {
                file_path: String::from("/some/path/file.lazycoder"),
                position: 2,
            }
        ));
    }

    #[test]
    fn save_returns_error_if_config_dir_fails() {
        CONFIG_DIR_ANSWER.set(None);
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(
            sut.save(&MockFileSystem::new(), true),
            Err(LazyCoderError::ConfigDirError)
        ));
    }

    #[test]
    fn save_creates_dir_if_doesnt_exist_and_requested() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let mut fs = MockFileSystem::new();
        fs.expect_exists().return_const(false);
        fs.expect_create_dir_all()
            .withf(|path| path == Path::new("/some/config/path"))
            .once()
            .returning(|_| Ok(()));
        fs.expect_write().once().returning(|_, _| Ok(()));
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(sut.save(&fs, true), Ok(())));
    }

    #[test]
    fn save_returns_error_if_doesnt_config_dir_doesnt_exist_and_create_dir_is_false() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let mut fs = MockFileSystem::new();
        fs.expect_exists().return_const(false);
        fs.expect_create_dir_all().never();
        fs.expect_write().never();
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(
            sut.save(&fs, false),
            Err(LazyCoderError::ConfigDirError)
        ));
    }

    #[test]
    fn save_stores_configuration_in_standard_file() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let fs = file_system_expecting_config("file_path = \"/some/config/path\"\nposition = 3\n");
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(sut.save(&fs, true), Ok(())));
    }

    #[test]
    fn next_snippet_increases_position_saves_and_returns_text() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        FILE_SYSTEM_ANSWER.set(Some(Box::new(file_system_expecting_config(
            "file_path = \"/some/config/path\"\nposition = 4\n",
        ))));
        let mut snippet_prov = MockSnippetProvider::new();
        snippet_prov
            .expect_get_snippet()
            .with(predicate::eq(3))
            .once()
            .returning(|_| Ok(String::from("Some snippet")));
        SNIPPET_PROVIDER_ANSWER.set(Some(Box::new(snippet_prov)));
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let snippet = sut.next();

        assert!(
            matches!(snippet, Ok(ref text) if text == "Some snippet"),
            "Snippet: {snippet:?}"
        );
    }

    #[test]
    fn next_snippet_fails_if_snippet_provider_fails() {
        // No file system answer: any file access would fail the test.
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        let mut snippet_prov = MockSnippetProvider::new();
        snippet_prov
            .expect_get_snippet()
            .with(predicate::eq(3))
            .once()
            .returning(|_| Err(LazyCoderError::RunOutOfSnippets));
        SNIPPET_PROVIDER_ANSWER.set(Some(Box::new(snippet_prov)));
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let snippet = sut.next();

        assert!(
            matches!(snippet, Err(LazyCoderError::RunOutOfSnippets)),
            "Snippet: {snippet:?}"
        );
    }

    #[test]
    fn next_snippet_fails_if_save_fails() {
        // No file system answer: any file access would fail the test.
        CONFIG_DIR_ANSWER.set(None);
        let mut snippet_prov = MockSnippetProvider::new();
        snippet_prov
            .expect_get_snippet()
            .with(predicate::eq(3))
            .once()
            .returning(|_| Ok(String::from("Some snippet")));
        SNIPPET_PROVIDER_ANSWER.set(Some(Box::new(snippet_prov)));
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let snippet = sut.next();

        assert!(
            matches!(snippet, Err(LazyCoderError::ConfigDirError)),
            "Snippet: {snippet:?}"
        );
    }

    #[test]
    fn peek_snippet_only_returns_text() {
        let mut snippet_prov = MockSnippetProvider::new();
        snippet_prov
            .expect_get_snippet()
            .with(predicate::eq(3))
            .once()
            .returning(|_| Ok(String::from("Some snippet")));
        SNIPPET_PROVIDER_ANSWER.set(Some(Box::new(snippet_prov)));
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let snippet = sut.peek();

        assert!(
            matches!(snippet, Ok(ref text) if text == "Some snippet"),
            "Snippet: {snippet:?}"
        );
    }

    #[test]
    fn peek_snippet_fails_if_snippet_provider_fails() {
        let mut snippet_prov = MockSnippetProvider::new();
        snippet_prov
            .expect_get_snippet()
            .with(predicate::eq(3))
            .once()
            .returning(|_| Err(LazyCoderError::RunOutOfSnippets));
        SNIPPET_PROVIDER_ANSWER.set(Some(Box::new(snippet_prov)));
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let snippet = sut.peek();

        assert!(
            matches!(snippet, Err(LazyCoderError::RunOutOfSnippets)),
            "Snippet: {snippet:?}"
        );
    }

    #[test]
    fn forward_increases_position_and_saves() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        FILE_SYSTEM_ANSWER.set(Some(Box::new(file_system_expecting_config(
            "file_path = \"/some/config/path\"\nposition = 7\n",
        ))));
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let result = sut.forward(4);

        assert!(result.is_ok(), "Unexpected error moving forward");
    }

    #[test]
    fn rewind_decreases_position_and_saves() {
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from("/some/config/path")));
        FILE_SYSTEM_ANSWER.set(Some(Box::new(file_system_expecting_config(
            "file_path = \"/some/config/path\"\nposition = 1\n",
        ))));
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let result = sut.rewind(2);

        assert!(result.is_ok(), "Unexpected error moving backward");
    }

    #[test]
    fn rewind_fails_when_decrement_moves_below_zero() {
        // No file system answer: any file access would fail the test.
        let mut sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        let result = sut.rewind(4);

        assert!(
            matches!(result, Err(LazyCoderError::OperationOutOfRange)),
            "Unexpected error moving backward"
        );
    }

    pub mod aux {
        use std::path::{Path, PathBuf};

        use crate::{
            file_system::{FileSystem, MockFileSystem},
            lazy_coder_error::LazyCoderError,
            snippet_handler::SnippetProvider,
        };

        use super::{CONFIG_DIR_ANSWER, FILE_SYSTEM_ANSWER, SNIPPET_PROVIDER_ANSWER};

        pub(crate) fn config_dir() -> Option<PathBuf> {
            CONFIG_DIR_ANSWER.take()
        }

        /// File system set by the test or, if none, a mock that fails on any access.
        pub fn file_system() -> Box<dyn FileSystem> {
            FILE_SYSTEM_ANSWER
                .take()
                .unwrap_or_else(|| Box::new(MockFileSystem::new()))
        }

        pub fn get_snippet_provider(
            _path: &Path,
        ) -> Result<Box<dyn SnippetProvider>, LazyCoderError> {
            match SNIPPET_PROVIDER_ANSWER.take() {
                Some(snippet_prov) => Ok(snippet_prov),
                None => Err(LazyCoderError::RunOutOfSnippets),
            }
        }
    }
}
