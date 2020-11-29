use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::not;
use nom::combinator::peek;
use nom::combinator::value;
use nom::sequence::delimited;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

use crate::parser::interface::*;

// whitespace wrapper
pub fn ws<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(
        alt((inline_comment, value((), space0))),
        inner,
        alt((inline_comment, value((), space0))),
    )
}

pub fn inline_comment<'a, E: nom::error::ParseError<Span<'a>>>(
    s: Span<'a>,
) -> IResult<Span<'a>, (), E> {
    value(
        (),
        alt((
            tuple((tag("/*"), take_until("*/"), tag("*/"))),
            tuple((
                terminated(tag("{"), not(peek(char('$')))),
                take_until("}"),
                tag("}"),
            )),
        )),
    )(s)
}

pub fn char_to_type(c: Option<char>) -> VariableType {
    match c {
        Some('i') => VariableType::Int,
        Some('f') => VariableType::Float,
        Some('s') => VariableType::ShortString,
        Some('v') => VariableType::LongString,
        _ => VariableType::Unknown,
    }
}
