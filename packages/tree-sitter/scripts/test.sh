#!/usr/bin/env bash
set -euo pipefail

GRAMMARS=(. tdr-yaml tdr-md tdr-md-inline)

for dir in "${GRAMMARS[@]}"; do
  (cd "$dir" && tree-sitter test)
done
