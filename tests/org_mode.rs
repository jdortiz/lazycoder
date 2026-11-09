//! Integration tests running lazycoder with Org mode snippet files.
// The configuration directory is redirected with `HOME`, which isn't used on Windows: running these tests there would
// modify the configuration of the user.
#![cfg(unix)]

mod common;

use common::Sandbox;

/// Demo with an indented block in a list, nested sections, an example block with a protective comma, and a trailing
/// section without snippets.
const DEMO: &str = "\
#+title: Demo
* Setup
1. Create the project
   #+begin_src shell
   cargo new demo
   #+end_src
* Code
** Main function
#+begin_src rust
fn main() {
    println!(\"Hello\");
}
#+end_src
** Escaped
#+begin_example
,* not a heading
#+end_example
* Empty
";
const SETUP: &str = "cargo new demo\n";
const MAIN: &str = "fn main() {\n    println!(\"Hello\");\n}\n";
const ESCAPED: &str = "* not a heading\n";

/// Sandbox where the demo has been started.
fn started_demo() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.write("demo.org", DEMO);
    sandbox.succeed(&["start", "demo.org"]);
    sandbox
}

#[test]
fn next_provides_snippets_in_document_order() {
    let sandbox = started_demo();

    assert_eq!(sandbox.succeed(&["next"]), SETUP);
    assert_eq!(sandbox.succeed(&["next"]), MAIN);
    assert_eq!(sandbox.succeed(&["next"]), ESCAPED);
}

#[test]
fn next_after_last_snippet_fails() {
    let sandbox = started_demo();
    sandbox.succeed(&["forward", "3"]);

    let error = sandbox.fail(&["next"]);

    assert!(
        error.contains("out of range of available snippets"),
        "Unexpected error: {error}"
    );
}

#[test]
fn peek_does_not_move_forward() {
    let sandbox = started_demo();

    assert_eq!(sandbox.succeed(&["peek"]), SETUP);
    assert_eq!(sandbox.succeed(&["peek"]), SETUP);
    assert_eq!(sandbox.succeed(&["next"]), SETUP);
}

#[test]
fn forward_and_rewind_move_the_position() {
    let sandbox = started_demo();

    sandbox.succeed(&["forward", "2"]);
    assert_eq!(sandbox.succeed(&["peek"]), ESCAPED);
    sandbox.succeed(&["rewind"]);
    assert_eq!(sandbox.succeed(&["peek"]), MAIN);
}

#[test]
fn rewind_before_first_snippet_fails() {
    let sandbox = started_demo();

    let error = sandbox.fail(&["rewind"]);

    assert!(
        error.contains("operation out of range"),
        "Unexpected error: {error}"
    );
}

#[test]
fn list_sections_shows_outline_with_current_section() {
    let sandbox = started_demo();

    assert_eq!(
        sandbox.succeed(&["list-sections"]),
        "\
> 1 Setup [f:0/o:1/t:1]
  2 Code [f:1/o:0/t:2]
    2.1 Main function [f:1/o:1/t:1]
    2.2 Escaped [f:2/o:1/t:1]
  3 Empty [f:3/o:0/t:0]
"
    );
}

#[test]
fn list_sections_marks_section_of_next_snippet() {
    let sandbox = started_demo();
    sandbox.succeed(&["forward", "2"]);

    assert_eq!(
        sandbox.succeed(&["list-sections"]),
        "  1 Setup [f:0/o:1/t:1]
  2 Code [f:1/o:0/t:2]
    2.1 Main function [f:1/o:1/t:1]
>   2.2 Escaped [f:2/o:1/t:1]
  3 Empty [f:3/o:0/t:0]
"
    );
}

#[test]
fn goto_moves_to_first_snippet_of_section() {
    let sandbox = started_demo();

    sandbox.succeed(&["goto", "2.2"]);

    assert_eq!(sandbox.succeed(&["next"]), ESCAPED);
}

#[test]
fn goto_unknown_section_fails() {
    let sandbox = started_demo();

    let error = sandbox.fail(&["goto", "9.1"]);

    assert!(
        error.contains("section not found"),
        "Unexpected error: {error}"
    );
}

#[test]
fn next_section_skips_sections_sharing_their_start() {
    let sandbox = started_demo();

    sandbox.succeed(&["next-section"]);
    assert_eq!(sandbox.succeed(&["peek"]), MAIN);
    sandbox.succeed(&["next-section"]);
    assert_eq!(sandbox.succeed(&["peek"]), ESCAPED);
}

#[test]
fn prev_section_goes_back_to_start_of_current_section_first() {
    let sandbox = started_demo();
    sandbox.succeed(&["goto", "2.1"]);
    sandbox.succeed(&["next"]);

    sandbox.succeed(&["prev-section"]);
    assert_eq!(sandbox.succeed(&["peek"]), MAIN);
    sandbox.succeed(&["prev-section"]);
    assert_eq!(sandbox.succeed(&["peek"]), SETUP);
}

#[test]
fn start_writes_index_next_to_document() {
    let sandbox = started_demo();

    assert!(sandbox.path().join("demo.org.lci").is_file());
}

#[test]
fn edited_document_is_parsed_again() {
    let sandbox = started_demo();
    sandbox.succeed(&["next"]);

    sandbox.write("demo.org", &DEMO.replace("Hello", "Bye"));

    assert_eq!(
        sandbox.succeed(&["next"]),
        "fn main() {\n    println!(\"Bye\");\n}\n"
    );
}

#[test]
fn start_with_absolute_path_works() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("demo.org", DEMO);

    sandbox.succeed(&["start", path.to_str().expect("Path isn't UTF-8")]);

    assert_eq!(sandbox.succeed(&["next"]), SETUP);
}

#[test]
fn unterminated_block_is_reported_and_previous_demo_is_kept() {
    let sandbox = started_demo();
    sandbox.write("broken.org", "* Title\n#+begin_src sh\nls\n");

    let error = sandbox.fail(&["start", "broken.org"]);

    assert!(
        error.contains("malformed snippet document at line 2"),
        "Unexpected error: {error}"
    );
    assert_eq!(sandbox.succeed(&["next"]), SETUP);
}

#[test]
fn start_with_missing_file_fails() {
    let sandbox = Sandbox::new();

    let error = sandbox.fail(&["start", "missing.org"]);

    assert!(
        error.contains("snippet file not found"),
        "Unexpected error: {error}"
    );
}

#[test]
fn next_without_start_fails() {
    let sandbox = Sandbox::new();

    let error = sandbox.fail(&["next"]);

    assert!(
        error.contains("Failed to read config file"),
        "Unexpected error: {error}"
    );
}
