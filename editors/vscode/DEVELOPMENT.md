# Development

## Requirements

- [Nix](https://nixos.org) with flakes enabled (provides Node.js, pnpm, cargo-watch, and Rust nightly)

## Setup

```bash
nix develop
pnpm install
```

## Testing Against a Local Build

Build the Rust LSP binary and compile the extension, then launch the Extension Development Host:

```bash
pnpm run compile:dev
```

Then press `F5` in VSCode (or use the "Run Extension (local dev)" launch configuration).

`compile:dev` builds `typedown-lsp` from source and copies the debug binary into `bin/` before bundling the extension.

To watch for changes across both Rust and TypeScript:

```bash
pnpm run watch:dev
```

This runs `cargo watch` on the Rust crates and esbuild/tsc in parallel. Relaunch the Extension Development Host after each Rust rebuild.

## Testing a Staging Release

1. Push a staging tag via `./publish.sh` from the repo root (choose a `pre*` bump type).
   CI builds and uploads the prerelease binaries automatically.

2. Download the staging binary matching the current version and compile:

   ```bash
   pnpm run compile:staging
   ```

   Then press `F5` (or use the "Run Extension (staging binary)" launch configuration).

   `fetch:staging` reads the version from `VERSION` at the repo root, constructs the `staging/vX.Y.Z-label.N` GitHub release URL, and downloads the matching binary into `bin/`.

## Lint and Type Check

```bash
pnpm run lint
pnpm run check-types
```

## Release

Releases are handled by `publish.sh` from the repo root. It bumps the version in `VERSION` alongside all other packages and pushes the tag that triggers CI to build and publish the VSIX.
