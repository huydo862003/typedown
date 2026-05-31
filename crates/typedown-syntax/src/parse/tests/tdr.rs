use super::helpers::*;

// Parses YAML frontmatter with nested mappings and sequences followed by rich markdown body
#[test]
fn parse_tdr_yaml_then_markdown() {
  let input = r#"---
title: test
tags:
  - rust
  - parser
config:
  debug: true
---
# Title

A paragraph with **bold** and *italic*.

- item with [link](url)
- plain item

> A blockquote
"#;
  let (ast, _) = parse(input);
  let tree = render_tree(&ast);
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    (YamlMapping
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "title")
        ":"
        (YamlMappingEntryValue
          (IdentLit
            " "
            "test")))
      "\n"
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "tags")
        ":"
        (YamlMappingEntryValue
          (YamlSequence
            "\n"
            "  "
            (YamlSequenceItem
              "-"
              (IdentLit
                " "
                "rust"))
            "\n"
            "  "
            (YamlSequenceItem
              "-"
              (IdentLit
                " "
                "parser")))))
      "\n"
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "config")
        ":"
        (YamlMappingEntryValue
          (YamlMapping
            "\n"
            "  "
            (YamlMappingEntry
              (YamlMappingEntryKey
                "debug")
              ":"
              (YamlMappingEntryValue
                (IdentLit
                  " "
                  "true"))))))
      "\n"
      "")
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
        "A"
        " "
        "paragraph"
        " "
        "with"
        " ")
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
        "."))
    "\n"
    "\n"
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "item"
            " "
            "with"
            " ")
          (MdLink
            "["
            (MdText
              "link"
              "]"
              "("
              "url"
              ")")
            "\n")
          (MdText
            "-"
            " "
            "plain"
            " "
            "item")))
      "\n")
    "\n"
    (MdBlockquote
      ">"
      " "
      (MdParagraph
        (MdText
          "A"
          " "
          "blockquote")))
    "\n"))"####
  );
}

// Parses YAML frontmatter with folded block scalar followed by markdown body
#[test]
fn parse_tdr_folded_block_then_markdown() {
  let input = r#"---
desc: >
  folded
  text
title: hi
---
# Hello

world
"#;
  let (ast, _) = parse(input);
  let tree = render_tree(&ast);
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    (YamlMapping
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "desc")
        ":"
        (YamlMappingEntryValue
          (StrLit
            (YamlFoldedBlockStrLit
              " "
              ">"
              "\n"
              "  "
              "folded"
              "\n"
              "  "
              "text"
              "\n"))))
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "title")
        ":"
        (YamlMappingEntryValue
          (IdentLit
            " "
            "hi")))
      "\n"
      "")
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "Hello"))
    "\n"
    "\n"
    (MdParagraph
      (MdText
        "world"))
    "\n"))"####
  );
}

// Parses simple frontmatter and markdown body
#[test]
fn parse_tdr_simple_frontmatter_and_body() {
  let input = r#"---
title: hello
---
# Welcome

paragraph here
"#;
  let (ast, _) = parse(input);
  let tree = render_tree(&ast);
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    (YamlMapping
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "title")
        ":"
        (YamlMappingEntryValue
          (IdentLit
            " "
            "hello")))
      "\n"
      "")
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "Welcome"))
    "\n"
    "\n"
    (MdParagraph
      (MdText
        "paragraph"
        " "
        "here"))
    "\n"))"####
  );
}

// Parses frontmatter with nested YAML and markdown body
#[test]
fn parse_tdr_complex_frontmatter_and_body() {
  let input = r#"---
title: hello
tags:
  - a
  - b
---
# Heading

- list item
"#;
  let (ast, _) = parse(input);
  let tree = render_tree(&ast);
  assert_eq!(
    tree,
    r####"(SourceFile
  (YamlFrontmatter
    ""
    "---"
    "\n"
    (YamlMapping
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "title")
        ":"
        (YamlMappingEntryValue
          (IdentLit
            " "
            "hello")))
      "\n"
      ""
      (YamlMappingEntry
        (YamlMappingEntryKey
          "tags")
        ":"
        (YamlMappingEntryValue
          (YamlSequence
            "\n"
            "  "
            (YamlSequenceItem
              "-"
              (IdentLit
                " "
                "a"))
            "\n"
            "  "
            (YamlSequenceItem
              "-"
              (IdentLit
                " "
                "b")))))
      "\n"
      "")
    "---"
    "\n")
  (MdBody
    (MdHeading
      "#"
      " "
      (MdText
        "Heading"))
    "\n"
    "\n"
    (MdBulletList
      (MdBulletListItem
        "-"
        " "
        (MdParagraph
          (MdText
            "list"
            " "
            "item")))
      "\n")))"####
  );
}
