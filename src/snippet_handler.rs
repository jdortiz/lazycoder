use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
use mockall::automock;

use crate::lazy_coder_error::LazyCoderError;

#[cfg_attr(test, automock)]
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

#[cfg_attr(test, automock)]
pub trait SnippetProvider {
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError>;
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

    #[cfg(test)]
    fn set_reader<R: WholeFileReader + 'a>(&mut self, reader: R) {
        self.reader = Box::new(reader);
    }
}

impl<'a> SnippetProvider for SnippetHandler<'a> {
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        match self.reader.read_to_string() {
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
    use std::{
        io::{Error, ErrorKind},
        path::PathBuf,
    };

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

    #[test]
    fn unavailable_snippet_file_causes_error() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader
            .expect_read_to_string()
            .returning(|| Err(Error::new(ErrorKind::NotFound, "")));

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(0);
        assert!(
            result.is_err(),
            "Expected error when file not found, but obtained {:?}",
            result
        );
        // assert!(matches!(result, Err(LazyCoderError::SnippetFileError(_))));
        let Err(LazyCoderError::SnippetFileError(err)) = result else {
            panic!("Unexpected error type: {:?}", result);
        };
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn first_snippet_is_returned() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader.expect_read_to_string().returning(|| {
            Ok(String::from(
                "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n",
            ))
        });

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(0);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {:?}",
            result
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "First snippet\n");
        }
    }

    #[test]
    fn middle_snippet_is_returned() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader.expect_read_to_string().returning(|| {
            Ok(String::from(
                "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n",
            ))
        });

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(1);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {:?}",
            result
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "Second snippet\n");
        }
    }

    #[test]
    fn last_snippet_is_returned() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader.expect_read_to_string().returning(|| {
            Ok(String::from(
                "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n",
            ))
        });

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(2);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {:?}",
            result
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "Third snippet\n");
        }
    }

    #[test]
    fn unexisting_snippet_returns_error() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader.expect_read_to_string().returning(|| {
            Ok(String::from(
                "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n",
            ))
        });

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(4);
        assert!(
            result.is_err(),
            "Expected error when requesting unexisting snippet, but got: {:?}",
            result
        );
        assert!(matches!(result, Err(LazyCoderError::RunOutOfSnippets)));
    }

    #[test]
    fn first_snippet_is_empty_for_empty_file() {
        let temp_file = NamedTempFile::new().expect("Unable to create temporary file");
        let path = temp_file.path();
        let mut sut = SnippetHandler::new(&path).unwrap();
        let mut mock_reader = MockWholeFileReader::new();
        mock_reader
            .expect_read_to_string()
            .returning(|| Ok(String::from("")));

        sut.set_reader(mock_reader);

        let result = sut.get_snippet(0);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {:?}",
            result
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "");
        }
    }
}
