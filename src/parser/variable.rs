use nom::branch::alt;
use nom::character::complete::char;
use nom::character::complete::one_of;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;

use crate::parser::helpers;
use crate::parser::interface::*;
use crate::parser::literal;

static LVAR_CHAR: char = '@';
static GVAR_CHAR: char = '$';

pub fn variable(s: Span) -> R<Variable> {
    alt((
        map(
            consumed(tuple((single_variable, array_element_scr))),
            |(span, (var, (index, len, _type)))| {
                Variable::ArrayElement(ArrayElementSCR {
                    array_var: Box::new(var),
                    index_var: Box::new(index),
                    len,
                    _type,
                    token: Token::from(span, SyntaxKind::ArrayElementSCR),
                })
            },
        ),
        map(
            consumed(tuple((single_variable, array_index))),
            |(span, (var, index))| {
                Variable::Indexed(IndexedVariable {
                    var: Box::new(var),
                    index: Box::new(index),
                    token: Token::from(span, SyntaxKind::IndexedVariable),
                })
            },
        ),
        single_variable,
    ))(s)
}

// $var($index,1i)
fn array_element_scr(s: Span) -> R<(Variable, Token, VariableType)> {
    delimited(
        char('('),
        tuple((
            terminated(single_variable, char(',')),
            literal::decimal,
            array_type,
        )),
        char(')'),
    )(s)
}

fn array_index(s: Span) -> R<Node> {
    delimited(
        char('['),
        alt((
            map(single_variable, |v| Node::Variable(v)),
            map(literal::decimal, |d| Node::Token(d)),
        )),
        char(']'),
    )(s)
}

fn single_variable(s: Span) -> R<Variable> {
    alt((local_var, global_var))(s)
}

fn local_var(s: Span) -> R<Variable> {
    map(
        consumed(tuple((
            literal::decimal,
            preceded(char(LVAR_CHAR), variable_type),
        ))),
        |(span, (name, _type))| {
            Variable::Local(SingleVariable {
                name,
                _type,
                token: Token::from(span, SyntaxKind::LocalVariable),
            })
        },
    )(s)
}

fn global_var(s: Span) -> R<Variable> {
    map(
        consumed(tuple((
            terminated(variable_type, char(GVAR_CHAR)),
            alt((literal::decimal, literal::identifier_any)),
        ))),
        |(span, (_type, name))| {
            Variable::Global(SingleVariable {
                name,
                _type,
                token: Token::from(span, SyntaxKind::GlobalVariable),
            })
        },
    )(s)
}

fn variable_type(s: Span) -> R<VariableType> {
    map(opt(one_of("sv")), |c| helpers::char_to_type(c))(s)
}

fn array_type(s: Span) -> R<VariableType> {
    map(opt(one_of("ifsv")), |c| helpers::char_to_type(c))(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    #[test]
    fn test_parser_variables01() {
        let (_, ast) = parse("0@").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Local(SingleVariable {
                    name: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::Unknown,
                    token: Token {
                        syntax_kind: SyntaxKind::LocalVariable,
                        start: 1,
                        len: 2,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables2() {
        let (_, ast) = parse("0@s").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Local(SingleVariable {
                    name: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::ShortString,
                    token: Token {
                        syntax_kind: SyntaxKind::LocalVariable,
                        start: 1,
                        len: 3,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables3() {
        let (_, ast) = parse("0@v").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Local(SingleVariable {
                    name: Token {
                        start: 1,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::LongString,
                    token: Token {
                        syntax_kind: SyntaxKind::LocalVariable,
                        start: 1,
                        len: 3,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables4() {
        let (_, ast) = parse("$var").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 2,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: VariableType::Unknown,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 4,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables5() {
        let (_, ast) = parse("s$var").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: VariableType::ShortString,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 5,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables6() {
        let (_, ast) = parse("v$var").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 3,
                        len: 3,
                        syntax_kind: SyntaxKind::Identifier
                    },
                    _type: VariableType::LongString,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 5,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables7() {
        let (_, ast) = parse("$1").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 2,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::Unknown,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 2,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables8() {
        let (_, ast) = parse("s$1").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 3,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::ShortString,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 3,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables9() {
        let (_, ast) = parse("v$1").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Global(SingleVariable {
                    name: Token {
                        start: 3,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    },
                    _type: VariableType::LongString,
                    token: Token {
                        syntax_kind: SyntaxKind::GlobalVariable,
                        start: 1,
                        len: 3,
                    }
                }))]
            }
        );
    }
    #[test]
    fn test_parser_variables10() {
        let (_, ast) = parse("$var($index,10i)").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::ArrayElement(ArrayElementSCR {
                    array_var: Box::new(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 3,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            len: 4,
                            start: 1,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    })),
                    index_var: Box::new(Variable::Global(SingleVariable {
                        name: Token {
                            start: 7,
                            len: 5,
                            syntax_kind: SyntaxKind::Identifier
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 6,
                            len: 6,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    })),
                    _type: VariableType::Int,
                    len: Token {
                        syntax_kind: SyntaxKind::IntegerLiteral,
                        start: 13,
                        len: 2
                    },
                    token: Token {
                        syntax_kind: SyntaxKind::ArrayElementSCR,
                        start: 1,
                        len: 16,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables11() {
        let (_, ast) = parse("$1($2,10f)").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::ArrayElement(ArrayElementSCR {
                    array_var: Box::new(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            len: 2,
                            start: 1,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    })),
                    index_var: Box::new(Variable::Global(SingleVariable {
                        name: Token {
                            start: 5,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 4,
                            len: 2,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    })),
                    _type: VariableType::Float,
                    len: Token {
                        syntax_kind: SyntaxKind::IntegerLiteral,
                        start: 7,
                        len: 2
                    },
                    token: Token {
                        syntax_kind: SyntaxKind::ArrayElementSCR,
                        start: 1,
                        len: 10,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables12() {
        let (_, ast) = parse("$1(11@,10s)").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::ArrayElement(ArrayElementSCR {
                    array_var: Box::new(Variable::Global(SingleVariable {
                        name: Token {
                            start: 2,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            len: 2,
                            start: 1,
                            syntax_kind: SyntaxKind::GlobalVariable
                        }
                    })),
                    index_var: Box::new(Variable::Local(SingleVariable {
                        name: Token {
                            start: 4,
                            len: 2,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 4,
                            len: 3,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    })),
                    _type: VariableType::ShortString,
                    len: Token {
                        syntax_kind: SyntaxKind::IntegerLiteral,
                        start: 8,
                        len: 2
                    },
                    token: Token {
                        syntax_kind: SyntaxKind::ArrayElementSCR,
                        start: 1,
                        len: 11,
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables13() {
        let (_, ast) = parse("$var[1]").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Indexed(IndexedVariable {
                    var: Box::new(Variable::Global(SingleVariable {
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
                    })),
                    index: Box::new(Node::Token(Token {
                        start: 6,
                        len: 1,
                        syntax_kind: SyntaxKind::IntegerLiteral
                    })),
                    token: Token {
                        start: 1,
                        len: 7,
                        syntax_kind: SyntaxKind::IndexedVariable
                    }
                }))]
            }
        );
    }

    #[test]
    fn test_parser_variables14() {
        let (_, ast) = parse("$var[0@]").unwrap();
        assert_eq!(
            ast,
            AST {
                body: vec![Node::Variable(Variable::Indexed(IndexedVariable {
                    var: Box::new(Variable::Global(SingleVariable {
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
                    })),
                    index: Box::new(Node::Variable(Variable::Local(SingleVariable {
                        name: Token {
                            start: 6,
                            len: 1,
                            syntax_kind: SyntaxKind::IntegerLiteral
                        },
                        _type: VariableType::Unknown,
                        token: Token {
                            start: 6,
                            len: 2,
                            syntax_kind: SyntaxKind::LocalVariable
                        }
                    }))),
                    token: Token {
                        start: 1,
                        len: 8,
                        syntax_kind: SyntaxKind::IndexedVariable
                    }
                }))]
            }
        );
    }
}
