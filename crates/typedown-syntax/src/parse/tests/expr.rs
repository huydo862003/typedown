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
