# Editor Extensions

Typedown provides editor extensions for VS Code, Neovim, and Zed. All three share the same LSP binary (`typedown-lsp`) for language features (diagnostics, completions, hover, etc.).

## Highlighting

Each editor has two layers of highlighting:

1. **Syntactic highlighting**:

- Pros:
  - Fast, regex/grammar-based, runs locally.
  - Provides instant coloring on file open.
- Cons: Semantic highlighting is not really possible.

2. **Semantic highlighting**: From the LSP, arrives asynchronously.

- Pros: More accurate (understands types, scopes, references).
- Cons: Slower.

When both are active, **semantic tokens take priority**. Syntactic highlighting serves as a fallback before the LSP responds.

Each editor uses a different system for syntactic highlighting:

| Editor  | Syntactic highlighting | Semantic highlighting |
| ------- | ---------------------- | --------------------- |
| VS Code | TextMate grammar       | LSP semantic tokens   |
| Neovim  | Tree-sitter            | LSP semantic tokens   |
| Zed     | Tree-sitter            | LSP semantic tokens   |

## Structure

- `vscode/`: VS Code extension (TypeScript client + TextMate grammar)
- `nvim/`: Neovim plugin (Lua, LSP client + semantic token theme)
- `zed/`: Zed extension (Rust/WASM)

See each directory's `DEVELOPMENT.md` for setup and build instructions.

## Naming conventions

Both syntactic and semantic highlighting rely on naming conventions so themes can style tokens without language-specific rules:

- **LSP semantic token types**: [LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokenTypes)
- **Tree-sitter captures** (Neovim/Zed): [nvim-treesitter highlight groups](https://neovim.io/doc/user/treesitter.html#treesitter-highlight)
- **TextMate scopes** (VS Code): See [vscode/DEVELOPMENT.md](vscode/DEVELOPMENT.md#scope-naming-conventions)

Our LSP currently only provides `TYPE` as a semantic token (for identifiers that resolve to types). All other highlighting is left to syntactic grammars.
