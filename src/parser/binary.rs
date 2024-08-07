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
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 1,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    operator: Token {
                        start: 4,
                        len: 2,
                        syntax_kind: SyntaxKind::OperatorPlusEqual
                    },
                    right: Box::new(Node::Variable(Variable::Global(SingleVariable {
                        name: Token {
                            start: 8,
                            len: 8,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 7,
                            len: 9,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 15,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        )
    }

    #[test]
    fn test_timed_addition() {
        let (_, ast) = parse("0@ +=@ 1@").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 1,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    operator: Token {
                        start: 4,
                        len: 3,
                        syntax_kind: SyntaxKind::OperatorTimedAdditionEqual
                    },
                    right: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 8,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 8,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 9,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        )
    }

    #[test]
    fn test_cast_assignment() {
        let (_, ast) = parse("0@ =# 1@").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 1,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    operator: Token {
                        start: 4,
                        len: 2,
                        syntax_kind: SyntaxKind::OperatorCastEqual
                    },
                    right: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 7,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 7,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 8,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        )
    }

    #[test]
    fn test_const_name() {
        let (_, ast) = parse("x &= 1").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Literal(Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::Identifier
                    })),
                    operator: Token {
                        start: 3,
                        len: 2,
                        syntax_kind: SyntaxKind::OperatorBitwiseAndEqual
                    },
                    right: Box::new(Node::Literal(Token {
                        start: 6,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    })),
                    token: Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );
    }

    #[test]
    fn test_const_name2() {
        let (_, ast) = parse("x &= &7").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Literal(Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::Identifier
                    })),
                    operator: Token {
                        start: 3,
                        len: 2,
                        syntax_kind: SyntaxKind::OperatorBitwiseAndEqual
                    },
                    right: Box::new(Node::Variable(Variable::Adma(SingleVariable {
                        name: Token {
                            syntax_kind: SyntaxKind::IntegerLiteral,
                            start: 7,
                            len: 1
                        },
                        token: Token {
                            syntax_kind: SyntaxKind::AdmaVariable,
                            start: 6,
                            len: 2
                        },
                        _type: VariableType::Unknown
                    }))),
                    token: Token {
                        start: 1,
                        len: 7,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );
    }
}
