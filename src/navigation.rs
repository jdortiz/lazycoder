//! Movement between the sections of a snippet document. Each function returns the new position, i.e. the index of the
//! next snippet to be provided.
use crate::{lazy_coder_error::LazyCoderError, outline::Section};

/// Position of the first snippet of the `count`-th section after the current position. Sections that start at the
/// same snippet count once.
pub fn next_section(
    sections: &[Section],
    position: usize,
    count: usize,
) -> Result<usize, LazyCoderError> {
    let starts = section_starts(sections)?;
    let Some(skipped) = count.checked_sub(1) else {
        return Ok(position);
    };
    starts
        .into_iter()
        .filter(|&start| start > position)
        .nth(skipped)
        .ok_or(LazyCoderError::OperationOutOfRange)
}

/// Position of the first snippet of the `count`-th section before the current position. If the current position is
/// past the start of its section, the first step goes back to that start.
pub fn prev_section(
    sections: &[Section],
    position: usize,
    count: usize,
) -> Result<usize, LazyCoderError> {
    let starts = section_starts(sections)?;
    let Some(skipped) = count.checked_sub(1) else {
        return Ok(position);
    };
    starts
        .into_iter()
        .rev()
        .filter(|&start| start < position)
        .nth(skipped)
        .ok_or(LazyCoderError::OperationOutOfRange)
}

/// Position of the first snippet of the section with the given outline number, e.g. "2.1".
pub fn goto(sections: &[Section], number: &str) -> Result<usize, LazyCoderError> {
    if sections.is_empty() {
        return Err(LazyCoderError::NoSections);
    }
    let number = parse_number(number).ok_or(LazyCoderError::SectionNotFound)?;
    sections
        .iter()
        .find(|section| section.number == number)
        .map(|section| section.first_snippet)
        .ok_or(LazyCoderError::SectionNotFound)
}

/// Distinct positions where the sections start, in document order.
fn section_starts(sections: &[Section]) -> Result<Vec<usize>, LazyCoderError> {
    if sections.is_empty() {
        return Err(LazyCoderError::NoSections);
    }
    let mut starts: Vec<usize> = sections
        .iter()
        .map(|section| section.first_snippet)
        .collect();
    // Starts never decrease in document order, so equal ones are consecutive.
    starts.dedup();
    Ok(starts)
}

/// Outline number from its dotted representation, e.g. "2.1" → `[2, 1]`.
fn parse_number(number: &str) -> Option<Vec<usize>> {
    number
        .trim()
        .split('.')
        .map(|part| part.parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_section_skips_sections_sharing_the_start() {
        assert_eq!(next_section(&sections(), 0, 1).unwrap(), 2);
    }

    #[test]
    fn next_section_from_the_middle_of_a_section() {
        assert_eq!(next_section(&sections(), 3, 1).unwrap(), 4);
    }

    #[test]
    fn next_section_several_times() {
        assert_eq!(next_section(&sections(), 1, 2).unwrap(), 4);
    }

    #[test]
    fn next_section_reaches_trailing_section_without_snippets() {
        assert_eq!(next_section(&sections(), 4, 1).unwrap(), 5);
    }

    #[test]
    fn next_section_zero_times_keeps_position() {
        assert_eq!(next_section(&sections(), 3, 0).unwrap(), 3);
    }

    #[test]
    fn next_section_beyond_last_fails() {
        assert!(matches!(
            next_section(&sections(), 4, 2),
            Err(LazyCoderError::OperationOutOfRange)
        ));
    }

    #[test]
    fn next_section_without_sections_fails() {
        assert!(matches!(
            next_section(&[], 0, 1),
            Err(LazyCoderError::NoSections)
        ));
    }

    #[test]
    fn prev_section_from_the_middle_goes_to_start_of_current() {
        assert_eq!(prev_section(&sections(), 3, 1).unwrap(), 2);
    }

    #[test]
    fn prev_section_from_the_start_goes_to_previous() {
        assert_eq!(prev_section(&sections(), 2, 1).unwrap(), 0);
    }

    #[test]
    fn prev_section_several_times() {
        assert_eq!(prev_section(&sections(), 5, 2).unwrap(), 2);
    }

    #[test]
    fn prev_section_zero_times_keeps_position() {
        assert_eq!(prev_section(&sections(), 3, 0).unwrap(), 3);
    }

    #[test]
    fn prev_section_before_first_fails() {
        assert!(matches!(
            prev_section(&sections(), 0, 1),
            Err(LazyCoderError::OperationOutOfRange)
        ));
    }

    #[test]
    fn prev_section_without_sections_fails() {
        assert!(matches!(
            prev_section(&[], 3, 1),
            Err(LazyCoderError::NoSections)
        ));
    }

    #[test]
    fn goto_top_level_section() {
        assert_eq!(goto(&sections(), "2").unwrap(), 2);
    }

    #[test]
    fn goto_nested_section() {
        assert_eq!(goto(&sections(), "2.1").unwrap(), 4);
    }

    #[test]
    fn goto_section_without_snippets_goes_to_next_snippet() {
        assert_eq!(goto(&sections(), "1.2").unwrap(), 2);
    }

    #[test]
    fn goto_ignores_surrounding_whitespace() {
        assert_eq!(goto(&sections(), " 2.1 ").unwrap(), 4);
    }

    #[test]
    fn goto_unexisting_section_fails() {
        assert!(matches!(
            goto(&sections(), "2.2"),
            Err(LazyCoderError::SectionNotFound)
        ));
    }

    #[test]
    fn goto_invalid_number_fails() {
        for number in ["", "2.", "a", "2.x", "-1"] {
            assert!(
                matches!(
                    goto(&sections(), number),
                    Err(LazyCoderError::SectionNotFound)
                ),
                "Unexpected result for {number:?}"
            );
        }
    }

    #[test]
    fn goto_without_sections_fails() {
        assert!(matches!(goto(&[], "1"), Err(LazyCoderError::NoSections)));
    }

    /// Sections of a document with 5 snippets. Distinct starts: 0, 2, 4, and 5 (end of document).
    fn sections() -> Vec<Section> {
        vec![
            Section::new(1, "Intro", &[1], 0),
            Section::new(2, "Setup", &[1, 1], 0),
            Section::new(2, "No snippets", &[1, 2], 2),
            Section::new(1, "Code", &[2], 2),
            Section::new(2, "More code", &[2, 1], 4),
            Section::new(1, "End", &[3], 5),
        ]
    }
}
