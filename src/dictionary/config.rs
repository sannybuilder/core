use super::ffi::{CaseFormat, Duplicates};

#[derive(Debug, Clone)]
pub struct Config {
    pub duplicates: Duplicates,
    pub case_format: CaseFormat,
    pub comments: String,
    pub delimiters: String,
    pub strip_whitespace: bool,
    pub hex_keys: bool,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder(Config::default())
    }

    pub fn set_duplicates<'a>(&'a mut self, duplicates: Duplicates) -> &'a mut ConfigBuilder {
        self.0.duplicates = duplicates;
        self
    }

    pub fn set_case_format<'a>(&'a mut self, case_format: CaseFormat) -> &'a mut ConfigBuilder {
        self.0.case_format = case_format;
        self
    }

    pub fn set_comments<'a>(&'a mut self, comments: String) -> &'a mut ConfigBuilder {
        self.0.comments = comments;
        self
    }

    pub fn set_delimiters<'a>(&'a mut self, delimiters: String) -> &'a mut ConfigBuilder {
        self.0.delimiters = delimiters;
        self
    }

    pub fn set_strip_whitespace<'a>(&'a mut self, strip_whitespace: bool) -> &'a mut ConfigBuilder {
        self.0.strip_whitespace = strip_whitespace;
        self
    }

    pub fn set_hex_keys<'a>(&'a mut self, hex_keys: bool) -> &'a mut ConfigBuilder {
        self.0.hex_keys = hex_keys;
        self
    }

    pub fn build(&mut self) -> Config {
        self.0.clone()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            duplicates: Duplicates::Replace,
            case_format: CaseFormat::NoFormat,
            comments: String::from(";"),
            delimiters: String::from("=,"),
            strip_whitespace: true,
            hex_keys: false,
        }
    }
}
