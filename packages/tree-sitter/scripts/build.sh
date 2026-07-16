#!/usr/bin/env bash
set -euo pipefail

GRAMMARS=(. typedown-yaml typedown-md typedown-md-inline)
NAMES=(typedown typedown_yaml typedown_md typedown_md_inline)

MODE="${1:-all}"

build_so() {
  mkdir -p artifacts/tree-sitter-so
  for index in "${!GRAMMARS[@]}"; do
    (cd "${GRAMMARS[$index]}" && tree-sitter build -o "../artifacts/tree-sitter-so/${NAMES[$index]}.so")
  done
}

build_wasm() {
  mkdir -p artifacts/tree-sitter-wasm
  for index in "${!GRAMMARS[@]}"; do
    (cd "${GRAMMARS[$index]}" && tree-sitter build --wasm -o "../artifacts/tree-sitter-wasm/${NAMES[$index]}.wasm")
  done
}

case "$MODE" in
  so)   build_so ;;
  wasm) build_wasm ;;
  all)  build_so && build_wasm ;;
  *)    echo "Usage: $0 {so|wasm|all}" >&2; exit 1 ;;
esac
