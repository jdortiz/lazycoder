use std::{fs, path::Path};

use crate::lazy_coder_error::LazyCoderError;

struct SnippetHandler {
    count: usize,
}

impl SnippetHandler {
    fn new(path: &Path) -> Result<SnippetHandler, LazyCoderError> {
        if path.exists() {
            let snippets: Vec<_> = fs::read_to_string(path)?
                .split("\n---\n\n")
                .map(|s| s.to_owned())
                .collect();
            println!("Read {} from {}", snippets.len(), path.display());
            Ok(SnippetHandler {
                count: snippets.len(),
            })
        } else {
            Err(LazyCoderError::ConfigError {})
        }
    }
}
