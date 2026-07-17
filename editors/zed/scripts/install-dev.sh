#!/usr/bin/env bash
# Build everything and generate extension.toml for local development.
# Embeds the absolute path to target/debug/tdr-lsp into the extension binary.
# See: https://github.com/zed-industries/zed/issues/42353
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ZED_DIR="$REPO_ROOT/editors/zed"
VERSION="$(cat "$REPO_ROOT/VERSION")"
COMMIT_SHA="$(git -C "$REPO_ROOT" rev-parse HEAD)"

# Build LSP binary
echo "Building tdr-lsp..."
cargo build -p tdr-lsp

# Build extension WASM with dev LSP path embedded
echo "Building extension WASM..."
TDR_DEV_LSP_PATH="$REPO_ROOT/target/debug/tdr-lsp" \
  cargo build -p typedown-zed --target wasm32-wasip1

# Clean stale grammar caches
rm -rf "$ZED_DIR/grammars/"

# Generate extension.toml from dev template
echo "Generating extension.toml (rev=$COMMIT_SHA)..."
sed \
  -e "s|\${REPO_URL}|file://$REPO_ROOT|g" \
  -e "s|\${VERSION}|$VERSION|g" \
  -e "s|\${COMMIT_SHA}|$COMMIT_SHA|g" \
  "$ZED_DIR/extension.dev.toml" > "$ZED_DIR/extension.toml"

echo "Done. Use 'zed: install dev extension' pointing to editors/zed/."
