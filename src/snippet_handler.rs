use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::lazy_coder_error::LazyCoderError;

trait WholeFileReader {
    fn read_to_string(&self) -> std::io::Result<String>;
}

struct ReaderShell {
    filename: PathBuf,
}

impl ReaderShell {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<ReaderShell, LazyCoderError> {
        if path.as_ref().is_file() {
            Ok(ReaderShell {
                filename: path.as_ref().to_path_buf(),
            })
        } else {
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }
}

impl WholeFileReader for ReaderShell {
    fn read_to_string(&self) -> std::io::Result<String> {
        fs::read_to_string(&self.filename)
    }
}

pub struct SnippetHandler<'a> {
    reader: Box<dyn WholeFileReader + 'a>,
}

impl<'a> SnippetHandler<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<SnippetHandler<'a>, LazyCoderError> {
        Ok(SnippetHandler {
            reader: Box::new(ReaderShell::new(path)?),
        })
    }

    pub fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        match self.reader.read_to_string() {
            Ok(string) => match string.split("\n---\n\n").nth(position) {
                Some(snippet) => Ok(snippet.to_owned()),
                None => Err(LazyCoderError::RunOutOfSnippets),
            },
            Err(err) => Err(LazyCoderError::SnippetFileError(err)),
        }
    }

    #[cfg(test)]
    fn set_reader<R: WholeFileReader + 'a>(&mut self, reader: R) {
        self.reader = Box::new(reader);
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
