const comma_separated = rule =>
  optional(seq(rule, repeat(seq(",", rule)), optional(",")));

module.exports = grammar({
  name: "gneiss",

  extras: $ => [/\s/, $.line_comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._statement),

    function_definition: $ =>
      seq(
        "fn",
        field("name", $.identifier),
        field("parameters", $.parameters),
        "->",
        field("return_type", $._type),
        field("body", $.block)
      ),

    parameters: $ => seq("(", ")"),

    _type: $ =>
      choice($.primitive_type, alias($.identifier, $.type_identifier)),

    block: $ =>
      seq(
        "{",
        repeat($._statement),
        field("result", optional($._expression)),
        "}"
      ),

    _statement: $ =>
      choice(
        $.function_definition,
        $.let_declaration,
        $.expression_statement,
        $.empty_statement
      ),

    empty_statement: $ => ";",

    expression_statement: $ =>
      choice(
        seq($._expression_requiring_semicolon, ";"),
        $._expression_not_requiring_semicolon
      ),

    _expression_requiring_semicolon: $ =>
      choice($.function_call, $.identifier, $.number),

    _expression_not_requiring_semicolon: $ => $.block,

    let_declaration: $ =>
      seq(
        "let",
        field("pattern", $._expression),
        "=",
        field("value", $._expression),
        ";"
      ),

    _expression: $ =>
      prec(
        1,
        choice(
          $._expression_requiring_semicolon,
          $._expression_not_requiring_semicolon
        )
      ),

    function_call: $ =>
      seq(field("name", $.identifier), field("arguments", $.arguments)),

    arguments: $ => seq("(", comma_separated($._expression), ")"),

    primitive_type: $ => /[ui](8|16|32|64)|unit/,

    identifier: $ => /@?[\p{XID_Start}_][\p{XID_Continue}-]*/,

    number: $ => /[+-]?\d[\d_]*_[ui](8|16|32|64)/,

    line_comment: $ => /\/\/.*/,
  },
});
