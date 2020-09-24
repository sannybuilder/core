use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::peek;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

pub enum HintParamValue<'a> {
    Empty,
    Text(&'a str),
}
pub enum HintParam<'a> {
    Text(&'a str),
    Enum(Vec<(&'a str, HintParamValue<'a>)>),
}

pub struct Param<'a> {
    pub name: &'a str,
    pub _type: HintParam<'a>,
}

pub fn hint(input: &str) -> IResult<&str, Vec<Param>> {
    delimited(
        char('('),
        many0(map(
            tuple((
                space0,
                delimited(
                    char('"'),
                    map(
                        tuple((
                            opt(is_not("%:\"")),
                            alt((hint_fixed_type, hint_arbitrary_type, hint_unknown_type)),
                        )),
                        |(name, _type)| {
                            let name = name.unwrap_or("_"); // empty param name is allowed
                            Param { name, _type }
                        },
                    ),
                    char('"'),
                ),
                space0,
            )),
            |(_, param, _)| param,
        )),
        char(')'),
    )(input)
}

pub fn hint_fixed_type(input: &str) -> IResult<&str, HintParam> {
    map(tuple((char('%'), is_not("\""))), |(_, param_type)| {
        HintParam::Text(match param_type {
            "h" => "Handle",
            "v" | "s" => "String",
            "b" => "Boolean",
            "f" => "Float",
            "i" => "Integer",
            _ => param_type,
        })
    })(input)
}

pub fn hint_arbitrary_type(input: &str) -> IResult<&str, HintParam> {
    map(
        tuple((char(':'), space0, alt((hint_enum_type, hint_plain_type)))),
        |(_, _, p)| p,
    )(input)
}

pub fn hint_unknown_type(input: &str) -> IResult<&str, HintParam> {
    Ok((input, HintParam::Text("Unknown")))
}

pub fn hint_enum_type(input: &str) -> IResult<&str, HintParam> {
    map(
        tuple((
            peek(char('^')),
            many1(map(
                tuple((
                    char('^'),
                    is_not("^=\""),
                    opt(tuple((char('='), is_not("^\"")))),
                )),
                |(_, enum_param, value)| match value {
                    Some((_, val)) => (enum_param, HintParamValue::Text(val)),
                    None => (enum_param, HintParamValue::Empty),
                },
            )),
        )),
        |(_, v)| HintParam::Enum(v),
    )(input)
}

pub fn hint_plain_type(input: &str) -> IResult<&str, HintParam> {
    map(is_not("\""), |x| HintParam::Text(x))(input)
}

pub fn method(input: &str) -> IResult<&str, (&str, &str, &str, &str, Vec<Param>)> {
    map(
        all_consuming(tuple((
            literal, // name
            comma,
            literal, // id
            comma,
            literal, // type
            comma,
            literal, // help_code,
            opt(comma),
            opt(hint),
        ))),
        |(name, _, id, _, r#type, _, help_code, _, hint)| {
            let hint = hint.unwrap_or(vec![]);
            (name, id, r#type, help_code, hint)
        },
    )(input)
}

pub fn property(
    input: &str,
) -> IResult<
    &str,
    (
        &str,
        std::vec::Vec<(&str, &str, &str, &str, &str)>,
        Vec<Param>,
    ),
> {
    map(
        all_consuming(tuple((
            char('^'),
            literal, // name
            comma,
            many0(map(
                tuple((
                    char('['),
                    space0,
                    take_while_m_n(4, 4, is_hex_digit), // opcode
                    comma,
                    literal, // operation
                    comma,
                    literal, // pos
                    comma,
                    literal, // type
                    comma,
                    literal, // help code
                    space0,
                    char(']'),
                    opt(comma),
                )),
                |(_, _, id, _, operation, _, pos, _, r#type, _, help_code, _, _, _)| {
                    (id, operation, pos, r#type, help_code)
                },
            )),
            opt(hint),
        ))),
        |(_, name, _, variations, hint)| {
            let hint = hint.unwrap_or(vec![]);
            (name, variations, hint)
        },
    )(input)
}

pub fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

pub fn literal(s: &str) -> IResult<&str, &str> {
    is_not(",] \t\r\n")(s)
}

pub fn comma(s: &str) -> IResult<&str, (&str, char, &str)> {
    tuple((space0, char(','), space0))(s)
}
