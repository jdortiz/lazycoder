use std::{error, fmt};

#[derive(Debug)]
pub enum LazyCoderError {
    NotImplementedError,
    ConfigError,
}

impl error::Error for LazyCoderError {}

impl fmt::Display for LazyCoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LazyCoderError::NotImplementedError => {
                write!(f, "ERR: Operation not implemented yet")
            }
            LazyCoderError::ConfigError => {
                write!(f, "Config error")
            }
        }
    }
}

impl From<std::io::Error> for LazyCoderError {
    fn from(_: std::io::Error) -> Self {
        LazyCoderError::ConfigError {}
    }
}

impl From<toml::de::Error> for LazyCoderError {
    fn from(_: toml::de::Error) -> Self {
        LazyCoderError::ConfigError {}
    }
}
