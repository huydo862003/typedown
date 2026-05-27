use super::helpers::*;

fn parse_expr(input: &str) -> String {
  let program = format!("---\nkey: {}\n---\n", input);

  let (ast, _) = parse(&program);
  let root = ast.as_node().unwrap();

  // Extract the fronmatter as we inject expressions inside the frontmatter
  let frontmatter = root.children()[0].as_node().unwrap();

  // Find the BlockMapping `key: ...`
  let mapping = frontmatter
    .children()
    .iter()
    .find(|c| {
      c.is_node()
        && c.as_node().unwrap().kind() == typedown_types::syntax_kind::SyntaxKind::BlockMappingLit
    })
    .expect("Expected BlockMapping in frontmatter");

  // Find they `key: ...` entry
  let entry = mapping.as_node().unwrap().children()[0].as_node().unwrap();

  // Find the last node child which should be the value expression
  let value = entry
    .children()
    .iter()
    .rev()
    .find(|c| c.is_node())
    .expect("Expected value in mapping entry");

  render_tree(value)
}

fn parse_expr_with_diagnostics(input: &str) -> Vec<String> {
  let full = format!("---\nkey: {}\n---\n", input);
  let (_, diagnostics) = parse(&full);
  diagnostics.iter().map(|d| format!("{:?}", d)).collect()
}

#[test]
fn parse_number_literal() {
  let tree = parse_expr("1");
  let expected = r#"(MappingEntryValue
  (NumberLit
    " "
    "1"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_decimal_literal() {
  let tree = parse_expr("3.14");
  let expected = r#"(MappingEntryValue
  (NumberLit
    " "
    "3.14"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_double_quoted_string() {
  let tree = parse_expr(r#""hello""#);
  let expected = r#"(MappingEntryValue
  (StrLit
    " "
    "\""
    "hello"
    "\""))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_single_quoted_string() {
  let tree = parse_expr("'hello'");
  let expected = r#"(MappingEntryValue
  (StrLit
    " "
    "'"
    "hello"
    "'"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_identifier_literal() {
  let tree = parse_expr("true");
  let expected = r#"(MappingEntryValue
  (IdentLit
    " "
    "true"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_list_literal() {
  let tree = parse_expr("[1, 2]");
  let expected = r#"(MappingEntryValue
  (ListLit
    " "
    "["
    (NumberLit
      "1")
    ","
    (NumberLit
      " "
      "2")
    "]"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_binary_expression() {
  let tree = parse_expr("1 + 2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "1")
    " "
    "+"
    (NumberLit
      " "
      "2")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_parenthesized_expression() {
  let tree = parse_expr("(1)");
  let expected = r#"(MappingEntryValue
  (ParenExpr
    " "
    "("
    (NumberLit
      "1")
    ")"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_subtraction() {
  let tree = parse_expr("3 - 1");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "3")
    " "
    "-"
    (NumberLit
      " "
      "1")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_multiplication() {
  let tree = parse_expr("2 * 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "2")
    " "
    "*"
    (NumberLit
      " "
      "3")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_division() {
  let tree = parse_expr("6 / 2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "6")
    " "
    "/"
    (NumberLit
      " "
      "2")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_unary_negation() {
  let tree = parse_expr("-1");
  let expected = r#"(MappingEntryValue
  (UnaryExpr
    " "
    "-"
    (NumberLit
      "1")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_precedence_multiply_add() {
  let tree = parse_expr("1 + 2 * 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "1")
    " "
    "+"
    (BinaryExpr
      (NumberLit
        " "
        "2")
      " "
      "*"
      (NumberLit
        " "
        "3"))))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_nested_parens() {
  let tree = parse_expr("(1 + 2) * 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (ParenExpr
      " "
      "("
      (BinaryExpr
        (NumberLit
          "1")
        " "
        "+"
        (NumberLit
          " "
          "2"))
      ")")
    " "
    "*"
    (NumberLit
      " "
      "3")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_empty_list_literal() {
  let tree = parse_expr("[]");
  let expected = r#"(MappingEntryValue
  (ListLit
    " "
    "["
    "]"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_nested_list_literal() {
  let tree = parse_expr("[[1], [2]]");
  let expected = r#"(MappingEntryValue
  (ListLit
    " "
    "["
    (ListLit
      "["
      (NumberLit
        "1")
      "]")
    ","
    (ListLit
      " "
      "["
      (NumberLit
        "2")
      "]")
    "]"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_dictionary_literal() {
  let tree = parse_expr("{a: 1}");
  let expected = r#"(MappingEntryValue
  (DictLit
    " "
    "{"
    (DictEntry
      (DictEntryKey
        "a")
      ":"
      (DictEntryValue
        (NumberLit
          " "
          "1")))
    "}"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_call_expression() {
  let tree = parse_expr("f(1, 2)");
  let expected = r#"(MappingEntryValue
  (CallExpr
    (IdentLit
      " "
      "f")
    "("
    (NumberLit
      "1")
    ","
    (NumberLit
      " "
      "2")
    ")"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_complex_expression() {
  let tree = parse_expr("f(1 + 2, [3])");
  let expected = r#"(MappingEntryValue
  (CallExpr
    (IdentLit
      " "
      "f")
    "("
    (BinaryExpr
      (NumberLit
        "1")
      " "
      "+"
      (NumberLit
        " "
        "2"))
    ","
    (ListLit
      " "
      "["
      (NumberLit
        "3")
      "]")
    ")"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_left_associative_addition() {
  let tree = parse_expr("1 + 2 + 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (BinaryExpr
      (NumberLit
        " "
        "1")
      " "
      "+"
      (NumberLit
        " "
        "2"))
    " "
    "+"
    (NumberLit
      " "
      "3")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_left_associative_subtraction() {
  let tree = parse_expr("5 - 3 - 1");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (BinaryExpr
      (NumberLit
        " "
        "5")
      " "
      "-"
      (NumberLit
        " "
        "3"))
    " "
    "-"
    (NumberLit
      " "
      "1")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_multiply_before_subtract() {
  let tree = parse_expr("5 - 2 * 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "5")
    " "
    "-"
    (BinaryExpr
      (NumberLit
        " "
        "2")
      " "
      "*"
      (NumberLit
        " "
        "3"))))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_divide_before_add() {
  let tree = parse_expr("1 + 6 / 2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "1")
    " "
    "+"
    (BinaryExpr
      (NumberLit
        " "
        "6")
      " "
      "/"
      (NumberLit
        " "
        "2"))))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_unary_minus_in_binary() {
  let tree = parse_expr("-1 + 2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (UnaryExpr
      " "
      "-"
      (NumberLit
        "1"))
    " "
    "+"
    (NumberLit
      " "
      "2")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_unary_minus_right_side() {
  let tree = parse_expr("1 + -2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "1")
    " "
    "+"
    (UnaryExpr
      " "
      "-"
      (NumberLit
        "2"))))"#;
  assert_eq!(tree, expected);
}


#[test]
fn parse_comparison() {
  let tree = parse_expr("1 == 2");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (NumberLit
      " "
      "1")
    " "
    "=="
    (NumberLit
      " "
      "2")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_logical_and() {
  let tree = parse_expr("true && false");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (IdentLit
      " "
      "true")
    " "
    "&&"
    (IdentLit
      " "
      "false")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_precedence() {
  let tree = parse_expr("1 + 2 == 3");
  let expected = r#"(MappingEntryValue
  (BinaryExpr
    (BinaryExpr
      (NumberLit
        " "
        "1")
      " "
      "+"
      (NumberLit
        " "
        "2"))
    " "
    "=="
    (NumberLit
      " "
      "3")))"#;
  assert_eq!(tree, expected);
}
