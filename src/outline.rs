//! Format-agnostic structure of a snippet document: its sections (headings) and the location of its snippets.
use std::ops::Range;

use serde::{Deserialize, Serialize};

/// Structure of a snippet document.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct Outline {
    pub sections: Vec<Section>,
    pub snippets: Vec<SnippetLocation>,
}

impl Outline {
    /// Snippets of the section at `index` that are not in any of its subsections: up to the next heading.
    ///
    /// # Panics
    ///
    /// If there is no section at `index`.
    pub fn own_snippets(&self, index: usize) -> Range<usize> {
        let end = self
            .sections
            .get(index + 1)
            .map_or(self.snippets.len(), |next| next.first_snippet);
        self.sections[index].first_snippet..end
    }

    /// Snippets of the section at `index`, including those of its subsections: up to the next heading of the same or
    /// a higher level.
    ///
    /// # Panics
    ///
    /// If there is no section at `index`.
    pub fn total_snippets(&self, index: usize) -> Range<usize> {
        let section = &self.sections[index];
        let end = self.sections[index + 1..]
            .iter()
            .find(|next| next.level <= section.level)
            .map_or(self.snippets.len(), |next| next.first_snippet);
        section.first_snippet..end
    }

    /// Index of the section that the snippet belongs to, i.e. the one whose own snippets include it. `None` if the
    /// snippet is before the first section or doesn't exist.
    pub fn section_of(&self, snippet: usize) -> Option<usize> {
        (0..self.sections.len()).find(|&index| self.own_snippets(index).contains(&snippet))
    }
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
    pub fn new(level: u8, title: &str, number: &[usize], first_snippet: usize) -> Self {
        Section {
            level,
            title: title.to_string(),
            number: number.to_vec(),
            first_snippet,
        }
    }

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

impl SnippetLocation {
    pub fn new(start_line: usize, end_line: usize, indent: usize) -> Self {
        SnippetLocation {
            start_line,
            end_line,
            indent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_string_of_top_level_section() {
        assert_eq!(Section::new(1, "Title", &[2], 0).number_string(), "2");
    }

    #[test]
    fn number_string_of_nested_section() {
        assert_eq!(
            Section::new(3, "Title", &[2, 1, 3], 0).number_string(),
            "2.1.3"
        );
    }

    #[test]
    fn own_snippets_end_at_next_section() {
        assert_eq!(outline().own_snippets(1), 2..3);
    }

    #[test]
    fn own_snippets_of_section_are_separated_from_subsection() {
        assert_eq!(outline().own_snippets(0), 0..2);
    }

    #[test]
    fn own_snippets_of_last_section_end_at_last_snippet() {
        assert_eq!(outline().own_snippets(5), 5..8);
    }

    #[test]
    fn own_snippets_of_section_without_snippets_are_empty() {
        assert!(outline().own_snippets(4).is_empty());
    }

    #[test]
    fn own_snippets_of_trailing_section_without_snippets_are_empty() {
        let mut outline = outline();
        outline.sections.push(Section::new(1, "End", &[3], 8));

        assert_eq!(outline.own_snippets(6), 8..8);
    }

    #[test]
    fn total_snippets_include_subsections() {
        assert_eq!(outline().total_snippets(0), 0..5);
    }

    #[test]
    fn total_snippets_of_subsection_end_at_parent_sibling() {
        assert_eq!(outline().total_snippets(1), 2..5);
    }

    #[test]
    fn total_snippets_of_deepest_section_end_at_next_section() {
        assert_eq!(outline().total_snippets(2), 3..5);
    }

    #[test]
    fn total_snippets_of_last_top_level_section_end_at_last_snippet() {
        assert_eq!(outline().total_snippets(3), 5..8);
    }

    #[test]
    fn total_snippets_with_skipped_level_end_at_higher_level() {
        let outline = Outline {
            sections: vec![
                Section::new(1, "Top", &[1], 0),
                Section::new(3, "Deep", &[1, 0, 1], 1),
                Section::new(2, "Middle", &[1, 1], 2),
            ],
            snippets: snippets(3),
        };

        assert_eq!(outline.total_snippets(1), 1..2);
    }

    #[test]
    fn snippet_belongs_to_top_level_section() {
        assert_eq!(outline().section_of(1), Some(0));
    }

    #[test]
    fn snippet_belongs_to_deepest_section() {
        assert_eq!(outline().section_of(4), Some(2));
    }

    #[test]
    fn snippet_belongs_to_section_with_own_snippets_among_those_sharing_start() {
        assert_eq!(outline().section_of(5), Some(5));
    }

    #[test]
    fn snippet_past_the_last_one_belongs_to_no_section() {
        assert_eq!(outline().section_of(8), None);
    }

    #[test]
    fn snippet_before_first_section_belongs_to_no_section() {
        let outline = Outline {
            sections: vec![Section::new(1, "Intro", &[1], 2)],
            snippets: snippets(3),
        };

        assert_eq!(outline.section_of(1), None);
    }

    #[test]
    fn snippet_belongs_to_no_section_without_sections() {
        let outline = Outline {
            sections: vec![],
            snippets: snippets(3),
        };

        assert_eq!(outline.section_of(0), None);
    }

    #[test]
    fn outline_survives_toml_round_trip() {
        let outline = Outline {
            sections: vec![Section::new(1, "Intro", &[1], 0)],
            snippets: vec![SnippetLocation::new(3, 5, 2)],
        };

        let text = toml::to_string(&outline).expect("Failed to encode outline");
        let decoded: Outline = toml::from_str(&text).expect("Failed to decode outline");

        assert_eq!(decoded, outline);
    }

    /// Outline with 8 snippets:
    ///
    /// | Index | Section     | First | Own | Total |
    /// |-------|-------------|-------|-----|-------|
    /// | 0     | 1 Intro     | 0     | 0–1 | 0–4   |
    /// | 1     | 1.1 Setup   | 2     | 2–2 | 2–4   |
    /// | 2     | 1.1.1 Steps | 3     | 3–4 | 3–4   |
    /// | 3     | 2 Code      | 5     | —   | 5–8   |
    /// | 4     | 2.1 Empty   | 5     | —   | 5–5   |
    /// | 5     | 2.2 More    | 5     | 5–8 | 5–8   |
    fn outline() -> Outline {
        Outline {
            sections: vec![
                Section::new(1, "Intro", &[1], 0),
                Section::new(2, "Setup", &[1, 1], 2),
                Section::new(3, "Steps", &[1, 1, 1], 3),
                Section::new(1, "Code", &[2], 5),
                Section::new(2, "Empty", &[2, 1], 5),
                Section::new(2, "More", &[2, 2], 5),
            ],
            snippets: snippets(8),
        }
    }

    /// Consecutive one-line snippets.
    fn snippets(count: usize) -> Vec<SnippetLocation> {
        (0..count)
            .map(|line| SnippetLocation::new(line, line + 1, 0))
            .collect()
    }
}
