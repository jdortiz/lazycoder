use std::{error, fmt, io};

#[derive(Debug)]
pub enum LazyCoderError {
    SnippetFileNotFound,
    SnippetFileError(io::Error),
    RunOutOfSnippets,
    ConfigDirError,
    ConfigFileError(io::Error),
    ConfigEncoding(toml::de::Error),
    OperationOutOfRange,
}

impl error::Error for LazyCoderError {}

impl fmt::Display for LazyCoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LazyCoderError::SnippetFileNotFound => {
                write!(f, "snippet file not found")
            }
            LazyCoderError::SnippetFileError(err) => {
                write!(f, "snippet file error: {err}")
            }
            LazyCoderError::RunOutOfSnippets => {
                write!(f, "out of range of available snippets")
            }
            LazyCoderError::ConfigDirError => {
                write!(f, "no valid home directory path could be retrieved")
            }
            LazyCoderError::ConfigFileError(err) => {
                write!(f, "configuration file error: {err}")
            }
            LazyCoderError::ConfigEncoding(err) => {
                write!(f, "configuration encoding error: {err}")
            }
            LazyCoderError::OperationOutOfRange => {
                write!(f, "operation out of range")
            }
        }
    }
}

impl From<std::io::Error> for LazyCoderError {
    fn from(err: std::io::Error) -> Self {
        LazyCoderError::ConfigFileError(err)
    }
}

impl From<toml::de::Error> for LazyCoderError {
    fn from(err: toml::de::Error) -> Self {
        LazyCoderError::ConfigEncoding(err)
    }
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
        )
    }

    #[test]
    fn display_snippet_file_error() {
        assert_eq!(
            LazyCoderError::SnippetFileError(std::io::Error::other("some file error".to_string()))
                .to_string(),
            "snippet file error: some file error"
        )
    }

    #[test]
    fn display_run_out_of_snippets_error() {
        assert_eq!(
            LazyCoderError::RunOutOfSnippets.to_string(),
            "out of range of available snippets"
        )
    }

    #[test]
    fn display_config_dir_error() {
        assert_eq!(
            LazyCoderError::ConfigDirError.to_string(),
            "no valid home directory path could be retrieved"
        )
    }

    #[test]
    fn display_config_file_error() {
        assert_eq!(
            LazyCoderError::ConfigFileError(std::io::Error::other("some file error".to_string()))
                .to_string(),
            "configuration file error: some file error"
        )
    }

    #[test]
    fn display_config_encoding_error() {
        assert_eq!(
            LazyCoderError::ConfigEncoding(toml::de::Error::custom("some file error")).to_string(),
            "configuration encoding error: some file error\n" // Newline added by toml::de::Error
        )
    }

    #[test]
    fn display_operation_out_of_range_error() {
        assert_eq!(
            LazyCoderError::OperationOutOfRange.to_string(),
            "operation out of range"
        )
    }

    #[test]
    fn toml_error_conversion() {
        let toml_error = toml::de::Error::custom("some file error");
        let new_error = LazyCoderError::from(toml_error.clone());

        assert!(matches!(
            new_error,
            LazyCoderError::ConfigEncoding(err) if err == toml_error
        ));
    }
}
