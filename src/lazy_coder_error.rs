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
