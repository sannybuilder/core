use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::{branch::alt, character::complete::hex_digit1};
use nom::{bytes::complete::tag, combinator::map_res};
use nom::{character::complete::alpha1, combinator::map_opt};

use crate::parser::interface::*;

pub fn number(s: Span) -> R<Token> {
    alt((
        map(float_span, |s| Token::from(s, SyntaxKind::FloatLiteral)),
        map(decimal_span, |s| Token::from(s, SyntaxKind::IntegerLiteral)),
    ))(s)
}

// combination of letters, digits and underscore, not starting with a digit
pub fn identifier(s: Span) -> R<Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(s)
}

// any combination of letters, digits and underscore
pub fn identifier_any_span(s: Span) -> R<Span> {
    recognize(many1(alt((alphanumeric1, tag("_")))))(s)
}

pub fn decimal_span(s: Span) -> R<Span> {
    recognize(digit1)(s)
}

fn hexadecimal_span(s: Span) -> R<Span> {
    preceded(alt((tag("0x"), tag("0X"))), recognize(hex_digit1))(s)
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
