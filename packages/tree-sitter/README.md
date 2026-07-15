# tree-sitter-typedown

Tree-sitter grammar for the Typedown language.

## Research

The tree-sitter grammar research is documented in the [loupe](https://github.com/huydo862003/loupe) repo.

## Structure

- `grammar.js`: Main grammar (owns `.tdr` files, injects into the two sub-grammars)
- `typedown-yaml/`: YAML sub-grammar
- `typedown-md/`: Markdown sub-grammar

## Scripts

- `pnpm run build:c`: Generate C parser sources from grammar.js files
- `pnpm run build:wasm`: Build WASM binaries (runs `build:c` first)
- `pnpm start`: Open the tree-sitter playground
