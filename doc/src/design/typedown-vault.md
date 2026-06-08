# Typedown Vault

A Typedown vault is a directory containing a `typedown.yaml` file at its root. This file is the entrypoint: its presence marks the directory as a Typedown vault and configures where the vault's content is located.

A vault is purely an organization convention: it defines how [TDR](./tdr.md) files are arranged on disk & how cross-file definitions are resolved. It has no meaning in the [Typedown abstract model](./typedown-model.md), which only concerns itself with the resource graph and its contents.

## Vault Layout

A typical vault looks like:

```
my-vault/
├── typedown.yaml
├── content/
│   ├── bob.tdr
│   └── mona-lisa.tdr
└── schema/
    ├── person.tdr
    └── artwork.tdr
```

See [TDR](./tdr.md) for how individual files are structured.

### Naming Conventions

Typedown uses **snake_case** throughout:

- **File names**: all `.tdr` files use snake_case (e.g. `my_note.tdr`, `blog_post.tdr`).
- **YAML keys**: all property names in the frontmatter use snake_case (e.g. `birth_date`, `first_name`, `topic_interest`).

### typedown.yaml

`typedown.yaml` (or `typedown.yml`) holds global vault configuration. It has the following fields:

- `version`: the TDR format version.
- `vault`: configuration for the vault.
  - `content_dir`: the directory where content `.tdr` files (resources) are located.
  - `schema_dir`: the directory where schema `.tdr` files (type definitions) are located.

```yaml
version: 1.0.0
vault:
  content_dir: ./content/
  schema_dir: ./schema/
```

Paths are relative to the vault root.
