#!/usr/bin/env bash
# Shared grammar definitions for build, generate, and test scripts.
GRAMMARS=(. typedown-yaml typedown-md typedown-md-inline)
NAMES=(typedown typedown_yaml typedown_md typedown_md_inline)
TREE_SITTER="${TREE_SITTER_PATH:-tree-sitter}"
