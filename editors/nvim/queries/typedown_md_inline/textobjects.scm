; Emphasis
(emphasis) @block.outer
(strong_emphasis) @block.outer

; Code spans
(code_span) @block.outer
(code_span
  (code_span_content) @block.inner)

; Links
(inline_link) @block.outer
(inline_link
  (link_text) @block.inner)

; Images
(image) @block.outer
(image
  (image_alt) @block.inner)

; Interpolation
(interpolation) @block.outer
