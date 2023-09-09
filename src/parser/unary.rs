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
        alt((
            map(variable::variable, |v| Node::Variable(v)),
            map(literal::number, |n| Node::Literal(n)),
            map(literal::identifier, |n| Node::Literal(n)),
        )),
    ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    // #[test]
    fn test_unary_not() {
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
    fn test_unary_minus() {
        let (_, ast) = parse("-10").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        syntax_kind: SyntaxKind::OperatorMinus,
                        start: 1,
                        len: 1
                    },
                    operand: Box::new(Node::Literal(Token {
                        syntax_kind: SyntaxKind::IntegerLiteral,
                        start: 2,
                        len: 2
                    })),
                    token: Token {
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                        start: 1,
                        len: 3
                    }
                })]
            }
        );

        let (_, ast) = parse("-10.23").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        syntax_kind: SyntaxKind::OperatorMinus,
                        start: 1,
                        len: 1
                    },
                    operand: Box::new(Node::Literal(Token {
                        syntax_kind: SyntaxKind::FloatLiteral,
                        start: 2,
                        len: 5
                    })),
                    token: Token {
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                        start: 1,
                        len: 6
                    }
                })]
            }
        );

        let (_, ast) = parse("-x").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Unary(UnaryPrefixExpr {
                    operator: Token {
                        syntax_kind: SyntaxKind::OperatorMinus,
                        start: 1,
                        len: 1
                    },
                    operand: Box::new(Node::Literal(Token {
                        syntax_kind: SyntaxKind::Identifier,
                        start: 2,
                        len: 1
                    })),
                    token: Token {
                        syntax_kind: SyntaxKind::UnaryPrefixExpr,
                        start: 1,
                        len: 2
                    }
                })]
            }
        );
    }
}
