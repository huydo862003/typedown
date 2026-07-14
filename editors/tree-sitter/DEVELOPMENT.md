# Tree-sitter Grammars for Typedown

## Overview

Typedown uses three tree-sitter grammars connected via language injection:

- **typedown/**: Host grammar. Detects `---` frontmatter delimiters and produces `frontmatter_content` and `body_content` nodes for injection.
- **typedown-yaml/**: Parses TDR-flavored YAML frontmatter (injected into `frontmatter_content`).
- **typedown-markdown/**: Parses TDR-flavored Markdown body (injected into `body_content`).

This follows the same pattern as `tree-sitter-markdown` / `markdown_inline`.

See also: [Tree-sitter research](https://huydo862003.github.io/loupe/research/analysis/syntactic-analysis.html)

## Requirements

- [tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/)
- Node.js (for `grammar.js`)
- C compiler (for the generated parser)

## Build

Generate parsers individually:

```bash
cd typedown && tree-sitter generate
cd typedown-yaml && tree-sitter generate
cd typedown-markdown && tree-sitter generate
```

## Test

Test individually:

```bash
cd typedown && tree-sitter test
cd typedown-yaml && tree-sitter test
cd typedown-markdown && tree-sitter test
```

Tests live in each grammar's `test/corpus/` directory. Each `.txt` file contains test cases:

```
==================
Test name
==================

<input source code>

---

<expected S-expression tree>
```

- The test name is between lines of `=` characters.
- The `---` separator divides the input from the expected output. When the input itself contains `---` (like TDR frontmatter), the **longest** `---` line is used as the divider. So use a longer separator (e.g., `----`) or put a blank line before it.
- The S-expression shows only named nodes (anonymous tokens like `:`, `"` are omitted).
- Field names appear before node names: `callee: (identifier)`.
- Attributes can be added below the test name: `:skip`, `:error` (expects parse errors), `:fail-fast`.

## Try interactively

Parse a file and print the tree:

```bash
cd typedown && tree-sitter parse path/to/file.tdr
```

Open the web playground:

```bash
cd typedown && tree-sitter playground
```
