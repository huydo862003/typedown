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
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "title")
        ":"
        (MappingEntryValue
          (IdentLit
            " "
            "test")))
      "\n"
      ""
      (MappingEntry
        (MappingEntryKey
          "tags")
        ":"
        (MappingEntryValue
          (BlockSeqLit
            "\n"
            "  "
            (SequenceItem
              "-"
              (IdentLit
                " "
                "rust"))
            "\n"
            "  "
            (SequenceItem
              "-"
              (IdentLit
                " "
                "parser")))))
      "\n"
      ""
      (MappingEntry
        (MappingEntryKey
          "config")
        ":"
        (MappingEntryValue
          (BlockMappingLit
            "\n"
            "  "
            (MappingEntry
              (MappingEntryKey
                "debug")
              ":"
              (MappingEntryValue
                (IdentLit
                  " "
                  "true"))))))
      "\n"
      "")
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
        "A"
        " "
        "paragraph"
        " "
        "with"
        " ")
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
        "."))
    "\n"
    "\n"
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "item"
            " "
            "with"
            " ")
          (Link
            "["
            (Text
              "link"
              "]"
              "("
              "url"
              ")")
            "\n")
          (Text
            "-"
            " "
            "plain"
            " "
            "item")))
      "\n")
    "\n"
    (Blockquote
      ">"
      " "
      (Paragraph
        (Text
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
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "desc")
        ":"
        (MappingEntryValue
          (FoldedBlockStrLit
            " "
            ">"
            "\n"
            "  "
            "folded"
            "\n"
            "  "
            "text"
            "\n")))
      ""
      (MappingEntry
        (MappingEntryKey
          "title")
        ":"
        (MappingEntryValue
          (IdentLit
            " "
            "hi")))
      "\n"
      "")
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "Hello"))
    "\n"
    "\n"
    (Paragraph
      (Text
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
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "title")
        ":"
        (MappingEntryValue
          (IdentLit
            " "
            "hello")))
      "\n"
      "")
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "Welcome"))
    "\n"
    "\n"
    (Paragraph
      (Text
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
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "title")
        ":"
        (MappingEntryValue
          (IdentLit
            " "
            "hello")))
      "\n"
      ""
      (MappingEntry
        (MappingEntryKey
          "tags")
        ":"
        (MappingEntryValue
          (BlockSeqLit
            "\n"
            "  "
            (SequenceItem
              "-"
              (IdentLit
                " "
                "a"))
            "\n"
            "  "
            (SequenceItem
              "-"
              (IdentLit
                " "
                "b")))))
      "\n"
      "")
    "---"
    "\n")
  (Body
    (Heading
      "#"
      " "
      (Text
        "Heading"))
    "\n"
    "\n"
    (BulletList
      (BulletListItem
        "-"
        " "
        (Paragraph
          (Text
            "list"
            " "
            "item")))
      "\n")))"####
  );
}
