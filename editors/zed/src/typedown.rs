use zed_extension_api as zed;

struct TypedownLsp;

impl zed::Extension for TypedownLsp {
  fn new() -> Self {
    Self
  }
  fn language_server_command(
    &mut self,
    _language_server_id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
  ) -> zed::Result<zed::Command> {
    let server = worktree
      .which("typedown-lsp")
      .unwrap_or_else(|| "typedown-lsp".to_string());
    Ok(zed::Command {
      command: server,
      args: vec![],
      env: vec![],
    })
  }
}

zed::register_extension!(TypedownLsp);
