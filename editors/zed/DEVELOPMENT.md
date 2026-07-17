# Development

For full project setup, see the [root DEVELOPMENT.md](../../DEVELOPMENT.md).

## Dependencies

- **Rust nightly**: To build the extension WASM
- **wasm32-wasip1 target**: WASM compilation target
- **Zed**

All provided automatically by `nix develop` from the repo root.

## Setup

No additional setup needed beyond the root project.

## Development

- Extension source: `src/typedown.rs`
- Tree-sitter queries: `languages/*/`
- Grammar references: `extension.toml`
- Build WASM: `cd editors/zed && cargo build`

## Testing

### Non-Nix

Requires Rust installed via [rustup](https://rustup.rs/) (not homebrew or system packages).

Install the extension locally in Zed via `zed: install dev extension`, pointing to the `editors/zed` directory. Zed handles the WASM compilation automatically.

### NixOS

Zed's `install dev extension` requires `rustup`, which is not available on NixOS ([zed-industries/zed#42353](https://github.com/zed-industries/zed/issues/42353)). Build and install manually instead:

1. Build the extension WASM:

   ```bash
   cd editors/zed
   cargo build
   ```

2. Copy files to Zed's extension directory:

   ```
   ~/.local/share/zed/extensions/installed/typedown/
     extension.toml          <- editors/zed/extension.toml
     extension.wasm          <- target/wasm32-wasip1/debug/typedown_zed.wasm
     languages/              <- editors/zed/languages/
     grammars/*.wasm         <- packages/tree-sitter/dist/tree-sitter-so/*.so (renamed to .wasm)
   ```

3. Restart Zed to pick up changes.

See also [nix-zed-extensions](https://github.com/DuskSystems/nix-zed-extensions) for a Nix-native approach to building and installing Zed extensions.

## Release

Releases are handled by `publish.sh` from the repo root. CI builds the WASM and packages the extension automatically.

## Tree-sitter

Syntactic highlighting in Zed uses Tree-sitter. The grammar lives in `packages/tree-sitter/`. Zed extensions reference the grammar via `extension.toml` and include query files under `languages/tdr/`.

See also: [Tree-sitter research](https://huydo862003.github.io/loupe/research/analysis/syntactic-analysis.html)
