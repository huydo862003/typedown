# Development

For full project setup, see the [root DEVELOPMENT.md](../../DEVELOPMENT.md).

## Dependencies

- **Node.js** (22+)
- **pnpm** (11+)
- **tree-sitter CLI** (0.26+)
- **clang** (21+): C compiler for external scanners
- **wasi-sdk** (25+): Only needed for WASM builds. Set `TREE_SITTER_WASI_SDK_PATH` to the extracted directory.

All provided automatically by `nix develop` from the repo root.

## Setup

```bash
pnpm install
```

## Development

- `pnpm run generate`: Generate C parser sources from grammar.js files
- `pnpm run build:so`: Build shared library (.so) parsers
- `pnpm run build:wasm`: Build WASM parsers
- `pnpm run build`: Build both .so and .wasm
- `pnpm run start`: Open the tree-sitter playground

## Testing

```bash
pnpm run test
```

Test corpus files are under each sub-grammar's `test/corpus/` directory.

## Release

Releases are handled by `publish.sh` from the repo root. CI builds the tree-sitter `.so` and `.wasm` artifacts automatically.
