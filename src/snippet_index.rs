//! Index file stored next to a structured snippet document, so it is only parsed again when its contents change.
use std::{
    fmt::Write,
    fs, io,
    path::{Path, PathBuf},
};

use log::{debug, info, warn};
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{document_parser::DocumentParser, lazy_coder_error::LazyCoderError, outline::Outline};

/// Appended to the name of the snippet document to obtain the name of its index file.
static INDEX_SUFFIX: &str = ".lci";

/// File operations required to maintain the index.
#[cfg_attr(test, automock)]
pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
}

/// Access to the actual file system of the computer.
pub struct DiskFileSystem;

impl FileSystem for DiskFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        fs::write(path, contents)
    }
}

/// Contents of the index file.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct IndexFile {
    /// Hash of the snippet document (org or otherwise) the outline was obtained from.
    hash: String,
    outline: Outline,
}

/// Path of the index file for the given snippet document.
pub fn index_path(source: &Path) -> PathBuf {
    let mut file_name = source.file_name().unwrap_or_default().to_os_string();
    file_name.push(INDEX_SUFFIX);
    source.with_file_name(file_name)
}

/// SHA-256 of the text as a lowercase hexadecimal string.
pub fn content_hash(text: &str) -> String {
    Sha256::digest(text)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Read the snippet document and obtain its outline, from the index file if it is up to date, or parsing the
/// document and writing a new index file otherwise.
pub fn load_or_build(
    fs: &dyn FileSystem,
    source: &Path,
    parser: &dyn DocumentParser,
) -> Result<(String, Outline), LazyCoderError> {
    let text = fs
        .read_to_string(source)
        .map_err(LazyCoderError::SnippetFileError)?;
    let hash = content_hash(&text);
    let path = index_path(source);
    if let Some(outline) = read_index(fs, &path, &hash) {
        return Ok((text, outline));
    }

    let index = IndexFile {
        hash,
        outline: parser.parse(&text)?,
    };
    let toml_text = toml::to_string(&index).expect("Failing to encode TOML");
    debug!("Writing index to file {}", path.display());
    fs.write(&path, &toml_text)
        .map_err(LazyCoderError::IndexFileError)?;
    Ok((text, index.outline))
}

/// Outline stored in the index file, if the file can be read and matches the hash of the document.
fn read_index(fs: &dyn FileSystem, path: &Path, hash: &str) -> Option<Outline> {
    let toml_text = fs
        .read_to_string(path)
        .inspect_err(|err| debug!("Index file {} not available: {err}", path.display()))
        .ok()?;
    match toml::from_str::<IndexFile>(&toml_text) {
        Ok(index) if index.hash == hash => Some(index.outline),
        Ok(_) => {
            info!("Index file {} is outdated", path.display());
            None
        }
        Err(err) => {
            warn!("Index file {} cannot be decoded: {err}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{document_parser::MockDocumentParser, outline::Section};

    use super::*;

    const SOURCE: &str = "/talks/demo.org";
    const INDEX: &str = "/talks/demo.org.lci";
    const TEXT: &str = "* Intro\n";

    #[test]
    fn index_path_is_next_to_source() {
        assert_eq!(index_path(Path::new(SOURCE)), PathBuf::from(INDEX));
    }

    #[test]
    fn content_hash_is_sha256_in_hex() {
        assert_eq!(
            content_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn index_file_survives_toml_round_trip() {
        let decoded: IndexFile =
            toml::from_str(&index_text("1234")).expect("Failed to decode index");

        assert_eq!(
            decoded,
            IndexFile {
                hash: String::from("1234"),
                outline: outline(),
            }
        );
    }

    #[test]
    fn unreadable_source_fails() {
        let mut fs = MockFileSystem::new();
        fs.expect_read_to_string()
            .returning(|_| Err(io::Error::from(io::ErrorKind::NotFound)));

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_never_called());

        assert!(
            matches!(result, Err(LazyCoderError::SnippetFileError(_))),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn missing_index_is_built_and_written() {
        let mut fs = file_system(None);
        expect_index_written(&mut fs);

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_returning_outline());

        assert_eq!(
            result.expect("Unexpected error"),
            (String::from(TEXT), outline())
        );
    }

    #[test]
    fn matching_index_is_used_without_parsing() {
        let mut fs = file_system(Some(index_text(&content_hash(TEXT))));
        fs.expect_write().never();

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_never_called());

        assert_eq!(
            result.expect("Unexpected error"),
            (String::from(TEXT), outline())
        );
    }

    #[test]
    fn outdated_index_is_rebuilt() {
        let mut fs = file_system(Some(index_text("outdated")));
        expect_index_written(&mut fs);

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_returning_outline());

        assert_eq!(
            result.expect("Unexpected error"),
            (String::from(TEXT), outline())
        );
    }

    #[test]
    fn corrupt_index_is_rebuilt() {
        let mut fs = file_system(Some(String::from("not = [valid")));
        expect_index_written(&mut fs);

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_returning_outline());

        assert_eq!(
            result.expect("Unexpected error"),
            (String::from(TEXT), outline())
        );
    }

    #[test]
    fn parse_error_is_returned_without_writing_index() {
        let mut fs = file_system(None);
        fs.expect_write().never();
        let mut parser = MockDocumentParser::new();
        parser
            .expect_parse()
            .returning(|_| Err(LazyCoderError::MalformedDocument { line: 3 }));

        let result = load_or_build(&fs, Path::new(SOURCE), &parser);

        assert!(
            matches!(result, Err(LazyCoderError::MalformedDocument { line: 3 })),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn index_write_failure_is_reported() {
        let mut fs = file_system(None);
        fs.expect_write()
            .returning(|_, _| Err(io::Error::from(io::ErrorKind::PermissionDenied)));

        let result = load_or_build(&fs, Path::new(SOURCE), &parser_returning_outline());

        assert!(
            matches!(result, Err(LazyCoderError::IndexFileError(_))),
            "Unexpected result: {result:?}"
        );
    }

    fn outline() -> Outline {
        Outline {
            sections: vec![Section::new(1, "Intro", &[1], 0)],
            snippets: vec![],
        }
    }

    fn index_text(hash: &str) -> String {
        toml::to_string(&IndexFile {
            hash: hash.to_string(),
            outline: outline(),
        })
        .expect("Failed to encode index")
    }

    /// File system with the snippet document and, optionally, its index file.
    fn file_system(index: Option<String>) -> MockFileSystem {
        let mut fs = MockFileSystem::new();
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(SOURCE))
            .returning(|_| Ok(String::from(TEXT)));
        fs.expect_read_to_string()
            .withf(|path| path == Path::new(INDEX))
            .returning(move |_| {
                index
                    .clone()
                    .ok_or(io::Error::from(io::ErrorKind::NotFound))
            });
        fs
    }

    fn expect_index_written(fs: &mut MockFileSystem) {
        let expected = index_text(&content_hash(TEXT));
        fs.expect_write()
            .withf(move |path, contents| path == Path::new(INDEX) && contents == expected)
            .once()
            .returning(|_, _| Ok(()));
    }

    fn parser_returning_outline() -> MockDocumentParser {
        let mut parser = MockDocumentParser::new();
        parser
            .expect_parse()
            .withf(|text| text == TEXT)
            .once()
            .returning(|_| Ok(outline()));
        parser
    }

    fn parser_never_called() -> MockDocumentParser {
        let mut parser = MockDocumentParser::new();
        parser.expect_parse().never();
        parser
    }
}
