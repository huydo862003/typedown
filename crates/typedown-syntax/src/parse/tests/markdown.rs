use super::helpers::*;

fn parse_body(input: &str) -> String {
  let full_input = format!("---\n---\n{}", input);
  let (ast, _) = parse(&full_input);
  render_tree(&ast)
}

fn parse_body_with_diags(input: &str) -> (String, Vec<typedown_types::diagnostic::Diagnostic>) {
  let full_input = format!("---\n---\n{}", input);
  let (ast, diags) = parse(&full_input);
  (render_tree(&ast), diags)
}

// Simple block elements

// Parses a single-line paragraph
#[test]
fn parse_paragraph_simple() {
  let tree = parse_body(r#"hello world
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "hello"
        " "
        "world"))
    "\n"))"####
  );
}

// Parses a level-1 heading
#[test]
fn parse_heading_simple() {
  let tree = parse_body(r#"# Hello
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "Hello"))
    "\n"))"####
  );
}

// Parses headings of levels 1, 2, and 3
#[test]
fn parse_heading_levels() {
  let tree = parse_body(r#"# H1
## H2
### H3
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "H"
        "1"))
    "\n"
    (Heading
      "##"
      " "
      (Text
        "H"
        "2"))
    "\n"
    (Heading
      "###"
      " "
      (Text
        "H"
        "3"))
    "\n"))"####
  );
}

// Parses a bullet list with dash markers
#[test]
fn parse_bullet_list_dash() {
  let tree = parse_body(r#"- item 1
- item 2
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "1")))
      "\n"
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses a bullet list with star markers
#[test]
fn parse_bullet_list_star() {
  let tree = parse_body(r#"* item 1
* item 2
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "*"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "1")))
      "\n"
      (BulletListItem
        "*"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses a bullet list with plus markers
#[test]
fn parse_bullet_list_plus() {
  let tree = parse_body(r#"+ item 1
+ item 2
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "+"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "1")))
      "\n"
      (BulletListItem
        "+"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "2")))
      "\n")))"####
  );
}

// Parses an ordered list
#[test]
fn parse_ordered_list_simple() {
  let tree = parse_body(r#"1. first
2. second
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (OrderedList
      (OrderedListItem
        "1"
        "."
        " "
        (Paragraph
          (Text
            "first")))
      "\n"
      (OrderedListItem
        "2"
        "."
        " "
        (Paragraph
          (Text
            "second")))
      "\n")))"####
  );
}

// Parses a blockquote
#[test]
fn parse_blockquote_simple() {
  let tree = parse_body(r#"> quoted text
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Blockquote
      ">"
      " "
      (Paragraph
        (Text
          "quoted"
          " "
          "text")))
    "\n"))"####
  );
}

// Parses a table
#[test]
fn parse_table_simple() {
  let tree = parse_body(r#"| a | b |
| --- | --- |
| 1 | 2 |
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Table
      (TableRow
        "|"
        (TableCell
          " "
          (Text
            "a"
            " "
            "|"
            " "
            "b"
            " "
            "|")))
      "\n"
      (TableSeparatorRow
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
      (TableRow
        "|"
        (TableCell
          " "
          (Text
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
  let tree = parse_body(r#">- summary

   details here
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (ToggleList
      (ToggleListItem
        ">"
        "-"
        " "
        (ToggleListSummary
          (Text
            "summary"))
        "\n"
        "\n"
        " "
        " "
        " "
        "details"
        " "
        "\n"
        (ToggleListDetails
          (Paragraph
            (Text
              "here"))))
      "")))"####
  );
}

// Parses a callout block
#[test]
fn parse_callout_block_simple() {
  let tree = parse_body(r#"::: note
content
:::
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (CalloutBlock
      ":::"
      " "
      "note"
      "\n"
      (Paragraph
        (Text
          "content"))
      (Text
        "\n")
      ":::")
    "\n"))"####
  );
}

// Parses a fenced code block
#[test]
fn parse_code_block_simple() {
  let tree = parse_body(r#"```
code
```
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
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
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body))"####
  );
}

// Parses a body with only blank lines
#[test]
fn parse_body_only_blank_lines() {
  let tree = parse_body("\n\n\n");
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    "\n"
    "\n"
    "\n"))"####
  );
}

// Simple inline elements

// Parses a link
#[test]
fn parse_link_simple() {
  let tree = parse_body(r#"[text](url)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Link
        "["
        (Text
          "text"
          "]"
          "("
          "url"
          ")")
        "\n"))))"####
  );
}

// Parses a media embed
#[test]
fn parse_media_simple() {
  let tree = parse_body(r#"![alt](image.png)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Media
      "!"
      "["
      (Text
        "alt"
        "]"
        "("
        "image"
        "."
        "png"
        ")")
      "\n")))"####
  );
}

// Parses bold text
#[test]
fn parse_bold_simple() {
  let tree = parse_body(r#"**bold**
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Bold
        "**"
        (Text
          "bold")
        "**"))
    "\n"))"####
  );
}

// Parses italic text
#[test]
fn parse_italic_simple() {
  let tree = parse_body(r#"*italic*
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Italic
        "*"
        (Text
          "italic")
        "*"))
    "\n"))"####
  );
}

// Parses bold italic text
#[test]
fn parse_bold_italic_simple() {
  let tree = parse_body(r#"***bold italic***
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (BoldItalic
        "***"
        (Text
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
  let tree = parse_body(r#"~~struck~~
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Strikethrough
        "~~"
        (Text
          "struck")
        "~~"))
    "\n"))"####
  );
}

// Parses inline code
#[test]
fn parse_inline_code_simple() {
  let tree = parse_body(r#"`code`
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "`code`"))
    "\n"))"####
  );
}

// Parses a citation
#[test]
fn parse_citation_simple() {
  let tree = parse_body(r#"[@key]
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Citation
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
  let tree = parse_body(r#"Hello *world* and [link](url)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "Hello"
        " ")
      (Italic
        "*"
        (Text
          "world")
        "*")
      (Text
        " "
        "and"
        " ")
      (Link
        "["
        (Text
          "link"
          "]"
          "("
          "url"
          ")")
        "\n"))))"####
  );
}

// Parses bold in paragraph
#[test]
fn parse_bold_in_paragraph() {
  let tree = parse_body(r#"Hello **world**!
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "Hello"
        " ")
      (Bold
        "**"
        (Text
          "world")
        "**")
      (Text
        "!"))
    "\n"))"####
  );
}

// Parses italic in heading
#[test]
fn parse_italic_in_heading() {
  let tree = parse_body(r#"# *emphasis* title
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Italic
        "*"
        (Text
          "emphasis")
        "*")
      (Text
        " "
        "title"))
    "\n"))"####
  );
}

// Parses strikethrough in heading
#[test]
fn parse_heading_with_strikethrough() {
  let tree = parse_body(r#"# ~~old~~ new
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Strikethrough
        "~~"
        (Text
          "old")
        "~~")
      (Text
        " "
        "new"))
    "\n"))"####
  );
}

// Parses link in blockquote
#[test]
fn parse_blockquote_with_link() {
  let tree = parse_body(r#"> see [here](url)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Blockquote
      ">"
      " "
      (Paragraph
        (Text
          "see"
          " ")
        (Link
          "["
          (Text
            "here"
            "]"
            "("
            "url"
            ")")
          "\n")))))"####
  );
}

// Parses strikethrough in blockquote
#[test]
fn parse_strikethrough_in_blockquote() {
  let tree = parse_body(r#"> ~~removed~~ text
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Blockquote
      ">"
      " "
      (Paragraph
        (Strikethrough
          "~~"
          (Text
            "removed")
          "~~")
        (Text
          " "
          "text")))
    "\n"))"####
  );
}

// Parses bold in list item
#[test]
fn parse_list_with_bold() {
  let tree = parse_body(r#"- **bold item**
- normal
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Bold
            "**"
            (Text
              "bold"
              " "
              "item")
            "**")))
      "\n"
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "normal")))
      "\n")))"####
  );
}

// Parses link in list item
#[test]
fn parse_link_in_list_item() {
  let tree = parse_body(r#"- see [here](url) for info
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "see"
            " ")
          (Link
            "["
            (Text
              "here"
              "]"
              "("
              "url"
              ")"
              " "
              "for"
              " "
              "info")
            "\n")))
      "")))"####
  );
}

// Parses media in paragraph
#[test]
fn parse_media_in_paragraph() {
  let tree = parse_body(r#"See ![photo](img.png) here
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "See"
        " ")
      (Media
        "!"
        "["
        (Text
          "photo"
          "]"
          "("
          "img"
          "."
          "png"
          ")"
          " "
          "here")
        "\n"))))"####
  );
}

// Parses nested bold inside italic
#[test]
fn parse_nested_bold_in_italic() {
  let tree = parse_body(r#"*hello **world***
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Italic
        "*"
        (Text
          "hello"
          " ")
        (Bold
          "**"
          (Text
            "world")
          (BoldItalic
            "***"
            "\n"))))))"####
  );
}

// Parses bold in table cells
#[test]
fn parse_table_with_bold_cells() {
  let tree = parse_body(r#"| **h** |
| --- |
| cell |
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Table
      (TableRow
        "|"
        (TableCell
          " "
          (Bold
            "**"
            (Text
              "h")
            "**")
          (Text
            " "
            "|")))
      "\n"
      (TableSeparatorRow
        "|"
        " "
        "---"
        " "
        "|")
      "\n"
      (TableRow
        "|"
        (TableCell
          " "
          (Text
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
  let tree = parse_body(r#"First paragraph.

Second paragraph.
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "First"
        " "
        "paragraph"
        ".")
      "\n"
      "\n"
      (Text
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
  let tree = parse_body(r#"# One

text

# Two
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "One"))
    "\n"
    "\n"
    (Paragraph
      (Text
        "text"))
    "\n"
    "\n"
    (Heading
      "#"
      " "
      (Text
        "Two"))
    "\n"))"####
  );
}

// Parses heading, paragraph, list sequence
#[test]
fn parse_heading_then_paragraph_then_list() {
  let tree = parse_body(r#"# Title

Some text.

- a
- b
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "Title"))
    "\n"
    "\n"
    (Paragraph
      (Text
        "Some"
        " "
        "text"
        "."))
    "\n"
    "\n"
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "a")))
      "\n"
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "b")))
      "\n")))"####
  );
}

// Parses table followed by paragraph
#[test]
fn parse_table_then_paragraph() {
  let tree = parse_body(r#"| h |
| - |
| c |

text
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Table
      (TableRow
        "|"
        (TableCell
          " "
          (Text
            "h"
            " "
            "|")))
      "\n"
      (TableSeparatorRow
        "|"
        " "
        "-"
        " "
        "|")
      "\n"
      (TableRow
        "|"
        (TableCell
          " "
          (Text
            "c"
            " "
            "|")))
      "\n")
    "\n"
    (Paragraph
      (Text
        "text"))
    "\n"))"####
  );
}

// Parses blockquote followed by bullet list
#[test]
fn parse_blockquote_then_list() {
  let tree = parse_body(r#"> quoted

- listed
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Blockquote
      ">"
      " "
      (Paragraph
        (Text
          "quoted")))
    "\n"
    "\n"
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "listed")))
      "\n")))"####
  );
}

// Parses bullet list followed by heading
#[test]
fn parse_list_then_heading() {
  let tree = parse_body(r#"- a
- b

# After
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "a")))
      "\n"
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "b")))
      "\n")
    "\n"
    (Heading
      "#"
      " "
      (Text
        "After"))
    "\n"))"####
  );
}

// Parses ordered list followed by unordered list
#[test]
fn parse_ordered_then_unordered() {
  let tree = parse_body(r#"1. first
2. second

- bullet
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (OrderedList
      (OrderedListItem
        "1"
        "."
        " "
        (Paragraph
          (Text
            "first")))
      "\n"
      (OrderedListItem
        "2"
        "."
        " "
        (Paragraph
          (Text
            "second")))
      "\n")
    "\n"
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "bullet")))
      "\n")))"####
  );
}

// Mixed inline formatting

// Parses interpolation in paragraph
#[test]
fn parse_interpolation_in_paragraph() {
  let tree = parse_body(r#"hello ${name} world
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "hello"
        " ")
      (InterpFragment
        "${"
        (IdentLit
          "name")
        "}")
      (Text
        " "
        "world"))
    "\n"))"####
  );
}

// Parses inline math
#[test]
fn parse_inline_math_simple() {
  let tree = parse_body(r#"the formula $E=mc^2$ is
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Text
        "the"
        " "
        "formula"
        " ")
      (Text
        "$E=mc^2$")
      (Text
        " "
        "is"))
    "\n"))"####
  );
}

// Parses bold and italic in one paragraph
#[test]
fn parse_bold_and_italic_mixed() {
  let tree = parse_body(r#"**bold** and *italic* text
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Bold
        "**"
        (Text
          "bold")
        "**")
      (Text
        " "
        "and"
        " ")
      (Italic
        "*"
        (Text
          "italic")
        "*")
      (Text
        " "
        "text"))
    "\n"))"####
  );
}

// Parses bold and strikethrough in one paragraph
#[test]
fn parse_bold_then_strikethrough() {
  let tree = parse_body(r#"**bold** ~~struck~~ end
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Bold
        "**"
        (Text
          "bold")
        "**")
      (Text
        " ")
      (Strikethrough
        "~~"
        (Text
          "struck")
        "~~")
      (Text
        " "
        "end"))
    "\n"))"####
  );
}

// Parses ordered list with links
#[test]
fn parse_ordered_list_with_links() {
  let tree = parse_body(r#"1. [first](a)
2. [second](b)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (OrderedList
      (OrderedListItem
        "1"
        "."
        " "
        (Paragraph
          (Link
            "["
            (Text
              "first"
              "]"
              "("
              "a"
              ")")
            "\n")
          (Text
            "2"
            "."
            " ")
          (Link
            "["
            (Text
              "second"
              "]"
              "("
              "b"
              ")")
            "\n")))
      "")))"####
  );
}

// Parses multiple links in paragraph
#[test]
fn parse_multiple_links_in_paragraph() {
  let tree = parse_body(r#"[a](x) and [b](y) and [c](z)
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Link
        "["
        (Text
          "a"
          "]"
          "("
          "x"
          ")"
          " "
          "and"
          " ")
        (Link
          "["
          (Text
            "b"
            "]"
            "("
            "y"
            ")"
            " "
            "and"
            " ")
          (Link
            "["
            (Text
              "c"
              "]"
              "("
              "z"
              ")")
            "\n"))))))"####
  );
}

// Parses links in table cells
#[test]
fn parse_table_with_links() {
  let tree = parse_body(r#"| [a](x) | [b](y) |
| --- | --- |
| 1 | 2 |
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Table
      (TableRow
        "|"
        (TableCell
          " "
          (Link
            "["
            (Text
              "a"
              "]"
              "("
              "x"
              ")"
              " "
              "|"
              " ")
            (Link
              "["
              (Text
                "b"
                "]"
                "("
                "y"
                ")"
                " "
                "|")
              "\n")
            (Text
              "|"
              " "
              "---"
              " "
              "|"
              " "
              "---"
              " "
              "|")
            "\n"))
        "|"
        (TableCell
          " "
          (Text
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
  let (tree, diags) = parse_body_with_diags(r#"[text without closing
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Link
        "["
        (Text
          "text"
          " "
          "without"
          " "
          "closing")
        "\n"))))"####
  );
  assert_eq!(
    diags,
    vec![typedown_types::diagnostic::Diagnostic::UnclosedLink {
      start_offset: 13,
      end_offset: 30,
    },]
  );
}

// Recovers from unclosed bold, emits UnclosedBold diagnostic
#[test]
fn recover_unclosed_bold() {
  let (tree, diags) = parse_body_with_diags(r#"**unclosed bold
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Bold
        "**"
        (Text
          "unclosed"
          " "
          "bold")
        "\n"))))"####
  );
  assert_eq!(
    diags,
    vec![typedown_types::diagnostic::Diagnostic::UnclosedBold {
      start_offset: 10,
      end_offset: 24
    },]
  );
}

// Recovers from mismatched italic and bold markers
#[test]
fn recover_mismatched_inline_formatting() {
  let (tree, diags) = parse_body_with_diags(r#"*italic **and bold*
"#);
  assert_eq!(
    tree,
    r####"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    ""
    "---"
    "\n")
  (Body
    (Paragraph
      (Italic
        "*"
        (Text
          "italic"
          " ")
        (Bold
          "**"
          (Text
            "and"
            " "
            "bold")
          (Italic
            "*"
            "\n"))))))"####
  );

  assert_eq!(
    diags,
    vec![
      typedown_types::diagnostic::Diagnostic::UnclosedItalic {
        start_offset: 27,
        end_offset: 28
      },
      typedown_types::diagnostic::Diagnostic::UnclosedBold {
        start_offset: 18,
        end_offset: 28
      },
      typedown_types::diagnostic::Diagnostic::UnclosedItalic {
        start_offset: 15,
        end_offset: 28
      },
    ]
  );
}
