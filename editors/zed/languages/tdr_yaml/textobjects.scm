(block_mapping_entry) @function.around
(block_mapping_entry
  value: (value) @function.inside)

(block_sequence_entry) @function.around

(block_scalar) @function.around
(block_scalar
  (block_scalar_content) @function.inside)

(comment)+ @comment.around
