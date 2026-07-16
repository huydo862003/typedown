#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/grammars.sh"

MODE="${1:-all}"

ROOT="$PWD"

build_so() {
  mkdir -p dist/tree-sitter-so
  for index in "${!GRAMMARS[@]}"; do
    (cd "${GRAMMARS[$index]}" && tree-sitter build -o "$ROOT/dist/tree-sitter-so/${NAMES[$index]}.so")
  done
}

build_wasm() {
  mkdir -p dist/tree-sitter-wasm
  for index in "${!GRAMMARS[@]}"; do
    (cd "${GRAMMARS[$index]}" && tree-sitter build --wasm -o "$ROOT/dist/tree-sitter-wasm/${NAMES[$index]}.wasm")
  done
}

case "$MODE" in
  so)   build_so ;;
  wasm) build_wasm ;;
  all)  build_so && build_wasm ;;
  *)    echo "Usage: $0 {so|wasm|all}" >&2; exit 1 ;;
esac
