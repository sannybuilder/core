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

    pub fn load_file<'a>(&mut self, file_name: &'a str) -> Option<()> {
        let content = std::fs::read_to_string(file_name).ok()?;
        self.parse_file(content)
    }

    pub fn parse_file<'a>(&mut self, content: String) -> Option<()> {
        let comments = self.comments.clone();
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
            .filter(|line| !(line.is_empty() || line.starts_with(&comments)));

        for line in lines {
            let v: Vec<&str> = line
                .split_terminator(|c| self.delimiters.contains(c))
                .map(|v| v.trim())
                .collect();

            if v.len() != 2 {
                continue;
            }

            self.add_raw(v[0], v[1]);
        }
        Some(())
    }

    pub fn add_raw(&mut self, key: &str, value: &str) -> Option<()> {
        let (key, value) = <(T, U)>::get_key_value(key, value, self.hex_keys, &self.case_format)?;
        self.add(key, value);
        Some(())
    }

    pub fn add(&mut self, key: T, value: U) {
        if self.should_add(&key) {
            self.map.insert(key, value);
        }
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
        let key = apply_format(v0, &CaseFormat::LowerCase)?;
        let value = apply_format(v1, case_format)?;
        Some((key, value))
    }
}

impl KeyValue for (i32, CString) {
    fn get_key_value(v0: &str, v1: &str, hex_keys: bool, case_format: &CaseFormat) -> Option<Self> {
        let key = parse_number(v0, hex_keys)?;
        let value = apply_format(v1, case_format)?;
        Some((key, value))
    }
}

impl KeyValue for (CString, i32) {
    fn get_key_value(v0: &str, v1: &str, hex_keys: bool, case_format: &CaseFormat) -> Option<Self> {
        let key = apply_format(v1, case_format)?;
        let value = parse_number(v0, hex_keys)?;
        Some((key, value))
    }
}

pub fn apply_format(s: &str, case_format: &CaseFormat) -> Option<CString> {
    let value = match case_format {
        CaseFormat::LowerCase => s.to_ascii_lowercase(),
        CaseFormat::UpperCase => s.to_ascii_uppercase(),
        CaseFormat::NoFormat => String::from(s),
    };
    CString::new(value).ok()
}

fn parse_number(s: &str, hex: bool) -> Option<i32> {
    if hex {
        i32::from_str_radix(s, 16).ok()
    } else {
        s.parse::<i32>().ok()
    }
}

#[derive(Debug, PartialEq)]
pub enum Duplicates {
    Replace, // 0
    Ignore,  // 1
}

impl From<u8> for Duplicates {
    fn from(i: u8) -> Self {
        match i {
            0 => Duplicates::Replace,
            _ => Duplicates::Ignore,
        }
    }
}

impl From<Duplicates> for u8 {
    fn from(f: Duplicates) -> Self {
        match f {
            Duplicates::Replace => 0,
            Duplicates::Ignore => 1,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum CaseFormat {
    NoFormat,  // 0
    UpperCase, // 1
    LowerCase, // 2
}

impl From<u8> for CaseFormat {
    fn from(i: u8) -> Self {
        match i {
            1 => CaseFormat::UpperCase,
            2 => CaseFormat::LowerCase,
            _ => CaseFormat::NoFormat,
        }
    }
}

impl From<CaseFormat> for u8 {
    fn from(f: CaseFormat) -> Self {
        match f {
            CaseFormat::NoFormat => 0,
            CaseFormat::UpperCase => 1,
            CaseFormat::LowerCase => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_case_format() {
        assert_eq!(CaseFormat::from(0), CaseFormat::NoFormat);
        assert_eq!(CaseFormat::from(1), CaseFormat::UpperCase);
        assert_eq!(CaseFormat::from(2), CaseFormat::LowerCase);

        assert_eq!(u8::from(CaseFormat::NoFormat), 0);
        assert_eq!(u8::from(CaseFormat::UpperCase), 1);
        assert_eq!(u8::from(CaseFormat::LowerCase), 2);
    }

    #[test]
    fn test_duplicates() {
        assert_eq!(Duplicates::from(0), Duplicates::Replace);
        assert_eq!(Duplicates::from(1), Duplicates::Ignore);

        assert_eq!(u8::from(Duplicates::Replace), 0);
        assert_eq!(u8::from(Duplicates::Ignore), 1);
    }
}
