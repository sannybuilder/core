use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::all_consuming;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

pub fn hint(input: &str) -> IResult<&str, String> {
    map_res(
        delimited(
            char('('),
            many0(map_res(
                tuple((
                    space0,
                    delimited(
                        char('"'),
                        map_res(
                            tuple((
                                opt(is_not("%^")),
                                opt(alt((hint_plain_param, hint_enum_param))),
                            )),
                            |(name, t)| -> Result<String, !> {
                                let name = name.unwrap_or("_"); // empty param name
                                match t {
                                    Some(t) => Ok(format!("{}: {}", name, t)),
                                    None => Ok(String::from(name)),
                                }
                            },
                        ),
                        char('"'),
                    ),
                    space0,
                )),
                |(_, params, _)| -> Result<String, !> { Ok(params) },
            )),
            char(')'),
        ),
        |params| -> Result<String, !> { Ok(params.join("; ")) },
    )(input)
}

pub fn hint_plain_param(input: &str) -> IResult<&str, &str> {
    map_res(
        tuple((char('%'), is_not("\""))),
        |(_, param_type)| -> Result<&str, !> {
            let x = match param_type {
                "h" => "Handle",
                "v" | "s" => "String",
                "b" => "Boolean",
                "f" => "Float",
                "i" => "Integer",
                _ => param_type,
            };
            Ok(x)
        },
    )(input)
}

pub fn hint_enum_param(input: &str) -> IResult<&str, &str> {
    map_res(tuple((char('^'), is_not("\""))), |_| -> Result<&str, !> {
        Ok("Extended")
    })(input)
}

pub fn property(
    input: &str,
) -> IResult<&str, (&str, std::vec::Vec<(&str, &str, &str, &str, &str)>, String)> {
    map_res(
        all_consuming(
            tuple((
                char('^'),
                literal, // name
                comma,
                many0(map_res(
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
                        comma
                    )),
                    |(_, _, id, _, operation, _, pos, _, r#type, _, help_code, _, _, _)| -> Result<(&str,&str,&str,&str,&str), !> {
                        Ok((id, operation, pos, r#type, help_code))
                    },
                )),
                hint,
            ))
        ),
        |(_, name, _, variations, hint)| -> Result<
            (&str, std::vec::Vec<(&str, &str, &str, &str, &str)>, String),
            !,
        > { Ok((name, variations, hint)) },
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
