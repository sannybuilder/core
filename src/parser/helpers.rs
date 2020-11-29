// whitespace wrapper
use crate::parser::interface::Span;
use nom::character::complete::space0;
use nom::sequence::delimited;
use nom::IResult;

use super::interface::VariableType;

pub fn ws<'a, F: 'a, O, E: nom::error::ParseError<Span<'a>>>(
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    delimited(space0, inner, space0)
}

pub fn char_to_type(c: Option<char>) -> VariableType {
    match c {
        Some('i') => VariableType::Int,
        Some('f') => VariableType::Float,
        Some('s') => VariableType::Str16,
        Some('v') => VariableType::Str256,
        _ => VariableType::Unknown,
    }
}
