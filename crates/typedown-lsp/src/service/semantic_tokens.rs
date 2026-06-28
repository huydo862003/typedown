use lsp_types::{
  SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensParams, SemanticTokensResult,
};
use ropey::Rope;
use typedown_db::derived::parse_file::parse_file;
use typedown_syntax::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

use crate::analysis::Analysis;
use crate::utils::ast::{ident_is_mapping_key, ident_is_type_ref};
use crate::utils::position::text_offset_to_lsp_position;
use crate::utils::uri::uri_to_path;

#[cfg(test)]
use typedown_db::{
  QueryStorage, TypedownDatabase,
  inputs::{File, FileHandle},
  types::Project,
};

pub const TOKEN_TYPES: &[SemanticTokenType] = &[
  SemanticTokenType::KEYWORD,
  SemanticTokenType::TYPE,
  SemanticTokenType::PROPERTY,
  SemanticTokenType::VARIABLE,
  SemanticTokenType::STRING,
  SemanticTokenType::NUMBER,
  SemanticTokenType::COMMENT,
  SemanticTokenType::OPERATOR,
];

fn token_type_index(tt: &SemanticTokenType) -> u32 {
  TOKEN_TYPES
    .iter()
    .position(|t| t == tt)
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
  let mut raw: Vec<(usize, usize, SemanticTokenType)> = Vec::new();
  collect_tokens(root, &mut raw);

  // Delta-encode and return
  let data = delta_encode(raw, &rope);
  Some(SemanticTokensResult::Tokens(SemanticTokens {
    result_id: None,
    data,
  }))
}

fn collect_tokens(node: RedNode, out: &mut Vec<(usize, usize, SemanticTokenType)>) {
  if node.is_token() {
    if let Some(token_type) = classify(&node) {
      let len = node.text_len();
      if len > 0 {
        out.push((node.offset(), len, token_type));
      }
    }
    return;
  }
  for child in node.children() {
    collect_tokens(child, out);
  }
}

fn classify(node: &RedNode) -> Option<SemanticTokenType> {
  match node.kind() {
    SyntaxKind::Ident if ident_is_mapping_key(node) => {
      let text = node.text();
      if text == "_type" || text == "_schema" {
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
      if ident_is_type_ref(node) {
        Some(SemanticTokenType::TYPE)
      } else {
        Some(SemanticTokenType::VARIABLE)
      }
    }

    SyntaxKind::YamlComment => Some(SemanticTokenType::COMMENT),

    SyntaxKind::YamlOp => Some(SemanticTokenType::OPERATOR),

    _ => None,
  }
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
    raw.into_iter().map(|(_, _, tt)| tt).collect()
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

fn delta_encode(raw: Vec<(usize, usize, SemanticTokenType)>, rope: &Rope) -> Vec<SemanticToken> {
  let mut result = Vec::with_capacity(raw.len());
  let mut prev_line = 0u32;
  let mut prev_start = 0u32;

  for (offset, length, token_type) in raw {
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
      token_modifiers_bitset: 0,
    });

    prev_line = line;
    prev_start = start;
  }

  result
}
