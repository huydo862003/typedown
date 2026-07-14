# Development

## Requirements

- Rust (nightly)
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

## Build

The Zed extension is compiled to WASM:

```bash
cargo build --target wasm32-wasip1 -p typedown-zed
```

## Testing

Install the extension locally in Zed via `zed: install dev extension`, pointing to the `editors/zed` directory.

## Tree-sitter

Syntactic highlighting in Zed uses Tree-sitter. The grammar lives in `editors/tree-sitter/`. Zed extensions reference the grammar via `extension.toml` and include query files under `languages/typedown/`.

See also: [Tree-sitter research](https://huydo862003.github.io/loupe/research/analysis/syntactic-analysis.html)
