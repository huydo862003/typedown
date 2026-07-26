; Mapping entries
(block_mapping_entry) @block.outer
(block_mapping_entry
  value: (value) @block.inner)

; Sequence entries
(block_sequence_entry) @block.outer

; Block scalars
(block_scalar) @block.outer
(block_scalar
  (block_scalar_content) @block.inner)

; Comments
(comment)+ @comment.outer
