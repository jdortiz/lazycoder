# LazyCoder

## This Repo

This repository contains code for the lazycoder tool, a snippet provider for
[espanso](https://github.com/federico-terzi/espanso) written in Rust. This is intentionally simple and I would like to
keep it that way.  Feel free to download and tinker with the code.  Pull requests are welcome.

## License

This project is licensed under the terms of the [Apache license 2.0](./LICENSE.txt).

## Snippet files

The format of the snippet file is chosen by its extension: files ending in `.org` are Org mode documents and any other
file is considered to be a plain text snippet file.

### Plain text

The file with the snippets is a plain text file with snippets separated by "\n---\n\n", i.e. an empty line, a line with
three dashes and another empty line.  If you don't want your snippet to be expanded with a new line at the end, don't
add an empty line at the beginning of the separator (just the three dashes right after your snippet and an empty line
after that).

### Org mode

You can write your demo as an Org mode document, with headings for its structure and ordered lists describing the
steps.  Every source block (`#+begin_src` … `#+end_src`) and example block (`#+begin_example` … `#+end_example`) is a
snippet, in the order they appear in the document.  The rest of the document is just for you.

```org
* Setup
1. Create the project
   #+begin_src shell
   cargo new demo
   #+end_src
* Code
** Main function
#+begin_src rust
fn main() {}
#+end_src
```

- Headings of levels 1 to 3 are sections that you can use to move around (see the commands below).  They are numbered
  like an outline: in the example above, "Code" is section 2 and "Main function" is section 2.1.  Deeper headings are
  ignored.
- The indentation of a block (e.g. a block inside a list item) is removed from its contents.
- Org mode's protective commas (`,*` and `,#+` at the beginning of a line) are removed.
- Each snippet ends with a new line.

When you start using an Org mode file, lazycoder stores its structure in an index file next to it, with the same name
plus `.lci` (e.g. `demo.org.lci`), and a hash of the document.  The index file is reused while the document doesn't
change and rebuilt otherwise, even if you edit the document in the middle of your demo.  You can delete it at any time,
and you probably want to add `*.lci` to your `.gitignore`.

## Commands

| Command                | Alias | Description                                                                    |
|------------------------|-------|--------------------------------------------------------------------------------|
| `start <filename>`     | `s`   | Use the file to provide snippets, starting from the first one                  |
| `next`                 | `n`   | Provide the next snippet and move forward                                      |
| `peek`                 | `p`   | Provide the next snippet without moving forward                                |
| `forward [n]`          | `f`   | Move *n* snippets forward (1 by default)                                       |
| `rewind [n]`           | `r`   | Move *n* snippets backward (1 by default)                                      |
| `list-sections`        | `l`   | List the sections with their numbers, indented by level                        |
| `next-section [n]`     | `ns`  | Move to the first snippet of the *n*-th next section (1 by default)            |
| `prev-section [n]`     | `ps`  | Move to the first snippet of the current section or the previous ones          |
| `goto <section>`       | `g`   | Move to the first snippet of the section with that number, e.g. `goto 2.1`     |

The section commands only work with Org mode files.  Use `list-sections` to find the number of the section you want to
go to.  `prev-section` behaves like the back button of a media player: if
you are past the first snippet of the current section, it takes you back to its start; otherwise, to the previous
section.

`list-sections` shows the snippets of each section as `[f:5/o:2/t:6]`: `f` is the number of its first snippet
(starting from 0), `o` the number of snippets of its own (before the next heading), and `t` the total number of
snippets, including those of its subsections.

## Author

(c) Jorge D. Ortiz Fuentes, 2022 - 2026

## Resources

- [🙋‍♂️ Jorge's Bluesky](https://bsky.app/profile/jdortiz.bsky.social)
- [🙋‍♂️ Jorge's Mastodon](https://fosstodon.org/@jdortiz)
- [🙋‍♂️ Jorge's X](https://x.com/jdortiz)
- [🙋‍♂️ Jorge's LinkedIn](https://www.linkedin.com/in/jorgeortiz/)
