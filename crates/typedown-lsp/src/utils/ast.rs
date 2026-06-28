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
pub fn is_in_value_position(node: &RedNode) -> bool {
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

/// Returns true if this `Ident` token is directly inside a `YamlMappingEntryKey`.
pub fn ident_is_mapping_key(node: &RedNode) -> bool {
  node
    .parent()
    .is_some_and(|parent| parent.kind() == SyntaxKind::YamlMappingEntryKey)
}

/// Returns true if this `Ident` token is used as a type reference (inside an `IdentLit` that is
/// the value of a `_type` mapping entry, or inside a `CallExpr`/`IndexExpr` type position).
pub fn ident_is_type_ref(node: &RedNode) -> bool {
  let Some(parent) = node.parent() else {
    return false;
  };
  if parent.kind() != SyntaxKind::IdentLit {
    return false;
  }
  // Check if the IdentLit sits inside a YamlMappingEntryValue whose key is `_type`.
  let Some(entry_value) = parent.parent() else {
    return false;
  };
  if entry_value.kind() != SyntaxKind::YamlMappingEntryValue {
    // Could be a CallExpr argument or similar, treat as variable.
    return false;
  }
  let Some(entry) = entry_value.parent() else {
    return false;
  };
  // Find the sibling key node.
  entry
    .children()
    .find(|child| child.kind() == SyntaxKind::YamlMappingEntryKey)
    .is_some_and(|key| key.text().trim() == "_type")
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
pub fn nearest_expr_ancestor(node: &RedNode) -> Option<RedNode> {
  let mut current = node.clone();
  loop {
    if Expr::cast(current.clone()).is_some() {
      return Some(current);
    }
    current = current.parent()?;
  }
}
