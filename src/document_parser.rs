//! Abstraction over the supported snippet file formats.
use std::path::Path;

use crate::{
    lazy_coder_error::LazyCoderError,
    outline::{Outline, SnippetLocation},
};

/// Parser for a structured document format (Org, Markdown, ...).
pub trait DocumentParser {
    /// Obtain the structure of the document.
    fn parse(&self, text: &str) -> Result<Outline, LazyCoderError>;
    /// Obtain the text of the snippet at the given location, removing any format-specific escaping.
    fn extract(&self, text: &str, location: &SnippetLocation) -> String;
}

/// Format of a snippet file.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SnippetFormat {
    /// Snippets separated by `\n---\n\n`.
    Plain,
    /// Org mode document with source and example blocks.
    Org,
}

impl SnippetFormat {
    /// Determine the format of the file from its extension.
    pub fn from_path(path: &Path) -> Self {
        match path.extension() {
            Some(ext) if ext.eq_ignore_ascii_case("org") => SnippetFormat::Org,
            _ => SnippetFormat::Plain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn org_extension_is_org_format() {
        assert_eq!(
            SnippetFormat::from_path(Path::new("/some/demo.org")),
            SnippetFormat::Org
        );
    }

    #[test]
    fn org_extension_is_case_insensitive() {
        assert_eq!(
            SnippetFormat::from_path(Path::new("/some/demo.ORG")),
            SnippetFormat::Org
        );
    }

    #[test]
    fn other_extension_is_plain_format() {
        assert_eq!(
            SnippetFormat::from_path(Path::new("/some/demo.lazycoder")),
            SnippetFormat::Plain
        );
    }

    #[test]
    fn no_extension_is_plain_format() {
        assert_eq!(
            SnippetFormat::from_path(Path::new("/some/demo")),
            SnippetFormat::Plain
        );
    }
}
