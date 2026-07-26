(parenthesized_expression "(" @delimiter ")" @delimiter) @container
(list_expression "[" @delimiter "]" @delimiter) @container
(dict_expression "{" @delimiter "}" @delimiter) @container
(index_expression "[" @delimiter "]" @delimiter) @container
(call_expression "(" @delimiter ")" @delimiter) @container
(interpolation "$" @delimiter "}" @delimiter) @container
