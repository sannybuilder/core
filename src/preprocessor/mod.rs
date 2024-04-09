use anyhow::{bail, Result};
use lines_lossy::LinesLossyExt;
use std::{
    collections::HashSet,
    ffi::CString,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use self::line_parser::{TokenType, TokenVal};
use crate::{
    dictionary::{config, ffi::CaseFormat, DictNumByString},
    preprocessor::scopes::Function,
    utils::{
        compiler_const::{
            TOKEN_END, TOKEN_FOR, TOKEN_FUNCTION, TOKEN_HEX, TOKEN_IF, TOKEN_INCLUDE,
            TOKEN_INCLUDE_ONCE, TOKEN_SWITCH, TOKEN_WHILE,
        },
        path::{normalize_file_name, resolve_path},
    },
    v4::helpers::token_str,
};

mod ffi;
mod line_parser;
mod scopes;

type FileName = PathBuf;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum SourceType {
    #[default]
    Memory,
    File(FileName),
}

#[derive(Default)]
pub struct Preprocessor {
    pub implicit_includes: HashSet<FileName>,
    pub source_type: SourceType,
    pub files: Vec<FileName>,
    pub open_files: HashSet<FileName>,
    pub parser: line_parser::DataParser,
    pub reserved_words: DictNumByString,
    pub current_file: isize,
    pub absolute_line_index: usize,
    pub scopes: scopes::Scopes,
}

pub struct LineLoc {
    file_index: isize, // -1 for memory
    line_index: usize, // 0-based
}

#[derive(Debug)]
pub struct PreProcessorBuilder {
    implicit_includes: HashSet<FileName>,
    reserved_words: DictNumByString,
}

impl PreProcessorBuilder {
    pub fn new() -> Self {
        Self {
            implicit_includes: HashSet::new(),
            reserved_words: DictNumByString::new(
                config::ConfigBuilder::new()
                    .set_case_format(CaseFormat::LowerCase)
                    .build(),
            ),
        }
    }

    pub fn implicit_includes(&mut self, includes: Vec<FileName>) -> &mut Self {
        self.implicit_includes.extend(includes);
        self
    }

    pub fn reserved_words(&mut self, reserved_words: FileName) -> &mut Self {
        self.reserved_words.load_file(&reserved_words);
        self
    }

    pub fn build(&mut self) -> Preprocessor {
        Preprocessor {
            implicit_includes: self.implicit_includes.clone(),
            parser: line_parser::DataParser::new(),
            reserved_words: self.reserved_words.clone(),
            scopes: scopes::Scopes::new(),
            ..Default::default()
        }
    }
}

impl Preprocessor {
    pub fn parse_file(&mut self, file_path: FileName) -> Result<()> {
        self.source_type = SourceType::File(file_path.clone());
        self.absolute_line_index = 0;
        self.load_implicit_includes()?;
        match self.load_file_source(&file_path) {
            Ok(_) => {}
            Err(e) => {
                log::error!("{e}");
            }
        }
        self.scopes.exit_scope(self.absolute_line_index);
        Ok(())
    }

    pub fn parse_in_memory(&mut self, source: &str) -> Result<()> {
        self.source_type = SourceType::Memory;
        self.absolute_line_index = 0;
        self.load_implicit_includes()?;

        self.current_file = -1;
        let mut in_hex_block = false;
        for (line_index, line) in source.lines().enumerate() {
            match self.process_line(line, line_index, &mut in_hex_block) {
                Ok(_) => {}
                Err(e) => {
                    bail!(e);
                }
            }
        }
        self.scopes.exit_scope(self.absolute_line_index);
        Ok(())
    }

    pub fn get_number_of_functions_this_scope(&self, line_index: usize) -> usize {
        self.scopes
            .functions
            .iter()
            .filter(|x| x.zone.start == line_index)
            .count()
    }

    pub fn get_function(&self, line_index: usize, index: usize) -> Option<&scopes::Function> {
        self.scopes
            .functions
            .iter()
            .filter(|x| x.zone.start == line_index)
            .nth(index)
    }

    fn load_implicit_includes(&mut self) -> Result<()> {
        let includes = self.implicit_includes.clone();
        for include in includes {
            if let Err(e) = self.load_file_source(&include) {
                return Err(e);
            }
        }
        Ok(())
    }

    fn load_file_source(&mut self, file_path: &FileName) -> Result<()> {
        use lines_lossy;

        let prev_file = self.current_file;
        self.current_file = self
            .files
            .iter()
            .position(|x| x == file_path)
            .unwrap_or_else(|| {
                self.files.push(normalize_file_name(file_path).unwrap());
                self.files.len() - 1
            }) as isize;

        let Ok(file) = std::fs::File::open(file_path) else {
            bail!("Can't open file: {:?}", file_path);
        };
        let reader = std::io::BufReader::new(file);
        let mut lines = reader.lines_lossy().enumerate();
        // let reader = std::io::BufReader::new(
        //     DecodeReaderBytesBuilder::new()
        //         .encoding(Some(encoding_rs::WINDOWS_1251))
        //         .build(file),
        // );
        // let mut lines = reader.lines().enumerate();
        let mut in_hex_block = false;
        while let Some((line_index, line)) = lines.next() {
            match line {
                Ok(line) => match self.process_line(line.as_str(), line_index, &mut in_hex_block) {
                    Ok(_) => {}
                    Err(e) => {
                        bail!(e);
                    }
                },
                Err(_) => {
                    bail!("Can't read line {line_index} from file {:?}", file_path);
                }
            }
        }

        self.current_file = prev_file;
        Ok(())
    }

    pub fn process_line(
        &mut self,
        line: &str,
        line_index: usize,
        in_hex_block: &mut bool,
    ) -> Result<()> {
        self.parser.line(line);
        self.absolute_line_index += 1;
        let token = self.parser.get_token();

        match token.token_type {
            TokenType::Directive if !*in_hex_block => {
                match token.val {
                    TokenVal::Ident(s) => {
                        let token_id = self.reserved_words.map.get(&s.to_ascii_lowercase());
                        match token_id {
                            Some(&TOKEN_INCLUDE) | Some(&TOKEN_INCLUDE_ONCE) => {
                                self.parser.skip_whitespace();
                                let token = self.parser.get_until1(b"}", TokenType::Ident);
                                if self.parser.get_token().token_type != TokenType::CloseCurly {
                                    bail!("Error parsing include directive")
                                }
                                match token.token_type {
                                    TokenType::Ident => {
                                        match token.val {
                                            TokenVal::Ident(include_path) => {
                                                let current_file_name = match self.current_file {
                                                    -1 => None,
                                                    x => {
                                                        match self.files.get(x as usize) {
                                                            None => {
                                                                bail!("Error loading include file: {}", x);
                                                            }
                                                            x => x,
                                                        }
                                                    }
                                                };

                                                let Some(path) =
                                                    resolve_path(&include_path, current_file_name)
                                                else {
                                                    let loc = self.parser.current_loc();
                                                    bail!("Error resolving path to include file at {}:{}", loc.0, loc.1);
                                                };

                                                if matches!(token_id, Some(&TOKEN_INCLUDE_ONCE))
                                                    && self.files.iter().any(|x| x == &path)
                                                {
                                                    // already included
                                                    return Ok(());
                                                }

                                                if !self.open_files.insert(path.clone()) {
                                                    let loc = self.parser.current_loc();
                                                    match current_file_name {
                                                        Some(x) => {
                                                            bail!("Circular include detected at {:?}:{}", x, loc.1);
                                                        }
                                                        None => {
                                                            // can't happen as current file is in-memory and can't include itself
                                                            bail!("Circular include detected at {}:{}", loc.0, loc.1);
                                                        }
                                                    }
                                                }

                                                if let Err(e) = self.load_file_source(&path) {
                                                    bail!(e);
                                                }

                                                self.open_files.remove(&path);
                                                return Ok(()); // don't add the include line to the source
                                            }
                                            _ => {}
                                        }
                                    }
                                    x => {
                                        bail!("Error parsing include directive: {:?}", x);
                                    }
                                }
                            }
                            Some(_) => {
                                // skip other directives
                            }
                            None => {
                                bail!("Unknown directive: {}", s);
                            }
                        }
                    }
                    _ => {}
                }
            }
            TokenType::Unknown if !*in_hex_block => {
                let loc = self.parser.current_loc();
                bail!("Unknown token at line {}:{}", loc.0, loc.1);
            }
            TokenType::Ident => {
                match token.val {
                    TokenVal::Ident(s) => {
                        if let Some(token_id) = self.reserved_words.map.get(&s.to_ascii_lowercase())
                        {
                            match *token_id {
                                TOKEN_FUNCTION => {
                                    if let Err(e) = self.process_new_function(line) {
                                        bail!(e)
                                    }
                                }
                                TOKEN_END => {
                                    let scope = self.scopes.get_current_scope();
                                    *in_hex_block = false; // hex blocks can't be nested, so any end will close it
                                    if !scope.is_root() {
                                        if scope.is_in_block() {
                                            // we are in a loop/if block
                                            scope.close_block();
                                        } else {
                                            // end of function
                                            self.scopes.exit_scope(self.absolute_line_index);
                                            // till 'end' line
                                        };
                                    };
                                }
                                TOKEN_IF | TOKEN_FOR | TOKEN_WHILE | TOKEN_SWITCH => {
                                    // these blocks use end to close, so we increment the block count
                                    let scope = self.scopes.get_current_scope();
                                    if !scope.is_root() {
                                        scope.add_block();
                                    }
                                }
                                TOKEN_HEX => {
                                    *in_hex_block = true;
                                }
                                _ => {
                                    // ignore
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            TokenType::Eol => {
                return Ok(()); // skip empty lines
            }
            _ => {}
        }
        return Ok(());
    }

    fn process_new_function(&mut self, line: &str) -> Result<()> {
        use crate::parser::{function_signature, Span};

        let line = Self::strip_comments(line);
        let line = line.as_str();
        let Ok((_, ref signature)) = function_signature(Span::from(line)) else {
            let loc = self.parser.current_loc();
            bail!("Can't parse function at {}:{}", loc.0, loc.1)
        };

        self.scopes
            .add_function(token_str(line, &signature.token).to_string());
        if signature.cc == crate::parser::FunctionCC::Local {
            self.scopes.enter_scope(self.absolute_line_index); // from 'function' line
        }
        Ok(())
    }

    fn strip_comments(s: &str) -> String {
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let mut chars = s.chars().peekable();
        let mut buf = &mut String::new();

        // iterate over all chars, skip comment fragments (/* */ and //)
        while let Some(c) = chars.next() {
            match c {
                _ if inside_comment => {
                    // skip until the end of the comment
                    if c == '*' {
                        if let Some('/') = chars.next() {
                            inside_comment = false;
                        }
                    }
                }
                _ if inside_comment2 => {
                    // skip until the end of the comment
                    if c == '}' {
                        inside_comment2 = false;
                    }
                }
                '/' if chars.peek() == Some(&'/') => {
                    // line comment //
                    // there is nothing left on this line, exiting
                    break;
                }
                '/' if chars.peek() == Some(&'*') => {
                    // block comment /* */
                    inside_comment = true;
                    chars.next(); // skip *
                }

                '{' if chars.peek() != Some(&'$') => {
                    // block comment {} but not directives {$...}
                    inside_comment2 = true;
                }
                _ if c.is_ascii_whitespace() && buf.is_empty() => {
                    // skip leading whitespace
                    continue;
                }
                _ => {
                    buf.push(c);
                }
            }
        }

        return buf.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_file_only() {
        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        preprocessor
            .parse_file("src/preprocessor/test/script.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 1);
        assert_eq!(
            preprocessor.source_type,
            SourceType::File("src/preprocessor/test/script.txt".into())
        );
    }

    #[test]
    fn test_parse_from_file() {
        let mut preprocessor = PreProcessorBuilder::new()
            .implicit_includes(vec!["src/preprocessor/test/const.txt".into()])
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        preprocessor
            .parse_file("src/preprocessor/test/script.txt".into())
            .unwrap();
        assert_eq!(
            preprocessor.source_type,
            SourceType::File("src/preprocessor/test/script.txt".into())
        );
        assert_eq!(preprocessor.files.len(), 2);
    }

    #[test]
    fn test_parse_in_memory() {
        let mut preprocessor = PreProcessorBuilder::new()
            .implicit_includes(vec!["src/preprocessor/test/const.txt".into()])
            .build();
        preprocessor
            .parse_in_memory("const a = 1;\nconst b = 2;\n")
            .unwrap();
        assert_eq!(preprocessor.source_type, SourceType::Memory);
        assert_eq!(preprocessor.files.len(), 1);
    }

    #[test]
    fn test_include() {
        let mut preprocessor = PreProcessorBuilder::new()
            .implicit_includes(vec!["src/preprocessor/test/const.txt".into()])
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();

        preprocessor
            .parse_file("src/preprocessor/test/script_with_include.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 3);

        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        let e = preprocessor.parse_in_memory(r#" {$include "#);
        assert!(e.is_err());

        let e = preprocessor.parse_in_memory(r#" {$include missing.txt } "#);
        assert!(e.is_err());
    }

    #[test]
    fn test_circular_include() {
        let mut preprocessor = PreProcessorBuilder::new()
            .implicit_includes(vec!["src/preprocessor/test/circular1.txt".into()])
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        let e = preprocessor.parse_file("src/preprocessor/test/circular1.txt".into());
        assert!(e.is_err());
    }

    #[test]
    fn test_function() {
        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        preprocessor
            .parse_file("src/preprocessor/test/scr_with_func.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 1);

        assert_eq!(preprocessor.get_number_of_functions_this_scope(0), 2);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(1), 2);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(9), 0);
    }

    #[test]
    fn test_foreign_function() {
        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();
        preprocessor
            .parse_file("src/preprocessor/test/scr_ffi.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 1);

        assert_eq!(preprocessor.get_number_of_functions_this_scope(0), 3);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(1), 0);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(2), 2);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(3), 0);
    }

    #[test]
    fn test_include_hex() {
        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .build();

        preprocessor
            .parse_file("src/preprocessor/test/hex_inc.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 1);
    }

    #[test]
    fn test_hoisting() {
        let mut preprocessor = PreProcessorBuilder::new()
            .reserved_words("src/preprocessor/test/compiler.ini".into())
            .implicit_includes(vec!["src/preprocessor/test/const.txt".into()])
            .build();

        preprocessor
            .parse_file("src/preprocessor/test/u.txt".into())
            .unwrap();
        assert_eq!(preprocessor.files.len(), 2);
        assert_eq!(preprocessor.get_number_of_functions_this_scope(50), 1);
    }

    #[test]
    fn test_parse_comments() {
        let mut preprocessor = PreProcessorBuilder::new()
            .implicit_includes(vec!["src/preprocessor/test/const.txt".into()])
            .build();
        // preprocessor
        //     .parse_in_memory("function foo\nend ")
        //     .unwrap();
        // assert_eq!(preprocessor.source_type, SourceType::Memory);
        // assert_eq!(preprocessor.scopes.functions.len(), 0);

        let res = preprocessor.parse_in_memory(
            r#"
            /*
            ;
            */
        "#,
        );

        assert_eq!(preprocessor.source_type, SourceType::Memory);
        assert!(res.is_ok());
    }
}
