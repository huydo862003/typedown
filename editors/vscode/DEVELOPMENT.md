# Development

## Requirements

- [Nix](https://nixos.org) with flakes enabled (provides Node.js, pnpm, cargo-watch, and Rust nightly)

## Setup

```bash
nix develop
pnpm install
```

## Testing Against a Local Build

Build the Rust LSP binary and compile the extension, then launch the Extension Development Host:

```bash
pnpm run compile:dev
```

Then press `F5` in VSCode (or use the "Run Extension (local dev)" launch configuration).

`compile:dev` builds `typedown-lsp` from source and copies the debug binary into `bin/` before bundling the extension.

To watch for changes across both Rust and TypeScript:

```bash
pnpm run watch:dev
```

This runs `cargo watch` on the Rust crates and esbuild/tsc in parallel. Relaunch the Extension Development Host after each Rust rebuild.

## Testing a Staging Release

1. Push a staging tag via `./publish.sh` from the repo root (choose a `pre*` bump type).
   CI builds and uploads the prerelease binaries automatically.

2. Download the staging binary matching the current version and compile:

   ```bash
   pnpm run compile:staging
   ```

   Then press `F5` (or use the "Run Extension (staging binary)" launch configuration).

   `fetch:staging` reads the version from `VERSION` at the repo root, constructs the `staging/vX.Y.Z-label.N` GitHub release URL, and downloads the matching binary into `bin/`.

## Lint and Type Check

```bash
pnpm run lint
pnpm run check-types
```

## Release

Releases are handled by `publish.sh` from the repo root. It bumps the version in `VERSION` alongside all other packages and pushes the tag that triggers CI to build and publish the VSIX.

## Authoring Textmate Grammar

In this section, I will document my personal experiences with Textmate grammar, maybe some pitfalls or high-level concepts to know.

### High-level structure

Top-level fields:

```json
{
  "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",

  "name": "Typedown",

  "scopeName": "source.tdr",

  "patterns": [
    {
      "include": "#frontmatter"
    }
  ],
  "repository": {
    "frontmatter": {
      "comment": "YAML mode between --- delimiters",
      "begin": "\\A(---)",
      "end": "^(---)",
      "beginCaptures": {
        "1": {
          "name": "punctuation.definition.frontmatter.begin.tdr"
        }
      },
      "endCaptures": {
        "1": {
          "name": "punctuation.definition.frontmatter.end.tdr"
        }
      },
      "patterns": []
    }
  }
}
```

- `$schema`: Normal JSON schema declaration.
- `name`: The name of the Textmate grammar, no functional effects.
- `scopeName: "source.tdr"`: The unique identifier for your grammar.
  1. How themes target your language: Rules can match broadly (`source`) or specifically (`source.tdr`)
  2. How other grammars embed yours: They reference `source.tdr` to include your grammar
  3. Scope hierarchy: Every token in your file inherits source.tdr as its root, and nested scopes build on top (e.g, `source.tdr > string.quoted.double.tdr`).

     Example: `string` would match `string.quoted`, but `quoted` would not match `string.quoted`. When both `string` and `string.quoted` match, the more specific one (`string.quoted`) wins, similar to CSS specificity.

  > The `.` is not a file part separator, it creates namespace prefixes, like folder hierarchy in `source/tdr`.

  > Initially, I thought this is a regex for filenames `*.tdr`.

- `patterns`: An array of rules applied to the file content.
  - Each rule is a regex that matches a region of text and assigns scope names to it.
  - Rules can nest: a matched region can have its own inner `patterns`, forming a tree. Inner rules see the raw text within the region and match against it independently. See [Rules](#rules) below.
  - Rules in `repository` are only applied when referenced from here (or from other active rules) via `include`.
  - Unmatched text (text that no pattern matches) stays in the parent scope. At the top level, that's `source.tdr`. No error occurs.
- `repository`: A dictionary of named rules referenced via `{ "include": "#rule-name" }` from `patterns` or other rules. Rules here are not applied unless included. Recursive includes are valid and useful for nested constructs (e.g., nested comments, nested brackets). The engine handles this by trying the rule at each position without infinite looping, since each match must consume text to advance.

### Rules

Each rule in `patterns` or `repository` can be one of three forms:

1. **Include**: References another rule. Three forms:
   - `{ "include": "#rule-name" }`: A rule from the `repository`.
   - `{ "include": "$self" }`: The entire current grammar (useful for recursive structures like nested brackets).
   - `{ "include": "source.js" }`: Another grammar by `scopeName` (for embedding languages).

   Recursive includes are valid (a rule can include itself). But only use them inside `begin`/`end` rules, not `match` rules, since each `begin` must consume text to advance. A `match` rule including itself would loop infinitely.

2. **Match**: A single-line regex match. The TextMate engine feeds one line at a time to the regex, so `match` can never span multiple lines. Multi-line constructs need `begin`/`end` instead.
   - `match`: Regex to match against.
   - `name`: Scope assigned to the entire matched text.
   - `captures`: Scopes assigned to individual capture groups. These are applied on top of `name`. For example, with the rule below, `# my comment` produces:
     - `#` has scopes: `comment.line.number-sign.tdr`, `punctuation.definition.comment.tdr` (targeted by either)
     - ` my comment` has scope: `comment.line.number-sign.tdr` only (targeted by `comment` but not `punctuation`)

   ```json
   {
     "match": "(#).*$",
     "name": "comment.line.number-sign.tdr",
     "captures": {
       "1": { "name": "punctuation.definition.comment.tdr" }
     }
   }
   ```

3. **Begin/End**: A multi-line region.
   - `begin`: Regex that opens the region.
   - `end`: Regex that closes the region.
   - `name`: Scope assigned to the entire region (delimiters + content).
   - `contentName`: Scope assigned to the content only (excludes delimiters).
   - `beginCaptures`/`endCaptures`: Scopes assigned to capture groups in `begin`/`end`.
   - `patterns`: Rules applied to the content between `begin` and `end`.

   ```json
   {
     "begin": "\"",
     "end": "\"",
     "name": "string.quoted.double.tdr",
     "patterns": [{ "include": "#escape" }, { "include": "#interpolation" }]
   }
   ```

### Scope naming conventions

Scope names are free-form strings. Technically, you can assign any name to any part of the document.

However, conventions exist for two reasons:

1. **Theme reuse**: A minimal theme only styles ~10 root groups. Using conventional names means your grammar gets colored by any theme without language-specific rules.
2. **Preference reuse**: VS Code has built-in behaviors tied to scope names. For example, inside any `comment` or `string` scope, it won't auto-pair `'` (since you're typing text, not code). If you use conventional names, you get these behaviors for free. If you use custom names like `my-custom-comment`, VS Code won't know it's a comment.

Suggestions:

- **Spread out across root groups**: Don't put everything under `keyword` just because your formal spec calls them keywords. Ask "would I want these two elements styled differently?". If yes, use different root groups.
- **Reuse existing sub-types**:
  - Within a group, use the established sub-names (e.g., `storage.modifier`, `storage.type`) rather than inventing new ones.
  - But append extra info: `storage.modifier.static.tdr` instead of just `storage.modifier`.
- **Put the language name last**:
  - Example: `string.quoted.double.tdr`, not `tdr.string.quoted.double`.
  - This matters for embedded languages where `source.tdr` context may not be available in selectors.

The 11 root groups:

| Group      | Purpose                            | Common sub-types                                           |
| ---------- | ---------------------------------- | ---------------------------------------------------------- |
| `comment`  | Comments                           | `line.number-sign`, `block`, `block.documentation`         |
| `constant` | Constants                          | `numeric`, `character.escape`, `language` (`true`/`false`) |
| `entity`   | Named parts of larger constructs   | `name.function`, `name.type`, `name.tag`, `name.section`   |
| `invalid`  | Invalid constructs                 | `illegal`, `deprecated`                                    |
| `keyword`  | Keywords                           | `control`, `operator`, `other`                             |
| `markup`   | Markup constructs                  | `heading`, `bold`, `italic`, `list`, `quote`, `raw`        |
| `meta`     | Structural regions (rarely styled) | `meta.function`, `meta.class`                              |
| `storage`  | Storage-related                    | `type` (`class`/`var`), `modifier` (`static`/`final`)      |
| `string`   | Strings                            | `quoted.single`, `quoted.double`, `interpolated`, `regexp` |
| `support`  | Framework/library-provided         | `function`, `class`, `type`, `constant`, `variable`        |
| `variable` | Variables                          | `parameter`, `language` (`self`/`this`/`super`), `other`   |

Full reference: https://macromates.com/manual/en/language_grammars#naming_conventions

### Embedded language highlighting

Fenced code blocks (` ```js ... ``` `) can be highlighted using the embedded language's grammar via **grammar injections**. This is how VS Code's built-in markdown does it.

Injections live in your own extension. They reference grammars provided by other extensions (e.g., `source.js` from VS Code's built-in JavaScript support) without modifying them.

The `injectionSelector` controls priority: `L:` means the injection is tried **before** the host grammar's patterns, `R:` (or no prefix) means **after**. Use `L:` for embedded code blocks so the injection matches before the host tries to parse it as regular content.

Two pieces are needed:

1. An injection grammar file (e.g., `syntaxes/js-codeblock.json`):

   ````json
   {
     "scopeName": "tdr.js.codeblock",
     "injectionSelector": "L:source.tdr",
     "patterns": [
       {
         "begin": "(^```)(js|javascript)\\s*$",
         "end": "^(```)\\s*$",
         "contentName": "meta.embedded.block.javascript",
         "patterns": [{ "include": "source.js" }]
       }
     ]
   }
   ````

2. A registration in `package.json`:

   ```json
   {
     "grammars": [
       {
         "scopeName": "tdr.js.codeblock",
         "path": "./syntaxes/js-codeblock.json",
         "injectTo": ["source.tdr"],
         "embeddedLanguages": {
           "meta.embedded.block.javascript": "javascript"
         }
       }
     ]
   }
   ```

This is one injection per language. The [vscode-markdown-tm-grammar](https://github.com/microsoft/vscode-markdown-tm-grammar) repo auto-generates these for ~50 languages.

Reference: https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide

### Regex

TextMate uses the [Oniguruma](https://macromates.com/manual/en/regular_expressions) regex engine (not JavaScript regex). Most syntax is the same, but Oniguruma has extra anchors. An anchor matches a **position** in the text (like "start of file") without consuming any characters:

| Anchor | Meaning                                      |
| ------ | -------------------------------------------- |
| `^`    | Start of a line                              |
| `$`    | End of a line                                |
| `\A`   | Start of the entire file (position 0 only)   |
| `\z`   | End of the entire file                       |
| `\Z`   | End of file, allowing optional trailing `\n` |

`^` and `$` match at every line boundary. `\A` and `\z` only match at the absolute start/end of the file, regardless of line boundaries.

For example, `\\A(---)` in the frontmatter rule ensures the `---` must be the very first thing in the file, not just any line starting with `---`.
