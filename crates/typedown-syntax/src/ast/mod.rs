//! Layer 3: Typed AST nodes wrapping untyped RedNodes.
//! Each AST type checks the SyntaxKind on cast, providing a type-safe API
//! over the generic tree structure.

use crate::green::syntax_kind::SyntaxKind;
use crate::red::RedNode;

/// All AST nodes implement this trait.
pub trait AstNode: Sized {
  /// Try to cast a RedNode into this AST type.
  /// Returns None if the SyntaxKind doesn't match.
  fn cast(syntax: RedNode) -> Option<Self>;

  /// Access the underlying RedNode.
  fn syntax(&self) -> &RedNode;
}

/// Helper: find the first child that casts to a given AST type.
fn child<T: AstNode>(parent: &RedNode) -> Option<T> {
  parent.children().find_map(T::cast)
}

/// Helper: find all children that cast to a given AST type.
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

impl SourceFile {
  pub fn frontmatter(&self) -> Option<Frontmatter> {
    child(&self.0)
  }
  pub fn body(&self) -> Option<Body> {
    child(&self.0)
  }
}

/* Frontmatter nodes */

pub struct Frontmatter(RedNode);

impl AstNode for Frontmatter {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Frontmatter => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl Frontmatter {
  pub fn mappings(&self) -> impl Iterator<Item = MappingEntry> {
    children(&self.0)
  }
}

pub struct Mapping(RedNode);

impl AstNode for Mapping {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Mapping => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl Mapping {
  pub fn entries(&self) -> impl Iterator<Item = MappingEntry> {
    children(&self.0)
  }
}

pub struct MappingEntry(RedNode);

impl AstNode for MappingEntry {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::MappingEntry => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Sequence(RedNode);

impl AstNode for Sequence {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Sequence => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl Sequence {
  pub fn items(&self) -> impl Iterator<Item = SequenceItem> {
    children(&self.0)
  }
}

pub struct SequenceItem(RedNode);

impl AstNode for SequenceItem {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::SequenceItem => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

/* Body nodes */

pub struct Body(RedNode);

impl AstNode for Body {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Body => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Heading(RedNode);

impl AstNode for Heading {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Heading => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Paragraph(RedNode);

impl AstNode for Paragraph {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Paragraph => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Blockquote(RedNode);

impl AstNode for Blockquote {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Blockquote => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct CodeBlock(RedNode);

impl AstNode for CodeBlock {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::CodeBlock => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct InlineCode(RedNode);

impl AstNode for InlineCode {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::InlineCode => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct MathBlock(RedNode);

impl AstNode for MathBlock {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::MathBlock => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct InlineMath(RedNode);

impl AstNode for InlineMath {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::InlineMath => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Table(RedNode);

impl AstNode for Table {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Table => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl Table {
  pub fn rows(&self) -> impl Iterator<Item = TableRow> {
    children(&self.0)
  }
}

pub struct TableRow(RedNode);

impl AstNode for TableRow {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::TableRow => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl TableRow {
  pub fn cells(&self) -> impl Iterator<Item = TableCell> {
    children(&self.0)
  }
}

pub struct TableCell(RedNode);

impl AstNode for TableCell {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::TableCell => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct BulletList(RedNode);

impl AstNode for BulletList {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::BulletList => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl BulletList {
  pub fn items(&self) -> impl Iterator<Item = BulletListItem> {
    children(&self.0)
  }
}

pub struct BulletListItem(RedNode);

impl AstNode for BulletListItem {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::BulletListItem => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct OrderedList(RedNode);

impl AstNode for OrderedList {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::OrderedList => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl OrderedList {
  pub fn items(&self) -> impl Iterator<Item = OrderedListItem> {
    children(&self.0)
  }
}

pub struct OrderedListItem(RedNode);

impl AstNode for OrderedListItem {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::OrderedListItem => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct ToggleList(RedNode);

impl AstNode for ToggleList {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::ToggleList => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

impl ToggleList {
  pub fn items(&self) -> impl Iterator<Item = ToggleListItem> {
    children(&self.0)
  }
}

pub struct ToggleListItem(RedNode);

impl AstNode for ToggleListItem {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::ToggleListItem => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct CalloutBlock(RedNode);

impl AstNode for CalloutBlock {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::CalloutBlock => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct FootnoteBlock(RedNode);

impl AstNode for FootnoteBlock {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::FootnoteBlock => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct BibliographyBlock(RedNode);

impl AstNode for BibliographyBlock {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::BibliographyBlock => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Link(RedNode);

impl AstNode for Link {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Link => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Media(RedNode);

impl AstNode for Media {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Media => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct FootnoteRef(RedNode);

impl AstNode for FootnoteRef {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::FootnoteRef => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct Citation(RedNode);

impl AstNode for Citation {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Citation => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

/* Expressions */

pub struct Expr(RedNode);

impl AstNode for Expr {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::Expr => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct InterpExpr(RedNode);

impl AstNode for InterpExpr {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::InterpExpr => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}

pub struct TaggedExpr(RedNode);

impl AstNode for TaggedExpr {
  fn cast(syntax: RedNode) -> Option<Self> {
    match syntax.kind() {
      SyntaxKind::TaggedExpr => Some(Self(syntax)),
      _ => None,
    }
  }
  fn syntax(&self) -> &RedNode {
    &self.0
  }
}
