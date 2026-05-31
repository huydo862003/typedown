//! Layer 3: Typed AST nodes wrapping untyped RedNodes.
//! Each AST type checks the SyntaxKind on cast, providing a type-safe API
//! over the generic tree structure.

use typedown_macros::{AstNode, wrapper_ast_node};
use typedown_types::either::Either;
use typedown_types::syntax_kind::SyntaxKind;
use typedown_types::unescape::{unescape, unescape_html_entity};

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

impl SourceFile {
  /// Return the frontmatter of the source file
  fn frontmatter(&self) -> Option<YamlFrontmatter> {
    child::<YamlFrontmatter>(&self.0)
  }

  /// Return the body of the source file
  fn body(&self) -> Option<MdBody> {
    child::<MdBody>(&self.0)
  }
}

/// The YAML frontmatter
#[derive(AstNode)]
pub struct YamlFrontmatter(RedNode);

impl YamlFrontmatter {
  /// Return the top-level mapping in the frontmatter
  fn mapping(&self) -> Option<YamlMapping> {
    child::<YamlMapping>(&self.0)
  }
}

/// The YAML mapping
#[derive(AstNode)]
pub struct YamlMapping(RedNode);

impl YamlMapping {
  /// Return an iterator over the mapping keys
  pub fn keys(&self) -> impl Iterator<Item = String> {
    children::<YamlMappingEntry>(&self.0).filter_map(|e| e.key())
  }

  /// Return an iterator over the mapping values
  pub fn values(&self) -> impl Iterator<Item = Expr> {
    children::<YamlMappingEntry>(&self.0).filter_map(|e| e.value())
  }

  /// Return an iterator over the entries
  pub fn entries(&self) -> impl Iterator<Item = (String, Expr)> {
    children::<YamlMappingEntry>(&self.0).filter_map(|e| e.entry())
  }
}

/// The YAML mapping's key-value pair
#[derive(AstNode)]
pub struct YamlMappingEntry(RedNode);

impl YamlMappingEntry {
  /// Return the key of this mapping entry
  pub fn key(&self) -> Option<String> {
    self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::YamlMappingEntryKey)
      .map(|v| v.chars().collect::<String>())
  }

  /// Return the value of this mapping entry
  pub fn value(&self) -> Option<Expr> {
    let entry_value = self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::YamlMappingEntryValue)?;
    entry_value.children().find_map(Expr::cast)
  }

  /// Return the entry of this mapping entry
  pub fn entry(&self) -> Option<(String, Expr)> {
    Some((self.key()?, self.value()?))
  }
}

/// The YAML sequence
#[derive(AstNode)]
pub struct YamlSequence(RedNode);
impl YamlSequence {
  /// Return the items of this sequence
  pub fn values(&self) -> impl Iterator<Item = Expr> {
    children::<YamlSequenceItem>(&self.0).filter_map(|e| e.value())
  }
}

/// The YAML sequence item
#[derive(AstNode)]
pub struct YamlSequenceItem(RedNode);

impl YamlSequenceItem {
  /// Return the value of this sequence item
  pub fn value(&self) -> Option<Expr> {
    self.0.children().find_map(Expr::cast)
  }
}

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

/// An HTML entity in markdown text, e.g. &amp; &#42; &#x2A;
#[derive(AstNode)]
pub struct MdHtmlEntity(RedNode);

impl MdHtmlEntity {
  pub fn decode(&self) -> Option<String> {
    let raw = self.0.as_token()?.text()?.to_string();
    Some(unescape_html_entity(&raw))
  }
}
//
// Expression nodes
#[wrapper_ast_node(SyntaxKind = [PrimaryExpr, ParenExpr, CallExpr, UnaryExpr, BinaryExpr])]
pub struct Expr(RedNode);

#[derive(AstNode)]
pub struct YamlOp(RedNode);

pub enum YamlOpKind {
  Plus,
  Minus,
  Tilde,
  Not,
  And,
  Or,
  Eq,
  NotEq,
  Lt,
  Gt,
  LtEq,
  GtEq,
  Mul,
  Div,
  Mod,
  Pow,
  Dot,
  Tag(String),
}

impl YamlOp {
  pub fn kind(&self) -> Option<YamlOpKind> {
    let text = self.0.as_token()?.text()?.to_string();
    let kind = match text.as_str() {
      "+" => YamlOpKind::Plus,
      "-" => YamlOpKind::Minus,
      "~" => YamlOpKind::Tilde,
      "!" => YamlOpKind::Not,
      "&&" => YamlOpKind::And,
      "||" => YamlOpKind::Or,
      "==" => YamlOpKind::Eq,
      "!=" => YamlOpKind::NotEq,
      "<" => YamlOpKind::Lt,
      ">" => YamlOpKind::Gt,
      "<=" => YamlOpKind::LtEq,
      ">=" => YamlOpKind::GtEq,
      "*" => YamlOpKind::Mul,
      "/" => YamlOpKind::Div,
      "%" => YamlOpKind::Mod,
      "**" => YamlOpKind::Pow,
      "." => YamlOpKind::Dot,
      op if op.starts_with('!') => YamlOpKind::Tag(op[1..].to_string()),
      _ => None?,
    };
    Some(kind)
  }
}

#[derive(AstNode)]
pub struct CallExpr(RedNode);

impl CallExpr {
  /// Return the callee expression
  pub fn callee(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  /// Return all argument expressions
  pub fn args(&self) -> Vec<Expr> {
    children::<Expr>(&self.0).skip(1).collect()
  }

  /// Return the nth argument (0-indexed)
  pub fn arg(&self, n: usize) -> Option<Expr> {
    children::<Expr>(&self.0).skip(1).nth(n)
  }
}

#[derive(AstNode)]
pub struct UnaryExpr(RedNode);

impl UnaryExpr {
  /// Return the operand expression
  pub fn expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }

  /// Return the operator
  pub fn op(&self) -> Option<YamlOp> {
    self.0.children().find_map(|c| YamlOp::cast(c))
  }
}

#[derive(AstNode)]
pub struct BinaryExpr(RedNode);

impl BinaryExpr {
  /// Return the left operand expression
  pub fn left(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  /// Return the operator
  pub fn op(&self) -> Option<YamlOp> {
    self.0.children().find_map(|c| YamlOp::cast(c))
  }

  /// Return the right operand expression
  pub fn right(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }
}

// Literals
#[derive(AstNode)]
pub struct ListLit(RedNode);

impl ListLit {
  pub fn items(&self) -> impl Iterator<Item = ListItem> {
    children::<ListItem>(&self.0)
  }
}

#[derive(AstNode)]
pub struct ListItem(RedNode);

impl ListItem {
  pub fn value(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(AstNode)]
pub struct DictLit(RedNode);

impl DictLit {
  pub fn entries(&self) -> impl Iterator<Item = DictEntry> {
    children::<DictEntry>(&self.0)
  }

  pub fn keys(&self) -> impl Iterator<Item = String> {
    children::<DictEntry>(&self.0).filter_map(|e| e.key())
  }

  pub fn values(&self) -> impl Iterator<Item = Expr> {
    children::<DictEntry>(&self.0).filter_map(|e| e.value())
  }
}

#[derive(AstNode)]
pub struct DictEntry(RedNode);

impl DictEntry {
  pub fn key(&self) -> Option<String> {
    self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::DictEntryKey)
      .map(|n| n.text())
  }

  pub fn value(&self) -> Option<Expr> {
    let red_node = self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::DictEntryValue)?;
    child::<Expr>(&red_node)
  }

  pub fn entry(&self) -> Option<(String, Expr)> {
    Some((self.key()?, self.value()?))
  }
}

#[derive(AstNode)]
pub struct StrLit(RedNode);

impl StrLit {
  pub fn is_interpolated(&self) -> bool {
    self
      .0
      .children()
      .any(|c| c.kind() == SyntaxKind::InterpFragment)
  }

  pub fn fragments(&self) -> impl Iterator<Item = Either<String, InterpFragment>> {
    self.0.children().filter_map(|child| match child.kind() {
      SyntaxKind::DqStrContent | SyntaxKind::SqStrContent => {
        let raw = child.as_token()?.text()?.to_string();
        Some(Either::Left(unescape(&raw).unwrap_or(raw)))
      }
      // Interp is currently not supported inside string literals
      SyntaxKind::YamlLiteralBlockStrLit | SyntaxKind::YamlFoldedBlockStrLit => {
        Some(Either::Left(child.text()))
      }
      SyntaxKind::InterpFragment => Some(Either::Right(InterpFragment(child))),
      _ => None,
    })
  }
}

#[derive(AstNode)]
pub struct InterpFragment(RedNode);

impl InterpFragment {
  pub fn expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(AstNode)]
pub struct MathLit(RedNode);

impl MathLit {
  pub fn value(&self) -> Option<String> {
    child::<InlineMath>(&self.0)
      .and_then(|n| n.value())
      .or_else(|| child::<MathBlock>(&self.0).and_then(|n| n.value()))
  }
}

#[derive(AstNode)]
pub struct CodeLit(RedNode);

impl CodeLit {
  pub fn label(&self) -> Option<String> {
    child::<CodeBlock>(&self.0).and_then(|n| n.label())
  }

  pub fn value(&self) -> Option<String> {
    child::<InlineCode>(&self.0)
      .and_then(|n| n.value())
      .or_else(|| child::<CodeBlock>(&self.0).and_then(|n| n.value()))
  }
}

#[derive(AstNode)]
pub struct NumberLit(RedNode);

impl NumberLit {
  pub fn value(&self) -> Option<String> {
    self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::Number)?
      .as_token()?
      .text()
      .map(str::to_string)
  }
}

#[derive(AstNode)]
pub struct IdentLit(RedNode);

impl IdentLit {
  pub fn value(&self) -> Option<String> {
    self
      .0
      .children()
      .find(|c| c.kind() == SyntaxKind::Ident)?
      .as_token()?
      .text()
      .map(str::to_string)
  }
}

#[derive(AstNode)]
pub struct InlineMath(RedNode);

impl InlineMath {
  pub fn value(&self) -> Option<String> {
    let text = self.0.as_token()?.text()?.to_string();
    let fence_count = text.chars().take_while(|c| *c == '$').count();
    // Strip opening and closing fence
    let content = text.get(fence_count..text.len().checked_sub(fence_count)?)?;
    Some(content.to_string())
  }
}

#[derive(AstNode)]
pub struct MathBlock(RedNode);

impl MathBlock {
  pub fn value(&self) -> Option<String> {
    let text = self.0.as_token()?.text()?.to_string();
    let fence_count = text.chars().take_while(|c| *c == '$').count();
    // Skip opening fence then the newline
    let after_fence = text.get(fence_count..)?;
    let content_start = after_fence.find('\n')? + 1;
    let content = after_fence.get(content_start..)?;
    // Strip closing fence
    let content = content.get(..content.len().checked_sub(fence_count)?)?;
    Some(content.to_string())
  }
}

#[derive(AstNode)]
pub struct InlineCode(RedNode);

impl InlineCode {
  pub fn value(&self) -> Option<String> {
    let text = self.0.as_token()?.text()?.to_string();
    let fence_count = text.chars().take_while(|c| *c == '`').count();
    // Strip opening and closing fence
    let content = text.get(fence_count..text.len().checked_sub(fence_count)?)?;
    Some(content.to_string())
  }
}

#[derive(AstNode)]
pub struct CodeBlock(RedNode);

impl CodeBlock {
  pub fn label(&self) -> Option<String> {
    let text = self.0.as_token()?.text()?.to_string();
    let fence_count = text.chars().take_while(|c| *c == '`').count();
    let after_fence = text.get(fence_count..)?;
    let label_end = after_fence.find('\n')?;
    let label = after_fence.get(..label_end)?.trim();
    if label.is_empty() {
      None
    } else {
      Some(label.to_string())
    }
  }

  pub fn value(&self) -> Option<String> {
    let text = self.0.as_token()?.text()?.to_string();
    let fence_count = text.chars().take_while(|c| *c == '`').count();
    // Skip opening fence and optional language tag, then the newline
    let after_fence = text.get(fence_count..)?;
    let content_start = after_fence.find('\n')? + 1;
    let content = after_fence.get(content_start..)?;
    // Strip closing fence
    let content = content.get(..content.len().checked_sub(fence_count)?)?;
    Some(content.to_string())
  }
}
