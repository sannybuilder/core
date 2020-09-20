#![cfg(windows)]

extern crate libc;
use std::collections::HashMap;
use std::ffi::CString;

pub struct Dict<T, U> {
    pub map: HashMap<T, U>,
    pub duplicates: Duplicates,
    pub case_format: CaseFormat,
    pub comments: String,
    pub delimiters: String,
    pub strip_whitespace: bool,
    pub hex_keys: bool,
}

impl<T, U> Dict<T, U>
where
    T: std::cmp::Eq + std::hash::Hash,
    (T, U): KeyValue,
{
    pub fn new(
        duplicates: Duplicates,
        case_format: CaseFormat,
        comments: String,
        delimiters: String,
        strip_whitespace: bool,
        hex_keys: bool,
    ) -> Self {
        Self {
            map: HashMap::new(),
            duplicates,
            case_format,
            comments,
            delimiters,
            strip_whitespace,
            hex_keys,
        }
    }

    pub fn should_add(&self, key: &T) -> bool {
        match self.duplicates {
            Duplicates::Replace => true,
            Duplicates::Ignore => match self.map.get(key) {
                Some(_) => false,
                None => true,
            },
        }
    }

    pub fn load_file<'a>(&mut self, file_name: &'a str) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(file_name)?;
        self.parse_file(content)
    }

    pub fn parse_file<'a>(&mut self, content: String) -> Result<(), std::io::Error> {
        let comments = self.comments.as_str();
        let strip = self.strip_whitespace;
        let lines = content
            .lines()
            .map(|line| {
                let mut line = String::from(line);
                if strip {
                    line.retain(|c| !c.is_whitespace());
                }
                line
            })
            .filter(|line| !(line.is_empty() || line.starts_with(comments)));

        for line in lines {
            let v: Vec<&str> = line
                .split_terminator(|c| self.delimiters.contains(c))
                .map(|v| v.trim())
                .collect();

            if v.len() != 2 {
                continue;
            }

            match <(T, U)>::get_key_value(v[0], v[1], self.hex_keys, &self.case_format) {
                Some((key, value)) => {
                    if self.should_add(&key) {
                        self.map.insert(key, value);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub trait KeyValue {
    fn get_key_value(v0: &str, v1: &str, hex_keys: bool, case_format: &CaseFormat) -> Option<Self>
    where
        Self: std::marker::Sized;
}

impl KeyValue for (CString, CString) {
    fn get_key_value(
        v0: &str,
        v1: &str,
        _hex_keys: bool,
        case_format: &CaseFormat,
    ) -> Option<Self> {
        if let Ok(key) = apply_format(v0, &CaseFormat::LowerCase) {
            if let Ok(value) = apply_format(v1, case_format) {
                return Some((key, value));
            }
        }

        return None;
    }
}

impl KeyValue for (i32, CString) {
    fn get_key_value(v0: &str, v1: &str, hex_keys: bool, case_format: &CaseFormat) -> Option<Self> {
        if let Ok(key) = parse_number(v0, hex_keys) {
            if let Ok(value) = apply_format(v1, case_format) {
                return Some((key, value));
            }
        }

        return None;
    }
}

impl KeyValue for (CString, i32) {
    fn get_key_value(v0: &str, v1: &str, hex_keys: bool, case_format: &CaseFormat) -> Option<Self> {
        if let Ok(key) = apply_format(v1, case_format) {
            if let Ok(value) = parse_number(v0, hex_keys) {
                return Some((key, value));
            }
        }
        return None;
    }
}

fn apply_format(s: &str, case_format: &CaseFormat) -> Result<CString, std::ffi::NulError> {
    let value = match case_format {
        CaseFormat::LowerCase => s.to_ascii_lowercase(),
        CaseFormat::UpperCase => s.to_ascii_uppercase(),
        CaseFormat::NoFormat => String::from(s),
    };
    CString::new(value)
}

fn parse_number(s: &str, hex: bool) -> std::result::Result<i32, std::num::ParseIntError> {
    if hex {
        i32::from_str_radix(s, 16)
    } else {
        s.parse::<i32>()
    }
}

pub enum Duplicates {
    Replace,
    Ignore,
}

impl From<u8> for Duplicates {
    fn from(i: u8) -> Self {
        match i {
            1 => Duplicates::Replace,
            _ => Duplicates::Ignore,
        }
    }
}

impl Into<u8> for Duplicates {
    fn into(self) -> u8 {
        match self {
            Duplicates::Ignore => 0,
            Duplicates::Replace => 1,
        }
    }
}

pub enum CaseFormat {
    NoFormat,
    UpperCase,
    LowerCase,
}

impl From<u8> for CaseFormat {
    fn from(i: u8) -> Self {
        match i {
            1 => CaseFormat::LowerCase,
            2 => CaseFormat::UpperCase,
            _ => CaseFormat::NoFormat,
        }
    }
}

impl Into<u8> for CaseFormat {
    fn into(self) -> u8 {
        match self {
            CaseFormat::NoFormat => 0,
            CaseFormat::LowerCase => 1,
            CaseFormat::UpperCase => 2,
        }
    }
}
