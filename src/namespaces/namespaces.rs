use crate::namespaces::*;
use error::ParseError;
use parser::hint;
use parser::property;
use std::collections::HashMap;

pub enum Member {
    opcode(usize),
    int(i32),
    float(f32),
    string(String),
}

pub struct Namespaces {
    pub names: Vec<String>,           // case-preserved
    pub map: HashMap<String, Member>, // ns.field
    pub opcodes: Vec<Opcode>,
    pub map_op_by_id: HashMap<u16, usize>,
    pub map_op_by_name: HashMap<String, usize>,
    pub map_enum: HashMap<String, String>, // ns.member_val = ns.member_name
}

#[derive(Debug)]
pub struct Opcode {
    pub name: String,
    pub id: u16,
    pub r#type: OpcodeType,
    pub help_code: i32,
    pub hint: String,
    pub extended_param_pos: u8,
    pub prop_pos: u8, // 1-left, 2-right
    pub prop_type: OpcodeType,
    pub operation: String, // used in decompiler output
}

#[derive(Debug)]
pub enum OpcodeType {
    Method,
    Condition,
    Property,
}

impl From<&str> for OpcodeType {
    fn from(s: &str) -> Self {
        match s {
            "1" => OpcodeType::Condition,
            "2" => OpcodeType::Property,
            _ => OpcodeType::Method,
        }
    }
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            names: vec![],
            opcodes: vec![],
            map_op_by_id: HashMap::new(),
            map_op_by_name: HashMap::new(),
            map_enum: HashMap::new(),
            map: HashMap::new(),
        }
    }

    pub fn load_file<'a>(&mut self, file_name: &'a str) -> Result<(), ParseError> {
        let content = std::fs::read_to_string(file_name)?;
        self.parse_file(content)
    }

    pub fn parse_file<'a>(&mut self, content: String) -> Result<(), ParseError> {
        let lines = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !(line.is_empty() || line.starts_with(";")))
            .take_while(|line| !line.eq_ignore_ascii_case("#eof"));

        let mut line_iter = lines.into_iter();

        if let Some(line) = line_iter.next() {
            if !line.eq_ignore_ascii_case("#classeslist") {
                return Err(ParseError::new("#CLASSESLIST not found"));
            };

            while let Some(line) = line_iter.next() {
                if !line.starts_with(|c| c == '#' || c == '$') {
                    self.names.push(String::from(line));
                    continue;
                }
                if !line.eq_ignore_ascii_case("#classes") || self.names.len() == 0 {
                    return Ok(());
                }
                break;
            }

            while let Some(line) = line_iter.next() {
                if !line.starts_with("$") {
                    continue;
                }
                if line.eq_ignore_ascii_case("$begin") || line.eq_ignore_ascii_case("$end") {
                    continue;
                }
                let name = &line[1..];

                let find_name = self.names.iter().find(|n| n.eq_ignore_ascii_case(name));

                if let Some(name) = find_name {
                    if let Some(line) = line_iter.next() {
                        if line.eq_ignore_ascii_case("$begin") {
                            let name = &name.clone();
                            for line in line_iter
                                .by_ref()
                                .take_while(|line| !line.starts_with(|c| c == '#' || c == '$'))
                            {
                                match self.parse_method(line, name) {
                                    Some(_) => {}
                                    None => {
                                        println!("Can't parse the line {}", line);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_method(&mut self, line: &str, class_name: &String) -> Option<()> {
        if line.starts_with("^") {
            return self.parse_prop(line, class_name);
        }
        let p: Vec<&str> = line.split(',').map(|x| x.trim()).collect();
        match p.as_slice() {
            [name, id, r#type, help_code, h] => {
                let id = u16::from_str_radix(*id, 16).ok()?;
                let (_, h) = hint(h).ok()?;
                self.register_opcode(Opcode {
                    id,
                    hint: h,
                    name: String::from(format!("{}.{}", class_name, name)),
                    r#type: (*r#type).into(),
                    help_code: i32::from_str_radix(*help_code, 10).ok()?,

                    prop_type: OpcodeType::Method,
                    operation: String::new(),
                    prop_pos: 0,
                    extended_param_pos: 0,
                });
                Some(())
            }
            _ => None,
        }
    }

    fn parse_prop(&mut self, line: &str, class_name: &String) -> Option<()> {
        match property(line) {
            Ok((_, (name, variations, hint))) => {
                for (id, operation, prop_pos, r#type, help_code) in variations {
                    let id = u16::from_str_radix(id, 16).ok()?;
                    self.register_opcode(Opcode {
                        id,
                        name: String::from(format!(
                            "{}.{}{}{}",
                            class_name, name, operation, prop_pos,
                        )),
                        r#type: OpcodeType::Property,
                        help_code: i32::from_str_radix(help_code, 10).ok()?,
                        hint: hint.clone(),
                        prop_type: r#type.into(),
                        operation: String::from(operation),
                        prop_pos: u8::from_str_radix(prop_pos, 10).ok()?,
                        extended_param_pos: 0,
                    });
                }
                Some(())
            }
            _ => None,
        }
    }

    fn register_opcode(&mut self, opcode: Opcode) {
        let name_lower = opcode.name.to_ascii_lowercase();
        let id = opcode.id;
        let index = self.opcodes.len();
        self.opcodes.push(opcode);
        self.map_op_by_id.insert(id, index);
        self.map_op_by_name.insert(name_lower, index);
    }

    pub fn find_by_opcode(&mut self, opcode: u16) -> Option<&usize> {
        self.map_op_by_id.get(&opcode)
    }
}
