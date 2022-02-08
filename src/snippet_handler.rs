use std::{fs, path::Path};

use crate::lazy_coder_error::LazyCoderError;

pub struct SnippetHandler {
    filename: String,
}

impl SnippetHandler {
    pub fn new(path: &Path) -> Result<SnippetHandler, LazyCoderError> {
        if path.exists() {
            Ok(SnippetHandler {
                filename: path.to_str().unwrap().to_owned(),
            })
        } else {
            Err(LazyCoderError::ConfigError {})
        }
    }

    pub fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        match fs::read_to_string(&self.filename)?
            .split("\n---\n\n")
            .nth(position)
        {
            Some(snippet) => Ok(snippet.to_owned()),
            None => Err(LazyCoderError::ConfigError {}),
        }
    }
}
