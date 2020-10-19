use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

pub struct Param<'a> {
    pub name: &'a str,
    pub _type: ParamType<'a>,
}

pub enum ParamType<'a> {
    Text(&'a str),
    Enum(Vec<ParamTypeEnum<'a>>),
}

pub struct ParamTypeEnum<'a> {
    pub name: &'a str,
    // pub is_anonymous: bool,
    pub value: ParamTypeEnumValue<'a>,
}

pub enum ParamTypeEnumValue<'a> {
    Empty,
    Text(&'a str),
}

pub fn params(input: &str) -> IResult<&str, Vec<Param>> {
    delimited(
        char('('),
        many0(delimited(
            space0,
            delimited(
                char('"'),
                map(
                    tuple((
                        opt(is_not("^%:\"")),
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
        char(')'),
    )(input)
}

pub fn hint_fixed_type(input: &str) -> IResult<&str, ParamType> {
    map(preceded(char('%'), is_not("\"")), |param_type| {
        ParamType::Text(match param_type {
            "h" => "Handle",
            "v" | "s" => "String",
            "b" => "Boolean",
            "f" => "Float",
            "i" => "Integer",
            _ => param_type,
        })
    })(input)
}

pub fn hint_arbitrary_type(input: &str) -> IResult<&str, ParamType> {
    preceded(
        terminated(char(':'), space0),
        alt((hint_anonymous_enum_type, hint_plain_type)),
    )(input)
}

pub fn hint_unknown_type(input: &str) -> IResult<&str, ParamType> {
    Ok((input, ParamType::Text("Unknown")))
}

pub fn hint_anonymous_enum_type(input: &str) -> IResult<&str, ParamType> {
    map(
        many1(preceded(
            char('^'),
            map(
                tuple((
                    is_not("^=\""),
                    map(opt(preceded(char('='), is_not("^\""))), |val| match val {
                        Some(val) => ParamTypeEnumValue::Text(val),
                        None => ParamTypeEnumValue::Empty,
                    }),
                )),
                |(name, value)| ParamTypeEnum {
                    name,
                    value,
                    // is_anonymous: true,
                },
            ),
        )),
        |v| ParamType::Enum(v),
    )(input)
}

pub fn hint_plain_type(input: &str) -> IResult<&str, ParamType> {
    map(is_not("\""), |x| ParamType::Text(x))(input)
}

pub fn method(input: &str) -> IResult<&str, (&str, &str, &str, &str, Vec<Param>)> {
    all_consuming(tuple((
        terminated(literal, comma), // name
        terminated(literal, comma), // id
        terminated(literal, comma), // type
        literal,                    // help_code,
        map(opt(preceded(comma, params)), |params| {
            params.unwrap_or(vec![])
        }),
    )))(input)
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
    all_consuming(preceded(
        char('^'),
        tuple((
            terminated(literal, comma), // name
            separated_list(
                comma,
                delimited(
                    terminated(char('['), space0),
                    tuple((
                        terminated(take_while_m_n(4, 4, is_hex_digit), comma), // opcode
                        terminated(literal, comma),                            // operation
                        terminated(literal, comma),                            // pos
                        terminated(literal, comma),                            // type
                        terminated(literal, space0),                           // help code
                    )),
                    char(']'),
                ),
            ),
            map(opt(preceded(comma, params)), |params| {
                params.unwrap_or(vec![])
            }),
        )),
    ))(input)
}

pub fn deprecated_anonymous_enum(input: &str) -> IResult<&str, Param> {
    all_consuming(map(
        tuple((
            delimited(space0, is_not(" \t\r\n"), space0),
            char('='),
            delimited(space0, hint_anonymous_enum_type, space0),
        )),
        |(name, _, _type)| Param { name, _type },
    ))(input)
}

pub fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

pub fn literal(s: &str) -> IResult<&str, &str> {
    is_not(",] \t\r\n")(s)
}

pub fn comma(s: &str) -> IResult<&str, char> {
    delimited(space0, char(','), space0)(s)
}
