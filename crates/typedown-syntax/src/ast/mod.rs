//! Layer 3: Typed AST nodes wrapping untyped RedNodes.
//! Each AST type checks the SyntaxKind on cast, providing a type-safe API
//! over the generic tree structure.

use typedown_macros::AstNode;

use crate::red::RedNode;

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
#[derive(AstNode)]
pub struct SourceFile(RedNode);

/// The YAML frontmatter
#[derive(AstNode)]
pub struct YamlFrontmatter(RedNode);

/// The YAML mapping
#[derive(AstNode)]
pub struct YamlMapping(RedNode);

/// The YAML mapping's key-value pair
#[derive(AstNode)]
pub struct YamlMappingEntry(RedNode);

/// The YAML sequence
#[derive(AstNode)]
pub struct YamlSequence(RedNode);

/// The YAML sequence item
#[derive(AstNode)]
pub struct YamlSequenceItem(RedNode);

/// The Markdown body
#[derive(AstNode)]
pub struct MdBody(RedNode);

/// The Markdown heading
/// Represented by: ## Heading
#[derive(AstNode)]
pub struct MdHeading(RedNode);

/// The Markdown paragraph
/// Represented by: Paragraph ...
#[derive(AstNode)]
pub struct MdParagraph(RedNode);

/// The Markdown blockquote
/// Represented by: > Blockquote
#[derive(AstNode)]
pub struct MdBlockquote(RedNode);

/// The Markdown table
/// Represented by:
/// | header 1 | header 2 |
/// | -------- | -------- |
/// | data 1   | data 2   |
#[derive(AstNode)]
pub struct MdTable(RedNode);

/// The Markdown data row in a table
#[derive(AstNode)]
pub struct MdTableDataRow(RedNode);

/// The Markdown header row in a table
#[derive(AstNode)]
pub struct MdTableHeaderRow(RedNode);

/// The Markdown cell in a table
#[derive(AstNode)]
pub struct MdTableCell(RedNode);

/// The Markdown bullet list
/// Represented by:
/// - item 1
/// - item 2
#[derive(AstNode)]
pub struct MdBulletList(RedNode);

/// The Markdown bullet list item
#[derive(AstNode)]
pub struct MdBulletListItem(RedNode);

/// The Markdown ordered list
/// Represented by:
/// 1. item 1
/// 2. item 2
#[derive(AstNode)]
pub struct MdOrderedList(RedNode);

/// The Markdown ordered list item
#[derive(AstNode)]
pub struct MdOrderedListItem(RedNode);

/// The Markdown toggle list
/// >- summary 1
///
///    details 1
#[derive(AstNode)]
pub struct MdToggleList(RedNode);

/// The Markdown toggle list item
/// >- summary
///
///    details
#[derive(AstNode)]
pub struct MdToggleListItem(RedNode);

/// The Markdown toggle list item summary
#[derive(AstNode)]
pub struct MdToggleListSummary(RedNode);

/// The Markdown toggle list item details
#[derive(AstNode)]
pub struct MdToggleListDetails(RedNode);

/// The Markdown callout block
/// Represented by:
/// ::: label
///  content
/// :::
#[derive(AstNode)]
pub struct MdCalloutBlock(RedNode);

/// The Markdown link
/// Represented by: [alt](link)
#[derive(AstNode)]
pub struct MdLink(RedNode);

/// The Markdown media
/// Represented by: ![alt](link)
#[derive(AstNode)]
pub struct MdMedia(RedNode);

/// The Markdown footnote ref
/// Represented by: [^key]
#[derive(AstNode)]
pub struct MdFootnoteRef(RedNode);

/// The Markdown citation
/// Represented by: [@citation]
#[derive(AstNode)]
pub struct MdCitation(RedNode);

/// The Markdown bold text
/// Represented by: **bold**
#[derive(AstNode)]
pub struct MdBold(RedNode);

/// The Markdown italic text
/// Represented by: _italic_ or *italic*
#[derive(AstNode)]
pub struct MdItalic(RedNode);

/// The Markdown bolditalic text
/// Represented by: ***italic***
#[derive(AstNode)]
pub struct MdBoldItalic(RedNode);

/// The Markdown strikethrough text
/// Represented by: ~strikethrough~
#[derive(AstNode)]
pub struct MdStrikethrough(RedNode);

/// The Markdown plaintext
/// Represented by: text
#[derive(AstNode)]
pub struct MdText(RedNode);

// Expression nodes
#[derive(AstNode)]
pub struct Expr(RedNode);

#[derive(AstNode)]
pub struct CallExpr(RedNode);

#[derive(AstNode)]
pub struct UnaryExpr(RedNode);

#[derive(AstNode)]
pub struct BinaryExpr(RedNode);

// Literals
#[derive(AstNode)]
pub struct TaggedLit(RedNode);

#[derive(AstNode)]
pub struct ListLit(RedNode);

#[derive(AstNode)]
pub struct BlockSeqLit(RedNode);

#[derive(AstNode)]
pub struct DictLit(RedNode);

#[derive(AstNode)]
pub struct DictEntry(RedNode);

#[derive(AstNode)]
pub struct StrLit(RedNode);

#[derive(AstNode)]
pub struct InterpFragment(RedNode);

#[derive(AstNode)]
pub struct MathLit(RedNode);

#[derive(AstNode)]
pub struct CodeLit(RedNode);

#[derive(AstNode)]
pub struct NumberLit(RedNode);

#[derive(AstNode)]
pub struct IdentLit(RedNode);

#[derive(AstNode)]
pub struct Tag(RedNode);

#[derive(AstNode)]
pub struct InlineMath(RedNode);

#[derive(AstNode)]
pub struct MathBlock(RedNode);

#[derive(AstNode)]
pub struct InlineCode(RedNode);

#[derive(AstNode)]
pub struct CodeBlock(RedNode);
