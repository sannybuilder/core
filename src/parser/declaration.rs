use crate::parser::interface::*;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{one_of, space0, space1};
use nom::combinator::{map, opt};
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::{branch::alt, combinator::consumed};
use nom::{character::complete::multispace0, sequence::terminated};

use crate::parser::expression;
use crate::parser::helpers;
use crate::parser::literal;
use crate::parser::statement;

pub fn declaration(s: Span) -> R<Node> {
    terminated(alt((statement::statement, const_declaration)), multispace0)(s)
}

pub fn function_signature(s: Span) -> R<FunctionSignature> {
    map(
        consumed(helpers::line(tuple((
            helpers::ws(terminated(tag_no_case("function"), space1)),
            helpers::ws(literal::identifier),
            helpers::ws(opt(function_cc)),
            function_arguments_and_return_types,
        )))),
        |(span, (_, name, cc, (parameters, return_types)))| {
            let (cc, address) = cc.unwrap_or((FunctionCC::Local, None));
            FunctionSignature {
                name,
                parameters,
                return_types,
                cc,
                address,
                token: Token::from(span, SyntaxKind::FunctionSignature),
            }
        },
    )(s)
}

pub fn function_arguments_and_return_types(
    s: Span,
) -> R<(Vec<FunctionParameter>, Vec<FunctionReturnType>)> {
    map(
        tuple((
            helpers::ws(opt(function_arguments)),
            helpers::ws(opt(function_return_types)),
        )),
        |(parameters, return_types)| {
            (
                parameters.unwrap_or_default(),
                return_types.unwrap_or_default(),
            )
        },
    )(s)
}

fn function_arguments(s: Span) -> R<Vec<FunctionParameter>> {
    map(
        delimited(
            helpers::ws(tag("(")),
            separated_list0(
                helpers::ws(tag(",")),
                consumed(tuple((
                    opt(delimited(
                        helpers::ws(opt(tag("..."))),
                        // param names are optional in define function
                        helpers::ws(literal::identifier),
                        helpers::ws(tag(":")),
                    )),
                    helpers::ws(literal::identifier),
                    opt(delimited(
                        helpers::ws(tag("[")),
                        helpers::ws(literal::number),
                        helpers::ws(tag("]")),
                    )),
                ))),
            ),
            helpers::ws(tag(")")),
        ),
        |args| {
            args.into_iter()
                .map(|(span, (name, _type, size))| FunctionParameter {
                    name,
                    _type,
                    size,
                    token: Token::from(span, SyntaxKind::LocalVariable),
                })
                .collect()
        },
    )(s)
}

fn function_return_types(s: Span) -> R<Vec<FunctionReturnType>> {
    map(
        tuple((
            helpers::ws(tag(":")),
            opt(tag_no_case("optional")),
            separated_list1(helpers::ws(tag(",")), helpers::ws(literal::identifier)),
        )),
        |(_, _, types)| {
            types
                .into_iter()
                .map(|_type| FunctionReturnType {
                    token: _type.clone(),
                    _type,
                })
                .collect()
        },
    )(s)
}

fn function_cc(s: Span) -> R<(FunctionCC, /*optional address*/ Option<Token>)> {
    delimited(
        tag("<"),
        helpers::ws(tuple((
            alt((
                map(tag_no_case("thiscall"), |_| FunctionCC::Thiscall),
                map(tag_no_case("cdecl"), |_| FunctionCC::Cdecl),
                map(tag_no_case("stdcall"), |_| FunctionCC::Stdcall),
            )),
            opt(preceded(
                helpers::ws(tag(",")),
                helpers::ws(literal::number),
            )),
        ))),
        tag(">"),
    )(s)
}

pub fn const_declaration(s: Span) -> R<Node> {
    map(
        consumed(delimited(
            helpers::line(tag_no_case("const")),
            map(
                many1(helpers::line(consumed(separated_pair(
                    literal::identifier,
                    helpers::ws(tag("=")),
                    expression::expression,
                )))),
                |v: Vec<(Span, (Token, Node))>| {
                    v.into_iter()
                        .map(|(span, (name, value))| ConstInitialization {
                            name,
                            value: Box::new(value),
                            token: Token::from(span, SyntaxKind::ConstInitialization),
                        })
                        .collect()
                },
            ),
            helpers::line(tag_no_case("end")),
        )),
        |(span, items)| {
            Node::ConstDeclaration(ConstDeclaration {
                items,
                token: Token::from(span, SyntaxKind::ConstDeclaration),
            })
        },
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_const_declaration() {
        let (_, node) = const_declaration(Span::from(
            r#"
const
    x = 1
    y = 2.0
end"#,
        ))
        .unwrap();
        assert_eq!(
            node,
            Node::ConstDeclaration(ConstDeclaration {
                items: vec![
                    ConstInitialization {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        value: Box::new(Node::Literal(Token {
                            start: 9,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        })),
                        token: Token {
                            start: 5,
                            len: 5,
                            syntax_kind: SyntaxKind::ConstInitialization
                        }
                    },
                    ConstInitialization {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        value: Box::new(Node::Literal(Token {
                            start: 9,
                            len: 3,
                            syntax_kind: SyntaxKind::FloatLiteral
                        })),
                        token: Token {
                            start: 5,
                            len: 7,
                            syntax_kind: SyntaxKind::ConstInitialization
                        }
                    }
                ],
                token: Token {
                    start: 1,
                    len: 32,
                    syntax_kind: SyntaxKind::ConstDeclaration
                }
            })
        )
    }

    #[test]
    fn test_function_signature() {
        let (_, node) = function_signature(Span::from(r#" function foo "#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 11,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 14,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo: string"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![FunctionReturnType {
                    token: Token {
                        start: 15,
                        len: 6,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: Token {
                        start: 15,
                        len: 6,
                        syntax_kind: SyntaxKind::Identifier
                    }
                }],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 20,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo()"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 14,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo(): int"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![FunctionReturnType {
                    token: Token {
                        start: 17,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: Token {
                        start: 17,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    }
                }],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 19,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo(a: int): int"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![FunctionParameter {
                    name: Some(Token {
                        start: 14,
                        len: 1,
                        syntax_kind: SyntaxKind::Identifier
                    }),
                    _type: Token {
                        start: 17,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    size: None,
                    token: Token {
                        start: 14,
                        len: 6,
                        syntax_kind: SyntaxKind::LocalVariable
                    }
                }],
                return_types: vec![FunctionReturnType {
                    token: Token {
                        start: 23,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: Token {
                        start: 23,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    }
                }],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 25,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) =
            function_signature(Span::from(r#"function foo(a: int, b: string): int"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![
                    FunctionParameter {
                        name: Some(Token {
                            start: 14,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        }),
                        _type: Token {
                            start: 17,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        size: None,
                        token: Token {
                            start: 14,
                            len: 6,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    },
                    FunctionParameter {
                        name: Some(Token {
                            start: 22,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        }),
                        _type: Token {
                            start: 25,
                            len: 6,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        size: None,
                        token: Token {
                            start: 22,
                            len: 9,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    }
                ],
                return_types: vec![FunctionReturnType {
                    token: Token {
                        start: 34,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: Token {
                        start: 34,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    }
                }],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 36,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(
            r#"function foo(a: int, b: string): int, int, int"#,
        ))
        .unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![
                    FunctionParameter {
                        name: Some(Token {
                            start: 14,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        }),
                        _type: Token {
                            start: 17,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        size: None,
                        token: Token {
                            start: 14,
                            len: 6,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    },
                    FunctionParameter {
                        name: Some(Token {
                            start: 22,
                            len: 1,
                            syntax_kind: SyntaxKind::Identifier
                        }),
                        _type: Token {
                            start: 25,
                            len: 6,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        size: None,
                        token: Token {
                            start: 22,
                            len: 9,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    }
                ],
                return_types: vec![
                    FunctionReturnType {
                        token: Token {
                            start: 34,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: Token {
                            start: 34,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        }
                    },
                    FunctionReturnType {
                        token: Token {
                            start: 39,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: Token {
                            start: 39,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        }
                    },
                    FunctionReturnType {
                        token: Token {
                            start: 44,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: Token {
                            start: 44,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        }
                    }
                ],
                cc: FunctionCC::Local,
                address: None,
                token: Token {
                    start: 1,
                    len: 46,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo<stdcall>()"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![],
                cc: FunctionCC::Stdcall,
                address: None,
                token: Token {
                    start: 1,
                    len: 23,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );

        let (_, node) = function_signature(Span::from(r#"function foo<cdecl, 0x135>()"#)).unwrap();

        assert_eq!(
            node,
            FunctionSignature {
                name: Token {
                    start: 10,
                    len: 3,
                    syntax_kind: SyntaxKind::Identifier
                },
                parameters: vec![],
                return_types: vec![],
                cc: FunctionCC::Cdecl,
                address: Some(Token {
                    start: 21,
                    len: 5,
                    syntax_kind: SyntaxKind::IntegerLiteral
                }),
                token: Token {
                    start: 1,
                    len: 28,
                    syntax_kind: SyntaxKind::FunctionSignature
                }
            }
        );
    }
}
