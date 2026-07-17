#!/usr/bin/env bash
# Build the Zed extension WASM and grammar WASMs in-place.
# After running, use `zed: install dev extension` pointing to editors/zed/.
# See: https://github.com/zed-industries/zed/issues/42353
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ZED_DIR="$REPO_ROOT/editors/zed"

RELEASE_FLAG=""
PROFILE="debug"
if [[ "${1:-}" == "--release" ]]; then
  RELEASE_FLAG="--release"
  PROFILE="release"
fi

# Build extension WASM
echo "Building extension WASM ($PROFILE)..."
cargo build -p typedown-zed --target wasm32-wasip2 $RELEASE_FLAG
cp "$REPO_ROOT/target/wasm32-wasip2/$PROFILE/typedown_zed.wasm" "$ZED_DIR/extension.wasm"

# Build grammar WASMs
echo "Building grammar WASMs..."
pnpm --filter tree-sitter-tdr run build wasm

mkdir -p "$ZED_DIR/grammars"
cp "$REPO_ROOT/packages/tree-sitter/dist/tree-sitter-wasm/"*.wasm "$ZED_DIR/grammars/"

echo "Done. Use 'zed: install dev extension' pointing to editors/zed/."
