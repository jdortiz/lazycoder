use std::path::{Path, PathBuf};

#[cfg(test)]
use mockall::automock;

use crate::{
    document_parser::{DocumentParser, SnippetFormat},
    file_system::FileSystem,
    lazy_coder_error::LazyCoderError,
    org_parser::OrgParser,
    outline::{Outline, Section},
    snippet_index::load_or_build,
};

#[cfg_attr(test, automock)]
pub trait SnippetProvider {
    /// Snippet at the given position (0-based), in document order. Fails with `RunOutOfSnippets` if there is no snippet
    /// at that position.
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError>;
    /// Sections of the snippet document, empty if the format has none.
    fn sections(&self) -> Result<Vec<Section>, LazyCoderError>;
}

/// Provider for the snippet file, chosen by its format.
pub fn snippet_provider<P: AsRef<Path>>(
    path: P,
    fs: Box<dyn FileSystem>,
) -> Result<Box<dyn SnippetProvider>, LazyCoderError> {
    Ok(match SnippetFormat::from_path(path.as_ref()) {
        SnippetFormat::Plain => Box::new(SnippetHandler::new(path, fs)?),
        SnippetFormat::Org => Box::new(OutlineSnippetHandler::new(path, fs, Box::new(OrgParser))?),
    })
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

    fn sections(&self) -> Result<Vec<Section>, LazyCoderError> {
        Ok(vec![])
    }
}

/// Provides the snippets of a structured document (e.g. org mode) using its outline, which is kept in an index file.
pub struct OutlineSnippetHandler {
    path: PathBuf,
    fs: Box<dyn FileSystem>,
    parser: Box<dyn DocumentParser>,
}

impl OutlineSnippetHandler {
    pub fn new<P: AsRef<Path>>(
        path: P,
        fs: Box<dyn FileSystem>,
        parser: Box<dyn DocumentParser>,
    ) -> Result<OutlineSnippetHandler, LazyCoderError> {
        if fs.is_file(path.as_ref()) {
            Ok(OutlineSnippetHandler {
                path: path.as_ref().to_path_buf(),
                fs,
                parser,
            })
        } else {
            Err(LazyCoderError::SnippetFileNotFound)
        }
    }

    /// Text and outline of the document. The index file is rebuilt if the document has changed.
    fn load(&self) -> Result<(String, Outline), LazyCoderError> {
        load_or_build(self.fs.as_ref(), &self.path, self.parser.as_ref())
    }
}

impl SnippetProvider for OutlineSnippetHandler {
    fn get_snippet(&self, position: usize) -> Result<String, LazyCoderError> {
        let (text, outline) = self.load()?;
        let location = outline
            .snippets
            .get(position)
            .ok_or(LazyCoderError::RunOutOfSnippets)?;
        Ok(self.parser.extract(&text, location))
    }

    fn sections(&self) -> Result<Vec<Section>, LazyCoderError> {
        Ok(self.load()?.1.sections)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};

    use crate::{
        document_parser::MockDocumentParser, file_system::MockFileSystem, outline::SnippetLocation,
        snippet_index::index_path,
    };

    use super::*;

    const PATH: &str = "/some/demo.lazycoder";
    const THREE_SNIPPETS: &str = "First snippet\n\n---\n\nSecond snippet\n\n---\n\nThird snippet\n";
    const ORG_PATH: &str = "/some/demo.org";
    const ORG_TEXT: &str =
        "* Intro\n#+begin_src\nhello\n#+end_src\n* Next\n#+begin_example\nbye\n#+end_example\n";

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

    #[test]
    fn plain_file_has_no_sections() {
        let sut = SnippetHandler::new(PATH, file_system_with(THREE_SNIPPETS)).unwrap();

        assert_eq!(sut.sections().expect("Unexpected error"), vec![]);
    }

    #[test]
    fn outline_handler_fails_with_non_existent_file() {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file().return_const(false);

        assert!(matches!(
            OutlineSnippetHandler::new(ORG_PATH, Box::new(fs), Box::new(MockDocumentParser::new())),
            Err(LazyCoderError::SnippetFileNotFound)
        ));
    }

    #[test]
    fn outline_handler_returns_first_snippet() {
        let sut = outline_handler(org_file_system(None), parser_returning(outline()));

        assert_eq!(sut.get_snippet(0).expect("Unexpected error"), "line 2\n");
    }

    #[test]
    fn outline_handler_returns_last_snippet() {
        let sut = outline_handler(org_file_system(None), parser_returning(outline()));

        assert_eq!(sut.get_snippet(1).expect("Unexpected error"), "line 6\n");
    }

    #[test]
    fn outline_handler_unexisting_snippet_returns_error() {
        let sut = outline_handler(org_file_system(None), parser_returning(outline()));

        let result = sut.get_snippet(2);
        assert!(
            matches!(result, Err(LazyCoderError::RunOutOfSnippets)),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn outline_handler_returns_sections() {
        let sut = outline_handler(org_file_system(None), parser_returning(outline()));

        assert_eq!(
            sut.sections().expect("Unexpected error"),
            outline().sections
        );
    }

    #[test]
    fn outline_handler_reports_malformed_document() {
        let mut parser = MockDocumentParser::new();
        parser
            .expect_parse()
            .returning(|_| Err(LazyCoderError::MalformedDocument { line: 2 }));
        let sut = outline_handler(org_file_system(None), parser);

        let result = sut.get_snippet(0);
        assert!(
            matches!(result, Err(LazyCoderError::MalformedDocument { line: 2 })),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn outline_handler_rebuilds_index_of_edited_document() {
        // Outdated index without snippets: the snippet can only be found if the document is parsed again.
        let stale_index = "hash = \"stale\"\n[outline]\nsections = []\nsnippets = []\n";
        let sut = outline_handler(
            org_file_system(Some(stale_index)),
            parser_returning(outline()),
        );

        assert_eq!(sut.get_snippet(0).expect("Unexpected error"), "line 2\n");
    }

    #[test]
    fn provider_for_org_file_extracts_blocks() {
        let sut = snippet_provider(ORG_PATH, org_file_system(None)).expect("Unexpected error");

        assert_eq!(sut.get_snippet(1).expect("Unexpected error"), "bye\n");
        assert_eq!(sut.sections().expect("Unexpected error").len(), 2);
    }

    #[test]
    fn provider_for_other_files_uses_plain_format() {
        let sut =
            snippet_provider(PATH, file_system_with(THREE_SNIPPETS)).expect("Unexpected error");

        assert_eq!(
            sut.get_snippet(1).expect("Unexpected error"),
            "Second snippet\n"
        );
        assert_eq!(sut.sections().expect("Unexpected error"), vec![]);
    }

    /// Outline with two sections, each with one snippet.
    fn outline() -> Outline {
        Outline {
            sections: vec![
                Section::new(1, "Intro", &[1], 0),
                Section::new(1, "Next", &[2], 1),
            ],
            snippets: vec![SnippetLocation::new(2, 3, 0), SnippetLocation::new(6, 7, 0)],
        }
    }

    fn outline_handler(
        fs: Box<MockFileSystem>,
        parser: MockDocumentParser,
    ) -> OutlineSnippetHandler {
        OutlineSnippetHandler::new(ORG_PATH, fs, Box::new(parser)).expect("Unexpected error")
    }

    /// File system with the org document, optionally its index file, and accepting the index to be written.
    fn org_file_system(index: Option<&'static str>) -> Box<MockFileSystem> {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file()
            .withf(|path| path == Path::new(ORG_PATH))
            .return_const(true);
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(ORG_PATH))
            .returning(|_| Ok(String::from(ORG_TEXT)));
        fs.expect_read_to_string()
            .withf(|path| path == index_path(Path::new(ORG_PATH)))
            .returning(move |_| {
                index
                    .map(String::from)
                    .ok_or(Error::from(ErrorKind::NotFound))
            });
        fs.expect_write()
            .withf(|path, _| path == index_path(Path::new(ORG_PATH)))
            .returning(|_, _| Ok(()));
        Box::new(fs)
    }

    /// Parser that returns the outline and extracts each snippet as the index of its first line.
    fn parser_returning(outline: Outline) -> MockDocumentParser {
        let mut parser = MockDocumentParser::new();
        parser
            .expect_parse()
            .returning(move |_| Ok(outline.clone()));
        parser
            .expect_extract()
            .withf(|text, _| text == ORG_TEXT)
            .returning(|_, location| format!("line {}\n", location.start_line));
        parser
    }

    /// File system where the snippet file exists and has the given contents.
    fn file_system_with(contents: &'static str) -> Box<MockFileSystem> {
        let mut fs = MockFileSystem::new();
        fs.expect_is_file().return_const(true);
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(PATH))
            .returning(move |_| Ok(String::from(contents)));
        Box::new(fs)
    }
}
