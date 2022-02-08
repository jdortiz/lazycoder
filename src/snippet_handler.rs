use std::{fs, path::Path};

use crate::lazy_coder_error::LazyCoderError;

pub struct SnippetHandler {
    filename: String,
    count: usize,
}

impl SnippetHandler {
    pub fn new(path: &Path) -> Result<SnippetHandler, LazyCoderError> {
        if path.exists() {
            let snippets: Vec<_> = fs::read_to_string(path)?
                .split("\n---\n\n")
                .map(|s| s.to_owned())
                .collect();
            eprintln!("Read {} from {}", snippets.len(), path.display());
            Ok(SnippetHandler {
                filename: path.to_str().unwrap().to_owned(),
                count: snippets.len(),
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
