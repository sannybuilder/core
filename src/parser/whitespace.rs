use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::not;
use nom::combinator::peek;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;
use nom::branch::alt;
use nom::character::complete::multispace1;

use crate::parser::interface::*;

/** whitespace wrapper */
pub fn ws<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(
        many0(alt((inline_comment, value((), space1)))),
        inner,
        many0(alt((inline_comment, eol_comment, value((), space1)))),
    )
}

/** whitespace wrapper */
pub fn mws<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(
        many0(alt((inline_comment, value((), multispace1)))),
        inner,
        many0(alt((inline_comment, eol_comment, value((), multispace1)))),
    )
}

/** standalone line */
// todo: should end with eol
pub fn line<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    mws(inner)
}

/** inline comment */
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

/** eol comment // */
pub fn eol_comment<'a, E: nom::error::ParseError<Span<'a>>>(
    s: Span<'a>,
) -> IResult<Span<'a>, (), E> {
    value((), pair(tag("//"), is_not("\n\r")))(s)
}
