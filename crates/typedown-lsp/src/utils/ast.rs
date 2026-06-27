use typedown_syntax::ast::{AstNode, Expr};
use typedown_syntax::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

/// Find the innermost red node whose source span contains `offset`.
pub fn node_at_offset(root: RedNode, offset: usize) -> Option<RedNode> {
  let start = root.offset();
  let end = start + root.text_len();

  if offset < start || offset >= end {
    return None;
  }

  // Descend into whichever child contains the offset
  for child in root.children() {
    if let Some(found) = node_at_offset(child, offset) {
      return Some(found);
    }
  }

  Some(root)
}

/// Returns true if the cursor is in a value position rather than a key position.
pub fn cursor_is_in_value_not_key(node: &RedNode) -> bool {
  let mut current = node.parent();
  while let Some(ref cur) = current {
    match cur.kind() {
      SyntaxKind::YamlMappingEntryValue => return true,
      SyntaxKind::YamlMappingEntryKey => return false,
      _ => current = cur.parent(),
    }
  }
  false
}

/// Walk up to find the nearest ancestor with the given syntax kind.
pub fn find_ancestor(node: &RedNode, kind: SyntaxKind) -> Option<RedNode> {
  let mut current = node.parent()?;
  loop {
    if current.kind() == kind {
      return Some(current);
    }
    current = current.parent()?;
  }
}

/// Walk up to find the nearest ancestor that can be cast to an Expr.
pub fn find_expr_ancestor(node: &RedNode) -> Option<RedNode> {
  let mut current = node.clone();
  loop {
    if Expr::cast(current.clone()).is_some() {
      return Some(current);
    }
    current = current.parent()?;
  }
}
