//! Export typedown documents to Markdown.

use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_resource::evaluate_resource;
use crate::db::derived::name_resolver::file_symbol::file_symbol;
use crate::db::types::{File, Project, SymbolKind, TdrObjectLike};

/// Export the markdown body of a resource file.
/// Returns `None` if the file is not a resource or has no `_content` field.
pub fn export_markdown(db: &TypedownDatabase, project: Project, file: File) -> Option<String> {
  let symbol = file_symbol(db, project, file).value(db)?;
  if !matches!(symbol.kind(db), SymbolKind::UserDefinedResource(..)) {
    return None;
  }

  let result = evaluate_resource(db, symbol);
  let obj = result.value(db)?;
  let content = obj.get_owned_field(db, "_content")?;
  let str_obj = content.as_tdr_str_obj()?;
  Some(str_obj.value(db))
}

#[cfg(test)]
mod tests {
  use super::export_markdown;
  use crate::db::fixtures::load_vault_fixture;

  #[test]
  fn exports_content_field_as_markdown() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/md_with_content.tdr");
    let output = export_markdown(&db, project, file);
    assert_eq!(
      output.as_deref(),
      Some(
        r#"Hello world
"#
      )
    );
  }

  #[test]
  fn exports_all_markdown_elements() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/all_md_elements.tdr");
    let output = export_markdown(&db, project, file).expect("should export content");
    assert_eq!(
      output,
      r#"# Heading 1

## Heading 2

Plain paragraph with **bold** and _italic_ and ***bold italic*** and ~strikethrough~.

`inline code`

```python
print("hello")
```

$E = mc^2$

$$
\int_0^\infty e^{-x^2} dx
$$

> blockquote line

| Col1 | Col2 |
| ---- | ---- |
| a    | b    |

- bullet one
- bullet two

1. ordered one
2. ordered two

::: note
callout content
:::

[link text](https://example.com)

![alt](image.png)

[^fn1]

[@cite1]
"#
    );
  }

  #[test]
  fn returns_none_for_schema_file() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/schema_in_content.tdr");
    // schema_in_content.tdr is treated as a resource (not a schema), so _content is absent, export should return None
    let output = export_markdown(&db, project, file);
    assert!(
      output.is_none(),
      "schema file with no _content should return None"
    );
  }
}
