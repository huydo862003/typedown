((inline) @injection.content
  (#set! injection.language "tdr_md_inline"))

(fenced_code_block
  (language) @injection.language
  (code_fence_content) @injection.content)
