fn main() {
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let manifest_path = std::path::Path::new(&manifest_dir);
  let extension_toml = manifest_path.join("extension.toml");

  // Detect dev mode: extension.toml has file:// URLs
  // A little ugly
  let is_dev = extension_toml
    .exists()
    .then(|| std::fs::read_to_string(&extension_toml).ok())
    .flatten()
    .is_some_and(|content| content.contains("file://"));

  if is_dev {
    let lsp_path = manifest_path.join("../../target/debug/tdr-lsp");
    if let Ok(canonical) = lsp_path.canonicalize() {
      println!("cargo:rustc-env=TDR_DEV_LSP_PATH={}", canonical.display());
    }
  }

  println!("cargo:rerun-if-changed=extension.toml");
  println!("cargo:rerun-if-changed=../../target/debug/tdr-lsp");
}
