use lsp_types::{
  SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensParams,
  SemanticTokensResult,
};
use ropey::Rope;
use typedown_lang::db::derived::parse_file::parse_file;
use typedown_lang::syntax::red::RedNode;
use typedown_lang::syntax::syntax_kind::SyntaxKind;

use crate::analysis::Analysis;
use crate::utils::ast::{ident_is_mapping_key, ident_is_type_ref};
use crate::utils::position::text_offset_to_lsp_position;
use crate::utils::uri::uri_to_path;

#[cfg(test)]
use typedown_lang::db::{
  QueryStorage, TypedownDatabase,
  types::{File, FileHandle, Project},
};

pub fn token_types() -> Vec<SemanticTokenType> {
  vec![
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::TYPE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::new("heading"),
    SemanticTokenType::new("punctuation.bracket"),
  ]
}

fn token_type_index(token_type: &SemanticTokenType) -> u32 {
  token_types()
    .iter()
    .position(|t| t == token_type)
    .expect("token type not in legend") as u32
}

// TODO: recompute incrementally in the future
pub fn semantic_tokens_full(
  analysis: &Analysis,
  params: SemanticTokensParams,
) -> Option<SemanticTokensResult> {
  // Resolve file
  let path = uri_to_path(&params.text_document.uri)?;
  let rope = analysis.file_rope(&path)?;
  let db = &analysis.db;
  let project = analysis.project;
  let file = *project.files(db).get(&path)?;

  // Collect tokens from the AST
  let root = parse_file(db, project, file).ast(db);
  let mut raw: Vec<(usize, usize, SemanticTokenType, u32)> = Vec::new();
  collect_tokens(root, &mut raw);

  // Delta-encode and return
  let data = delta_encode(raw, &rope);
  Some(SemanticTokensResult::Tokens(SemanticTokens {
    result_id: None,
    data,
  }))
}

// Push a span only if it has non-zero length.
fn emit(
  node: &RedNode,
  token_type: SemanticTokenType,
  modifiers: u32,
  out: &mut Vec<(usize, usize, SemanticTokenType, u32)>,
) {
  let len = node.text_len();
  if len > 0 {
    out.push((node.offset(), len, token_type, modifiers));
  }
}

// Walk the AST and collect (offset, length, type, modifiers) spans for all highlighted regions.
fn collect_tokens(node: RedNode, out: &mut Vec<(usize, usize, SemanticTokenType, u32)>) {
  // Node-level: emit the whole span and skip children
  if !node.is_token() {
    if let Some(token_type) = classify_node(&node) {
      emit(&node, token_type, node_modifiers(&node), out);
      return;
    }
    for child in node.children() {
      collect_tokens(child, out);
    }
    return;
  }
  // Leaf token
  if let Some(token_type) = classify_token(&node) {
    emit(&node, token_type, 0, out);
  }
}

// Modifier bit positions must match the order returned by token_modifiers().
const MODIFIER_BOLD: u32 = 1 << 1;
const MODIFIER_ITALIC: u32 = 1 << 2;
const MODIFIER_STRIKETHROUGH: u32 = 1 << 3;

pub fn token_modifiers() -> Vec<SemanticTokenModifier> {
  vec![
    SemanticTokenModifier::READONLY,             // index 0
    SemanticTokenModifier::new("bold"),          // index 1
    SemanticTokenModifier::new("italic"),        // index 2
    SemanticTokenModifier::new("strikethrough"), // index 3
  ]
}

// Returns the modifier bitset for nodes that carry formatting information.
fn node_modifiers(node: &RedNode) -> u32 {
  match node.kind() {
    SyntaxKind::MdHeading => MODIFIER_BOLD,
    SyntaxKind::MdBold => MODIFIER_BOLD,
    SyntaxKind::MdItalic => MODIFIER_ITALIC,
    SyntaxKind::MdBoldItalic => MODIFIER_BOLD | MODIFIER_ITALIC,
    SyntaxKind::MdStrikethrough => MODIFIER_STRIKETHROUGH,
    _ => 0,
  }
}

// Classify a non-leaf node
// Returning Some stops descent into children.
fn classify_node(node: &RedNode) -> Option<SemanticTokenType> {
  match node.kind() {
    SyntaxKind::MdHeading => Some(SemanticTokenType::new("heading")),
    SyntaxKind::MdTableSeparatorRow => Some(SemanticTokenType::OPERATOR),
    SyntaxKind::MdBold
    | SyntaxKind::MdBoldItalic
    | SyntaxKind::MdItalic
    | SyntaxKind::MdStrikethrough => Some(SemanticTokenType::MODIFIER),
    // Not at node level: descend so content inside is highlighted normally.
    // The `>` marker is highlighted via MdSymbol in classify_token.
    // SyntaxKind::MdBlockquote
    SyntaxKind::InlineCode | SyntaxKind::CodeBlock => Some(SemanticTokenType::STRING),
    SyntaxKind::InlineMath | SyntaxKind::MathBlock => Some(SemanticTokenType::OPERATOR),
    SyntaxKind::MdLink | SyntaxKind::MdMedia => Some(SemanticTokenType::STRING),
    SyntaxKind::MdFootnoteRef | SyntaxKind::MdCitation => Some(SemanticTokenType::VARIABLE),
    SyntaxKind::MdCheckbox | SyntaxKind::MdCalloutBlock => Some(SemanticTokenType::KEYWORD),
    _ => None,
  }
}

// Classify a leaf token.
fn classify_token(node: &RedNode) -> Option<SemanticTokenType> {
  match node.kind() {
    SyntaxKind::Ident if ident_is_mapping_key(node) => {
      let text = node.text();
      if text == "_type" || text == "_label" || text == "self" {
        Some(SemanticTokenType::KEYWORD)
      } else {
        Some(SemanticTokenType::PROPERTY)
      }
    }
    SyntaxKind::DqStrStart
    | SyntaxKind::DqStrContent
    | SyntaxKind::DqStrEnd
    | SyntaxKind::SqStrStart
    | SyntaxKind::SqStrContent
    | SyntaxKind::SqStrEnd
    | SyntaxKind::YamlLiteralBlockStrLit
    | SyntaxKind::YamlFoldedBlockStrLit => Some(SemanticTokenType::STRING),
    SyntaxKind::Number => Some(SemanticTokenType::NUMBER),
    SyntaxKind::Ident => {
      if node
        .parent()
        .is_some_and(|p| p.kind() == SyntaxKind::MdText)
      {
        return None;
      }
      if node
        .parent()
        .is_some_and(|p| p.kind() == SyntaxKind::IdentLit)
        && node
          .parent()
          .and_then(|p| p.parent())
          .is_some_and(|gp| gp.kind() == SyntaxKind::CallExpr)
      {
        return Some(SemanticTokenType::FUNCTION);
      }
      if ident_is_type_ref(node) {
        Some(SemanticTokenType::TYPE)
      } else {
        Some(SemanticTokenType::VARIABLE)
      }
    }
    SyntaxKind::YamlComment => Some(SemanticTokenType::COMMENT),
    SyntaxKind::Colon | SyntaxKind::YamlOp => Some(SemanticTokenType::OPERATOR),
    SyntaxKind::LParen
    | SyntaxKind::RParen
    | SyntaxKind::LBracket
    | SyntaxKind::RBracket
    | SyntaxKind::InterpStart
    | SyntaxKind::InterpEnd => Some(SemanticTokenType::new("punctuation.bracket")),
    SyntaxKind::MdNumber => Some(SemanticTokenType::NUMBER),
    SyntaxKind::MdHtmlEntity => Some(SemanticTokenType::STRING),
    // Only highlight structural markers, not arbitrary symbols.
    // MdTableHeaderRow and MdTableSeparatorRow are classified at node level (no descent).
    SyntaxKind::MdSymbol => {
      let parent = node.parent()?;
      let parent_kind = parent.kind();
      match parent_kind {
        // List markers: -, +, *
        SyntaxKind::MdBulletListItem
        | SyntaxKind::MdOrderedListItem
        | SyntaxKind::MdTaskListItem => Some(SemanticTokenType::OPERATOR),
        // Toggle list marker: >-
        SyntaxKind::MdToggleListItem => Some(SemanticTokenType::OPERATOR),
        // Blockquote marker: >
        SyntaxKind::MdBlockquote => Some(SemanticTokenType::COMMENT),
        // Table leading pipe: direct child of the row
        SyntaxKind::MdTableHeaderRow | SyntaxKind::MdTableDataRow => {
          Some(SemanticTokenType::OPERATOR)
        }
        // Table separator/trailing pipes: parse_text consumes | into MdText inside MdTableCell
        SyntaxKind::MdText if node.text() == "|" => {
          let grandparent_kind = parent.parent()?.kind();
          if matches!(
            grandparent_kind,
            SyntaxKind::MdTableCell | SyntaxKind::MdTableHeaderRow | SyntaxKind::MdTableDataRow
          ) {
            Some(SemanticTokenType::OPERATOR)
          } else {
            None
          }
        }
        _ => None,
      }
    }
    _ => None,
  }
}

// LSP tokens are encoded as deltas: each token's line and column are relative to the previous
// token, not absolute
fn delta_encode(
  raw: Vec<(usize, usize, SemanticTokenType, u32)>,
  rope: &Rope,
) -> Vec<SemanticToken> {
  let mut result = Vec::with_capacity(raw.len());
  let mut prev_line = 0u32;
  let mut prev_start = 0u32;

  for (offset, length, token_type, modifiers) in raw {
    // Convert char offset to line/character position
    let pos = text_offset_to_lsp_position(rope, offset);
    let line = pos.line;
    let start = pos.character;

    // Compute deltas relative to the previous token
    let delta_line = line - prev_line;
    let delta_start = if delta_line == 0 {
      start - prev_start
    } else {
      start
    };

    result.push(SemanticToken {
      delta_line,
      delta_start,
      length: length as u32,
      token_type: token_type_index(&token_type),
      token_modifiers_bitset: modifiers,
    });

    prev_line = line;
    prev_start = start;
  }

  result
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;

  use super::*;

  fn parse_tokens(content: &str) -> Vec<SemanticTokenType> {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };
    let path = PathBuf::from("/test.tdr");
    let file = File::new(&db, FileHandle::Content(content.to_string()));
    let project = Project::new(
      &db,
      PathBuf::from("/"),
      HashMap::from([(path.clone(), file)]),
    );
    let ast = parse_file(&db, project, file).ast(&db);
    let mut raw = Vec::new();
    collect_tokens(ast, &mut raw);
    raw
      .into_iter()
      .map(|(_, _, token_type, _)| token_type)
      .collect()
  }

  #[test]
  fn keyword_and_type_tokens() {
    let types = parse_tokens(
      r#"---
_type: Person
name: "Alice"
age: 30
---
"#,
    );
    assert!(
      types.contains(&SemanticTokenType::KEYWORD),
      "expected keyword for _type, got: {types:?}"
    );
    assert!(
      types.contains(&SemanticTokenType::TYPE),
      "expected type for Person, got: {types:?}"
    );
    assert!(
      types.contains(&SemanticTokenType::PROPERTY),
      "expected property for name/age, got: {types:?}"
    );
    assert!(
      types.contains(&SemanticTokenType::STRING),
      "expected string for Alice, got: {types:?}"
    );
    assert!(
      types.contains(&SemanticTokenType::NUMBER),
      "expected number for 30, got: {types:?}"
    );
  }
}
