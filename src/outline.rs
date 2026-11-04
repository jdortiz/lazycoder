//! Format-agnostic structure of a snippet document: its sections (headings) and the location of its snippets.
use serde_derive::{Deserialize, Serialize};

/// Structure of a snippet document.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct Outline {
    pub sections: Vec<Section>,
    pub snippets: Vec<SnippetLocation>,
}

/// Heading that can be used to navigate the document.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Section {
    pub level: u8,
    pub title: String,
    /// Outline path, e.g. `[2, 1]` for section "2.1".
    pub number: Vec<usize>,
    /// Index of the first snippet at or after this heading.
    pub first_snippet: usize,
}

impl Section {
    /// Dotted representation of the outline number, e.g. "2.1".
    pub fn number_string(&self) -> String {
        self.number
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Lines of the document that contain a snippet.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SnippetLocation {
    /// First content line (0-based).
    pub start_line: usize,
    /// Line after the last content line (exclusive).
    pub end_line: usize,
    /// Indentation of the block to be removed from each line.
    pub indent: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(number: Vec<usize>) -> Section {
        Section {
            level: u8::try_from(number.len()).unwrap(),
            title: String::from("Title"),
            number,
            first_snippet: 0,
        }
    }

    #[test]
    fn number_string_of_top_level_section() {
        assert_eq!(section(vec![2]).number_string(), "2");
    }

    #[test]
    fn number_string_of_nested_section() {
        assert_eq!(section(vec![2, 1, 3]).number_string(), "2.1.3");
    }

    #[test]
    fn outline_survives_toml_round_trip() {
        let outline = Outline {
            sections: vec![Section {
                level: 1,
                title: String::from("Intro"),
                number: vec![1],
                first_snippet: 0,
            }],
            snippets: vec![SnippetLocation {
                start_line: 3,
                end_line: 5,
                indent: 2,
            }],
        };

        let text = toml::to_string(&outline).expect("Failed to encode outline");
        let decoded: Outline = toml::from_str(&text).expect("Failed to decode outline");

        assert_eq!(decoded, outline);
    }
}
