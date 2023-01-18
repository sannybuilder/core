use nom::branch::alt;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::sequence::tuple;

use crate::parser::interface::*;
use crate::parser::literal;
use crate::parser::operator;
use crate::parser::string;
use crate::parser::variable;

use super::binary;

pub fn unary(s: Span) -> R<Node> {
    alt((
        map(
            consumed(tuple((operator::op_minus, literal::number))),
            |(span, (operator, right))| {
                Node::Unary(UnaryPrefixExpr::new(
                    operator,
                    Box::new(Node::Literal(right)),
                    Token::from(span, SyntaxKind::UnaryPrefixExpr),
                ))
            },
        ),
        map(
            consumed(tuple((operator::op_bitwise_not, variable::variable))),
            |(span, (operator, right))| {
                Node::Unary(UnaryPrefixExpr::new(
                    operator,
                    Box::new(Node::Variable(right)),
                    Token::from(span, SyntaxKind::UnaryPrefixExpr),
                ))
            },
        ),
        map(
            consumed(tuple((operator::op_not, binary::assignment))),
            |(span, (operator, right))| {
                Node::Unary(UnaryPrefixExpr::new(
                    operator,
                    Box::new(right),
                    Token::from(span, SyntaxKind::UnaryPrefixExpr),
                ))
            },
        ),
        map(variable::variable, |v| Node::Variable(v)),
        map(literal::number, |n| Node::Literal(n)),
        map(string::string, |n| Node::Literal(n)),
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
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 3,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorBitwiseNot,
                    },
                    Box::new(Node::Variable(Variable::Local(SingleVariable {
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
                    Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );

        // not 0@
        let (_, ast) = parse("not 0@").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 1,
                        len: 3,
                        syntax_kind: SyntaxKind::OperatorNot,
                    },
                    Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral,
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 5,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );

        // not 0@ <= 1
        let (_, ast) = parse("not 0@ <= 1").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 1,
                        len: 3,
                        syntax_kind: SyntaxKind::OperatorNot,
                    },
                    Box::new(Node::Binary(BinaryExpr {
                        left: Box::new(Node::Variable(Variable::Local(SingleVariable {
                            name: Token {
                                start: 5,
                                len: 1,
                                syntax_kind: SyntaxKind::IntegerLiteral,
                            },
                            _type: VariableType::Unknown,
                            token: Token {
                                start: 5,
                                len: 2,
                                syntax_kind: SyntaxKind::LocalVariable,
                            }
                        }))),
                        operator: Token {
                            start: 8,
                            len: 2,
                            syntax_kind: SyntaxKind::OperatorLessEqual,
                        },
                        right: Box::new(Node::Literal(Token {
                            start: 11,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral,
                        })),
                        token: Token {
                            start: 5,
                            len: 7,
                            syntax_kind: SyntaxKind::BinaryExpr,
                        }
                    })),
                    Token {
                        start: 1,
                        len: 11,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );
    }

    #[test]
    fn test_negative_numbers() {
        let (_, ast) = parse(" -1.0 ").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 2,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    Box::new(Node::Literal(Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::FloatLiteral,
                    })),
                    Token {
                        start: 2,
                        len: 4,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );

        let (_, ast) = parse("-10234").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    Box::new(Node::Literal(Token {
                        start: 2,
                        len: 5,
                        syntax_kind: SyntaxKind::IntegerLiteral,
                    })),
                    Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );

        let (_, ast) = parse("-0x123").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr::new(
                    Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorMinus,
                    },
                    Box::new(Node::Literal(Token {
                        start: 4,
                        len: 3,
                        syntax_kind: SyntaxKind::HexadecimalLiteral,
                    })),
                    Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                    },
                ))]
            }
        );
    }
}
