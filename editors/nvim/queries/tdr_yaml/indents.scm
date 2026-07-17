(block_mapping_entry) @indent.begin
(block_sequence_entry) @indent.begin
(block_scalar) @indent.begin

(parenthesized_expression "(" @indent.begin ")" @indent.branch)
(list_expression "[" @indent.begin "]" @indent.branch)
(dict_expression "{" @indent.begin "}" @indent.branch)
(fixed_key_dict_type "{" @indent.begin "}" @indent.branch)
