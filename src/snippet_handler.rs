use std::path::{Path, PathBuf};

#[cfg(test)]
use mockall::automock;

use crate::{file_system::FileSystem, lazy_coder_error::LazyCoderError};

#[cfg_attr(test, automock)]
pub trait SnippetProvider {
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError>;
}

pub struct SnippetHandler {
    path: PathBuf,
    fs: Box<dyn FileSystem>,
}

impl SnippetHandler {
    pub fn new<P: AsRef<Path>>(
        path: P,
        fs: Box<dyn FileSystem>,
    ) -> Result<SnippetHandler, LazyCoderError> {
        if fs.is_file(path.as_ref()) {
            Ok(SnippetHandler {
                path: path.as_ref().to_path_buf(),
                fs,
            })
        } else {
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }
}

impl SnippetProvider for SnippetHandler {
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        match self.fs.read_to_string(&self.path) {
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
    use std::io::{Error, ErrorKind};

    use crate::file_system::MockFileSystem;

    use super::*;

    const PATH: &str = "/some/demo.lazycoder";
    const THREE_SNIPPETS: &str = "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n";

    /// File system where the snippet file exists and has the given contents.
    fn file_system_with(contents: &'static str) -> Box<MockFileSystem> {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file().return_const(true);
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(PATH))
            .returning(move |_| Ok(String::from(contents)));
        Box::new(fs)
    }

    #[test]
    fn configuration_successful_with_existing_file() {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file()
            .withf(|path| path == Path::new(PATH))
            .return_const(true);

        assert!(SnippetHandler::new(PATH, Box::new(fs)).is_ok());
    }

    #[test]
    fn configuration_fails_with_non_existent_file() {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file().return_const(false);

        assert!(matches!(
            SnippetHandler::new(PATH, Box::new(fs)),
            Err(LazyCoderError::SnippetFileNotFound)
        ));
    }

    #[test]
    fn unavailable_snippet_file_causes_error() {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file().return_const(true);
        fs.expect_read_to_string()
            .returning(|_| Err(Error::new(ErrorKind::NotFound, "")));
        let sut = SnippetHandler::new(PATH, Box::new(fs)).unwrap();

        let result = sut.get_snippet(0);
        assert!(
            result.is_err(),
            "Expected error when file not found, but obtained {result:?}"
        );

        let Err(LazyCoderError::SnippetFileError(err)) = result else {
            panic!("Unexpected error type: {result:?}");
        };
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn first_snippet_is_returned() {
        let sut = SnippetHandler::new(PATH, file_system_with(THREE_SNIPPETS)).unwrap();

        let result = sut.get_snippet(0);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {result:?}"
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "First snippet\n");
        }
    }

    #[test]
    fn empty_snippet_is_returned_from_emptly_file() {
        let sut = SnippetHandler::new(PATH, file_system_with("")).unwrap();

        let result = sut.get_snippet(0);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {result:?}"
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "");
        }
    }

    #[test]
    fn middle_snippet_is_returned() {
        let sut = SnippetHandler::new(PATH, file_system_with(THREE_SNIPPETS)).unwrap();

        let result = sut.get_snippet(1);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {result:?}"
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "Second snippet\n");
        }
    }

    #[test]
    fn last_snippet_is_returned() {
        let sut = SnippetHandler::new(PATH, file_system_with(THREE_SNIPPETS)).unwrap();

        let result = sut.get_snippet(2);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {result:?}"
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "Third snippet\n");
        }
    }

    #[test]
    fn unexisting_snippet_returns_error() {
        let sut = SnippetHandler::new(PATH, file_system_with(THREE_SNIPPETS)).unwrap();

        let result = sut.get_snippet(4);
        assert!(
            result.is_err(),
            "Expected error when requesting unexisting snippet, but got: {result:?}"
        );
        assert!(matches!(result, Err(LazyCoderError::RunOutOfSnippets)));
    }

    #[test]
    fn first_snippet_is_empty_for_empty_file() {
        let sut = SnippetHandler::new(PATH, file_system_with("")).unwrap();

        let result = sut.get_snippet(0);
        assert!(
            result.is_ok(),
            "Unexpected error when getting snippet: {result:?}"
        );
        if let Ok(snippet) = result {
            assert_eq!(snippet, "");
        }
    }
}
