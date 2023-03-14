(line_comment) @comment.line

(number) @constant.numeric

"fn" @keyword.function
"let" @keyword.storage

[
  ";"
  ","
] @punctuation.delimiter

[
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

[
  "->"
  "="
] @operator

(function_definition
  name: (identifier) @function)

(function_call 
  name: (identifier) @function)

(primitive_type) @type.builtin
(identifier) @variable
