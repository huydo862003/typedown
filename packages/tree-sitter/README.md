# tree-sitter-tdr

Tree-sitter grammar for the Typedown TDR language.

## Research

The tree-sitter grammar research is documented in the [loupe](https://github.com/huydo862003/loupe) repo.

## Structure

- `grammar.js`: Main grammar (owns `.tdr` files, injects into the two sub-grammars)
- `tdr-yaml/`: YAML sub-grammar
- `tdr-md/`: Markdown sub-grammar
- `tdr-md-inline/`: Markdown inline sub-grammar

## Scripts

- `pnpm run generate`: Generate C parser sources from grammar.js files
- `pnpm run build:so`: Build shared library (.so) parsers
- `pnpm run build:wasm`: Build WASM parsers
- `pnpm run build`: Build both .so and .wasm
- `pnpm run test`: Run tree-sitter tests
- `pnpm start`: Open the tree-sitter playground
