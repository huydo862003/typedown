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
