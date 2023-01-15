use nom::branch::alt;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::sequence::tuple;

use crate::parser::interface::*;
use crate::parser::literal;
use crate::parser::operator;
use crate::parser::variable;
use crate::parser::string;

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
        alt((
            map(variable::variable, |v| Node::Variable(v)),
            map(literal::number, |n| Node::Literal(n)),
            map(string::string, |n| Node::Literal(n)),
        )),
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
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        start: 3,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorBitwiseNot,
                    },
                    operand: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 4,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral,
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 4,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    token: Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                })]
            }
        )
    }

    #[test]
    fn test_negative_numbers() {
        let (_, ast) = parse(" -1.0 ").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        start: 2,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    operand: Box::new(Node::Literal(Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::FloatLiteral,
                    })),
                    token: Token {
                        start: 2,
                        len: 4,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                })]
            }
        );

        let (_, ast) = parse("-10234").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    operand: Box::new(Node::Literal(Token {
                        start: 2,
                        len: 5,
                        syntax_kind: SyntaxKind::IntegerLiteral,
                    })),
                    token: Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                })]
            }
        );

        let (_, ast) = parse("-0x123").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    operand: Box::new(Node::Literal(Token {
                        start: 4,
                        len: 3,
                        syntax_kind: SyntaxKind::HexadecimalLiteral,
                    })),
                    token: Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                })]
            }
        );
    }
}
