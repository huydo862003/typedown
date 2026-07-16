; Keys
(property_key) @property
(reserved_key) @keyword

; Type values
(type_value "!type" @keyword)

; Values
(number) @number
(boolean) @boolean
(string) @string
(escape_sequence) @string.escape

; Types
(primitive_type) @type
(list_type "list" @type)
(dict_type "dict" @type)
(fixed_key_dict_type) @type
(union_type) @type
(fixed_key_entry key: (identifier) @property)

; Expressions
(identifier) @variable
(self_expression) @variable.special
(fref "fref" @function)
(fref) @function
(tag_operator) @operator
(access_expression "." @punctuation.delimiter)
(dict_entry key: (identifier) @property)

; Binary operators
(binary_expression "+" @operator)
(binary_expression "-" @operator)
(binary_expression "*" @operator)
(binary_expression "/" @operator)
(binary_expression "%" @operator)
(binary_expression "**" @operator)
(binary_expression "==" @operator)
(binary_expression "!=" @operator)
(binary_expression "<" @operator)
(binary_expression ">" @operator)
(binary_expression "<=" @operator)
(binary_expression ">=" @operator)
(binary_expression "||" @keyword)
(binary_expression "&&" @keyword)

; Unary operators
(unary_expression "~" @operator)

; Function calls
(call_expression (identifier) @function)
(call_expression (access_expression (identifier) @function))

; Interpolation
(interpolation "${" @punctuation.special)
(interpolation "}" @punctuation.special)

; Comments
(comment) @comment

; Punctuation
":" @punctuation.delimiter
"-" @punctuation.delimiter
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"|" @punctuation.delimiter
