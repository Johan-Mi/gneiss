#![forbid(clippy::unwrap_used)] // Syntax errors should never cause crashes.

use crate::{text::node_text, typ::Type};
use internment::Intern;
use ropey::Rope;
use tree_sitter::{Node, Tree};

// TODO: Incremental reparsing

#[derive(Debug)]
pub struct SyntaxError;

type Result<T> = std::result::Result<T, SyntaxError>;

#[derive(Debug)]
pub struct File {
    functions: Vec<Function>,
}

impl File {
    pub fn parse(tree: &Tree, text: &Rope) -> Self {
        Self {
            functions: tree
                .root_node()
                .children(&mut tree.walk())
                .filter(|child| !child.is_extra())
                .map(|child| Function::parse(child, text))
                .collect(),
        }
    }
}

#[derive(Debug)]
struct Function {
    signature: FunctionSignature,
    body: Result<Block>,
}

impl Function {
    fn parse(node: Node, text: &Rope) -> Self {
        let name = node
            .child_by_field_name("name")
            .filter(|node| node.kind() == "identifier")
            .map(|node| (&*node_text(node, text)).into())
            .ok_or(SyntaxError);
        let parameters = node
            .child_by_field_name("parameters")
            .map(FunctionParameters::parse)
            .ok_or(SyntaxError);
        let return_type = node
            .child_by_field_name("return_type")
            .ok_or(SyntaxError)
            .and_then(|node| Type::parse(node, text));
        let body = node
            .child_by_field_name("body")
            .ok_or(SyntaxError)
            .and_then(|node| Block::parse(node, text));
        Self {
            signature: FunctionSignature {
                name,
                parameters,
                return_type,
            },
            body,
        }
    }
}

#[derive(Debug)]
struct FunctionSignature {
    name: Result<Intern<str>>,
    parameters: Result<FunctionParameters>,
    return_type: Result<Type>,
}

#[derive(Debug)]
struct FunctionParameters(Vec<(Expr, Type)>);

impl FunctionParameters {
    fn parse(node: Node) -> Self {
        // TODO
        Self(Vec::new())
    }
}

#[derive(Debug)]
struct Block {
    statements: Vec<Result<Statement>>,
    result: Option<Result<Box<Expr>>>,
}

impl Block {
    fn parse(node: Node, text: &Rope) -> Result<Self> {
        if node.kind() != "block" {
            return Err(SyntaxError);
        }
        let result_node = node.child_by_field_name("result");
        Ok(Self {
            statements: node
                .named_children(&mut node.walk())
                .filter(|child| {
                    !child.is_extra()
                        && Some(child.id())
                            != result_node.as_ref().map(Node::id)
                })
                .map(|node| Statement::parse(node, text))
                .collect(),
            result: result_node
                .map(|node| Expr::parse(node, text).map(Box::new)),
        })
    }
}

#[derive(Debug)]
enum Statement {
    Expr(Result<Expr>),
    Let {
        pattern: Result<Expr>,
        value: Result<Expr>,
    },
}

impl Statement {
    fn parse(node: Node, text: &Rope) -> Result<Self> {
        match node.kind() {
            "expression_statement" => node
                .child(0)
                .map(|node| Expr::parse(node, text))
                .map(Self::Expr)
                .ok_or(SyntaxError),
            "let_declaration" => Ok(Self::Let {
                pattern: node
                    .child_by_field_name("pattern")
                    .ok_or(SyntaxError)
                    .and_then(|node| Expr::parse(node, text)),
                value: node
                    .child_by_field_name("value")
                    .ok_or(SyntaxError)
                    .and_then(|node| Expr::parse(node, text)),
            }),
            _ => Err(SyntaxError),
        }
    }
}

#[derive(Debug)]
enum Expr {
    Block(Block),
    Identifier(Intern<str>),
    FunctionCall {
        name: Result<Intern<str>>,
        arguments: Result<FunctionArguments>,
    },
    IntLiteral(IntLiteral),
}

impl Expr {
    fn parse(node: Node, text: &Rope) -> Result<Self> {
        match node.kind() {
            "identifier" => parse_identifier(node, text).map(Self::Identifier),
            "block" => Block::parse(node, text).map(Self::Block),
            "function_call" => Ok(Self::FunctionCall {
                name: node
                    .child_by_field_name("name")
                    .ok_or(SyntaxError)
                    .and_then(|node| parse_identifier(node, text)),
                arguments: node
                    .child_by_field_name("arguments")
                    .map(|node| FunctionArguments::parse(node, text))
                    .ok_or(SyntaxError),
            }),
            "number" => IntLiteral::parse(node, text).map(Self::IntLiteral),
            _ => Err(SyntaxError),
        }
    }
}

#[derive(Debug)]
struct FunctionArguments(Vec<Result<Expr>>);

impl FunctionArguments {
    fn parse(node: Node, text: &Rope) -> Self {
        Self(
            node.named_children(&mut node.walk())
                .filter(|child| child.is_extra() == child.is_error())
                .map(|node| {
                    if node.is_error() {
                        Err(SyntaxError)
                    } else {
                        Expr::parse(node, text)
                    }
                })
                .collect(),
        )
    }
}

#[derive(Debug)]
enum IntLiteral {
    U8(Result<u8>),
    U16(Result<u16>),
    U32(Result<u32>),
    U64(Result<u64>),
    I8(Result<i8>),
    I16(Result<i16>),
    I32(Result<i32>),
    I64(Result<i64>),
}

impl IntLiteral {
    fn parse(node: Node, text: &Rope) -> Result<Self> {
        if node.kind() != "number" {
            return Err(SyntaxError);
        }
        let text = &*node_text(node, text);
        let (digits_with_separators, typ) = text
            .rsplit_once('_')
            .expect("integer literal missing underscore before type suffix");

        let digits_without_separators;
        let digits = if digits_with_separators.contains('_') {
            // Most integer literals don't contain any separators, so this
            // avoids some allocations.
            digits_with_separators
        } else {
            digits_without_separators = digits_with_separators.replace('_', "");
            &*digits_without_separators
        };

        Ok(match typ {
            "u8" => Self::U8(digits.parse().map_err(|_| SyntaxError)),
            "u16" => Self::U16(digits.parse().map_err(|_| SyntaxError)),
            "u32" => Self::U32(digits.parse().map_err(|_| SyntaxError)),
            "u64" => Self::U64(digits.parse().map_err(|_| SyntaxError)),
            "i8" => Self::I8(digits.parse().map_err(|_| SyntaxError)),
            "i16" => Self::I16(digits.parse().map_err(|_| SyntaxError)),
            "i32" => Self::I32(digits.parse().map_err(|_| SyntaxError)),
            "i64" => Self::I64(digits.parse().map_err(|_| SyntaxError)),
            _ => panic!("invalid integer literal type suffix"),
        })
    }
}

fn parse_identifier(node: Node, text: &Rope) -> Result<Intern<str>> {
    let identifier = node_text(node, text);
    if identifier.is_empty() {
        Err(SyntaxError)
    } else {
        Ok((&*identifier).into())
    }
}
