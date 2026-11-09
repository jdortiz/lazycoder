//! Minimal Markdown parser: ATX headings (levels 1–3) and fenced code blocks.
use crate::{
    document_parser::{DocumentParser, dedent, indentation},
    lazy_coder_error::LazyCoderError,
    outline::{MAX_SECTION_LEVEL, Outline, SnippetLocation},
};

/// Shortest sequence of markers that makes a fence.
const MIN_FENCE_LENGTH: usize = 3;
/// Deepest level of a Markdown heading.
const MAX_HEADING_LEVEL: u8 = 6;
/// Largest indentation of a heading, in spaces.
const MAX_HEADING_INDENT: usize = 3;

/// Parser for Markdown documents.
pub struct MarkdownParser;

/// Fenced block being parsed.
struct OpenFence {
    begin_line: usize,
    marker: char,
    length: usize,
    indent: usize,
}

impl DocumentParser for MarkdownParser {
    fn parse(&self, text: &str) -> Result<Outline, LazyCoderError> {
        let mut outline = Outline::default();
        let mut counters = [0; MAX_SECTION_LEVEL as usize];
        let mut open_fence: Option<OpenFence> = None;

        for (index, line) in text.lines().enumerate() {
            if let Some(fence) = &open_fence {
                if is_fence_end(line, fence.marker, fence.length) {
                    outline.snippets.push(SnippetLocation::new(
                        fence.begin_line + 1,
                        index,
                        fence.indent,
                    ));
                    open_fence = None;
                }
            } else if let Some((marker, length)) = fence_begin(line) {
                open_fence = Some(OpenFence {
                    begin_line: index,
                    marker,
                    length,
                    indent: indentation(line),
                });
            } else if let Some((level, title)) = heading(line)
                && level <= MAX_SECTION_LEVEL
            {
                outline.add_section(&mut counters, level, title);
            }
        }

        match open_fence {
            Some(fence) => Err(LazyCoderError::MalformedDocument {
                line: fence.begin_line + 1,
            }),
            None => Ok(outline),
        }
    }

    fn extract(&self, text: &str, location: &SnippetLocation) -> String {
        text.lines()
            .skip(location.start_line)
            .take(location.end_line.saturating_sub(location.start_line))
            .map(|line| dedent(line, location.indent).to_string() + "\n")
            .collect()
    }
}

/// Level and title of the heading in the line, if it is one.
fn heading(line: &str) -> Option<(u8, &str)> {
    let content = line.trim_start_matches(' ');
    if line.len() - content.len() > MAX_HEADING_INDENT {
        return None;
    }
    let rest = content.trim_start_matches('#');
    let level = u8::try_from(content.len() - rest.len()).ok()?;
    let title = without_closing_sequence(rest.trim());
    ((1..=MAX_HEADING_LEVEL).contains(&level) && rest.starts_with([' ', '\t']) && !title.is_empty())
        .then_some((level, title))
}

/// Title without its optional closing sequence of `#`, which has to be separated from the text.
fn without_closing_sequence(title: &str) -> &str {
    let text = title.trim_end_matches('#');
    if text.is_empty() || text.ends_with([' ', '\t']) {
        text.trim_end()
    } else {
        title
    }
}

/// Marker and length of the fence opened in the line, if any.
fn fence_begin(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let info = trimmed.trim_start_matches(marker);
    let length = trimmed.len() - info.len();
    (length >= MIN_FENCE_LENGTH && (marker == '~' || !info.contains('`')))
        .then_some((marker, length))
}

/// Whether the line closes a fence with the given marker and length.
fn is_fence_end(line: &str, marker: char, length: usize) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= length && trimmed.chars().all(|c| c == marker)
}

#[cfg(test)]
mod tests {
    use crate::outline::Section;

    use super::*;

    #[test]
    fn text_without_blocks_produces_empty_outline() {
        assert_eq!(parse(&["Some text", "more text"]), Outline::default());
    }

    #[test]
    fn backtick_fence_is_located() {
        let outline = parse(&["```", "fn main() {}", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
        assert_eq!(outline.sections, vec![]);
    }

    #[test]
    fn tilde_fence_is_located() {
        let outline = parse(&["Intro", "~~~", "a", "b", "~~~"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(2, 4, 0)]);
    }

    #[test]
    fn info_string_does_not_affect_location() {
        let outline = parse(&["```rust title=\"main\"", "fn main() {}", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn tilde_fence_info_string_can_contain_backticks() {
        let outline = parse(&["~~~ a`b", "ls", "~~~"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn inline_triple_backticks_are_not_a_fence() {
        assert_eq!(parse(&["```inline``` text"]), Outline::default());
    }

    #[test]
    fn two_markers_are_not_a_fence() {
        assert_eq!(parse(&["``", "ls", "``"]), Outline::default());
    }

    #[test]
    fn empty_block_is_a_snippet() {
        let outline = parse(&["```", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 1, 0)]);
    }

    #[test]
    fn indented_fence_records_indent() {
        let outline = parse(&["1. Step one:", "   ```sh", "   ls", "   ```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(2, 3, 3)]);
    }

    #[test]
    fn blocks_are_located_in_order() {
        let outline = parse(&["```", "ls", "```", "~~~", "pwd", "~~~"]);

        assert_eq!(
            outline.snippets,
            vec![SnippetLocation::new(1, 2, 0), SnippetLocation::new(4, 5, 0)]
        );
    }

    #[test]
    fn longer_fence_can_contain_shorter_one() {
        let outline = parse(&["````markdown", "```", "ls", "```", "````"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 4, 0)]);
    }

    #[test]
    fn longer_closing_fence_closes_block() {
        let outline = parse(&["```", "ls", "`````"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn other_marker_does_not_close_block() {
        let outline = parse(&["```", "~~~", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn closing_fence_with_trailing_text_does_not_close_block() {
        let outline = parse(&["```", "``` sh", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn closing_fence_can_have_trailing_whitespace() {
        let outline = parse(&["```", "ls", "```  "]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn blank_line_does_not_close_block() {
        let outline = parse(&["```", "", "```"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn heading_inside_block_is_not_a_section() {
        let outline = parse(&["```sh", "# Not a heading", "```"]);

        assert_eq!(outline.sections, vec![]);
        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn unterminated_block_is_malformed() {
        let result = MarkdownParser.parse(&text(&["# Title", "```sh", "ls"]));

        assert!(
            matches!(result, Err(LazyCoderError::MalformedDocument { line: 2 })),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn headings_of_levels_one_to_three_are_sections() {
        let outline = parse(&[
            "# One",
            "## One.One",
            "### One.One.One",
            "# Two",
            "## Two.One",
        ]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 0),
                Section::new(2, "One.One", &[1, 1], 0),
                Section::new(3, "One.One.One", &[1, 1, 1], 0),
                Section::new(1, "Two", &[2], 0),
                Section::new(2, "Two.One", &[2, 1], 0),
            ]
        );
    }

    #[test]
    fn deep_headings_are_ignored() {
        let outline = parse(&["# One", "#### Deep", "###### Deeper", "```", "ls", "```"]);

        assert_eq!(outline.sections, vec![Section::new(1, "One", &[1], 0)]);
        assert_eq!(outline.snippets, vec![SnippetLocation::new(4, 5, 0)]);
    }

    #[test]
    fn skipped_level_gets_zero_counter() {
        let outline = parse(&["# One", "### Deep"]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 0),
                Section::new(3, "Deep", &[1, 0, 1], 0)
            ]
        );
    }

    #[test]
    fn heading_needs_whitespace_and_title() {
        assert_eq!(
            parse(&["#hashtag", "#!/bin/sh", "#", "# ", "# ##", "####### Seven"]),
            Outline::default()
        );
    }

    #[test]
    fn heading_can_be_indented_up_to_three_spaces() {
        let outline = parse(&["   # Indented", "    # Code"]);

        assert_eq!(outline.sections, vec![Section::new(1, "Indented", &[1], 0)]);
    }

    #[test]
    fn heading_can_be_separated_with_a_tab() {
        let outline = parse(&["#\tTabbed"]);

        assert_eq!(outline.sections, vec![Section::new(1, "Tabbed", &[1], 0)]);
    }

    #[test]
    fn heading_title_is_trimmed() {
        let outline = parse(&["#   Spaced title   "]);

        assert_eq!(
            outline.sections,
            vec![Section::new(1, "Spaced title", &[1], 0)]
        );
    }

    #[test]
    fn closing_sequence_is_removed_from_title() {
        let outline = parse(&["## Title ##"]);

        assert_eq!(outline.sections, vec![Section::new(2, "Title", &[0, 1], 0)]);
    }

    #[test]
    fn hash_attached_to_title_is_kept() {
        let outline = parse(&["# C#"]);

        assert_eq!(outline.sections, vec![Section::new(1, "C#", &[1], 0)]);
    }

    #[test]
    fn section_points_to_its_first_snippet() {
        let outline = parse(&[
            "```", "a", "```", "# One", "```", "b", "```", "# Two", "```", "c", "```",
        ]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 1),
                Section::new(1, "Two", &[2], 2)
            ]
        );
        assert_eq!(outline.snippets.len(), 3);
    }

    #[test]
    fn extract_returns_lines_with_trailing_newline() {
        let text = text(&["```", "ls", "pwd", "```"]);

        assert_eq!(
            MarkdownParser.extract(&text, &SnippetLocation::new(1, 3, 0)),
            "ls\npwd\n"
        );
    }

    #[test]
    fn extract_of_empty_block_is_empty() {
        let text = text(&["```", "```"]);

        assert_eq!(
            MarkdownParser.extract(&text, &SnippetLocation::new(1, 1, 0)),
            ""
        );
    }

    #[test]
    fn extract_removes_block_indent() {
        let text = text(&[
            "   ```rust",
            "   fn main() {",
            "       todo!()",
            "   }",
            "   ```",
        ]);

        assert_eq!(
            MarkdownParser.extract(&text, &SnippetLocation::new(1, 4, 3)),
            "fn main() {\n    todo!()\n}\n"
        );
    }

    #[test]
    fn extract_tolerates_lines_with_less_indent() {
        let text = text(&["  ```", "  a", "", "b", "  ```"]);

        assert_eq!(
            MarkdownParser.extract(&text, &SnippetLocation::new(1, 4, 2)),
            "a\n\nb\n"
        );
    }

    #[test]
    fn extract_keeps_commas() {
        let text = text(&["```", ",* not escaped", "```"]);

        assert_eq!(
            MarkdownParser.extract(&text, &SnippetLocation::new(1, 2, 0)),
            ",* not escaped\n"
        );
    }

    fn text(lines: &[&str]) -> String {
        lines.join("\n") + "\n"
    }

    fn parse(lines: &[&str]) -> Outline {
        MarkdownParser
            .parse(&text(lines))
            .expect("Unexpected error parsing document")
    }
}
