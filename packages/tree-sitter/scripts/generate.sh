#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/grammars.sh"

for dir in "${GRAMMARS[@]}"; do
  (cd "$dir" && tree-sitter generate)
done
