use super::helpers::*;
use typedown_types::diagnostic::Diagnostic;

fn parse_frontmatter(input: &str) -> String {
  let full = format!("---\n{}\n---\n", input);
  let (ast, _) = parse(&full);
  let root = ast.as_node().unwrap();
  let frontmatter = &root.children()[0];
  render_tree(frontmatter)
}

fn parse_frontmatter_with_diagnostics(input: &str) -> (String, Vec<Diagnostic>) {
  let full = format!("---\n{}\n---\n", input);
  let (ast, diagnostics) = parse(&full);
  let root = ast.as_node().unwrap();
  let frontmatter = &root.children()[0];
  (render_tree(frontmatter), diagnostics)
}

// Yaml frontmatter
#[test]
fn empty_frontmatter() {
  let tree = parse_frontmatter(r#""#);
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  "\n"
  ""
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

#[test]
fn error_missing_closing_marker() {
  let full = r#"---
key: 1
"#;
  let (ast, diags) = parse(full);
  let root = ast.as_node().unwrap();
  let frontmatter = &root.children()[0];
  let tree = render_tree(frontmatter);
  assert!(tree.starts_with("(Frontmatter"));
  assert!(diags
    .iter()
    .any(|d| matches!(d, Diagnostic::MissingFrontmatterMarker { .. })));
}

// Mapping

#[test]
fn single_key_value() {
  let tree = parse_frontmatter(r#"key: 1"#);
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "key")
      ":"
      (MappingEntryValue
        (NumberLit
          " "
          "1")))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

#[test]
fn multiple_key_values() {
  let tree = parse_frontmatter(
    r#"a: 1
b: 2"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "a")
      ":"
      (MappingEntryValue
        (NumberLit
          " "
          "1")))
    "\n"
    ""
    (MappingEntry
      (MappingEntryKey
        "b")
      ":"
      (MappingEntryValue
        (NumberLit
          " "
          "2")))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Strings
#[test]
fn string_value() {
  let tree = parse_frontmatter(r#"title: "hello""#);
  let expected = r#"(Frontmatter
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
        (StrLit
          " "
          "\""
          "hello"
          "\"")))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Flow sequences
#[test]
fn flow_list_value() {
  let tree = parse_frontmatter(r#"tags: [1, 2]"#);
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "tags")
      ":"
      (MappingEntryValue
        (ListLit
          " "
          "["
          (NumberLit
            "1")
          ","
          (NumberLit
            " "
            "2")
          "]")))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Block sequences
#[test]
fn block_sequence() {
  let tree = parse_frontmatter(
    r#"items:
  - 1
  - 2"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "items")
      ":"
      (MappingEntryValue
        (BlockSeqLit
          "\n"
          "  "
          (SequenceItem
            "-"
            (NumberLit
              " "
              "1"))
          "\n"
          "  "
          (SequenceItem
            "-"
            (NumberLit
              " "
              "2")))))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Nested mappings
#[test]
fn nested_mapping() {
  let tree = parse_frontmatter(
    r#"outer:
  inner: 1"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "outer")
      ":"
      (MappingEntryValue
        (BlockMappingLit
          "\n"
          "  "
          (MappingEntry
            (MappingEntryKey
              "inner")
            ":"
            (MappingEntryValue
              (NumberLit
                " "
                "1"))))))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Sequence in mapping
#[test]
fn sequence_in_mapping() {
  let tree = parse_frontmatter(
    r#"key:
  - a
  - b"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "key")
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
  "\n")"#;
  assert_eq!(tree, expected);
}

// Mapping in sequence
#[test]
fn mapping_in_sequence() {
  let tree = parse_frontmatter(
    r#"items:
  - name: alice
  - name: bob"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "items")
      ":"
      (MappingEntryValue
        (BlockSeqLit
          "\n"
          "  "
          (SequenceItem
            "-"
            (BlockMappingLit
              " "
              (MappingEntry
                (MappingEntryKey
                  "name")
                ":"
                (MappingEntryValue
                  (IdentLit
                    " "
                    "alice")))
              "\n"
              "  "
              (Error
                "-"
                " ")
              (MappingEntry
                (MappingEntryKey
                  "name")
                ":"
                (MappingEntryValue
                  (IdentLit
                    " "
                    "bob"))))))))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Nested sequences
#[test]
fn nested_sequence() {
  let tree = parse_frontmatter(
    r#"matrix:
  -
    - 1
    - 2
  -
    - 3
    - 4"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "matrix")
      ":"
      (MappingEntryValue
        (BlockSeqLit
          "\n"
          "  "
          (SequenceItem
            "-"
            (BlockSeqLit
              "\n"
              "    "
              (SequenceItem
                "-"
                (NumberLit
                  " "
                  "1"))
              "\n"
              "    "
              (SequenceItem
                "-"
                (NumberLit
                  " "
                  "2"))))
          "\n"
          "  "
          (SequenceItem
            "-"
            (BlockSeqLit
              "\n"
              "    "
              (SequenceItem
                "-"
                (NumberLit
                  " "
                  "3"))
              "\n"
              "    "
              (SequenceItem
                "-"
                (NumberLit
                  " "
                  "4")))))))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

#[test]
fn inline_nested_sequence() {
  let tree = parse_frontmatter(
    r#"items:
  - - 1
  - - 2"#,
  );
  assert!(tree.contains("BlockSeqLit"));
}

#[test]
fn block_string_in_sequence() {
  let tree = parse_frontmatter(
    r#"items:
  - >
    hello
    world
  - >
    foo"#,
  );
  assert!(tree.contains("FoldedBlockStrLit"));
}

#[test]
fn multi_entry_inline_mapping() {
  let tree = parse_frontmatter(
    r#"items:
  - name: alice
    age: 30
  - name: bob
    age: 25"#,
  );
  let expected = r#"(Frontmatter
  ""
  "---"
  "\n"
  (BlockMappingLit
    ""
    (MappingEntry
      (MappingEntryKey
        "items")
      ":"
      (MappingEntryValue
        (BlockSeqLit
          "\n"
          "  "
          (SequenceItem
            "-"
            (BlockMappingLit
              " "
              (MappingEntry
                (MappingEntryKey
                  "name")
                ":"
                (MappingEntryValue
                  (IdentLit
                    " "
                    "alice")))
              "\n"
              "    "
              (MappingEntry
                (MappingEntryKey
                  "age")
                ":"
                (MappingEntryValue
                  (NumberLit
                    " "
                    "30")))))
          "\n"
          "  "
          (SequenceItem
            "-"
            (BlockMappingLit
              " "
              (MappingEntry
                (MappingEntryKey
                  "name")
                ":"
                (MappingEntryValue
                  (IdentLit
                    " "
                    "bob")))
              "\n"
              "    "
              (MappingEntry
                (MappingEntryKey
                  "age")
                ":"
                (MappingEntryValue
                  (NumberLit
                    " "
                    "25"))))))))
    "\n"
    "")
  "---"
  "\n")"#;
  assert_eq!(tree, expected);
}

// Inline nested sequence with continuation items
#[test]
fn inline_nested_sequence_multi_items() {
  let full = format!(
    "---\n{}\n---\n",
    r#"items:
  - - 1
    - 2
  - - 3
    - 4"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(!tree.contains("Error"));
}

// Multiple top-level keys, values mixing sequences and mappings
#[test]
fn multi_key_mixed_seq_and_map() {
  let full = format!(
    "---\n{}\n---\n",
    r#"users:
  - name: alice
    tags:
      - admin
      - editor
  - name: bob
    tags:
      - viewer
config:
  - ports:
      - 8080
      - 443
    host: localhost
  - ports:
      - 3000
    host: staging
extra:
  enabled: true"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(!tree.contains("Error"));
}

// Sequence where first item is a list, second is a map, third is a list again
#[test]
fn seq_alternating_list_and_map() {
  let full = format!(
    "---\n{}\n---\n",
    r#"data:
  - - 10
    - 20
    - 30
  - key: value
    other: stuff
  - - 40
    - 50"#
  );
  let (ast, _diags) = parse(&full);
  let tree = render_tree(&ast);
  let expected = r#"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "data")
        ":"
        (MappingEntryValue
          (BlockSeqLit
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockSeqLit
                " "
                (SequenceItem
                  "-"
                  (NumberLit
                    " "
                    "10"))
                "\n"
                "    "
                (SequenceItem
                  "-"
                  (NumberLit
                    " "
                    "20"))
                "\n"
                "    "
                (SequenceItem
                  "-"
                  (NumberLit
                    " "
                    "30"))))
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockMappingLit
                " "
                (MappingEntry
                  (MappingEntryKey
                    "key")
                  ":"
                  (MappingEntryValue
                    (IdentLit
                      " "
                      "value")))
                "\n"
                "    "
                (MappingEntry
                  (MappingEntryKey
                    "other")
                  ":"
                  (MappingEntryValue
                    (IdentLit
                      " "
                      "stuff")))))
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockSeqLit
                " "
                (SequenceItem
                  "-"
                  (NumberLit
                    " "
                    "40"))
                "\n"
                "    "
                (SequenceItem
                  "-"
                  (NumberLit
                    " "
                    "50")))))))
      "\n"
      "")
    "---"
    "\n")
  (Body))"#;
  assert_eq!(tree, expected);
}

// Deeply nested: sequence of sequences of sequences
#[test]
fn triple_nested_sequence() {
  let full = format!(
    "---\n{}\n---\n",
    r#"matrix:
  - - - 1
      - 2
    - - 3
      - 4
  - - - 5
      - 6"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(!tree.contains("Error"));
}

// Map value is a sequence whose items are maps containing sequences
#[test]
fn map_of_seq_of_map_of_seq() {
  let full = format!(
    "---\n{}\n---\n",
    r#"teams:
  - members:
      - alice
      - bob
    projects:
      - alpha
      - beta
  - members:
      - charlie
    projects:
      - gamma
settings:
  debug: true"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  let expected = r#"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "teams")
        ":"
        (MappingEntryValue
          (BlockSeqLit
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockMappingLit
                " "
                (MappingEntry
                  (MappingEntryKey
                    "members")
                  ":"
                  (MappingEntryValue
                    (BlockSeqLit
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "alice"))
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "bob")))))
                "\n"
                "    "
                (MappingEntry
                  (MappingEntryKey
                    "projects")
                  ":"
                  (MappingEntryValue
                    (BlockSeqLit
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "alpha"))
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "beta")))))))
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockMappingLit
                " "
                (MappingEntry
                  (MappingEntryKey
                    "members")
                  ":"
                  (MappingEntryValue
                    (BlockSeqLit
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "charlie")))))
                "\n"
                "    "
                (MappingEntry
                  (MappingEntryKey
                    "projects")
                  ":"
                  (MappingEntryValue
                    (BlockSeqLit
                      "\n"
                      "      "
                      (SequenceItem
                        "-"
                        (IdentLit
                          " "
                          "gamma"))))))))))
      "\n"
      ""
      (MappingEntry
        (MappingEntryKey
          "settings")
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
  (Body))"#;
  assert_eq!(tree, expected);
}

// Inline sequence items where one is a map and next is a sequence
#[test]
fn inline_seq_first_map_then_list() {
  let full = format!(
    "---\n{}\n---\n",
    r#"mixed:
  - a: 1
    b: 2
  - - x
    - y
  - c: 3"#
  );
  let (ast, _diags) = parse(&full);
  let tree = render_tree(&ast);
  let expected = r#"(SourceFile
  (Frontmatter
    ""
    "---"
    "\n"
    (BlockMappingLit
      ""
      (MappingEntry
        (MappingEntryKey
          "mixed")
        ":"
        (MappingEntryValue
          (BlockSeqLit
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockMappingLit
                " "
                (MappingEntry
                  (MappingEntryKey
                    "a")
                  ":"
                  (MappingEntryValue
                    (NumberLit
                      " "
                      "1")))
                "\n"
                "    "
                (MappingEntry
                  (MappingEntryKey
                    "b")
                  ":"
                  (MappingEntryValue
                    (NumberLit
                      " "
                      "2")))))
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockSeqLit
                " "
                (SequenceItem
                  "-"
                  (IdentLit
                    " "
                    "x"))
                "\n"
                "    "
                (SequenceItem
                  "-"
                  (IdentLit
                    " "
                    "y"))))
            "\n"
            "  "
            (SequenceItem
              "-"
              (BlockMappingLit
                " "
                (MappingEntry
                  (MappingEntryKey
                    "c")
                  ":"
                  (MappingEntryValue
                    (NumberLit
                      " "
                      "3")))
                "\n"
                ""))))))
    "---"
    "\n")
  (Body))"#;
  assert_eq!(tree, expected);
}

#[test]
fn seq_mixed_single_and_multi_inner() {
  let full = format!(
    "---\n{}\n---\n",
    r#"values:
  - - 1
  - - 2
    - 3
  - - 4"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(!tree.contains("Error"));
}

// Sequence items with folded block strings
#[test]
fn seq_with_folded_block_string() {
  let full = format!(
    "---\n{}\n---\n",
    r#"key:
  - >
    hello
    world
  - value2"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("FoldedBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Sequence items with literal block strings
#[test]
fn seq_with_literal_block_string() {
  let full = format!(
    "---\n{}\n---\n",
    r#"key:
  - |
    line1
    line2
  - other"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("LiteralBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Mapping with folded block string value
#[test]
fn mapping_with_folded_block_string() {
  let full = format!(
    "---\n{}\n---\n",
    r#"desc: >
  this is
  folded
title: hello"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("FoldedBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Mapping with literal block string value
#[test]
fn mapping_with_literal_block_string() {
  let full = format!(
    "---\n{}\n---\n",
    r#"desc: |
  line one
  line two
title: hello"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("LiteralBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Mixed: mapping values are sequences with block strings and nested mappings
#[test]
fn complex_mixed_block_strings_and_nesting() {
  let full = format!(
    "---\n{}\n---\n",
    r#"items:
  - name: alice
    bio: >
      alice is
      great
  - name: bob
    bio: |
      bob is
      cool
settings:
  debug: true"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("FoldedBlockStrLit"));
  assert!(tree.contains("LiteralBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Deeply nested: map -> seq -> map -> folded block string
#[test]
fn deep_nested_with_block_string() {
  let full = format!(
    "---\n{}\n---\n",
    r#"outer:
  - inner:
      desc: >
        nested
        folded"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(tree.contains("FoldedBlockStrLit"));
  assert!(!tree.contains("Error"));
}

// Sequence with empty items followed by nested content
#[test]
fn seq_empty_dash_then_nested() {
  let full = format!(
    "---\n{}\n---\n",
    r#"matrix:
  -
    - 1
    - 2
  - 3"#
  );
  let (ast, _) = parse(&full);
  let tree = render_tree(&ast);
  assert!(!tree.contains("Error"));
}
