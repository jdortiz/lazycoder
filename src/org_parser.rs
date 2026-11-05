//! Minimal Org mode parser: headings (levels 1–3) and source/example blocks.
use std::borrow::Cow;

use crate::{
    document_parser::DocumentParser,
    lazy_coder_error::LazyCoderError,
    outline::{Outline, Section, SnippetLocation},
};

/// Deepest heading level that creates a section.
const MAX_SECTION_LEVEL: u8 = 3;
/// Kinds of blocks that contain snippets, i.e. `#+begin_<kind>` ... `#+end_<kind>`.
const BLOCK_KINDS: [&str; 2] = ["src", "example"];

/// Parser for Org mode documents.
pub struct OrgParser;

/// Block being parsed.
struct OpenBlock {
    begin_line: usize,
    kind: &'static str,
    indent: usize,
}

impl DocumentParser for OrgParser {
    fn parse(&self, text: &str) -> Result<Outline, LazyCoderError> {
        let mut outline = Outline::default();
        let mut counters = [0; MAX_SECTION_LEVEL as usize];
        let mut open_block: Option<OpenBlock> = None;

        for (index, line) in text.lines().enumerate() {
            if let Some(block) = &open_block {
                if is_block_end(line, block.kind) {
                    outline.snippets.push(SnippetLocation::new(
                        block.begin_line + 1,
                        index,
                        block.indent,
                    ));
                    open_block = None;
                }
            } else if let Some(kind) = block_begin(line) {
                open_block = Some(OpenBlock {
                    begin_line: index,
                    kind,
                    indent: indentation(line),
                });
            } else if let Some((level, title)) = heading(line)
                && level <= MAX_SECTION_LEVEL
            {
                let depth = usize::from(level);
                counters[depth - 1] += 1;
                counters[depth..].fill(0);
                outline.sections.push(Section::new(
                    level,
                    title,
                    &counters[..depth],
                    outline.snippets.len(),
                ));
            }
        }

        match open_block {
            Some(block) => Err(LazyCoderError::MalformedDocument {
                line: block.begin_line + 1,
            }),
            None => Ok(outline),
        }
    }

    fn extract(&self, text: &str, location: &SnippetLocation) -> String {
        text.lines()
            .skip(location.start_line)
            .take(location.end_line.saturating_sub(location.start_line))
            .map(|line| unescape(dedent(line, location.indent)) + "\n")
            .collect()
    }
}

/// Level and title of the heading in the line, if it is one.
fn heading(line: &str) -> Option<(u8, &str)> {
    let rest = line.trim_start_matches('*');
    let level = u8::try_from(line.len() - rest.len()).ok()?;
    let title = rest.trim();
    (level > 0 && rest.starts_with(char::is_whitespace) && !title.is_empty())
        .then_some((level, title))
}

/// Kind of the block started in the line, if any.
fn block_begin(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    BLOCK_KINDS.into_iter().find(|kind| {
        let marker = format!("#+begin_{kind}");
        trimmed
            .get(..marker.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(&marker))
            && trimmed[marker.len()..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace)
    })
}

/// Whether the line closes a block of the given kind.
fn is_block_end(line: &str, kind: &str) -> bool {
    line.trim().eq_ignore_ascii_case(&format!("#+end_{kind}"))
}

/// Number of leading whitespace characters.
fn indentation(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

/// Line without up to `indent` leading whitespace characters.
fn dedent(line: &str, indent: usize) -> &str {
    let mut rest = line;
    for _ in 0..indent {
        match rest.strip_prefix(char::is_whitespace) {
            Some(stripped) => rest = stripped,
            None => break,
        }
    }
    rest
}

/// Line without the comma Org uses to protect lines starting with `*` or `#+` inside blocks.
fn unescape(line: &str) -> Cow<'_, str> {
    let content = line.trim_start();
    match content.strip_prefix(',') {
        Some(escaped) if escaped.starts_with('*') || escaped.starts_with("#+") => {
            let indentation = &line[..line.len() - content.len()];
            Cow::Owned(format!("{indentation}{escaped}"))
        }
        _ => Cow::Borrowed(line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_without_blocks_produces_empty_outline() {
        assert_eq!(parse(&["Some text", "more text"]), Outline::default());
    }

    #[test]
    fn src_block_is_located() {
        let outline = parse(&["#+begin_src rust", "fn main() {}", "#+end_src"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
        assert_eq!(outline.sections, vec![]);
    }

    #[test]
    fn example_block_is_located() {
        let outline = parse(&["Intro", "#+begin_example", "a", "b", "#+end_example"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(2, 4, 0)]);
    }

    #[test]
    fn block_markers_are_case_insensitive() {
        let outline = parse(&["#+BEGIN_SRC sh", "ls", "#+END_SRC"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn header_arguments_do_not_affect_location() {
        let outline = parse(&[
            "#+begin_src rust :tangle src/main.rs :noweb yes",
            "fn main() {}",
            "#+end_src",
        ]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn similar_keyword_is_not_a_block() {
        assert_eq!(
            parse(&["#+begin_srcfoo", "ls", "#+end_src"]),
            Outline::default()
        );
    }

    #[test]
    fn indented_block_records_indent() {
        let outline = parse(&["1. Step one:", "   #+begin_src sh", "   ls", "   #+end_src"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(2, 3, 3)]);
    }

    #[test]
    fn empty_block_is_a_snippet() {
        let outline = parse(&["#+begin_src sh", "#+end_src"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 1, 0)]);
    }

    #[test]
    fn blocks_are_located_in_order() {
        let outline = parse(&[
            "#+begin_src sh",
            "ls",
            "#+end_src",
            "#+begin_example",
            "pwd",
            "#+end_example",
        ]);

        assert_eq!(
            outline.snippets,
            vec![SnippetLocation::new(1, 2, 0), SnippetLocation::new(4, 5, 0)]
        );
    }

    #[test]
    fn end_marker_of_other_kind_does_not_close_block() {
        let outline = parse(&["#+begin_src org", "#+end_example", "#+end_src"]);

        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn heading_inside_block_is_not_a_section() {
        let outline = parse(&["#+begin_src org", "* Not a heading", "#+end_src"]);

        assert_eq!(outline.sections, vec![]);
        assert_eq!(outline.snippets, vec![SnippetLocation::new(1, 2, 0)]);
    }

    #[test]
    fn unterminated_block_is_malformed() {
        let result = OrgParser.parse(&text(&["* Title", "#+begin_src sh", "ls"]));

        assert!(
            matches!(result, Err(LazyCoderError::MalformedDocument { line: 2 })),
            "Unexpected result: {result:?}"
        );
    }

    #[test]
    fn headings_of_levels_one_to_three_are_sections() {
        let outline = parse(&[
            "* One",
            "** One.One",
            "*** One.One.One",
            "* Two",
            "** Two.One",
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
    fn level_four_heading_is_ignored() {
        let outline = parse(&["* One", "**** Deep", "#+begin_src sh", "ls", "#+end_src"]);

        assert_eq!(outline.sections, vec![Section::new(1, "One", &[1], 0)]);
        assert_eq!(outline.snippets, vec![SnippetLocation::new(3, 4, 0)]);
    }

    #[test]
    fn skipped_level_gets_zero_counter() {
        let outline = parse(&["* One", "*** Deep"]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 0),
                Section::new(3, "Deep", &[1, 0, 1], 0)
            ]
        );
    }

    #[test]
    fn stars_without_title_are_not_headings() {
        assert_eq!(parse(&["*bold* text", "**", "* "]), Outline::default());
    }

    #[test]
    fn heading_title_is_trimmed() {
        let outline = parse(&["*   Spaced title   "]);

        assert_eq!(
            outline.sections,
            vec![Section::new(1, "Spaced title", &[1], 0)]
        );
    }

    #[test]
    fn section_points_to_its_first_snippet() {
        let outline = parse(&[
            "#+begin_src sh",
            "a",
            "#+end_src",
            "* One",
            "#+begin_src sh",
            "b",
            "#+end_src",
            "* Two",
            "#+begin_src sh",
            "c",
            "#+end_src",
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
    fn sections_without_snippets_share_next_start() {
        let outline = parse(&[
            "* One",
            "** Empty",
            "* Two",
            "#+begin_src sh",
            "a",
            "#+end_src",
        ]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 0),
                Section::new(2, "Empty", &[1, 1], 0),
                Section::new(1, "Two", &[2], 0),
            ]
        );
    }

    #[test]
    fn trailing_section_without_snippets_points_past_last() {
        let outline = parse(&["* One", "#+begin_src sh", "a", "#+end_src", "* Two"]);

        assert_eq!(
            outline.sections,
            vec![
                Section::new(1, "One", &[1], 0),
                Section::new(1, "Two", &[2], 1)
            ]
        );
    }

    #[test]
    fn extract_returns_lines_with_trailing_newline() {
        let text = text(&["#+begin_src sh", "ls", "pwd", "#+end_src"]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 3, 0)),
            "ls\npwd\n"
        );
    }

    #[test]
    fn extract_of_empty_block_is_empty() {
        let text = text(&["#+begin_src sh", "#+end_src"]);

        assert_eq!(OrgParser.extract(&text, &SnippetLocation::new(1, 1, 0)), "");
    }

    #[test]
    fn extract_removes_block_indent() {
        let text = text(&[
            "   #+begin_src rust",
            "   fn main() {",
            "       todo!()",
            "   }",
            "   #+end_src",
        ]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 4, 3)),
            "fn main() {\n    todo!()\n}\n"
        );
    }

    #[test]
    fn extract_tolerates_lines_with_less_indent() {
        let text = text(&["  #+begin_src sh", "  a", "", "b", "  #+end_src"]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 4, 2)),
            "a\n\nb\n"
        );
    }

    #[test]
    fn extract_unescapes_protected_heading() {
        let text = text(&["#+begin_src org", ",* Heading", "#+end_src"]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 2, 0)),
            "* Heading\n"
        );
    }

    #[test]
    fn extract_unescapes_indented_protected_keyword() {
        let text = text(&["#+begin_src org", "  ,#+begin_src sh", "#+end_src"]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 2, 0)),
            "  #+begin_src sh\n"
        );
    }

    #[test]
    fn extract_keeps_other_commas() {
        let text = text(&["#+begin_src sh", ",not escaped", "#+end_src"]);

        assert_eq!(
            OrgParser.extract(&text, &SnippetLocation::new(1, 2, 0)),
            ",not escaped\n"
        );
    }

    fn text(lines: &[&str]) -> String {
        lines.join("\n") + "\n"
    }

    fn parse(lines: &[&str]) -> Outline {
        OrgParser
            .parse(&text(lines))
            .expect("Unexpected error parsing document")
    }
}
