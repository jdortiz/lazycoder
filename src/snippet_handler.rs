use std::{fs, path::Path};

use crate::lazy_coder_error::LazyCoderError;

pub struct SnippetHandler {
    filename: String,
}

impl SnippetHandler {
    pub fn new(path: &Path) -> Result<SnippetHandler, LazyCoderError> {
        if path.is_file() {
            Ok(SnippetHandler {
                filename: path.to_str().unwrap().to_owned(),
            })
        } else {
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }

    pub fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        match fs::read_to_string(&self.filename) {
            Ok(string) => match string.split("\n---\n\n").nth(position) {
                Some(snippet) => Ok(snippet.to_owned()),
                None => Err(LazyCoderError::RunOutOfSnippets),
            },
            Err(err) => Err(LazyCoderError::SnippetFileError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn configuration_successful_with_existing_file() {
        // Create temporary file
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();

        assert!(SnippetHandler::new(&path).is_ok());
    }

    #[test]
    fn configuration_fails_with_non_existent_file() {
        let path: PathBuf;
        {
            let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
            path = temp_file.path().to_path_buf();
        } // temp file is deleted here.

        assert!(SnippetHandler::new(&path).is_err());
    }
}
