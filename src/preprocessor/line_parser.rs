use std::collections::HashMap;

static CHARS_DIGIT: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
static CHARS_IDENTIFIER: [u8; 63] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_', b'a', b'b', b'c', b'd', b'e',
    b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u',
    b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K',
    b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
];

const CHARS_MODEL: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_@";
const CHARS_VAR: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_&";

static CHARS_HEX: [u8; 22] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    b'A', b'B', b'C', b'D', b'E', b'F',
];
static CHARS_BIN: [u8; 2] = [b'0', b'1'];
const CHARS_WHITESPACE: &[u8] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31, 32,
];

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Eol,
    Ident,
    Int,
    HexNumber,
    BinNumber,
    Float,
    Eq,
    Plus,
    Minus,
    Mul,
    Div,
    PlusEq,
    MinusEq,
    MulEq,
    DivEq,
    PlusPlus,
    MinusMinus,
    EqEq,
    NotEq,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    EqCast,
    Colon,
    Comma,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    OpenCurly,
    CloseCurly,
    Directive,
    Global,
    GlobalString8,
    GlobalString16,
    Adma,
    Local,
    LocalString8,
    LocalString16,
    Model,
    Label,
    Period,
    VString,
    SString,
    // Hex,
    OpcodeId,
    StringUnterm,
    // FunctionCall,
    // Class,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum TokenVal {
    Eol,
    Unknown(u8),
    Ident(String),
    Int(i64),
    Float(f32),
    Punctuator,
}

type Handler = fn(&mut DataParser) -> Token;

#[derive(Default)]
pub struct DataParser {
    handlers: HashMap<u8, Handler>,
    line: Vec<u8>,
    current_char: usize,
    in_comment_curly: bool,
    in_comment_cpp: bool,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub val: TokenVal,
}

impl DataParser {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        for i in 0..=255 {
            match i {
                0 | 10 | 13 => handlers.insert(i, eof_proc as Handler),
                1..=9 | 11..=12 | 14..=32 => handlers.insert(i, space_proc),
                b'$' => handlers.insert(i, global_var_proc),
                b'&' => handlers.insert(i, adma_var_proc),
                b'(' => handlers.insert(i, open_brace_proc),
                b')' => handlers.insert(i, close_brace_proc),
                b'{' => handlers.insert(i, open_curly_proc),
                b'}' => handlers.insert(i, close_curly_proc),
                b'#' => handlers.insert(i, model_proc),
                b'[' => handlers.insert(i, open_bracket_proc),
                b']' => handlers.insert(i, close_bracket_proc),
                b'@' => handlers.insert(i, label_proc),
                b'"' => handlers.insert(i, v_string_proc), // todo! if SBOptionEnabled(eoSB3Compat)
                b'\'' => handlers.insert(i, s_string_proc), // todo! if SBOptionEnabled(eoSB3Compat)
                b'+' => handlers.insert(i, plus_proc),
                b'-' => handlers.insert(i, minus_proc),
                b'*' => handlers.insert(i, mul_proc),
                b'/' => handlers.insert(i, div_proc),
                b'>' => handlers.insert(i, greater_than_proc),
                b'<' => handlers.insert(i, less_than_proc),
                b'=' => handlers.insert(i, eq_proc),
                b'.' => handlers.insert(i, period_proc),
                b',' => handlers.insert(i, comma_proc),
                b'0'..=b'9' => handlers.insert(i, |p| opcode_or(p, integer_proc)),
                b'a'..=b'f' | b'A'..=b'F' => handlers.insert(i, |p| opcode_or(p, identifier_proc)),
                b's' => handlers.insert(i, s_proc),
                b'v' => handlers.insert(i, v_proc),
                b':' => handlers.insert(i, colon_proc),
                b'~' => handlers.insert(i, tilde_proc),
                b'_' | b'a'..=b'z' | b'A'..=b'Z' => handlers.insert(i, identifier_proc),
                _ => handlers.insert(i, unknown_proc),
            };
        }
        Self {
            handlers,
            line: Vec::new(),
            current_char: 0,
            in_comment_cpp: false,
            in_comment_curly: false,
            // current_char: std::ptr::null(),
        }
    }

    pub fn line(&mut self, line: &str) {
        self.line = line.as_bytes().to_vec();
        self.current_char = 0;
    }

    pub fn get_token(&mut self) -> Token {
        let char = self.this_char();
        self.handlers[&char](self)
    }

    fn eol(&mut self) -> Token {
        self.next();
        Token {
            token_type: TokenType::Eol,
            val: TokenVal::Eol,
        }
    }

    fn punctuator(&mut self, token_type: TokenType) -> Token {
        self.next();
        Token {
            token_type,
            val: TokenVal::Punctuator,
        }
    }

    fn unknown(&mut self) -> Token {
        Token {
            token_type: TokenType::Unknown,
            val: TokenVal::Unknown(self.this_char()),
        }
    }

    fn at(&mut self, n: usize) -> u8 {
        if n >= self.line.len() {
            0
        } else {
            self.line[n]
        }
    }

    fn read_char(&mut self) -> u8 {
        match self.at(self.current_char) {
            _ if self.in_comment_curly => {
                if self.skip_until(b"}") {
                    self.next(); // }
                    self.in_comment_curly = false;
                    return self.this_char(); // return char after comment
                } else {
                    0
                }
            }
            _ if self.in_comment_cpp => {
                loop {
                    match self.at(self.current_char) {
                        0 | 10 | 13 => return 0,
                        b'*' if self.at(self.current_char + 1) == b'/' => {
                            self.next_n(2); // */
                            self.in_comment_cpp = false;
                            return self.this_char(); // return char after comment
                        }
                        _ => self.next(),
                    }
                }
            }
            b'{' if self.at(self.current_char + 1) != b'$' => {
                self.next(); // {
                if self.skip_until(b"}") {
                    self.next(); // }
                    self.in_comment_curly = false;
                    return self.this_char(); // return char after comment
                } else {
                    self.in_comment_curly = true;
                    0
                }
            }
            b'/' if self.at(self.current_char + 1) == b'*' => {
                self.next_n(2); // /*
                loop {
                    match self.at(self.current_char) {
                        0 | 10 | 13 => {
                            self.in_comment_cpp = true;
                            return 0;
                        }
                        b'*' if self.at(self.current_char + 1) == b'/' => {
                            self.next_n(2); // */
                            self.in_comment_cpp = false;
                            return self.this_char(); // return char after comment
                        }
                        _ => {
                            self.next();
                        }
                    }
                }
            }
            x => x, // 0 goes here
        }
    }

    fn this_char(&mut self) -> u8 {
        self.read_char()
    }

    fn next(&mut self) {
        self.next_n(1)
    }

    fn next_n(&mut self, n: usize) {
        self.current_char += n;
    }

    fn peek(&mut self) -> u8 {
        self.peek_n(1)
    }

    fn peek_n(&mut self, n: usize) -> u8 {
        let current_char = self.current_char;
        self.next_n(n);
        let peek = self.this_char();
        self.current_char = current_char;
        peek
    }

    fn slice(&mut self, start: usize) -> String {
        let mut buf = vec![];
        let end = self.current_char;

        self.current_char = start; // restore the start position
        while self.current_char < end {
            buf.push(self.this_char());
            self.next();
        }

        String::from_utf8_lossy(&buf).to_string()
        // String::from_utf8_lossy(&self.line[start..self.current_char]).to_string()
    }

    fn get_while(&mut self, chars: &[u8]) -> String {
        let start = self.current_char;
        self.skip_while(chars);
        self.slice(start)
    }

    // fn get_until(&mut self, chars: &[u8]) -> String {
    //     let start = self.current_char;
    //     self.skip_until(chars);
    //     self.slice(start)
    // }

    fn get_while1(&mut self, chars: &[u8], token_type: TokenType) -> Token {
        let start = self.current_char;
        self.skip_while(chars);
        if start == self.current_char {
            self.unknown()
        } else {
            Token {
                token_type,
                val: TokenVal::Ident(self.slice(start)),
            }
        }
    }

    pub fn get_until1(&mut self, chars: &[u8], token_type: TokenType) -> Token {
        let start = self.current_char;
        self.skip_until(chars);
        if start == self.current_char {
            self.unknown()
        } else {
            Token {
                token_type,
                val: TokenVal::Ident(self.slice(start)),
            }
        }
    }

    fn skip_while(&mut self, chars: &[u8]) {
        while chars.contains(&self.this_char()) {
            self.next();
        }
    }

    fn skip_until(&mut self, chars: &[u8]) -> bool {
        let stop_chars = [0, 10, 13];
        loop {
            let c = self.at(self.current_char);
            if chars.contains(&c) {
                return true;
            }
            if stop_chars.contains(&c) {
                return false;
            }
            self.next();
        }
    }

    fn skip_while1(&mut self, chars: &[u8]) -> bool {
        let start = self.current_char;
        self.skip_while(chars);
        start != self.current_char
    }

    // fn skip_until1(&mut self, chars: &[u8]) -> bool {
    //     let start = self.current_char;
    //     self.skip_until(chars);
    //     start != self.current_char
    // }

    pub fn skip_whitespace(&mut self) {
        self.skip_while(&CHARS_WHITESPACE)
    }

    fn try_char(&mut self, c: &[u8]) -> bool {
        if c.contains(&self.this_char()) {
            self.next();
            true
        } else {
            false
        }
    }

    fn try_token(&mut self, tokens: &[TokenType]) -> Option<Token> {
        let cur_pos = self.current_char;
        let token = self.get_token();

        if tokens.contains(&token.token_type) {
            Some(token)
        } else {
            self.current_char = cur_pos;
            None
        }
    }

    pub fn current_loc(&self) -> (String, usize) {
        (
            String::from_utf8_lossy(&self.line).to_string(),
            self.current_char,
        )
    }
}

fn eof_proc(p: &mut DataParser) -> Token {
    p.eol()
}

fn space_proc(p: &mut DataParser) -> Token {
    p.skip_whitespace();
    p.get_token()
}

fn unknown_proc(p: &mut DataParser) -> Token {
    p.unknown()
}

fn global_var_proc(p: &mut DataParser) -> Token {
    p.next();
    p.get_while1(&CHARS_VAR, TokenType::Global)
}

fn adma_var_proc(p: &mut DataParser) -> Token {
    p.next();
    p.get_while1(&CHARS_DIGIT, TokenType::Adma)
}

fn open_brace_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::OpenBrace)
}

fn close_brace_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::CloseBrace)
}

fn open_curly_proc(p: &mut DataParser) -> Token {
    if p.peek() == b'$' {
        let start = p.current_char;
        p.next_n(2); // ${
        if p.skip_while1(&CHARS_IDENTIFIER) {
            Token {
                token_type: TokenType::Directive,
                val: TokenVal::Ident(p.slice(start)),
            }
        } else {
            p.unknown()
        }
    } else {
        p.in_comment_curly = true;
        p.get_token()
    }
}

fn close_curly_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::CloseCurly)
}

fn model_proc(p: &mut DataParser) -> Token {
    p.next(); // #
    p.get_while1(&CHARS_MODEL, TokenType::Model)
}

fn open_bracket_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::OpenBracket)
}

fn close_bracket_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::CloseBracket)
}

fn label_proc(p: &mut DataParser) -> Token {
    p.next(); // @
    p.get_while1(&CHARS_IDENTIFIER, TokenType::Label)
}

fn v_string_proc(p: &mut DataParser) -> Token {
    let s = grab_string_escaped(p, b'"');
    if p.try_char(b"\"") {
        Token {
            token_type: TokenType::VString,
            val: TokenVal::Ident(s),
        }
    } else {
        Token {
            token_type: TokenType::StringUnterm,
            val: TokenVal::Ident(s),
        }
    }
}

fn s_string_proc(p: &mut DataParser) -> Token {
    let s = grab_string_escaped(p, b'\'');
    if p.try_char(b"'") {
        Token {
            token_type: TokenType::SString,
            val: TokenVal::Ident(s),
        }
    } else {
        Token {
            token_type: TokenType::StringUnterm,
            val: TokenVal::Ident(s),
        }
    }
}

fn plus_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'+' => {
            p.next(); // +
            p.punctuator(TokenType::PlusPlus)
        }
        b'=' => {
            p.next(); // +
            p.punctuator(TokenType::PlusEq)
        }
        b'0'..=b'9' => {
            p.next(); // +
            integer_proc(p)
        }
        b'.' => {
            p.next(); // +
            period_proc(p)
        }
        _ => p.punctuator(TokenType::Plus),
    }
}

fn minus_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'-' => {
            p.next(); // -
            p.punctuator(TokenType::MinusMinus)
        }
        b'=' => {
            p.next(); // -
            p.punctuator(TokenType::MinusEq)
        }
        b'0'..=b'9' => {
            let start = p.current_char;
            p.next(); // -
            let token = integer_proc(p);
            Token {
                token_type: token.token_type,
                val: TokenVal::Ident(p.slice(start)),
            }
        }
        b'.' => {
            let start = p.current_char;
            p.next(); // -
            let token = period_proc(p);
            Token {
                token_type: token.token_type,
                val: TokenVal::Ident(p.slice(start)),
            }
        }
        _ => p.punctuator(TokenType::Minus),
    }
}

fn mul_proc(p: &mut DataParser) -> Token {
    if p.peek() == b'=' {
        p.next(); // *
        p.punctuator(TokenType::MulEq)
    } else {
        p.punctuator(TokenType::Mul)
    }
}

fn div_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'=' => {
            p.next(); // /
            p.punctuator(TokenType::DivEq)
        }
        b'/' => {
            // line ends here
            p.eol()
        }
        b'*' => {
            // block comment /*
            p.next(); // *
            p.in_comment_cpp = true;
            p.get_token()
        }
        _ => p.punctuator(TokenType::Div),
    }
}

fn greater_than_proc(p: &mut DataParser) -> Token {
    if p.peek() == b'=' {
        p.next(); // >
        p.punctuator(TokenType::GreaterThanEq)
    } else {
        p.punctuator(TokenType::GreaterThan)
    }
}

fn less_than_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'=' => {
            p.next(); // <
            p.punctuator(TokenType::LessThanEq)
        }
        b'>' => {
            p.next(); // <
            p.punctuator(TokenType::NotEq)
        }
        _ => p.punctuator(TokenType::LessThan),
    }
}

fn eq_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'=' => {
            p.next(); // =
            p.punctuator(TokenType::EqEq)
        }
        b'#' => {
            p.next(); // =
            p.punctuator(TokenType::EqCast)
        }
        _ => p.punctuator(TokenType::Eq),
    }
}

fn period_proc(p: &mut DataParser) -> Token {
    match p.peek() {
        b'0'..=b'9' => {
            let start = p.current_char;
            p.next(); // .
            p.skip_while(&CHARS_DIGIT);
            match p.this_char() {
                b'e' | b'E' => {
                    p.next(); // E
                    match p.this_char() {
                        b'+' | b'-' => p.next(),
                        _ => {}
                    }
                    if !CHARS_DIGIT.contains(&p.this_char()) {
                        return p.unknown();
                    }
                    p.skip_while(&CHARS_DIGIT);
                }
                _ => {}
            }
            Token {
                token_type: TokenType::Float,
                val: TokenVal::Ident(p.slice(start)),
            }
        }
        _ => p.punctuator(TokenType::Period),
    }
}

fn s_proc(p: &mut DataParser) -> Token {
    if p.peek() == b'$' {
        p.next_n(2); // skip s$
        p.get_while1(&CHARS_IDENTIFIER, TokenType::GlobalString8)
    } else {
        Token {
            token_type: TokenType::Ident,
            val: TokenVal::Ident(p.get_while(&CHARS_IDENTIFIER)),
        }
    }
}

fn v_proc(p: &mut DataParser) -> Token {
    if p.peek() == b'$' {
        p.next_n(2); // skip v$
        p.get_while1(&CHARS_IDENTIFIER, TokenType::GlobalString16)
    } else {
        Token {
            token_type: TokenType::Ident,
            val: TokenVal::Ident(p.get_while(&CHARS_IDENTIFIER)),
        }
    }
}

fn comma_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::Comma)
}

fn opcode_or(p: &mut DataParser, other_handler: Handler) -> Token {
    // 0..9, A-F
    if CHARS_HEX.contains(&p.peek_n(1))
        && CHARS_HEX.contains(&p.peek_n(2))
        && CHARS_HEX.contains(&p.peek_n(3))
        && p.peek_n(4) == b':'
    {
        let start = p.current_char;
        p.next_n(5); // 0000:
        Token {
            token_type: TokenType::OpcodeId,
            val: TokenVal::Ident(p.slice(start)),
        }
    } else {
        other_handler(p)
    }
}

fn integer_proc(p: &mut DataParser) -> Token {
    let start = p.current_char;
    let next_char = p.peek();
    let mut token_type;

    match (p.this_char(), next_char) {
        (b'0', b'x') | (b'0', b'X') => {
            p.next_n(2); // skip 0x
                         // can be empty?
            if !p.skip_while1(&CHARS_HEX) {
                return p.unknown();
            }
            Token {
                token_type: TokenType::HexNumber,
                val: TokenVal::Ident(p.slice(start)),
            }
        }

        (b'0', b'b') | (b'0', b'B') => {
            p.next_n(2); // skip 0b
            if !p.skip_while1(&CHARS_BIN) {
                return p.unknown();
            }
            Token {
                token_type: TokenType::BinNumber,
                val: TokenVal::Ident(p.slice(start)),
            }
        }
        _ => {
            p.skip_while(&CHARS_DIGIT);

            match p.this_char() {
                b'@' => {
                    let val = TokenVal::Ident(p.slice(start));
                    p.next(); // @

                    if p.try_char(b"s") {
                        // @s
                        return Token {
                            token_type: TokenType::LocalString8,
                            val,
                        };
                    }

                    if p.try_char(b"v") {
                        // @v
                        return Token {
                            token_type: TokenType::LocalString16,
                            val,
                        };
                    }

                    return Token {
                        // @
                        token_type: TokenType::Local,
                        val,
                    };
                }
                b'.' => {
                    p.next(); // .

                    if !p.skip_while1(&CHARS_DIGIT) {
                        return p.unknown();
                    }
                    token_type = TokenType::Float;
                }
                _ => {
                    token_type = TokenType::Int;
                    // fall down
                }
            }

            //  100E4, -100.0E4, -100E-2, -100.0E-2
            if p.this_char() == b'E' || p.this_char() == b'e' {
                token_type = TokenType::Float;
                p.next(); // E
                if p.this_char() == b'+' || p.this_char() == b'-' {
                    p.next();
                }
                if !CHARS_DIGIT.contains(&p.this_char()) {
                    return p.unknown();
                }

                p.skip_while(&CHARS_DIGIT);
            }

            Token {
                token_type,
                val: TokenVal::Ident(p.slice(start)),
            }
        }
    }
}

fn colon_proc(p: &mut DataParser) -> Token {
    p.punctuator(TokenType::Colon)
}

fn tilde_proc(p: &mut DataParser) -> Token {
    p.next(); // ~
              // todo: check whitespace?

    p.try_token(&[TokenType::Global, TokenType::Local])
        .unwrap_or_else(|| Token {
            token_type: TokenType::Unknown,
            val: TokenVal::Unknown(b'~'),
        })
}

fn identifier_proc(p: &mut DataParser) -> Token {
    Token {
        token_type: TokenType::Ident,
        val: TokenVal::Ident(p.get_while(&CHARS_IDENTIFIER)),
    }
}

fn grab_string_escaped(p: &mut DataParser, terminator: u8) -> String {
    let mut buf = Vec::new();
    let mut this_char: u8;

    p.next(); // skip the start quote (" or ')
    loop {
        this_char = p.this_char();

        if [0, 10, 13, terminator].contains(&this_char) {
            break;
        }

        if p.try_char(b"\\") {
            match p.this_char() {
                0 | 10 | 13 => break,
                b'0' => buf.push(0),
                b'b' => buf.push(8),
                b't' => buf.push(9),
                b'n' => buf.push(10),
                b'r' => buf.push(13),
                b'x' => {
                    p.next(); // skip x
                    let a = p.this_char();
                    let a_num = match a {
                        b'0'..=b'9' => a - b'0',
                        b'a'..=b'f' => a - b'a' + 10,
                        b'A'..=b'F' => a - b'A' + 10,
                        _ => break,
                    };

                    p.next(); // skip the first digit

                    let b = p.this_char();
                    let b_num = match b {
                        b'0'..=b'9' => b - b'0',
                        b'a'..=b'f' => b - b'a' + 10,
                        b'A'..=b'F' => b - b'A' + 10,
                        _ => break,
                    };

                    buf.push((a_num << 4) + b_num);
                }
                b'\\' => buf.push(b'\\'),
                x => buf.push(x),
            }
        } else {
            buf.push(this_char);
        }
        p.next();
    }

    String::from_utf8_lossy(&buf).to_string()
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn test_empty_line() {
        let mut parser = DataParser::new();
        let line = "  ";
        parser.current_char = line.as_ptr() as _;

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_punctuators() {
        let mut parser = DataParser::new();
        parser.line(": [] {} = == =# ,- -- -= . > < <>");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Colon);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::OpenBracket);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::CloseBracket);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eq);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::EqEq);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::EqCast);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Comma);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Minus);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::MinusMinus);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::MinusEq);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Period);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::GreaterThan);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::LessThan);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::NotEq);
        assert_eq!(token.val, TokenVal::Punctuator);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_vars() {
        let mut parser = DataParser::new();
        parser.line("$global &100 s$var v$103");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Global);
        assert_eq!(token.val, TokenVal::Ident("global".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Adma);
        assert_eq!(token.val, TokenVal::Ident("100".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::GlobalString8);
        assert_eq!(token.val, TokenVal::Ident("var".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::GlobalString16);
        assert_eq!(token.val, TokenVal::Ident("103".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_identifiers() {
        let mut parser = DataParser::new();
        parser.line("abc DE_F sanny vvv");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Ident);
        assert_eq!(token.val, TokenVal::Ident("abc".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Ident);
        assert_eq!(token.val, TokenVal::Ident("DE_F".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Ident);
        assert_eq!(token.val, TokenVal::Ident("sanny".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Ident);
        assert_eq!(token.val, TokenVal::Ident("vvv".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_directives() {
        let mut parser = DataParser::new();
        parser.line("{$include a.txt}");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Directive);
        assert_eq!(token.val, TokenVal::Ident("{$include".to_string()));

        parser.line("{$cleo}");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Directive);
        assert_eq!(token.val, TokenVal::Ident("{$cleo".to_string()));
    }

    #[test]
    fn test_model() {
        let mut parser = DataParser::new();
        parser.line("#model #023@egg");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Model);
        assert_eq!(token.val, TokenVal::Ident("model".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Model);
        assert_eq!(token.val, TokenVal::Ident("023@egg".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_label() {
        let mut parser = DataParser::new();
        parser.line(" @label ");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Label);
        assert_eq!(token.val, TokenVal::Ident("label".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_tilde() {
        let mut parser = DataParser::new();
        parser.line("~$var ~10@ ~");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Global);
        assert_eq!(token.val, TokenVal::Ident("var".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Local);
        assert_eq!(token.val, TokenVal::Ident("10".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Unknown);
        assert_eq!(token.val, TokenVal::Unknown(b'~'));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_numbers() {
        let mut parser = DataParser::new();
        parser.line("0 0x10 0b10 100 100.0 100E4 -100.0E4 -100E-2 -100.0E-2 .1 .1E4 -.0E-4 .0E+10");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("0".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::HexNumber);
        assert_eq!(token.val, TokenVal::Ident("0x10".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::BinNumber);
        assert_eq!(token.val, TokenVal::Ident("0b10".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("100".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("100.0".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("100E4".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("-100.0E4".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("-100E-2".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("-100.0E-2".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident(".1".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident(".1E4".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident("-.0E-4".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Float);
        assert_eq!(token.val, TokenVal::Ident(".0E+10".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_strings() {
        let mut parser = DataParser::new();
        parser.line(r#" "hello" "\0\t\n\\" "\x20\x4A" 'abc' "world "#);

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::VString);
        assert_eq!(token.val, TokenVal::Ident("hello".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::VString);
        assert_eq!(token.val, TokenVal::Ident("\0\t\n\\".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::VString);
        assert_eq!(token.val, TokenVal::Ident(" J".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::SString);
        assert_eq!(token.val, TokenVal::Ident("abc".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::StringUnterm);
        assert_eq!(token.val, TokenVal::Ident("world ".to_string()));

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);
    }

    #[test]
    fn test_comments() {
        let mut parser = DataParser::new();

        parser.line(" /* comment */ 1");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("1".to_string()));
        parser.line(" // comment");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);

        parser.line(" {comment}123");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("123".to_string()));

        parser.line(" /* comment */");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert_eq!(token.val, TokenVal::Eol);

        parser.line(" /* comment */ 1");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("1".to_string()));

        parser.line(" {comment}1{}2{}3");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("123".to_string()));

        parser.line(" {comment}1/*555*/2{}3");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.val, TokenVal::Ident("123".to_string()));

        parser.in_comment_curly = false;
        parser.in_comment_cpp = false;
        parser.line("{");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert!(parser.in_comment_curly);

        parser.in_comment_curly = false;
        parser.in_comment_cpp = false;
        parser.line("/*");

        let token = parser.get_token();
        assert_eq!(token.token_type, TokenType::Eol);
        assert!(parser.in_comment_cpp);
    }
}
