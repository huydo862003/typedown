((inline) @content
  (#set! "language" "typedown_md_inline"))

(fenced_code_block
  (language) @language
  (code_fence_content) @content)

((math_block_content) @content
  (#set! "language" "latex"))
