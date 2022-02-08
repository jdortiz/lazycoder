use crate::{lazy_coder_error::LazyCoderError, snippet_handler::SnippetHandler};
use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path};

static FILE_NAME: &str = "lazycoder.toml";

/// LazyCoder configuration.
#[derive(Default, Deserialize, Serialize)]
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
            eprintln!("{:?} does exist", absolute_path);
            let new_config = Config {
                file_path: absolute_path.to_str().unwrap().to_string(),
                position: 0,
            };
            new_config.save(true)?;
            Ok(new_config)
        } else {
            eprintln!("{} doesn't exist", path.display());
            Err(LazyCoderError::ConfigError)
        }
    }

    pub fn read() -> Result<Config, LazyCoderError> {
        if let Some(project_dirs) = ProjectDirs::from("com", "mongodb", "lazycoder") {
            let mut config_file = project_dirs.config_dir().to_path_buf();
            config_file.push(FILE_NAME);
            eprintln!(
                "Reading configuration from file {}",
                config_file.as_path().display()
            );
            let toml_text = fs::read_to_string(config_file)?;
            let cfg: Config = toml::from_str(&toml_text)?;
            // TODO: Check that the file_path is stil valid?
            Ok(cfg)
        } else {
            Err(LazyCoderError::ConfigError {})
        }
    }

    pub fn next(&mut self) -> Result<String, LazyCoderError> {
        let snippet_hdlr: SnippetHandler = SnippetHandler::new(self.file_path.as_ref())?;
        let snippet = snippet_hdlr.get_snippet(self.position)?;
        self.position += 1;
        self.save(false)?;
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
            Err(LazyCoderError::ConfigError)
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
            eprintln!(
                "Writing configuration to file {}",
                config_file.as_path().display()
            );
            fs::write(config_file, toml_text)?;
        }
        Ok(())
    }
}
