use nom::combinator::opt;
use nom::combinator::{consumed, map};
use nom::sequence::tuple;

use crate::parser::helpers::ws;
use crate::parser::interface::*;
use crate::parser::operator;
use crate::parser::unary::unary;

fn map_binary(span: Span, left: Node, op: Option<(Token, Node)>) -> Node {
    match op {
        Some((operator, right)) => Node::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            token: Token::from(span, SyntaxKind::BinaryExpr),
        }),
        _ => left,
    }
}

pub fn assignment(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            equality,
            opt(tuple((ws(operator::assignment), equality))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn equality(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            bitwise,
            opt(tuple((ws(operator::equality), bitwise))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn bitwise(s: Span) -> R<Node> {
    map(
        consumed(tuple((
            comparison,
            opt(tuple((ws(operator::bitwise), comparison))),
        ))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn comparison(s: Span) -> R<Node> {
    map(
        consumed(tuple((term, opt(tuple((ws(operator::comparison), term)))))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn term(s: Span) -> R<Node> {
    map(
        consumed(tuple((factor, opt(tuple((ws(operator::add_sub), factor)))))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

fn factor(s: Span) -> R<Node> {
    map(
        consumed(tuple((unary, opt(tuple((ws(operator::mul_div), unary)))))),
        |(span, (left, op))| map_binary(span, left, op),
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_binary() {
        let (_, ast) = parse("0@ += $_t_e_s_t").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Binary(BinaryExpr {
                    left: Box::new(Node::Token(Token {
                        start: 1,
                        len: 2,
                        // text: String::from("0@"),
                        syntax_kind: SyntaxKind::LocalVariable,
                    })),
                    operator: Token {
                        start: 4,
                        len: 2,
                        // text: String::from("+="),
                        syntax_kind: SyntaxKind::OperatorPlusEqual
                    },
                    right: Box::new(Node::Token(Token {
                        start: 7,
                        len: 9,
                        // text: String::from("$_t_e_s_t"),
                        syntax_kind: SyntaxKind::GlobalVariable
                    })),
                    token: Token {
                        start: 1,
                        len: 15,
                        // text: String::from("0@ += $_t_e_s_t"),
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })
            }
        )
    }
}
