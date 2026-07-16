#!/usr/bin/env bash
set -euo pipefail

GRAMMARS=(. typedown-yaml typedown-md typedown-md-inline)

for dir in "${GRAMMARS[@]}"; do
  (cd "$dir" && tree-sitter test)
done
