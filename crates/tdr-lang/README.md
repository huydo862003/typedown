# tdr-lang

## Formatter vs Linter

We follow [Google's markdown style guide](https://google.github.io/styleguide/docguide/style.html) with some divergences noted below.

These are separate concerns:

- **Formatter**: Rewrites the markdown body to match style rules. Deterministic, idempotent, auto-applied. Handles whitespace, spacing, indentation. Running it twice produces the same output.
- **Linter**: Reports problems as diagnostics. Some are auto-fixable, some are not. Handles correctness and style rules that require human judgment.

## Formatter Rules

The formatter is a pretty-printer that operates on the AST of the markdown body. Frontmatter is passed through verbatim.

| Rule                  | Description                                          |
| --------------------- | ---------------------------------------------------- |
| Heading spacing       | Exactly one space after `#` symbols                  |
| Heading blank lines   | Exactly one blank line before and after headings     |
| Trailing whitespace   | Remove trailing whitespace on all lines              |
| Blank line collapsing | Collapse multiple consecutive blank lines to one     |
| Nested list indent    | 2-space indentation (diverges from Google's 4-space) |
| Code blocks           | Pass through verbatim, no reformatting               |
| Paragraph content     | Pass through verbatim, no reflowing                  |
| Trailing newline      | Ensure file ends with exactly one newline            |

## Divergences from Google

| Rule               | Google   | Typedown | Reason                                           |
| ------------------ | -------- | -------- | ------------------------------------------------ |
| Nested list indent | 4 spaces | 2 spaces | Consistent with the YAML frontmatter indentation |
