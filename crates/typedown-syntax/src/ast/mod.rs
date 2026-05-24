//! Layer 3: Typed AST nodes wrapping untyped RedNodes.
//! Each AST type checks the SyntaxKind on cast, providing a type-safe API
//! over the generic tree structure.

use crate::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

/// All AST nodes implement this trait.
pub trait AstNode: Sized {
  /// Try to cast a RedNode into this AST type.
  /// Returns None if the SyntaxKind doesn't match.
  fn cast(syntax: RedNode) -> Option<Self>;

  /// Access the underlying RedNode.
  fn syntax(&self) -> &RedNode;
}

fn child<T: AstNode>(parent: &RedNode) -> Option<T> {
  parent.children().find_map(T::cast)
}

fn children<T: AstNode>(parent: &RedNode) -> impl Iterator<Item = T> {
  parent.children().filter_map(T::cast)
}

/* Top-level nodes */

/// The root of a TDR file: frontmatter + body.
pub struct SourceFile(RedNode);

impl AstNode for SourceFile {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::SourceFile => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}
