use crate::lazy_coder_error::LazyCoderError;
use directories::ProjectDirs;
use serde_derive::Serialize;
use std::{fs, path};

static FILE_NAME: &str = "lazycoder.toml";

/// LazyCoder configuration.
#[derive(Default, Serialize)]
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
    /// * `filename` - name of the file with the snippets that will be stored in the configuration.
    pub fn new(filename: &str) -> Result<Config, LazyCoderError> {
        let path = path::PathBuf::from(filename);

        if let Ok(absolute_path) = fs::canonicalize(&path) {
            println!("{:?} does exist", absolute_path);
            let new_config = Config {
                file_path: absolute_path.to_str().unwrap().to_string(),
                position: 0,
            };
            let toml_text = toml::to_string(&new_config).expect("Failing to encode TOML");
            if let Some(project_dirs) = ProjectDirs::from("com", "mongodb", "lazycoder") {
                let config_dir = project_dirs.config_dir();
                if !config_dir.exists() {
                    fs::create_dir_all(config_dir)?;
                }
                let mut config_file = project_dirs.config_dir().to_path_buf();
                config_file.push(FILE_NAME);
                println!(
                    "Writing configuration to file {}",
                    config_file.as_path().display()
                );
                fs::write(config_file, toml_text)?;
            }
            Ok(new_config)
        } else {
            println!("{} doesn't exist", path.display());
            Err(LazyCoderError::ConfigError)
        }
    }

    pub fn read() -> Result<Config, LazyCoderError> {
        Err(LazyCoderError::NotImplementedError {})
    }

    pub fn next(&self) -> Result<String, LazyCoderError> {
        Err(LazyCoderError::NotImplementedError {})
    }

    pub fn forward(&self, count: usize) -> Result<(), LazyCoderError> {
        Err(LazyCoderError::NotImplementedError {})
    }

    pub fn rewind(&self, count: usize) -> Result<(), LazyCoderError> {
        Err(LazyCoderError::NotImplementedError {})
    }
}
