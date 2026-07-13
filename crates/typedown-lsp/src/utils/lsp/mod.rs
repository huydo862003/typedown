use lsp_server::{Notification, Request};
use lsp_types::{TextDocumentIdentifier, Uri};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HasDocumentUri {
  pub text_document: TextDocumentIdentifier,
}

pub fn try_extract_path_from_request(req: &Request) -> Option<Uri> {
  Some(
    serde_json::from_value::<HasDocumentUri>(req.params.clone())
      .ok()?
      .text_document
      .uri,
  )
}

pub fn try_extract_path_from_notification(note: &Notification) -> Option<Uri> {
  Some(
    serde_json::from_value::<HasDocumentUri>(note.params.clone())
      .ok()?
      .text_document
      .uri,
  )
}
