; highlights.scm — Typedown
;
; FALLBACK LAYER only. Authoritative styling lives in
; `editors/zed/semantic-token-rules.json` once the user opts in via
; `"languages": { "Typedown": { "semantic_tokens": "combined" } }`.
;
; Capture names below mirror the `style` entries in the rules file so
; themes paint the same construct the same color in either mode.

; =========================================================================
; Frontmatter delimiters (function as structural markers; not tokens)
; =========================================================================

"---" @punctuation.delimiter

; =========================================================================
; Typedown `_type:` and `type:` references
; =========================================================================

(type_reference) @type

; =========================================================================
; Built-in primitive types inside schema definitions
; =========================================================================

[
  "string"
  "integer"
  "number"
  "date"
  "boolean"
] @type.builtin

; =========================================================================
; Enum members
; =========================================================================

(enum_member) @constant

; =========================================================================
; Optional / required flag
; =========================================================================

[
  "optional"
  "required"
] @keyword

; =========================================================================
; Function references (file refs, embeds, etc.)
; =========================================================================

(function_reference
  function_name: (identifier) @function.builtin)

(function_reference
  (arguments (string) @string.path))

; =========================================================================
; Inline formulas
; =========================================================================

(formula
  "${" @punctuation.bracket
  "}"   @punctuation.bracket)

; =========================================================================
; Markdown body — Typedown-specific overlays
; =========================================================================

(task_list_item
  (task_list_item_marker
    "[" @punctuation.bracket
    "]" @punctuation.bracket
    (_)? @constant.builtin))

(pipe_table_delimiter_row) @punctuation.delimiter

(task_list_unchecked) @punctuation.special

; =========================================================================
; Mirror of LSP semantic-token behavior so semantic_tokens: "off" users
; see the same construct family themes would paint anyway.
; =========================================================================

(type_identifier)  @type
(const_identifier) @constant
(strong_emphasis)  @emphasis @type
(emphasis)         @emphasis @variable
(strikethrough)    @emphasis @comment

; Number, operator, comment mirrors for the markdown body. Whichever layer
; covers these wins in tree-sitter mode; semantic-token mode overrides.
(node: (MdNumber) @number)
(MdTableSeparatorRow_start: "--" @operator)
(node: (MdStrikethrough) @emphasis @strikethrough)
