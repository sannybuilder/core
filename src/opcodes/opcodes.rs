extern crate nom;

use nom::bytes::complete::take_till;
use nom::character::complete::{alpha1, digit1};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple,
    IResult,
};

#[derive(Debug, PartialEq)]
struct OpcodeDefinition {
    id: u16,
    num_params: i8,
    words: Vec<String>,
    params: Vec<Param>,
}

#[derive(Debug, PartialEq)]
struct Param {
    r#type: String,
    position: u8,
}

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

fn from_hex(input: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(input, 16)
}

fn to_i8(input: &str) -> Result<i8, std::num::ParseIntError> {
    input.parse()
}

fn to_u8(input: &str) -> Result<u8, std::num::ParseIntError> {
    input.parse()
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn string_chunk(s: &str) -> IResult<&str, &str> {
    take_till(|c: char| c == '%')(s)
}

fn param(input: &str) -> IResult<&str, Param> {
    let (input, (position, r#type)) =
        delimited(tag("%"), tuple((map_res(digit1, to_u8), alpha1)), tag("%"))(input)?;

    Ok((
        input,
        Param {
            r#type: r#type.to_string(),
            position,
        },
    ))
}

fn opcode(input: &str) -> IResult<&str, OpcodeDefinition> {
    let (input, id) = map_res(take_while_m_n(4, 4, is_hex_digit), from_hex)(input)?;

    let (input, num_params) =
        delimited(tag("="), map_res(alt((digit1, tag("-1"))), to_i8), tag(","))(input)?;

    let mut op = OpcodeDefinition {
        id,
        num_params,
        words: vec![],
        params: vec![],
    };

    // let (input, mut d) = many0(tuple((string_chunk, param)))(input)?;

    // for (word, param) in d.drain(..) {
    //     op.words.push(word.to_string());
    //     op.params.push(param);
    // }

    let (input, mut d) = many0(tuple((string_chunk, param)))(input)?;

    for (word, param) in d.drain(..) {
        op.words.push(word.to_string());
        op.params.push(param);
    }

    op.words.push(input.to_string());
    Ok((input, op))
}

#[test]
fn parse() {
    assert_eq!(
        opcode("0051=2,return %1dddd% wait %2x% last"),
        Ok((
            " last",
            OpcodeDefinition {
                id: 81,
                num_params: 2,
                words: vec![
                    String::from("return "),
                    String::from(" wait "),
                    String::from(" last")
                ],
                params: vec![
                    Param {
                        position: 1,
                        r#type: String::from("dddd")
                    },
                    Param {
                        position: 2,
                        r#type: String::from("x")
                    }
                ]
            }
        ))
    );
}
