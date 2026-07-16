use super::helpers::*;
use crate::syntax::diagnostic::Diagnostic;

fn parse_body(input: &str) -> String {
  let full_input = format!("---\n---\n{}", input);
  let (ast, _) = parse(&full_input);
  render_tree(&ast)
}

fn parse_body_with_diags(input: &str) -> (String, Vec<Diagnostic>) {
  let full_input = format!("---\n---\n{}", input);
  let (ast, diags) = parse(&full_input);
  (render_tree(&ast), diags)
}

// Simple block elements

// Parses a single-line paragraph
#[test]
fn parse_paragraph_simple() {
  let tree = parse_body(
    r#"hello world
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "hello"
        " "
        "world"))
    "\n"))"####
  );
}

// Parses a level-1 heading
#[test]
fn parse_heading_simple() {
  let tree = parse_body(
    r#"# Hello
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "Hello"))
    "\n"))"####
  );
}

// Parses headings of levels 1, 2, and 3
#[test]
fn parse_heading_levels() {
  let tree = parse_body(
    r#"# H1
## H2
### H3
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "H"
        "1"))
    "\n"
    (MdHeading
      "##"
      " "
      (MdText
        "H"
        "2"))
    "\n"
    (MdHeading
      "###"
      " "
      (MdText
        "H"
        "3"))
    "\n"))"####
  );
}

// Parses a bullet list with dash markers
#[test]
fn parse_bullet_list_dash() {
  let tree = parse_body(
    r#"- item 1
- item 2
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "1")))
      "\n"
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses a bullet list with star markers
#[test]
fn parse_bullet_list_star() {
  let tree = parse_body(
    r#"* item 1
* item 2
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "*"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "1")))
      "\n"
      (MdBulletListItem
        "*"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses a bullet list with plus markers
#[test]
fn parse_bullet_list_plus() {
  let tree = parse_body(
    r#"+ item 1
+ item 2
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "+"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "1")))
      "\n"
      (MdBulletListItem
        "+"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses a task list with unchecked and checked items
#[test]
fn parse_task_list_simple() {
  let tree = parse_body(
    r#"- [ ] unchecked
- [x] checked
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdTaskListItem
        "-"
        " "
        (MdCheckbox
          "["
          " "
          "]")
        (MdParagraph
          (MdText
            " "
            "unchecked")))
      "\n"
      (MdTaskListItem
        "-"
        " "
        (MdCheckbox
          "["
          "x"
          "]")
        (MdParagraph
          (MdText
            " "
            "checked")))
      "\n")))"####
  );
}

// Parses a bullet list mixed with a task list item
#[test]
fn parse_task_list_mixed() {
  let tree = parse_body(
    r#"- plain item
- [ ] task item
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "plain"
            " "
            "item")))
      "\n"
      (MdTaskListItem
        "-"
        " "
        (MdCheckbox
          "["
          " "
          "]")
        (MdParagraph
          (MdText
            " "
            "task"
            " "
            "item")))
      "\n")))"####
  );
}

// Parses an ordered list
#[test]
fn parse_ordered_list_simple() {
  let tree = parse_body(
    r#"1. first
2. second
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdOrderedList
      (MdOrderedListItem
        "1"
        "."
        " "
        (MdParagraph
          (MdText
            "first")))
      "\n"
      (MdOrderedListItem
        "2"
        "."
        " "
        (MdParagraph
          (MdText
            "second")))
      "\n")))"####
  );
}

// Parses a blockquote
#[test]
fn parse_blockquote_simple() {
  let tree = parse_body(
    r#"> quoted text
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBlockquote
      ">"
      " "
      (MdParagraph
        (MdText
          "quoted"
          " "
          "text")))
    "\n"))"####
  );
}

// Parses a table
#[test]
fn parse_table_simple() {
  let tree = parse_body(
    r#"| a | b |
| --- | --- |
| 1 | 2 |
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdTable
      (MdTableHeaderRow
        "|"
        (MdTableCell
          " "
          (MdText
            "a"
            " "
            "|"
            " "
            "b"
            " "
            "|")))
      "\n"
      (MdTableSeparatorRow
        "|"
        " "
        "---"
        " "
        "|"
        " "
        "---"
        " "
        "|")
      "\n"
      (MdTableDataRow
        "|"
        (MdTableCell
          " "
          (MdText
            "1"
            " "
            "|"
            " "
            "2"
            " "
            "|")))
      "\n")))"####
  );
}

// Parses a toggle list
#[test]
fn parse_toggle_list_simple() {
  let tree = parse_body(
    r#">- summary

   details here
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdToggleList
      (MdToggleListItem
        ">"
        "-"
        " "
        (MdToggleListSummary
          (MdText
            "summary"))
        "\n"
        "\n"
        " "
        " "
        " "
        "details"
        " "
        "\n"
        (MdToggleListDetails
          (MdParagraph
            (MdText
              "here"))))
      "")))"####
  );
}

// Parses a callout block
#[test]
fn parse_callout_block_simple() {
  let tree = parse_body(
    r#"::: note
content
:::
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdCalloutBlock
      ":::"
      " "
      "note"
      "\n"
      (MdParagraph
        (MdText
          "content"))
      (MdText
        "\n")
      ":::")
    "\n"))"####
  );
}

// Parses a fenced code block
#[test]
fn parse_code_block_simple() {
  let tree = parse_body(
    r#"```
code
```
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (CodeBlock
      "```\ncode\n```")
    "\n"))"####
  );
}

// Parses an empty body
#[test]
fn parse_body_empty() {
  let tree = parse_body("");
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody))"####
  );
}

// Parses a body with only blank lines
#[test]
fn parse_body_only_blank_lines() {
  let tree = parse_body("\n\n\n");
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    "\n"
    "\n"
    "\n"))"####
  );
}

// Simple inline elements

// Parses a link
#[test]
fn parse_link_simple() {
  let tree = parse_body(
    r#"[text](url)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdLink
        "["
        (MdText
          "text")
        "]"
        "("
        (MdText
          "url")
        ")"))
    "\n"))"####
  );
}

// Parses a media embed
#[test]
fn parse_media_simple() {
  let tree = parse_body(
    r#"![alt](image.png)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdMedia
      "!"
      "["
      (MdText
        "alt")
      "]"
      "("
      (MdText
        "image"
        "."
        "png")
      ")")
    "\n"))"####
  );
}

// Parses bold text
#[test]
fn parse_bold_simple() {
  let tree = parse_body(
    r#"**bold**
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdBold
        "**"
        (MdText
          "bold")
        "**"))
    "\n"))"####
  );
}

// Parses italic text
#[test]
fn parse_italic_simple() {
  let tree = parse_body(
    r#"*italic*
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdItalic
        "*"
        (MdText
          "italic")
        "*"))
    "\n"))"####
  );
}

// Parses bold italic text
#[test]
fn parse_bold_italic_simple() {
  let tree = parse_body(
    r#"***bold italic***
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdBoldItalic
        "***"
        (MdText
          "bold"
          " "
          "italic")
        "***"))
    "\n"))"####
  );
}

// Parses strikethrough text
#[test]
fn parse_strikethrough_simple() {
  let tree = parse_body(
    r#"~~struck~~
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdStrikethrough
        "~~"
        (MdText
          "struck")
        "~~"))
    "\n"))"####
  );
}

// Parses inline code
#[test]
fn parse_inline_code_simple() {
  let tree = parse_body(
    r#"`code`
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (InlineCode
        "`code`"))
    "\n"))"####
  );
}

// Parses a citation
#[test]
fn parse_citation_simple() {
  let tree = parse_body(
    r#"[@key]
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdCitation
        "["
        "@"
        "key"
        "]"))
    "\n"))"####
  );
}

// Inline elements inside block elements

// Parses a paragraph with italic and link
#[test]
fn parse_paragraph_with_inline() {
  let tree = parse_body(
    r#"Hello *world* and [link](url)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "Hello"
        " ")
      (MdItalic
        "*"
        (MdText
          "world")
        "*")
      (MdText
        " "
        "and"
        " ")
      (MdLink
        "["
        (MdText
          "link")
        "]"
        "("
        (MdText
          "url")
        ")"))
    "\n"))"####
  );
}

// Parses bold in paragraph
#[test]
fn parse_bold_in_paragraph() {
  let tree = parse_body(
    r#"Hello **world**!
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "Hello"
        " ")
      (MdBold
        "**"
        (MdText
          "world")
        "**")
      (MdText
        "!"))
    "\n"))"####
  );
}

// Parses italic in heading
#[test]
fn parse_italic_in_heading() {
  let tree = parse_body(
    r#"# *emphasis* title
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdItalic
        "*"
        (MdText
          "emphasis")
        "*")
      (MdText
        " "
        "title"))
    "\n"))"####
  );
}

// Parses strikethrough in heading
#[test]
fn parse_heading_with_strikethrough() {
  let tree = parse_body(
    r#"# ~~old~~ new
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdStrikethrough
        "~~"
        (MdText
          "old")
        "~~")
      (MdText
        " "
        "new"))
    "\n"))"####
  );
}

// Parses link in blockquote
#[test]
fn parse_blockquote_with_link() {
  let tree = parse_body(
    r#"> see [here](url)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBlockquote
      ">"
      " "
      (MdParagraph
        (MdText
          "see"
          " ")
        (MdLink
          "["
          (MdText
            "here")
          "]"
          "("
          (MdText
            "url")
          ")")))
    "\n"))"####
  );
}

// Parses strikethrough in blockquote
#[test]
fn parse_strikethrough_in_blockquote() {
  let tree = parse_body(
    r#"> ~~removed~~ text
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBlockquote
      ">"
      " "
      (MdParagraph
        (MdStrikethrough
          "~~"
          (MdText
            "removed")
          "~~")
        (MdText
          " "
          "text")))
    "\n"))"####
  );
}

// Parses bold in list item
#[test]
fn parse_list_with_bold() {
  let tree = parse_body(
    r#"- **bold item**
- normal
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdBold
            "**"
            (MdText
              "bold"
              " "
              "item")
            "**")))
      "\n"
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "normal")))
      "\n")))"####
  );
}

// Parses link in list item
#[test]
fn parse_link_in_list_item() {
  let tree = parse_body(
    r#"- see [here](url) for info
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "see"
            " ")
          (MdLink
            "["
            (MdText
              "here")
            "]"
            "("
            (MdText
              "url")
            ")")
          (MdText
            " "
            "for"
            " "
            "info")))
      "\n")))"####
  );
}

// Parses media in paragraph
#[test]
fn parse_media_in_paragraph() {
  let tree = parse_body(
    r#"See ![photo](img.png) here
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "See"
        " ")
      (MdMedia
        "!"
        "["
        (MdText
          "photo")
        "]"
        "("
        (MdText
          "img"
          "."
          "png")
        ")")
      (MdText
        " "
        "here"))
    "\n"))"####
  );
}

// Parses nested bold inside italic
#[test]
fn parse_nested_bold_in_italic() {
  let tree = parse_body(
    r#"*hello **world***
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdItalic
        "*"
        (MdText
          "hello"
          " ")
        (MdBold
          "**"
          (MdText
            "world")
          (MdBoldItalic
            "***"
            "\n"))))))"####
  );
}

// Parses bold in table cells
#[test]
fn parse_table_with_bold_cells() {
  let tree = parse_body(
    r#"| **h** |
| --- |
| cell |
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdTable
      (MdTableHeaderRow
        "|"
        (MdTableCell
          " "
          (MdBold
            "**"
            (MdText
              "h")
            "**")
          (MdText
            " "
            "|")))
      "\n"
      (MdTableSeparatorRow
        "|"
        " "
        "---"
        " "
        "|")
      "\n"
      (MdTableDataRow
        "|"
        (MdTableCell
          " "
          (MdText
            "cell"
            " "
            "|")))
      "\n")))"####
  );
}

// Multiple block elements in sequence

// Parses multiple paragraphs
#[test]
fn parse_paragraph_multiple() {
  let tree = parse_body(
    r#"First paragraph.

Second paragraph.
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "First"
        " "
        "paragraph"
        ".")
      "\n"
      "\n"
      (MdText
        "Second"
        " "
        "paragraph"
        "."))
    "\n"))"####
  );
}

// Parses heading, paragraph, heading sequence
#[test]
fn parse_heading_paragraph_heading() {
  let tree = parse_body(
    r#"# One

text

# Two
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "One"))
    "\n"
    "\n"
    (MdParagraph
      (MdText
        "text"))
    "\n"
    "\n"
    (MdHeading
      "#"
      " "
      (MdText
        "Two"))
    "\n"))"####
  );
}

// Parses heading, paragraph, list sequence
#[test]
fn parse_heading_then_paragraph_then_list() {
  let tree = parse_body(
    r#"# Title

Some text.

- a
- b
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "Title"))
    "\n"
    "\n"
    (MdParagraph
      (MdText
        "Some"
        " "
        "text"
        "."))
    "\n"
    "\n"
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "a")))
      "\n"
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "b")))
      "\n")))"####
  );
}

// Parses table followed by paragraph
#[test]
fn parse_table_then_paragraph() {
  let tree = parse_body(
    r#"| h |
| - |
| c |

text
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdTable
      (MdTableHeaderRow
        "|"
        (MdTableCell
          " "
          (MdText
            "h"
            " "
            "|")))
      "\n"
      (MdTableSeparatorRow
        "|"
        " "
        "-"
        " "
        "|")
      "\n"
      (MdTableDataRow
        "|"
        (MdTableCell
          " "
          (MdText
            "c"
            " "
            "|")))
      "\n")
    "\n"
    (MdParagraph
      (MdText
        "text"))
    "\n"))"####
  );
}

// Parses blockquote followed by bullet list
#[test]
fn parse_blockquote_then_list() {
  let tree = parse_body(
    r#"> quoted

- listed
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBlockquote
      ">"
      " "
      (MdParagraph
        (MdText
          "quoted")))
    "\n"
    "\n"
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "listed")))
      "\n")))"####
  );
}

// Parses bullet list followed by heading
#[test]
fn parse_list_then_heading() {
  let tree = parse_body(
    r#"- a
- b

# After
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "a")))
      "\n"
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "b")))
      "\n")
    "\n"
    (MdHeading
      "#"
      " "
      (MdText
        "After"))
    "\n"))"####
  );
}

// Parses ordered list followed by unordered list
#[test]
fn parse_ordered_then_unordered() {
  let tree = parse_body(
    r#"1. first
2. second

- bullet
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdOrderedList
      (MdOrderedListItem
        "1"
        "."
        " "
        (MdParagraph
          (MdText
            "first")))
      "\n"
      (MdOrderedListItem
        "2"
        "."
        " "
        (MdParagraph
          (MdText
            "second")))
      "\n")
    "\n"
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "bullet")))
      "\n")))"####
  );
}

// Mixed inline formatting

// Parses interpolation in paragraph
#[test]
fn parse_interpolation_in_paragraph() {
  let tree = parse_body(
    r#"hello ${name} world
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "hello"
        " ")
      (InterpFragment
        "${"
        (IdentLit
          "name")
        "}")
      (MdText
        " "
        "world"))
    "\n"))"####
  );
}

// Parses inline math
#[test]
fn parse_inline_math_simple() {
  let tree = parse_body(
    r#"the formula $E=mc^2$ is
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdText
        "the"
        " "
        "formula"
        " ")
      (InlineMath
        "$E=mc^2$")
      (MdText
        " "
        "is"))
    "\n"))"####
  );
}

// Parses bold and italic in one paragraph
#[test]
fn parse_bold_and_italic_mixed() {
  let tree = parse_body(
    r#"**bold** and *italic* text
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdBold
        "**"
        (MdText
          "bold")
        "**")
      (MdText
        " "
        "and"
        " ")
      (MdItalic
        "*"
        (MdText
          "italic")
        "*")
      (MdText
        " "
        "text"))
    "\n"))"####
  );
}

// Parses bold and strikethrough in one paragraph
#[test]
fn parse_bold_then_strikethrough() {
  let tree = parse_body(
    r#"**bold** ~~struck~~ end
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdBold
        "**"
        (MdText
          "bold")
        "**")
      (MdText
        " ")
      (MdStrikethrough
        "~~"
        (MdText
          "struck")
        "~~")
      (MdText
        " "
        "end"))
    "\n"))"####
  );
}

// Parses ordered list with links
#[test]
fn parse_ordered_list_with_links() {
  let tree = parse_body(
    r#"1. [first](a)
2. [second](b)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdOrderedList
      (MdOrderedListItem
        "1"
        "."
        " "
        (MdParagraph
          (MdLink
            "["
            (MdText
              "first")
            "]"
            "("
            (MdText
              "a")
            ")")))
      "\n"
      (MdOrderedListItem
        "2"
        "."
        " "
        (MdParagraph
          (MdLink
            "["
            (MdText
              "second")
            "]"
            "("
            (MdText
              "b")
            ")")))
      "\n")))"####
  );
}

// Parses multiple links in paragraph
#[test]
fn parse_multiple_links_in_paragraph() {
  let tree = parse_body(
    r#"[a](x) and [b](y) and [c](z)
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdLink
        "["
        (MdText
          "a")
        "]"
        "("
        (MdText
          "x")
        ")")
      (MdText
        " "
        "and"
        " ")
      (MdLink
        "["
        (MdText
          "b")
        "]"
        "("
        (MdText
          "y")
        ")")
      (MdText
        " "
        "and"
        " ")
      (MdLink
        "["
        (MdText
          "c")
        "]"
        "("
        (MdText
          "z")
        ")"))
    "\n"))"####
  );
}

// Parses links in table cells
#[test]
fn parse_table_with_links() {
  let tree = parse_body(
    r#"| [a](x) | [b](y) |
| --- | --- |
| 1 | 2 |
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdTable
      (MdTableHeaderRow
        "|"
        (MdTableCell
          " "
          (MdLink
            "["
            (MdText
              "a")
            "]"
            "("
            (MdText
              "x")
            ")")
          (MdText
            " "
            "|"
            " ")
          (MdLink
            "["
            (MdText
              "b")
            "]"
            "("
            (MdText
              "y")
            ")")
          (MdText
            " "
            "|")))
      "\n"
      (MdTableSeparatorRow
        "|"
        " "
        "---"
        " "
        "|"
        " "
        "---"
        " "
        "|")
      "\n"
      (MdTableDataRow
        "|"
        (MdTableCell
          " "
          (MdText
            "1"
            " "
            "|"
            " "
            "2"
            " "
            "|")))
      "\n")))"####
  );
}

// Error recovery

// Recovers from unclosed link, emits UnclosedLink diagnostic
#[test]
fn recover_unclosed_link() {
  let (tree, diags) = parse_body_with_diags(
    r#"[text without closing
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdLink
        "["
        (MdText
          "text"
          " "
          "without"
          " "
          "closing")))
    "\n"))"####
  );
  assert_eq!(
    diags,
    vec![Diagnostic::UnclosedLink {
      start_offset: 13,
      end_offset: 30,
    },]
  );
}

// Recovers from unclosed bold, emits UnclosedBold diagnostic
#[test]
fn recover_unclosed_bold() {
  let (tree, diags) = parse_body_with_diags(
    r#"**unclosed bold
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdBold
        "**"
        (MdText
          "unclosed"
          " "
          "bold")
        "\n"))))"####
  );
  assert_eq!(
    diags,
    vec![Diagnostic::UnclosedBold {
      start_offset: 10,
      end_offset: 24
    },]
  );
}

// Recovers from mismatched italic and bold markers
#[test]
fn recover_mismatched_inline_formatting() {
  let (tree, diags) = parse_body_with_diags(
    r#"*italic **and bold*
"#,
  );
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (MdBody
    (MdParagraph
      (MdItalic
        "*"
        (MdText
          "italic"
          " ")
        (MdBold
          "**"
          (MdText
            "and"
            " "
            "bold")
          (MdItalic
            "*"
            "\n"))))))"####
  );

  assert_eq!(
    diags,
    vec![
      Diagnostic::UnclosedItalic {
        start_offset: 27,
        end_offset: 28
      },
      Diagnostic::UnclosedBold {
        start_offset: 18,
        end_offset: 28
      },
      Diagnostic::UnclosedItalic {
        start_offset: 15,
        end_offset: 28
      },
    ]
  );
}
