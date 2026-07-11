use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::Ordering;

use tempfile::TempDir;
use typedown_incremental::{CacheSession, InputId, SerializableQueryDatabase};
use typedown_lang::db::TypedownDatabase;
use typedown_lang::db::derived::evaluate::evaluate_resource::evaluate_resource;
use typedown_lang::db::derived::name_resolver::file_symbol::file_symbol;
use typedown_lang::db::derived::parse_file::parse_file;
use typedown_lang::db::types::Project;

use super::utils::{copy_dir_recursive, setup_db_cached, setup_db_fresh};

// Dump cache in session 1, reload and run queries in session 2 (child process).
// Uses a child process to get clean statics.
#[test]
fn cache_roundtrip_with_project() {
  // Session 2: re-invoked as child process
  if std::env::var("CACHE_ROUNDTRIP_SESSION").as_deref() == Ok("2") {
    let project_dir = PathBuf::from(std::env::var("CACHE_ROUNDTRIP_PROJECT").unwrap());
    let cache_dir = PathBuf::from(std::env::var("CACHE_ROUNDTRIP_CACHE").unwrap());

    let db = setup_db_cached(&cache_dir, &project_dir);
    run_diagnostics(&db);
    return;
  }

  // Session 1
  let source = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("examples/project_tracker");
  if !source.exists() {
    eprintln!("skipping: examples/project_tracker not found");
    return;
  }

  let tmp = TempDir::new().unwrap();
  let project_dir = tmp.path().join("project_tracker");
  copy_dir_recursive(&source, &project_dir);

  let cache_dir = project_dir.join(".typedown/cache");

  // Load project, run queries, dump
  let db = setup_db_fresh(&project_dir);
  run_diagnostics(&db);

  let serialized = db.dump();

  let (session, _) = CacheSession::open(&cache_dir).unwrap();
  let revision = db.storage.revision.load(Ordering::Acquire) as u64;
  session.finalize(&serialized, revision).unwrap();

  // Session 2: re-run this test binary in a child process
  let status = Command::new(std::env::current_exe().unwrap())
    .env("CACHE_ROUNDTRIP_SESSION", "2")
    .env("CACHE_ROUNDTRIP_PROJECT", project_dir.to_str().unwrap())
    .env("CACHE_ROUNDTRIP_CACHE", cache_dir.to_str().unwrap())
    .arg("cache_roundtrip::cache_roundtrip_with_project")
    .arg("--exact")
    .arg("--nocapture")
    .output()
    .expect("failed to spawn child process");

  if !status.status.success() {
    panic!(
      "Session 2 failed:\nstdout: {}\nstderr: {}",
      String::from_utf8_lossy(&status.stdout),
      String::from_utf8_lossy(&status.stderr),
    );
  }
}

// Trigger parsing, symbol resolution, and typechecking for all .tdr files
fn run_diagnostics(db: &TypedownDatabase) {
  let project = Project::iter(db)
    .into_iter()
    .next()
    .expect("project should exist");
  for (path, file) in project.files(db) {
    if path.extension().and_then(|e| e.to_str()) != Some("tdr") {
      continue;
    }
    let result = parse_file(db, project, file);
    let _ = result.diagnostics(db);
    if let Some(sym) = file_symbol(db, project, file).value(db) {
      let eval = evaluate_resource(db, sym);
      let _ = eval.diagnostics(db);
    }
  }
}
