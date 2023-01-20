use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::combinator::map_opt;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::{branch::alt, character::complete::hex_digit1};

use crate::parser::interface::*;

pub fn number(s: Span) -> R<Literal> {
    alt((
        map(hexadicimal, |i| Literal::Int(i)),
        map(float, |f| Literal::Float(f)),
        map(integer, |i| Literal::Int(i)),
    ))(s)
}

// combination of letters, digits and underscore, not starting with a digit
pub fn identifier(s: Span) -> R<Token> {
    map(
        consumed(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |(span, _)| Token::from(span, SyntaxKind::Identifier),
    )(s)
}

// any combination of letters, digits and underscore
pub fn identifier_any(s: Span) -> R<Token> {
    map(
        consumed(many1(alt((alphanumeric1, tag("_"))))),
        |(span, _)| Token::from(span, SyntaxKind::Identifier),
    )(s)
}

pub fn integer(s: Span) -> R<IntLiteral> {
    map_opt(decimal_span, |s| {
        Some(IntLiteral {
            value: s.fragment().parse::<i32>().ok()?,
            token: Token::from(s, SyntaxKind::IntegerLiteral),
        })
    })(s)
}

pub fn float(s: Span) -> R<FloatLiteral> {
    map_opt(float_span, |s| {
        Some(FloatLiteral {
            value: s.fragment().parse::<f32>().ok()?,
            token: Token::from(s, SyntaxKind::FloatLiteral),
        })
    })(s)
}

pub fn hexadicimal(s: Span) -> R<IntLiteral> {
    map_opt(hexadecimal_span, |s| {
        Some(IntLiteral {
            value: i32::from_str_radix(s.fragment(), 16).ok()?,
            token: Token::from(s, SyntaxKind::HexadecimalLiteral),
        })
    })(s)
}

pub fn decimal_span(s: Span) -> R<Span> {
    recognize(digit1)(s)
}

fn hexadecimal_span(s: Span) -> R<Span> {
    preceded(alt((tag("0x"), tag("0X"))), hex_digit1)(s)
}

pub fn float_span(s: Span) -> R<Span> {
    alt((
        // Case one: .42
        recognize(tuple((
            char('.'),
            decimal_span,
            opt(tuple((one_of("eE"), opt(one_of("+-")), decimal_span))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            decimal_span,
            opt(preceded(char('.'), decimal_span)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal_span,
        ))), // Case three: 42. and 42.42
        recognize(tuple((decimal_span, char('.'), opt(decimal_span)))),
    ))(s)
}

#[test]
fn literal_1() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse(" 1 ").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Int(IntLiteral {
                value: 1,
                token: Token {
                    start: 2,
                    len: 1,
                    syntax_kind: SyntaxKind::IntegerLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_2() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse(" 1.0 ").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Float(FloatLiteral {
                value: 1.0,
                token: Token {
                    start: 2,
                    len: 3,
                    syntax_kind: SyntaxKind::FloatLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_3() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("1.0e1").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Float(FloatLiteral {
                value: 10.0,
                token: Token {
                    start: 1,
                    len: 5,
                    syntax_kind: SyntaxKind::FloatLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_4() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("1e1").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Float(FloatLiteral {
                value: 10.0,
                token: Token {
                    start: 1,
                    len: 3,
                    syntax_kind: SyntaxKind::FloatLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_5() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("1.0e-1").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Float(FloatLiteral {
                value: 0.1,
                token: Token {
                    start: 1,
                    len: 6,
                    syntax_kind: SyntaxKind::FloatLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_6() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("1.0e+1").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Float(FloatLiteral {
                value: 10.0,
                token: Token {
                    start: 1,
                    len: 6,
                    syntax_kind: SyntaxKind::FloatLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_7() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("0x1").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Int(IntLiteral {
                value: 1,
                token: Token {
                    start: 3,
                    len: 1,
                    syntax_kind: SyntaxKind::HexadecimalLiteral,
                },
            }))],
        }
    );
}

#[test]
fn literal_8() {
    use super::*;
    use crate::parser::parse;
    let (_, ast) = parse("-0xABC").unwrap();
    assert_eq!(
        ast,
        AST {
            body: vec![Node::Literal(Literal::Int(IntLiteral {
                value: -2748,
                token: Token {
                    start: 1,
                    len: 6,
                    syntax_kind: SyntaxKind::HexadecimalLiteral,
                },
            }))],
        }
    );
}
