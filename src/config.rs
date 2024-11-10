use crate::{lazy_coder_error::LazyCoderError, snippet_handler::SnippetProvider};
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(not(test))]
use aux::{config_dir, get_snippet_provider, path_exists};
#[cfg(not(test))]
use std::fs::{canonicalize, create_dir_all, read_to_string, write};
#[cfg(test)]
use tests::aux::{
    canonicalize, config_dir, create_dir_all, get_snippet_provider, path_exists, read_to_string,
    write,
};

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
        if let Ok(absolute_path) = canonicalize(path) {
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

    /// Read snippet from the file in the configuration, increment position, and update config file.
    ///
    ///
    pub fn next(&mut self) -> Result<String, LazyCoderError> {
        let path = PathBuf::from(self.file_path.clone());
        let snippet_prov = get_snippet_provider(&path)?;
        let snippet = snippet_prov.get_snippet(self.position)?;
        self.position += 1;
        self.save(false)?;
        Ok(snippet)
    }

    /// Read snippet from the file in the configuration without updating the config file.
    ///
    ///
    pub fn peek(&mut self) -> Result<String, LazyCoderError> {
        let path = PathBuf::from(self.file_path.clone());
        let snippet_prov = get_snippet_provider(&path)?;
        let snippet = snippet_prov.get_snippet(self.position)?;
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

    /// Saves this configuration to the standard path and file.
    fn save(&self, create_dir: bool) -> Result<(), LazyCoderError> {
        let toml_text = toml::to_string(&self).expect("Failing to encode TOML");
        if let Some(mut config_path) = config_dir() {
            if !path_exists(&config_path) {
                if create_dir {
                    create_dir_all(config_path.clone())?;
                } else {
                    return Err(LazyCoderError::ConfigDirError);
                }
            }
            config_path.push(FILE_NAME);
            debug!(
                "Writing configuration to file {}",
                config_path.as_path().display()
            );
            write(config_path, toml_text)?;
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
        lazy_coder_error::LazyCoderError,
        snippet_handler::{SnippetHandler, SnippetProvider},
    };

    pub(crate) fn config_dir() -> Option<PathBuf> {
        Some(
            ProjectDirs::from("dev", "jorgeortiz", "lazycoder")?
                .config_dir()
                .to_path_buf(),
        )
    }

    pub fn get_snippet_provider(path: &Path) -> Result<Box<dyn SnippetProvider>, LazyCoderError> {
        Ok(Box::new(SnippetHandler::new(path)?))
    }

    pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, str::FromStr};

    use mockall::predicate;

    use crate::snippet_handler::MockSnippetProvider;

    use super::*;

    thread_local! {
        static CANONIZALIZE_ANSWER: Cell<Option<PathBuf>> = Cell::new(None);
        static CONFIG_DIR_ANSWER: Cell<Option<PathBuf>> = Cell::new(None);
        static CREATE_DIR_OK_ANSWER: Cell<bool> = Cell::new(true);
        static CREATE_DIR_ALL_ARG: Cell<Option<PathBuf>> = Cell::new(None);
        static PATH_EXISTS_ANSWER: Cell<bool> = Cell::new(true);
        static READ_TO_STRING_ANSWER: Cell<Option<String>> = Cell::new(None);
        static SNIPPET_PROVIDER_ANSWER: Cell<Option<Box<dyn SnippetProvider>>> = Cell::new(None);
        static WRITE_OK_ANSWER: Cell<bool> = Cell::new(true);
        static WRITE_ARG_PATH: Cell<Option<PathBuf>> = Cell::new(None);
        static WRITE_ARG_CONTENTS: Cell<Option<String>> = Cell::new(None);
    }

    #[test]
    fn config_new_from_non_existing_path_fails() {
        CANONIZALIZE_ANSWER.set(None);

        let sut = Config::new(Path::new(""));

        assert!(matches!(sut, Err(LazyCoderError::SnippetFileNotFound)));
    }

    #[test]
    fn config_new_from_existing_path_is_created() {
        let mut path = PathBuf::new();
        path.push("/tmp");
        CANONIZALIZE_ANSWER.set(Some(path));
        CONFIG_DIR_ANSWER.set(Some(PathBuf::from_str("Some path").unwrap()));

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

    #[test]
    fn save_returns_error_if_config_dir_fails() {
        CONFIG_DIR_ANSWER.set(None);
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(
            sut.save(true),
            Err(LazyCoderError::ConfigDirError)
        ));
    }

    #[test]
    fn save_creates_dir_if_doesnt_exist_and_rnquested() {
        let path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(false);
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(sut.save(true), Ok(())));

        assert_eq!(CREATE_DIR_ALL_ARG.take(), Some(path_buf));
    }

    #[test]
    fn save_returns_error_if_doesnt_config_dir_doesnt_exist_and_create_dir_is_false() {
        let path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(false);
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        assert!(matches!(
            sut.save(false),
            Err(LazyCoderError::ConfigDirError)
        ));
    }

    #[test]
    fn save_stores_configuration_in_stanard_file() {
        let mut path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(true);
        let sut = Config {
            file_path: String::from("/some/config/path"),
            position: 3,
        };

        // TODO: path.exists() should also be mocked. and then create_dir_all should be a spy.
        assert!(matches!(sut.save(true), Ok(())));

        path_buf.push(FILE_NAME);
        assert_eq!(WRITE_ARG_PATH.take(), Some(path_buf));
        assert_eq!(
            WRITE_ARG_CONTENTS.take(),
            Some(String::from(
                "file_path = \"/some/config/path\"\nposition = 3\n"
            ))
        );
    }

    #[test]
    fn next_snippet_increases_position_saves_and_returns_text() {
        let mut path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(true);
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
            "Snippet: {:?}",
            snippet
        );
        path_buf.push(FILE_NAME);
        assert_eq!(WRITE_ARG_PATH.take(), Some(path_buf));
        assert_eq!(
            WRITE_ARG_CONTENTS.take(),
            Some(String::from(
                "file_path = \"/some/config/path\"\nposition = 4\n"
            ))
        );
    }

    #[test]
    fn next_snippet_fails_if_snippet_provider_fails() {
        let path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(true);
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
            "Snippet: {:?}",
            snippet
        );
        assert_eq!(WRITE_ARG_PATH.take(), None);
        assert_eq!(WRITE_ARG_CONTENTS.take(), None);
    }

    #[test]
    fn next_snippet_fails_if_save_fails() {
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
            "Snippet: {:?}",
            snippet
        );
        assert_eq!(WRITE_ARG_PATH.take(), None);
        assert_eq!(WRITE_ARG_CONTENTS.take(), None);
    }

    #[test]
    fn peek_snippet_only_returns_text() {
        let path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(true);
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

        let snippet = sut.peek();

        assert!(
            matches!(snippet, Ok(ref text) if text == "Some snippet"),
            "Snippet: {:?}",
            snippet
        );
    }

    #[test]
    fn peek_snippet_fails_if_snippet_provider_fails() {
        let path_buf = PathBuf::from("/some/config/path");
        CONFIG_DIR_ANSWER.set(Some(path_buf.clone()));
        PATH_EXISTS_ANSWER.set(true);
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

        let snippet = sut.peek();

        assert!(
            matches!(snippet, Err(LazyCoderError::RunOutOfSnippets)),
            "Snippet: {:?}",
            snippet
        );
    }

    pub mod aux {
        use std::{
            io,
            path::{Path, PathBuf},
        };

        use crate::{lazy_coder_error::LazyCoderError, snippet_handler::SnippetProvider};

        use super::{
            CANONIZALIZE_ANSWER, CONFIG_DIR_ANSWER, CREATE_DIR_ALL_ARG, CREATE_DIR_OK_ANSWER,
            PATH_EXISTS_ANSWER, READ_TO_STRING_ANSWER, SNIPPET_PROVIDER_ANSWER, WRITE_ARG_CONTENTS,
            WRITE_ARG_PATH, WRITE_OK_ANSWER,
        };

        pub fn canonicalize<P: AsRef<Path>>(_path: P) -> io::Result<PathBuf> {
            match CANONIZALIZE_ANSWER.take() {
                Some(path_buf) => Ok(path_buf),
                None => Err(io::Error::other("Some error")),
            }
        }

        pub(crate) fn config_dir() -> Option<PathBuf> {
            CONFIG_DIR_ANSWER.take()
        }

        pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
            CREATE_DIR_ALL_ARG.set(Some(path.as_ref().to_path_buf()));
            if CREATE_DIR_OK_ANSWER.get() {
                Ok(())
            } else {
                Err(io::Error::other("Some error"))
            }
        }

        pub fn get_snippet_provider(
            _path: &Path,
        ) -> Result<Box<dyn SnippetProvider>, LazyCoderError> {
            match SNIPPET_PROVIDER_ANSWER.take() {
                Some(snippet_prov) => Ok(snippet_prov),
                None => Err(LazyCoderError::RunOutOfSnippets),
            }
        }

        pub fn path_exists<P: AsRef<Path>>(_path: P) -> bool {
            PATH_EXISTS_ANSWER.get()
        }

        pub(crate) fn read_to_string<P: AsRef<Path>>(_path: P) -> std::io::Result<String> {
            match READ_TO_STRING_ANSWER.take() {
                Some(lines) => Ok(lines),
                None => Err(io::Error::other("Some error")),
            }
        }

        pub(crate) fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
            path: P,
            contents: C,
        ) -> io::Result<()> {
            WRITE_ARG_PATH.set(Some(path.as_ref().to_path_buf()));
            if let Ok(cont_str) = std::str::from_utf8(contents.as_ref()) {
                WRITE_ARG_CONTENTS.set(Some(cont_str.to_string()));
            }
            if WRITE_OK_ANSWER.get() {
                Ok(())
            } else {
                Err(io::Error::other("Some error"))
            }
        }
    }
}
