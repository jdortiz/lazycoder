use std::io;

use thiserror::Error;

/// Errors of lazycoder. Wrapped errors are their source, so they aren't repeated in the message.
#[derive(Debug, Error)]
pub enum LazyCoderError {
    #[error("snippet file not found")]
    SnippetFileNotFound,
    #[error("snippet file error")]
    SnippetFileError(#[source] io::Error),
    #[error("snippet file path is not valid UTF-8")]
    SnippetPathNotUtf8,
    #[error("out of range of available snippets")]
    RunOutOfSnippets,
    #[error("no valid home directory path could be retrieved")]
    ConfigDirError,
    #[error("configuration file error")]
    ConfigFileError(#[from] io::Error),
    #[error("configuration encoding error")]
    ConfigEncoding(#[from] toml::de::Error),
    #[error("operation out of range")]
    OperationOutOfRange,
    #[error("malformed snippet document at line {line}")]
    MalformedDocument { line: usize },
    #[error("index file error")]
    IndexFileError(#[source] io::Error),
    #[error("section not found")]
    SectionNotFound,
    #[error("snippet file has no sections")]
    NoSections,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::de::Error;

    #[test]
    fn display_snippet_file_not_found_error() {
        assert_eq!(
            LazyCoderError::SnippetFileNotFound.to_string(),
            "snippet file not found"
        );
    }

    #[test]
    fn display_snippet_path_not_utf8_error() {
        assert_eq!(
            LazyCoderError::SnippetPathNotUtf8.to_string(),
            "snippet file path is not valid UTF-8"
        );
    }

    #[test]
    fn display_snippet_file_error() {
        assert_eq!(
            LazyCoderError::SnippetFileError(std::io::Error::other("some file error".to_string()))
                .to_string(),
            "snippet file error"
        );
    }

    #[test]
    fn display_run_out_of_snippets_error() {
        assert_eq!(
            LazyCoderError::RunOutOfSnippets.to_string(),
            "out of range of available snippets"
        );
    }

    #[test]
    fn display_config_dir_error() {
        assert_eq!(
            LazyCoderError::ConfigDirError.to_string(),
            "no valid home directory path could be retrieved"
        );
    }

    #[test]
    fn display_config_file_error() {
        assert_eq!(
            LazyCoderError::ConfigFileError(std::io::Error::other("some file error".to_string()))
                .to_string(),
            "configuration file error"
        );
    }

    #[test]
    fn display_config_encoding_error() {
        assert_eq!(
            LazyCoderError::ConfigEncoding(toml::de::Error::custom("some file error")).to_string(),
            "configuration encoding error"
        );
    }

    #[test]
    fn display_operation_out_of_range_error() {
        assert_eq!(
            LazyCoderError::OperationOutOfRange.to_string(),
            "operation out of range"
        );
    }

    #[test]
    fn display_malformed_document_error() {
        assert_eq!(
            LazyCoderError::MalformedDocument { line: 7 }.to_string(),
            "malformed snippet document at line 7"
        );
    }

    #[test]
    fn display_index_file_error() {
        assert_eq!(
            LazyCoderError::IndexFileError(std::io::Error::other("some file error".to_string()))
                .to_string(),
            "index file error"
        );
    }

    #[test]
    fn wrapped_error_is_the_source() {
        let error =
            LazyCoderError::IndexFileError(std::io::Error::other("some file error".to_string()));

        assert_eq!(
            std::error::Error::source(&error).map(ToString::to_string),
            Some(String::from("some file error"))
        );
    }

    #[test]
    fn io_error_is_converted_to_custom_error() {
        let new_error = LazyCoderError::from(std::io::Error::other("some file error".to_string()));

        assert!(matches!(new_error, LazyCoderError::ConfigFileError(_)));
    }

    #[test]
    fn display_section_not_found_error() {
        assert_eq!(
            LazyCoderError::SectionNotFound.to_string(),
            "section not found"
        );
    }

    #[test]
    fn display_no_sections_error() {
        assert_eq!(
            LazyCoderError::NoSections.to_string(),
            "snippet file has no sections"
        );
    }

    #[test]
    fn toml_error_is_converted_to_custom_error() {
        let toml_error = toml::de::Error::custom("some file error");
        let new_error = LazyCoderError::from(toml_error.clone());

        assert!(matches!(
            new_error,
            LazyCoderError::ConfigEncoding(err) if err == toml_error
        ));
    }
}
