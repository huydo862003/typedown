use lsp_server::{Notification, Request};
use lsp_types::{TextDocumentIdentifier, Uri};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HasDocumentUri {
  pub text_document: TextDocumentIdentifier,
}

pub fn try_extract_path_from_request(req: &Request) -> anyhow::Result<Uri> {
  Ok(
    serde_json::from_value::<HasDocumentUri>(req.params.clone())?
      .text_document
      .uri,
  )
}

pub fn try_extract_path_from_notification(note: &Notification) -> anyhow::Result<Uri> {
  Ok(
    serde_json::from_value::<HasDocumentUri>(note.params.clone())?
      .text_document
      .uri,
  )
}
