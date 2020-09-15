use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub details: String,
}

impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        ParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::convert::From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        Self {
            details: e.to_string(),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.details
    }
}
