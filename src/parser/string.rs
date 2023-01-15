use crate::parser::interface::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::char;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::{combinator::map, sequence::delimited};

pub fn string(s: Span) -> R<Token> {
    map(alt((string1_span, string2_span)), |s| {
        Token::from(s, SyntaxKind::StringLiteral)
    })(s)
}
fn string1_span(s: Span) -> R<Span> {
    delimited(
        char('"'),
        recognize(many0(alt((
            is_not(r#""\"#),
            alt((tag(r#"\""#), tag("\\"))),
        )))),
        char('"'),
    )(s)
}

fn string2_span(s: Span) -> R<Span> {
    delimited(
        char('\''),
        recognize(many0(alt((
            is_not(r#"'\"#),
            alt((tag(r#"\'"#), tag("\\"))),
        )))),
        char('\''),
    )(s)
}

#[test]
fn test_parse_string1() {
    use super::*;
    use crate::parser::parse;

    let (_, ast) = parse(r#""test""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 4,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
}

#[test]
fn test_parse_string_with_spaces() {
    use super::*;
    use crate::parser::parse;

    let (_, ast) = parse(r#""word1 word2 word3""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 17,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
    let (_, ast) = parse(r#"'word1 word2 word3'"#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 17,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
}

#[test]
fn test_parse_empty_string() {
    use super::*;
    use crate::parser::parse;

    let (_, ast) = parse(r#""""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 0,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
    let (_, ast) = parse(r#"''"#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 0,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
}

#[test]
fn test_parse_string_escaped() {
    use super::*;
    use crate::parser::parse;

    let (_, ast) = parse(r#""word1 \n\n word2""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 16,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#""\"""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 2,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#""\"b""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 3,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#""a\"\"\nb""#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 8,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#"'\''"#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 2,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#"'\'b'"#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 3,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );

    let (_, ast) = parse(r#"'a\'\'\nb'"#).unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Token {
                start: 2,
                len: 8,
                syntax_kind: SyntaxKind::StringLiteral
            })]
        }
    );
}
