use nom::branch::alt;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;

use crate::parser::interface::*;
use crate::parser::literal;

static LVAR_CHAR: char = '@';
static GVAR_CHAR: char = '$';

pub fn variable(s: Span) -> R<Token> {
    alt((array, local_var, global_var))(s)
}

fn array(s: Span) -> R<Token> {
    alt((array_typed, array_indexed))(s)
}

fn variable_type_char(s: Span) -> R<Option<char>> {
    opt(one_of("sv"))(s)
}

fn array_typed(s: Span) -> R<Token> {
    map(
        recognize(tuple((
            variable_span,
            delimited(
                char('('),
                tuple((variable_span, char(','), digit1, opt(one_of("ifvs")))),
                char(')'),
            ),
        ))),
        |s: Span| Token::from(s, SyntaxKind::Array),
    )(s)
}

fn array_indexed(s: Span) -> R<Token> {
    map(
        recognize(tuple((
            variable_span,
            delimited(
                char('['),
                alt((variable_span, literal::decimal_span)),
                char(']'),
            ),
        ))),
        |s: Span| Token::from(s, SyntaxKind::Array),
    )(s)
}

fn local_var(s: Span) -> R<Token> {
    map(local_var_span, |s: Span| {
        Token::from(s, SyntaxKind::LocalVariable)
    })(s)
}

fn global_var(s: Span) -> R<Token> {
    map(global_var_span, |s: Span| {
        Token::from(s, SyntaxKind::GlobalVariable)
    })(s)
}

fn variable_span(s: Span) -> R<Span> {
    alt((local_var_span, global_var_span))(s)
}

fn local_var_span(s: Span) -> R<Span> {
    recognize(terminated(
        digit1,
        tuple((char(LVAR_CHAR), variable_type_char)),
    ))(s)
}

fn global_var_span(s: Span) -> R<Span> {
    recognize(preceded(
        tuple((variable_type_char, char(GVAR_CHAR))),
        literal::identifier_any_span,
    ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    #[test]
    fn test_variables() {
        let (_, ast) = parse("0@").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::LocalVariable,
                    start: 1,
                    len: 2,
                })
            }
        );
        let (_, ast) = parse("0@s").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::LocalVariable,
                    start: 1,
                    len: 3,
                })
            }
        );
        let (_, ast) = parse("0@v").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::LocalVariable,
                    start: 1,
                    len: 3,
                })
            }
        );
        let (_, ast) = parse("$var").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 4,
                })
            }
        );
        let (_, ast) = parse("s$var").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 5,
                })
            }
        );
        let (_, ast) = parse("v$var").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 5,
                })
            }
        );
        let (_, ast) = parse("$1").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 2,
                })
            }
        );
        let (_, ast) = parse("s$1").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 3,
                })
            }
        );
        let (_, ast) = parse("v$1").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::GlobalVariable,
                    start: 1,
                    len: 3,
                })
            }
        );

        let (_, ast) = parse("$var($index,10i)").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::Array,
                    start: 1,
                    len: 16,
                })
            }
        );

        let (_, ast) = parse("$1($2,10f)").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::Array,
                    start: 1,
                    len: 10,
                })
            }
        );
        let (_, ast) = parse("$1(11@,10s)").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::Array,
                    start: 1,
                    len: 11,
                })
            }
        );
        let (_, ast) = parse("$var[1]").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::Array,
                    start: 1,
                    len: 7,
                })
            }
        );
        let (_, ast) = parse("$var[0@]").unwrap();
        assert_eq!(
            ast,
            AST {
                node: Node::Token(Token {
                    syntax_kind: SyntaxKind::Array,
                    start: 1,
                    len: 8,
                })
            }
        );
    }
}
