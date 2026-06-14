//! Tracked query to get the type of a green node

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, YamlMapping};
use typedown_types::diagnostic::Diagnostic;
use typedown_types::syntax_kind::SyntaxKind;

use crate::derived::evaluate::evaluate_schema::evaluate_schema;
use crate::derived::name_resolver::referee::referee;
use crate::types::{TdrNode, TdrObjectType, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_type(db: &TypedownDatabase, node: TdrNode) -> TypeResult {
  // If this is a top-level mapping, look up _schema to determine its type
  if let Some(mapping) = node.try_cast::<YamlMapping>(db) {
    let is_top_level = mapping
      .syntax()
      .parent()
      .is_some_and(|parent| parent.kind() == SyntaxKind::YamlFrontmatter);
    if is_top_level {
      return get_mapping_type(db, node, &mapping);
    }
  }

  todo!();
}

fn get_mapping_type(db: &TypedownDatabase, node: TdrNode, mapping: &YamlMapping) -> TypeResult {
  // Look for _schema field in the mapping
  for (key, value_expr) in mapping.entries() {
    if key == "_schema" {
      let schema_node = TdrNode::new(
        db,
        node.project(db),
        node.file(db),
        value_expr.syntax().clone(),
      );
      let resolved = referee(db, schema_node);
      if let Some(symbol) = resolved.value(db) {
        return evaluate_schema(db, symbol);
      }

      // _schema field found but could not resolve
      return TypeResult::new(
        db,
        Box::new(TdrObjectType::get(db)),
        vec![Diagnostic::UnresolvedSchema {
          name: value_expr.syntax().text(),
          start_offset: value_expr.syntax().offset(),
          end_offset: value_expr.syntax().offset() + value_expr.syntax().text_len(),
        }],
      );
    }
  }

  // No _schema field found
  TypeResult::new(
    db,
    Box::new(TdrObjectType::get(db)),
    vec![Diagnostic::MissingSchemaField {
      start_offset: mapping.syntax().offset(),
      end_offset: mapping.syntax().offset() + mapping.syntax().text_len(),
    }],
  )
}
