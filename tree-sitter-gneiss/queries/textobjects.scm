(line_comment) @comment.inside
(line_comment)+ @comment.around

(arguments
  ((_) @parameter.inside . ","? @parameter.around) @parameter.around)

(function_definition
  body: (_) @function.inside) @function.around
