//! Serialize TDR objects to plain JSON for RPC/document serving

use std::collections::HashSet;

use tdr_incremental::Id;

use super::evaluate_lazy_field;
use crate::db::TypedownDatabase;
use crate::db::types::{FileHandle, TdrObjectEnum};

/// Serialize a FileHandle to a JSON object
pub fn handle_to_json(handle: &FileHandle) -> serde_json::Value {
  match handle {
    FileHandle::Path(path, mtime) => {
      let mtime_secs = mtime
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
      serde_json::json!({
        "type": "path",
        "path": path.to_string_lossy(),
        "mtime": mtime_secs,
      })
    }
    FileHandle::Content(path, content) => {
      serde_json::json!({
        "type": "content",
        "path": path.to_string_lossy(),
        "content": content,
      })
    }
  }
}

/// Returned when a cycle is detected during serialization
#[derive(Debug)]
pub struct CircularRef;

/// Serialize a TDR object to a plain JSON value
pub fn to_json(
  db: &TypedownDatabase,
  obj: &TdrObjectEnum,
) -> Result<serde_json::Value, CircularRef> {
  serialize(db, obj, &mut HashSet::new())
}

fn serialize(
  db: &TypedownDatabase,
  obj: &TdrObjectEnum,
  visiting: &mut HashSet<(usize, usize)>,
) -> Result<serde_json::Value, CircularRef> {
  match obj {
    TdrObjectEnum::TdrStrObj(str_obj) => Ok(serde_json::Value::String(str_obj.value(db))),

    TdrObjectEnum::TdrNumObj(num_obj) => {
      // NaN and Infinity are not valid JSON, fall back to null
      let value = num_obj.value(db);
      match serde_json::Number::from_f64(value) {
        Some(num) => Ok(serde_json::Value::Number(num)),
        None => Ok(serde_json::Value::Null),
      }
    }

    TdrObjectEnum::TdrBoolObj(bool_obj) => Ok(serde_json::Value::Bool(bool_obj.value(db))),

    TdrObjectEnum::TdrMathObj(math_obj) => Ok(serde_json::Value::String(math_obj.value(db))),

    TdrObjectEnum::TdrDateTimeObj(dt) => Ok(serde_json::Value::String(dt.value(db))),
    TdrObjectEnum::TdrDateObj(dt) => Ok(serde_json::Value::String(dt.value(db))),
    TdrObjectEnum::TdrTimeObj(dt) => Ok(serde_json::Value::String(dt.value(db))),

    TdrObjectEnum::TdrListObj(list) => {
      let mut items = Vec::with_capacity(list.len(db));
      for idx in 0..list.len(db) {
        match list.get(db, idx) {
          Some(item) => items.push(serialize(db, &item, visiting)?),
          None => items.push(serde_json::Value::Null),
        }
      }
      Ok(serde_json::Value::Array(items))
    }

    TdrObjectEnum::TdrDictObj(dict) => {
      let mut map = serde_json::Map::new();
      for (key, entry) in dict.entries(db) {
        if let Some(item) = evaluate_lazy_field(db, entry) {
          map.insert(key, serialize(db, &item, visiting)?);
        }
      }
      Ok(serde_json::Value::Object(map))
    }

    TdrObjectEnum::TdrProductObj(product) => {
      // If this object is already on the call stack we have a cycle
      let id = product.as_id();
      if !visiting.insert(id) {
        return Err(CircularRef);
      }
      let mut map = serde_json::Map::new();
      for (key, entry) in product.fields(db) {
        if let Some(item) = evaluate_lazy_field(db, entry) {
          map.insert(key, serialize(db, &item, visiting)?);
        }
      }
      visiting.remove(&id);
      Ok(serde_json::Value::Object(map))
    }

    TdrObjectEnum::TdrBlobObj(blob) => {
      let format = blob.asset_kind(db).as_format_str();
      let handle = handle_to_json(&blob.file(db).handle(db));
      Ok(serde_json::json!({ "format": format, "handle": handle }))
    }

    // Type objects and functions are not meaningful as document values
    _ => Ok(serde_json::Value::Null),
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::time::SystemTime;

  use tdr_types::either::Either;

  use std::collections::HashMap;

  use super::*;
  use crate::db::derived::evaluate::evaluate_resource::evaluate_resource;
  use crate::db::derived::name_resolver::file_symbol::file_symbol;
  use crate::db::fixtures::load_vault_fixture;
  use crate::db::types::{
    AssetKind, File, FileHandle, TdrBlobObj, TdrBoolObj, TdrDateObj, TdrDateTimeObj, TdrDictObj,
    TdrListObj, TdrMathObj, TdrNumObj, TdrProductObj, TdrStrObj, TdrStrType, TdrTimeObj,
  };
  use crate::db::{QueryStorage, TypedownDatabase};

  fn empty_db() -> TypedownDatabase {
    TypedownDatabase {
      storage: QueryStorage::default(),
    }
  }

  #[test]
  fn serializes_string() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrStrObj::new(&db, "hello".to_string()));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::String("hello".to_string()));
  }

  #[test]
  fn serializes_number() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrNumObj::new(&db, 42.0));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::json!(42.0));
  }

  #[test]
  fn non_finite_float_serializes_to_null() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrNumObj::new(&db, f64::NAN));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::Null);
  }

  #[test]
  fn infinity_serializes_to_null() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrNumObj::new(&db, f64::INFINITY));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::Null);
  }

  #[test]
  fn serializes_bool() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrBoolObj::new(&db, true));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::Bool(true));
  }

  #[test]
  fn serializes_list() {
    let db = empty_db();
    let items = vec![
      Either::Right(TdrObjectEnum::from(TdrNumObj::new(&db, 1.0))),
      Either::Right(TdrObjectEnum::from(TdrStrObj::new(&db, "two".to_string()))),
      Either::Right(TdrObjectEnum::from(TdrBoolObj::new(&db, false))),
    ];
    let obj = TdrObjectEnum::from(TdrListObj::new(&db, items));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::json!([1.0, "two", false]));
  }

  #[test]
  fn serializes_dict() {
    let db = empty_db();
    let mut entries = HashMap::new();
    entries.insert(
      "x".to_string(),
      Either::Right(TdrObjectEnum::from(TdrNumObj::new(&db, 10.0))),
    );
    entries.insert(
      "y".to_string(),
      Either::Right(TdrObjectEnum::from(TdrStrObj::new(
        &db,
        "hello".to_string(),
      ))),
    );
    let obj = TdrObjectEnum::from(TdrDictObj::new(&db, entries));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value["x"], serde_json::json!(10.0));
    assert_eq!(value["y"], serde_json::json!("hello"));
  }

  #[test]
  fn serializes_math_as_string() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrMathObj::new(&db, "$E = mc^2$".to_string()));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::String("$E = mc^2$".to_string()));
  }

  #[test]
  fn serializes_datetime_as_string() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrDateTimeObj::new(&db, "2024-01-15T10:30:00Z".to_string()));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(
      value,
      serde_json::Value::String("2024-01-15T10:30:00Z".to_string())
    );
  }

  #[test]
  fn serializes_date_as_string() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrDateObj::new(&db, "2024-01-15".to_string()));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::String("2024-01-15".to_string()));
  }

  #[test]
  fn serializes_time_as_string() {
    let db = empty_db();
    let obj = TdrObjectEnum::from(TdrTimeObj::new(&db, "10:30:00".to_string()));
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value, serde_json::Value::String("10:30:00".to_string()));
  }

  #[test]
  fn serializes_product_fields() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/valid_person.tdr");
    let result = evaluate_resource(&db, file_symbol(&db, project, file).value(&db).unwrap());
    let obj = result.value(&db).expect("should evaluate resource");
    let value = to_json(&db, &obj).expect("should serialize without cycle");
    assert!(value.is_object(), "product should serialize to object");
    assert_eq!(
      value["name"],
      serde_json::Value::String("Alice".to_string())
    );
    assert_eq!(value["age"], serde_json::json!(30.0));
  }

  // Two distinct product objects nested inside each other are not a cycle.
  // This confirms shared non-cyclic products serialize fully without error.
  // A true cycle (product A contains product A) is not constructible through
  // the public API since ids are assigned at construction time.
  #[test]
  fn nested_product_serializes_without_cycle() {
    let db = empty_db();
    let schema: crate::db::types::TdrTypeEnum = TdrStrType::get(&db).into();
    let inner = TdrProductObj::new(&db, schema.clone(), HashMap::new());
    let mut fields = HashMap::new();
    fields.insert(
      "inner".to_string(),
      Either::Right(TdrObjectEnum::from(inner)),
    );
    let outer = TdrProductObj::new(&db, schema, fields);

    let result = to_json(&db, &TdrObjectEnum::from(outer));
    assert!(result.is_ok(), "non-cyclic nested product should serialize");
  }

  #[test]
  fn blob_includes_format_and_path() {
    let db = empty_db();
    let path = PathBuf::from("/vault/assets/photo.png");
    let file = File::new(&db, FileHandle::Path(path.clone(), SystemTime::UNIX_EPOCH));
    let blob = TdrBlobObj::new(&db, AssetKind::Png, file);
    let obj = TdrObjectEnum::from(blob);
    let value = to_json(&db, &obj).unwrap();
    assert_eq!(value["format"], "png");
    assert_eq!(value["handle"]["type"], "path");
    assert_eq!(value["handle"]["path"], "/vault/assets/photo.png");
  }
}
