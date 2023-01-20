use nom::combinator::opt;
use nom::combinator::{consumed, map};
use nom::sequence::tuple;

use crate::parser::interface::*;
use crate::parser::operator;
use crate::parser::unary::unary;
use crate::parser::whitespace::ws;

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
    fn test_binary_1() {
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
    fn test_binary_assignment() {
        let (_, ast) = parse("0@ = 5").unwrap();
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
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 5,
                        token: Token {
                            start: 6,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );

        let (_, ast) = parse("$var = 5").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 4,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    }))),
                    operator: Token {
                        start: 6,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 5,
                        token: Token {
                            start: 8,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 8,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );

        // &123 = 5
        let (_, ast) = parse("&123 = 5").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Adma(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 3,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 4,
                            syntax_kind: SyntaxKind::AdmaVariable
                        }
                    }))),
                    operator: Token {
                        start: 6,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 5,
                        token: Token {
                            start: 8,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 8,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );

        // $2 = 1 /*comment*/
        let (_, ast) = parse("$2 = 1 /*comment*/").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 2,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    }))),
                    operator: Token {
                        start: 4,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 1,
                        token: Token {
                            start: 6,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 6,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );

        // $2 = 1 // (int)
        let (_, ast) = parse("$2 = 1 // (int)").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Binary(BinaryExpr {
                    left: Box::new(Node::Variable(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 1,
                            len: 2,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    }))),
                    operator: Token {
                        start: 4,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::Int(IntLiteral {
                        value: 1,
                        token: Token {
                            start: 6,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        }
                    }))),
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
    fn test_string_assignment() {
        let (_, ast) = parse("0@v = \"test\"").unwrap();
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
                        _type: VariableType::LongString,
                        token: Token {
                            start: 1,
                            len: 3,
                            syntax_kind: SyntaxKind::LocalVariable,
                        }
                    }))),
                    operator: Token {
                        start: 5,
                        len: 1,
                        syntax_kind: SyntaxKind::OperatorEqual
                    },
                    right: Box::new(Node::Literal(Literal::String(Token {
                        start: 8,
                        len: 4,
                        syntax_kind: SyntaxKind::StringLiteral
                    }))),
                    token: Token {
                        start: 1,
                        len: 12,
                        syntax_kind: SyntaxKind::BinaryExpr
                    }
                })]
            }
        );
    }
}
