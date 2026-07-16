# Development

## Testing Against a Local Build

Build the debug binary, then launch with `local_init.lua`:

```bash
pnpm build -r tree-sitter
cargo build -p tdr-lsp
nvim -u editors/nvim/local_init.lua path/to/file.tdr
```

`local_init.lua` sets `vim.g.typedown_dev = true`, which tells the plugin to use
`target/debug/tdr-lsp` instead of downloading a binary.

Run `:LspInfo` inside Neovim to confirm the server attached.

## Testing a Staging Release

1. Push a staging tag via `./publish.sh` from the repo root (choose a `pre*` bump type).
   This bumps `VERSION` and all derived version files, then pushes the tag.
   CI builds and uploads the prerelease binaries automatically.

2. Launch with `staging_init.lua`:

   ```bash
   nvim -u editors/nvim/staging_init.lua path/to/file.tdr
   ```

   The plugin reads `version.lua` (derived from `VERSION`), constructs the
   `staging/vX.Y.Z-label.N` release URL, and downloads the matching binary automatically
   on first launch.

## Tree-sitter

Syntactic highlighting in Neovim uses Tree-sitter. The grammar lives in `editors/tree-sitter/` and the query files live in `queries/typedown/` inside this plugin.

Once the grammar is built, register the parser with nvim-treesitter and the query files will provide highlighting, injections, and folds.

See also: [Tree-sitter research](https://huydo862003.github.io/loupe/research/analysis/syntactic-analysis.html)

## Release

Releases are handled by `publish.sh` from the repo root. It bumps the version in `VERSION`
alongside all other derived version files and pushes the tag that triggers CI.
