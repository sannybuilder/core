/*

enum %name
    %item[,|=%value]+
end

*/

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric1;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::line_ending;
use nom::character::complete::multispace0;
use nom::character::complete::space0;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::combinator::verify;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    pub name: &'a str,
    pub items: EnumItems<'a>,
}

type IntEnum<'a> = Vec<(&'a str, i32)>;
type TextEnum<'a> = Vec<(&'a str, &'a str)>;

#[derive(Debug, PartialEq)]
pub enum EnumItems<'a> {
    Int(IntEnum<'a>),
    Text(TextEnum<'a>),
}

#[derive(Debug, PartialEq)]
pub struct EnumItemRaw<'a> {
    pub name: &'a str,
    pub value: EnumItemValueRaw<'a>,
}

#[derive(Debug, PartialEq)]
pub enum EnumItemValueRaw<'a> {
    Empty,
    Int(i32),
    Text(&'a str),
}

pub fn parse_enums(input: &str) -> IResult<&str, Vec<Enum>> {
    many0(parse_enum)(input)
}

fn parse_enum(input: &str) -> IResult<&str, Enum> {
    map(
        delimited(
            delimited(multispace0, tag("enum"), space1),
            pair(enum_name, enum_items),
            tuple((tag("end"), space0, optional_line_ending)),
        ),
        |(name, items)| Enum { name, items },
    )(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn optional_line_ending(input: &str) -> IResult<&str, &str> {
    if input.len() == 0 {
        Ok(("", ""))
    } else {
        line_ending(input)
    }
}

fn enum_items(input: &str) -> IResult<&str, EnumItems> {
    let (input, raw_items) = many1(enum_item)(input)?;

    #[derive(PartialEq)]
    enum TT {
        Unknown,
        Int,
        Text,
    }

    let mut _type = TT::Unknown;
    for item in &raw_items {
        match &item.value {
            EnumItemValueRaw::Empty => continue,
            EnumItemValueRaw::Int(_) => {
                if _type == TT::Unknown {
                    _type = TT::Int;
                } else if _type != TT::Int {
                    return Err(nom::Err::Failure(nom::error::Error {
                        input: "Mixed type",
                        code: nom::error::ErrorKind::Verify,
                    }));
                }
            }
            EnumItemValueRaw::Text(_) => {
                if _type == TT::Unknown {
                    _type = TT::Text;
                } else if _type != TT::Text {
                    return Err(nom::Err::Failure(nom::error::Error {
                        input: "Mixed type",
                        code: nom::error::ErrorKind::Verify,
                    }));
                }
            }
        }
    }

    let items = match _type {
        TT::Unknown | TT::Int => {
            let mut index = 0;
            let items = raw_items
                .iter()
                .map(|i| {
                    let value = match i.value {
                        EnumItemValueRaw::Empty => index,
                        EnumItemValueRaw::Int(x) => {
                            index = x;
                            x
                        }
                        _ => panic!(),
                    };
                    index += 1;
                    (i.name, value)
                })
                .collect::<Vec<_>>();

            EnumItems::Int(items)
        }

        TT::Text => {
            let items = raw_items
                .iter()
                .map(|i| {
                    let value = match i.value {
                        EnumItemValueRaw::Empty => i.name,
                        EnumItemValueRaw::Text(x) => x,
                        _ => panic!(),
                    };
                    (i.name, value)
                })
                .collect::<Vec<_>>();

            EnumItems::Text(items)
        }
    };

    Ok((input, items))
}

fn enum_name(input: &str) -> IResult<&str, &str> {
    terminated(identifier, delimited(space0, line_ending, multispace0))(input)
}

fn enum_item(input: &str) -> IResult<&str, EnumItemRaw> {
    map(
        terminated(
            pair(verify(identifier, |s: &str| s != "end"), enum_value),
            delimited(
                space0,
                alt((value((), line_ending), value((), char(',')))),
                multispace0,
            ),
        ),
        |(name, value)| EnumItemRaw { name, value },
    )(input)
}

fn enum_value(input: &str) -> IResult<&str, EnumItemValueRaw> {
    map(
        opt(preceded(
            delimited(space0, char('='), space0),
            alt((number, text)),
        )),
        |v| match v {
            Some(v) => v,
            _ => EnumItemValueRaw::Empty,
        },
    )(input)
}

fn number(input: &str) -> IResult<&str, EnumItemValueRaw> {
    let (input, d) = digit1(input)?;
    match i32::from_str_radix(d, 10) {
        Ok(d) => Ok((input, EnumItemValueRaw::Int(d))),
        _ => Err(nom::Err::Error(nom::error::Error {
            input: d,
            code: nom::error::ErrorKind::Digit,
        })),
    }
}

fn text(input: &str) -> IResult<&str, EnumItemValueRaw> {
    map(delimited(char('"'), is_not("\""), char('"')), |v| {
        EnumItemValueRaw::Text(v)
    })(input)
}

#[test]
fn test_enum0() {
    assert_eq!(
        parse_enums(
            r"
    enum X
    ending=1
    b=2
    end"
        ),
        Ok((
            "",
            vec![Enum {
                name: "X",
                items: EnumItems::Int(vec![("ending", 1), ("b", 2)])
            }]
        ))
    )
}

#[test]
fn test_enum1() {
    assert_eq!(
        parse_enums(
            r"
    enum X
     a, b
    end"
        ),
        Ok((
            "",
            vec![Enum {
                name: "X",
                items: EnumItems::Int(vec![("a", 0), ("b", 1)])
            }]
        ))
    )
}

#[test]
fn test_enum2() {
    assert_eq!(
        parse_enums(
            r"
    enum X
     a=10, b
    end"
        ),
        Ok((
            "",
            vec![Enum {
                name: "X",
                items: EnumItems::Int(vec![("a", 10), ("b", 11)])
            }]
        ))
    )
}

#[test]
fn test_enum3() {
    assert_eq!(
        parse_enums(
            r#"
    enum X
     a=" x ", b
    end"#
        ),
        Ok((
            "",
            vec![Enum {
                name: "X",
                items: EnumItems::Text(vec![("a", " x "), ("b", "b")])
            }]
        ))
    )
}

#[test]
fn test_enum4() {
    assert_eq!(
        parse_enums(
            r#"
    enum X
     x=1,a="a"
    end"#
        ),
        Err(nom::Err::Failure(nom::error::Error {
            input: "Mixed type",
            code: nom::error::ErrorKind::Verify
        }))
    )
}

#[test]
fn test_enum5() {
    assert_eq!(
        parse_enums(
            r#"
    enum X
     a, b
    end 
    
    
    
    enum X2
    a, b
   end"#,
        ),
        Ok((
            "",
            vec![
                Enum {
                    name: "X",
                    items: EnumItems::Int(vec![("a", 0), ("b", 1)])
                },
                Enum {
                    name: "X2",
                    items: EnumItems::Int(vec![("a", 0), ("b", 1)])
                }
            ]
        ))
    )
}
