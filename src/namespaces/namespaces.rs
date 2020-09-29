use crate::namespaces::*;
use error::ParseError;
use parser::{method, property, ParamType, ParamTypeEnumValue};
use std::collections::HashMap;
use std::ffi::CString;

pub struct EnumMember {
    pub is_anonymous: bool,
    pub name: CString,
    pub value: EnumMemberValue,
}

pub enum EnumMemberValue {
    Int(i32),
    Float(f32),
    Text(String),
}

#[repr(C)]
pub struct OpcodeParam {
    pub is_enum: bool,
    pub name: CString,
    pub _type: CString,
}

// impl std::convert::From<&OpcodeParam> for String {
//     fn from(x: &OpcodeParam) -> Self {
//         let _type = if x.is_enum {
//             "Extended"
//         } else {
//             let z = x._type.clone();
//             z.into_string().unwrap().as_str()
//         };
//         format!(
//             "{}: {}",
//             x.name.clone().into_string().unwrap().as_str(),
//             _type
//         )
//     }
// }

pub struct Namespaces {
    pub names: Vec<String>, // case-preserved
    pub opcodes: Vec<Opcode>,
    pub map_op_by_id: HashMap<u16, usize>,
    pub map_op_by_name: HashMap<String, HashMap<String, usize>>,
    pub map_enum: HashMap<String, HashMap<String, EnumMember>>,
}

#[repr(C)]
pub struct Opcode {
    pub id: u16,
    pub help_code: i32,
    pub op_type: OpcodeType,
    pub prop_type: OpcodeType,
    pub prop_pos: u8, // 1-left, 2-right
    pub name: CString,
    pub operation: CString, // used in decompiler output
    pub params_len: i32,
    pub params: Vec<OpcodeParam>,
}

pub enum OpcodeType {
    Method,
    Condition,
    Property,
}

struct PropKey<'a> {
    name: &'a str,
    prop_pos: u8,
    operation: &'a str,
}

impl<'a> From<PropKey<'a>> for String {
    fn from(key: PropKey) -> Self {
        format!("{}{}{}", key.name, key.prop_pos, key.operation)
    }
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
        }
    }

    pub fn load_classes<'a>(&mut self, file_name: &'a str) -> Result<(), ParseError> {
        let content = std::fs::read_to_string(file_name)?;
        self.parse_classes(content)
    }

    fn parse_classes<'a>(&mut self, content: String) -> Result<(), ParseError> {
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
                            let mut map: HashMap<String, usize> = HashMap::new();
                            for line in line_iter
                                .by_ref()
                                .take_while(|line| !line.starts_with(|c| c == '#' || c == '$'))
                            {
                                match self.parse_method(line, name, &mut map) {
                                    Some(_) => {}
                                    None => {
                                        println!("Can't parse the line {}", line);
                                    }
                                }
                            }
                            self.map_op_by_name.insert(name.to_ascii_lowercase(), map);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_method(
        &mut self,
        line: &str,
        class_name: &String,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        if line.starts_with("^") {
            return self.parse_prop(line, class_name, map);
        }
        match method(line) {
            Ok((_, (name, id, r#type, help_code, hint_params))) => {
                let id = u16::from_str_radix(id, 16).ok()?;
                let full_name = String::from(format!("{}.{}", class_name, name));
                let params = self.parse_params(&hint_params, &full_name);

                let op_index = self.register_opcode(Opcode {
                    params_len: params.len() as i32,
                    id,
                    params,
                    name: CString::new(full_name).ok()?,
                    op_type: r#type.into(),
                    help_code: i32::from_str_radix(help_code, 10).ok()?,

                    prop_type: OpcodeType::Method,
                    operation: CString::new("").ok()?,
                    prop_pos: 0,
                });
                map.insert(name.to_ascii_lowercase(), op_index);
                Some(())
            }
            _ => None,
        }
    }

    fn parse_prop(
        &mut self,
        line: &str,
        class_name: &String,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        match property(line) {
            Ok((_, (name, variations, hint_params))) => {
                for (id, operation, prop_pos, r#type, help_code) in variations {
                    let id = u16::from_str_radix(id, 16).ok()?;
                    let full_name = String::from(format!("{}.{}", class_name, name));
                    let params = self.parse_params(&hint_params, &full_name);
                    let prop_pos = u8::from_str_radix(prop_pos, 10).ok()?;
                    let key = PropKey {
                        name,
                        prop_pos,
                        operation,
                    };

                    let op_index = self.register_opcode(Opcode {
                        params_len: params.len() as i32,
                        id,
                        params,
                        prop_pos,
                        name: CString::new(full_name).ok()?,
                        op_type: r#type.into(),
                        help_code: i32::from_str_radix(help_code, 10).ok()?,
                        prop_type: OpcodeType::Property,
                        operation: CString::new(operation).ok()?,
                    });
                    map.insert(String::from(key).to_ascii_lowercase(), op_index);
                }
                Some(())
            }
            _ => None,
        }
    }

    fn parse_params(
        &mut self,
        params: &Vec<namespaces::parser::Param>,
        full_name: &String,
    ) -> Vec<OpcodeParam> {
        params
            .iter()
            .enumerate()
            .map(|(param_index, param)| match &param._type {
                ParamType::Text(_type) => OpcodeParam {
                    is_enum: false,
                    name: CString::new(param.name).unwrap(),
                    _type: CString::new(*_type).unwrap(),
                },
                ParamType::Enum(enum_members) => {
                    let mut index = 0;
                    let enum_name = format!("{}.{}", full_name, param_index);
                    let mut members = HashMap::new();
                    for enum_member in enum_members {
                        let member = match enum_member.value {
                            ParamTypeEnumValue::Empty => EnumMemberValue::Int(index),
                            ParamTypeEnumValue::Text(text) => match i32::from_str_radix(text, 10) {
                                Ok(v) => {
                                    index = v;
                                    EnumMemberValue::Int(v)
                                }
                                Err(_) => EnumMemberValue::Text(text.to_string()),
                            },
                        };
                        index += 1;

                        let key = enum_member.name;
                        members.insert(
                            key.to_ascii_lowercase(),
                            EnumMember {
                                name: CString::new(key).unwrap(),
                                value: member,
                                is_anonymous: enum_member.is_anonymous,
                            },
                        );
                    }

                    self.map_enum
                        .insert(enum_name.to_ascii_lowercase(), members);
                    OpcodeParam {
                        is_enum: true,
                        name: CString::new(param.name).unwrap(),
                        _type: CString::new(enum_name).unwrap(),
                    }
                }
            })
            .collect::<Vec<_>>()
    }

    fn register_opcode(&mut self, opcode: Opcode) -> usize {
        let id = opcode.id;
        let index = self.opcodes.len();
        self.opcodes.push(opcode);
        self.map_op_by_id.insert(id, index);
        index
    }

    pub fn get_opcode_index_by_opcode(&self, opcode: u16) -> Option<&usize> {
        self.map_op_by_id.get(&opcode)
    }

    pub fn get_class_method_index_by_name(
        &self,
        class_name: &str,
        member_name: &str,
    ) -> Option<&usize> {
        self.map_op_by_name
            .get(&class_name.to_ascii_lowercase())?
            .get(&member_name.to_ascii_lowercase())
    }

    pub fn get_class_property_index_by_name(
        &self,
        class_name: &str,
        member_name: &str,
        prop_pos: u8,
        operation: &str,
    ) -> Option<&usize> {
        let key = PropKey {
            name: member_name,
            prop_pos,
            operation,
        };
        self.get_class_method_index_by_name(class_name, String::from(key).as_str())
    }

    pub fn get_opcode_by_index(&self, op_index: usize) -> Option<&Opcode> {
        Some(self.opcodes.get(op_index)?)
    }

    fn get_enum_by_name(&self, name: &str) -> Option<&HashMap<String, EnumMember>> {
        self.map_enum.get(&name.to_ascii_lowercase())
    }

    pub fn get_opcode_param_at(&self, op_index: usize, param_index: usize) -> Option<&OpcodeParam> {
        Some(
            self.get_opcode_by_index(op_index)?
                .params
                .get(param_index)?,
        )
    }

    pub fn get_enum_value_by_name(
        &self,
        enum_name: &str,
        member_name: &str,
    ) -> Option<&EnumMemberValue> {
        Some(
            &self
                .get_enum_by_name(enum_name)?
                .get(&member_name.to_ascii_lowercase())?
                .value,
        )
    }

    /**
     * used to represent anonymous enumerated type available in Sanny classes
     * class.member(0, 0, Torso)
     * Torso is the member of enum defined specifically in class.member
     *
     * From now on, those enums should be decompiled using namespace
     * class.member(0, 0, BodyPart.Torso)
     */
    pub fn get_anonymous_enum_name_by_member_value(
        &self,
        enum_name: &str,
        value: &EnumMemberValue,
    ) -> Option<&CString> {
        let members = self.get_enum_by_name(enum_name)?;
        members
            .iter()
            // .filter(|(_, v)| std::mem::discriminant(value) == std::mem::discriminant(v))
            .find_map(|(_, member)| match &member.value {
                EnumMemberValue::Text(x) => match value {
                    EnumMemberValue::Text(t) => t.eq_ignore_ascii_case(x).then_some(&member.name),
                    _ => None,
                },
                EnumMemberValue::Int(x) => match value {
                    EnumMemberValue::Int(t) => (t == x).then_some(&member.name),
                    _ => None,
                },
                EnumMemberValue::Float(x) => match value {
                    EnumMemberValue::Float(t) => (t == x).then_some(&member.name),
                    _ => None,
                },
            })
    }

    pub fn get_anonymous_enum_value_by_member_name(
        &self,
        op_index: usize,
        param_index: usize,
        member_name: &str,
    ) -> Option<&EnumMemberValue> {
        let param = self.get_opcode_param_at(op_index, param_index)?;
        if param.is_enum {
            self.get_enum_value_by_name(&param._type.to_str().unwrap(), member_name)
        } else {
            None
        }
    }
}
