use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::combinator::consumed;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::{branch::alt, character::complete::hex_digit1};

use crate::parser::interface::*;

pub fn number(s: Span) -> R<Token> {
    alt((hexadecimal, float, decimal, label))(s)
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

pub fn decimal(s: Span) -> R<Token> {
    map(decimal_span, |s| Token::from(s, SyntaxKind::IntegerLiteral))(s)
}

pub fn float(s: Span) -> R<Token> {
    map(float_span, |s| Token::from(s, SyntaxKind::FloatLiteral))(s)
}

pub fn decimal_span(s: Span) -> R<Span> {
    recognize(digit1)(s)
}

pub fn hexadecimal(s: Span) -> R<Token> {
    map(
        recognize(pair(tag_no_case("0x"), hex_digit1)), 
        |s| Token::from(s, SyntaxKind::IntegerLiteral)
    )(s)
}

pub fn label(s: Span) -> R<Token> {
    map(
        recognize(pair(tag_no_case("@"), identifier_any)), 
        |s| Token::from(s, SyntaxKind::LabelLiteral)
    )(s)
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
