use nom::branch::alt;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::sequence::tuple;

use crate::parser::interface::*;
use crate::parser::literal;
use crate::parser::operator;
use crate::parser::variable;

pub fn unary(s: Span) -> R<Node> {
    alt((
        map(
            consumed(tuple((operator::unary, unary))),
            |(span, (operator, right))| {
                Node::Unary(UnaryPrefixExpr {
                    operator,
                    operand: Box::new(right),
                    token: Token::from(span, SyntaxKind::UnaryPrefixExpr),
                })
            },
        ),
        map(alt((variable::variable, literal::number)), |token| {
            Node::Token(token)
        }),
    ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    #[test]
    fn test_unary() {
        let (_, ast) = parse("  ~1@  ").unwrap();

        assert_eq!(
            ast,
            AST {
                node: Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        start: 3,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorBitwiseNot,
                        // text: String::from("~")
                    },
                    operand: Box::new(Node::Token(Token {
                        start: 4,
                        len: 2,
                        // text: String::from("1@"),
                        syntax_kind: SyntaxKind::LocalVariable,
                    })),
                    token: Token {
                        // text: String::from("~1@"),
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                })
            }
        )
    }
}
