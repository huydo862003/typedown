use lsp_server::Notification;
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, PublishDiagnosticsParams};
use ropey::Rope;
use typedown_db::derived::evaluate::evaluate_resource::evaluate_resource;
use typedown_db::derived::name_resolver::file_symbol::file_symbol;
use typedown_db::derived::parse_file::parse_file;
use typedown_types::diagnostic::Diagnostic as TdrDiagnostic;

use crate::analysis::Analysis;
use crate::utils::position::text_offset_to_lsp_position;
use crate::utils::uri::path_to_uri;

pub fn publish_diagnostics(analysis: &Analysis) -> Vec<Notification> {
  let db = &analysis.db;
  let project = analysis.project;
  let files = project.files(db);

  let mut notifications = Vec::new();

  for (path, file) in &files {
    let rope = match analysis.file_rope(path) {
      Some(rope) => rope,
      None => continue,
    };

    let parse_result = parse_file(db, project, *file);
    let mut tdr_diags: Vec<TdrDiagnostic> = parse_result.diagnostics(db).to_vec();

    if let Some(sym) = file_symbol(db, project, *file).value(db) {
      let eval_result = evaluate_resource(db, sym);
      tdr_diags.extend(eval_result.diagnostics(db).iter().cloned());
    }

    let lsp_diags: Vec<Diagnostic> = tdr_diags
      .iter()
      .filter_map(|diag| to_lsp_diagnostic(diag, &rope))
      .collect();

    let scheme = analysis
      .scheme_map
      .get(path)
      .map(String::as_str)
      .unwrap_or("file");
    let uri = path_to_uri(path, scheme);
    let params = PublishDiagnosticsParams {
      uri,
      diagnostics: lsp_diags,
      version: None,
    };

    notifications.push(Notification::new(
      PublishDiagnostics::METHOD.to_string(),
      params,
    ));
  }

  notifications
}

fn to_lsp_diagnostic(diag: &TdrDiagnostic, rope: &Rope) -> Option<Diagnostic> {
  let (start_offset, end_offset) = diag.offsets()?;

  let start_offset = start_offset.min(rope.len_chars());
  let end_offset = end_offset.min(rope.len_chars());

  let range = lsp_types::Range {
    start: text_offset_to_lsp_position(rope, start_offset),
    end: text_offset_to_lsp_position(rope, end_offset),
  };

  Some(Diagnostic {
    range,
    severity: Some(DiagnosticSeverity::ERROR),
    code: Some(NumberOrString::String(diag.code().into())),
    source: Some("typedown".into()),
    message: diag.message(),
    ..Default::default()
  })
}
