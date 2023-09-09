use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case};
use nom::character::complete::{alpha1, char};
use nom::combinator::{not, rest};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{
    character::complete::{digit1, hex_digit1},
    combinator::{map, opt, recognize},
    sequence::tuple,
    IResult,
};
use nom_locate::LocatedSpan;
use std::collections::HashMap;
use std::ffi::CString;

type Span<'a> = LocatedSpan<&'a str>;
type R<'a, T> = IResult<Span<'a>, T>;

#[derive(Debug, Default)]
pub struct Opcode {
    num_params: i8,
    params: HashMap<usize, Param>,
    words: HashMap<usize, CString>,
}

#[derive(Debug)]
pub struct Param {
    real_index: u8,
    param_type: ParamType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ParamType {
    Any = 0,
    Gxt = 2,
    Pointer = 3,
    AnyModel = 4,
    ScriptId = 5,
    String8 = 6, // only lcs
    IdeModel = 7,
    Byte128 = 8, // only SA
}

#[derive(Debug)]
pub enum Game {
    GTA3,
    VC,
    SA,
    LCS,
    VCS,
    SAMOBILE,
}

impl From<u8> for Game {
    fn from(game: u8) -> Self {
        match game {
            0 => Game::GTA3,
            1 => Game::VC,
            2 => Game::SA,
            3 => Game::LCS,
            4 => Game::VCS,
            5 => Game::SAMOBILE,
            _ => {
                log::error!("Unknown game: {game}, using SA as the default value");
                Game::SA
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct OpcodeTable {
    opcodes: HashMap<u16, Opcode>,
    files: Vec<String>,
    date: Option<String>,
    publisher: Option<String>,
    max_opcode: u16,
    max_params: u8,
}

#[derive(Debug)]
enum Token {
    Param(Param),
    Word(String),
}

pub enum Line {
    Date(String),
    Publisher(String),
    Opcode(u16, Opcode),
    Section(String),
}

pub fn line_parser(s: &str) -> R<Line> {
    alt((date_line, publisher_line, opcode_line, section_line))(Span::from(s))
}

pub fn date_line(s: Span) -> R<Line> {
    map(
        preceded(tag_no_case("DATE="), rest),
        |s: LocatedSpan<&str>| Line::Date(s.to_string()),
    )(s)
}

pub fn publisher_line(s: Span) -> R<Line> {
    map(
        preceded(tag_no_case("PUBLISHER="), rest),
        |s: LocatedSpan<&str>| Line::Publisher(s.to_string()),
    )(s)
}

pub fn section_line(s: Span) -> R<Line> {
    map(
        delimited(char('['), is_not("]"), char(']')),
        |s: LocatedSpan<&str>| Line::Section(s.to_string()),
    )(s)
}

pub fn opcode_line(s: Span) -> R<Line> {
    map(
        tuple((
            terminated(hexadecimal_span, char('=')),
            terminated(num_params, char(',')),
            many0(token),
        )),
        |(id, num_params, tokens)| {
            let mut index: usize = 0;
            let mut params = HashMap::new();
            let mut words: HashMap<usize, CString> = HashMap::new();

            for tok in tokens {
                match tok {
                    Token::Param(p) => {
                        params.insert(index, p);
                        index += 1
                    }
                    Token::Word(w) => {
                        words.insert(index, CString::new(w).unwrap());
                    }
                }
            }

            if words.is_empty() {
                words.insert(0, CString::new("(unknown) ").unwrap());
            }

            if params.len() < num_params as usize {
                for i in params.len().max(1) as i8..num_params {
                    // log::debug!("{id} Adding empty words");
                    log::debug!("{id} Adding empty param {}", i);
                    words.insert(i as usize, CString::new(" ").unwrap());
                }
            }

            Line::Opcode(
                id,
                Opcode {
                    num_params,
                    params,
                    words,
                },
            )
        },
    )(s)
}

fn token(s: Span) -> R<Token> {
    alt((param, bool_param, word))(s)
}

fn word(s: Span) -> R<Token> {
    map(
        recognize(tuple((
            is_not("%"),
            opt(tuple((char('%'), not(digit1), opt(is_not("%"))))),
        ))),
        |s: LocatedSpan<&str>| Token::Word(s.to_string()),
    )(s)
}

fn param(s: Span) -> R<Token> {
    map(
        delimited(char('%'), pair(digit1, alpha1), char('%')),
        |(d, a): (LocatedSpan<&str>, LocatedSpan<&str>)| {
            Token::Param(Param {
                real_index: u8::from_str_radix(*d, 10).unwrap() - 1,
                param_type: match *a {
                    "g" | "z" => ParamType::Gxt,
                    "p" => ParamType::Pointer,
                    "o" | "t" => ParamType::AnyModel,
                    "x" => ParamType::ScriptId,
                    "s" => ParamType::String8, // only lcs
                    "m" => ParamType::IdeModel,
                    "k" => ParamType::Byte128, // only sa
                    _ => ParamType::Any,
                },
            })
        },
    )(s)
}

fn bool_param(s: Span) -> R<Token> {
    map(
        delimited(char('%'), terminated(digit1, tag("b")), is_not(" ")),
        |d: LocatedSpan<&str>| {
            Token::Param(Param {
                real_index: u8::from_str_radix(*d, 10).unwrap() - 1,
                param_type: ParamType::Any,
            })
        },
    )(s)
}

fn hexadecimal_span(s: Span) -> R<u16> {
    map(recognize(hex_digit1), |s: LocatedSpan<&str>| {
        u16::from_str_radix(*s, 16).unwrap()
    })(s)
}

fn num_params(s: Span) -> R<i8> {
    map(
        recognize(tuple((opt(char('-')), decimal_span))),
        |s: LocatedSpan<&str>| i8::from_str_radix(*s, 10).unwrap(),
    )(s)
}

pub fn decimal_span(s: Span) -> R<Span> {
    recognize(digit1)(s)
}

impl OpcodeTable {
    pub fn new(game: Game) -> Self {
        OpcodeTable {
            max_params: get_game_limit(game),
            ..Default::default()
        }
    }

    pub fn add_opcode(&mut self, id: u16, opcode: Opcode) {
        if id > self.max_opcode {
            self.max_opcode = id;
        }
        self.opcodes.insert(id, opcode);
    }

    pub fn load_from_file(&mut self, file_name: &str) -> bool {
        if self.files.iter().any(|f| f.eq_ignore_ascii_case(file_name)) {
            return false;
        }
        self.files.push(file_name.to_string());
        let content = std::fs::read_to_string(file_name).unwrap();
        for line in content.lines() {
            self.parse_line(line);
        }
        return true;
    }

    pub fn parse_line(&mut self, line: &str) {
        let line = line.trim();
        let line = line.replace(";;", "//");
        let line = line.replace(";", "//");

        if line.starts_with("//") || line.is_empty() {
            return;
        }
        match line_parser(&line) {
            Ok((_, Line::Opcode(id, opcode))) => {
                if opcode.num_params > 0 && opcode.params.len() as i8 != opcode.num_params {
                    log::error!(
                        "Invalid number of params for opcode {:04X}: expected {}, found {}",
                        id,
                        opcode.num_params,
                        opcode.params.len()
                    );
                }
                self.add_opcode(id, opcode);
            }
            Ok((_, Line::Date(date))) => {
                self.date = Some(date);
            }
            Ok((_, Line::Publisher(publisher))) => {
                self.publisher = Some(publisher);
            }
            Ok((_, Line::Section(_))) => {
                // ignore section
            }
            Err(e) => {
                log::error!("{e}");
            }
        }
    }

    fn get_opcode(&self, id: u16) -> Option<&Opcode> {
        self.opcodes.get(&id)
    }

    pub fn get_params_count(&self, id: u16) -> u8 {
        self.get_opcode(id)
            .map(|opcode| {
                if opcode.num_params < 0 {
                    self.max_params + 1
                } else {
                    opcode.num_params as u8
                }
            })
            .unwrap_or(0)
    }

    pub fn is_variadic_opcode(&self, id: u16) -> bool {
        self.get_opcode(id)
            .map(|opcode| opcode.num_params < 0)
            .unwrap_or(false)
    }

    pub fn get_param_real_index(&self, id: u16, index: usize) -> u8 {
        self.get_opcode(id)
            .and_then(|opcode| opcode.params.get(&index))
            .map(|x| x.real_index)
            .unwrap_or(index as u8)
    }

    pub fn get_param_type(&self, id: u16, index: usize) -> ParamType {
        self.get_opcode(id)
            .and_then(|opcode| opcode.params.get(&index))
            .map(|x| x.param_type)
            .unwrap_or(ParamType::Any)
    }

    pub fn does_word_exist(&self, id: u16, index: usize) -> bool {
        self.get_word(id, index).is_some()
    }

    pub fn get_word(&self, id: u16, index: usize) -> Option<&CString> {
        self.get_opcode(id)
            .and_then(|opcode| opcode.words.get(&index))
    }

    pub fn get_max_opcode(&self) -> u16 {
        self.max_opcode
    }

    pub fn get_publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    pub fn get_date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    pub fn len(&self) -> usize {
        self.opcodes.len()
    }
}

fn get_game_limit(game: Game) -> u8 {
    match game {
        Game::GTA3 => 16 + 2,
        Game::VC => 16 + 2,
        Game::SA => 32 + 2,
        Game::LCS => 96 + 2,
        Game::VCS => 96 + 2,
        Game::SAMOBILE => 40 + 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_sa() {
        let opcode_table = OpcodeTable::new(Game::SA);
        assert_eq!(opcode_table.max_params, 34);
    }

    #[test]
    fn test_nop() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0000=0,NOP");
        let id = 0x0000;

        assert_eq!(opcode_table.get_params_count(id), 0);
        assert!(opcode_table.does_word_exist(id, 0));
        assert_eq!(opcode_table.does_word_exist(id, 1), false);
        assert_eq!(
            opcode_table.get_word(id, 0),
            Some(&CString::new("NOP").unwrap())
        );
    }

    #[test]
    fn test_0001() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0001=1,wait %1d% ms");
        let id = 0x0001;

        assert_eq!(opcode_table.get_params_count(id), 1);
        assert!(opcode_table.does_word_exist(id, 0));
        assert!(opcode_table.does_word_exist(id, 1));
        assert_eq!(
            opcode_table.get_word(id, 0),
            Some(&CString::new("wait ").unwrap())
        );
        assert_eq!(
            opcode_table.get_word(id, 1),
            Some(&CString::new(" ms").unwrap())
        );
    }

    #[test]
    fn test_004f() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("004F=-1,start_new_script %1p%");
        let id = 0x004F;

        assert_eq!(opcode_table.get_params_count(id), 35);
        assert!(opcode_table.does_word_exist(id, 0));
        assert_eq!(
            opcode_table.get_word(id, 0),
            Some(&CString::new("start_new_script ").unwrap())
        );
    }

    #[test]
    fn test_real_index() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0053=5,%5d% = create_player %1d% at %2d% %3d% %4d%");

        let id = 0x0053;
        assert_eq!(opcode_table.get_params_count(id), 5);
        assert_eq!(opcode_table.get_param_real_index(id, 0), 4);
        assert_eq!(opcode_table.get_param_real_index(id, 1), 0);
        assert_eq!(opcode_table.get_param_real_index(id, 2), 1);
        assert_eq!(opcode_table.get_param_real_index(id, 3), 2);
        assert_eq!(opcode_table.get_param_real_index(id, 4), 3);
    }

    #[test]
    fn test_bool_param() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line(
            "00E3=6,  player %1d% %6b:in-sphere/%near_point %2d% %3d% radius %4d% %5d%",
        );

        let id = 0x00E3;
        assert_eq!(opcode_table.get_params_count(id), 6);
        assert_eq!(opcode_table.get_param_real_index(id, 0), 0);
        assert_eq!(opcode_table.get_param_real_index(id, 1), 5);
        assert_eq!(opcode_table.get_param_real_index(id, 2), 1);
        assert_eq!(opcode_table.get_param_real_index(id, 3), 2);
        assert_eq!(opcode_table.get_param_real_index(id, 4), 3);
        assert_eq!(opcode_table.get_param_real_index(id, 5), 4);
    }

    #[test]
    fn test_percentage() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0B1B=2,%1d% %= %2d%");
        let id = 0x0B1B;

        assert_eq!(opcode_table.get_params_count(id), 2);
        assert!(opcode_table.does_word_exist(id, 1));
        assert_eq!(
            opcode_table.get_word(id, 1),
            Some(&CString::new(" %= ").unwrap())
        );
    }

    #[test]
    fn test_percentage2() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0B1B=1,%1d% 100%");
        let id = 0x0B1B;

        assert_eq!(opcode_table.get_params_count(id), 1);
        assert!(opcode_table.does_word_exist(id, 1));
        assert_eq!(
            opcode_table.get_word(id, 1),
            Some(&CString::new(" 100%").unwrap())
        );
    }

    #[test]
    fn test_unary_() {
        let mut opcode_table = OpcodeTable::new(Game::SA);
        opcode_table.parse_line("0B1A=1,~%1d%");
        let id = 0x0B1A;

        assert_eq!(opcode_table.get_params_count(id), 1);
        assert!(opcode_table.does_word_exist(id, 0));
        assert_eq!(
            opcode_table.get_word(id, 0),
            Some(&CString::new("~").unwrap())
        );
    }
}
