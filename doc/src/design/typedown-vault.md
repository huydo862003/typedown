# Typedown Vault

A Typedown vault is a directory containing a `typedown.yaml` file at its root. This file is the entrypoint: its presence marks the directory as a Typedown vault and configures where the vault's content is located.

A vault is purely an organization convention: it defines how [TDR](./tdr.md) files are arranged on disk & how cross-file definitions are resolved. It has no meaning in the [Typedown abstract model](./typedown-model.md), which only concerns itself with the resource graph and its contents.

## Vault Layout

A typical vault looks like:

```
my-vault/
├── typedown.yaml
└── vault/
    ├── bob.tdr
    ├── mona-lisa.tdr
    ├── person.tdr
    └── artwork.tdr
```

All `.tdr` files live in the same directory. See [TDR](./tdr.md) for how individual files are structured.

### Naming Conventions

Typedown uses **snake_case** throughout:

- **File names**: all `.tdr` files use snake_case (e.g. `my_note.tdr`, `blog_post.tdr`).
- **YAML keys**: all property names in the frontmatter use snake_case (e.g. `birth_date`, `first_name`, `topic_interest`).

### typedown.yaml

`typedown.yaml` holds global vault configuration. It has the following fields:

- `version`: the TDR format version.
- `vault`: configuration for the vault.
  - `root_dir`: the directory where all `.tdr` files (both resources and schemas) are located.

```yaml
version: 1.0.0
vault:
  root_dir: ./vault/
```

The path is relative to the vault root. Whether a `.tdr` file is a schema or a resource is determined by its contents, not its location.
