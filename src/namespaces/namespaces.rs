use std::collections::HashMap;
use std::ffi::CString;

/**
 * this is a remnant of old Sanny type system where built-in types as Int, Float, Handle, etc
 * defined in the compiler.ini took first 20 slots and classes started their numeration from 20
 *
 * this should not be needed once all types moved here
 */
static CID: usize = 20;

pub struct Namespaces {
    names: Vec<CString>, // case-preserved
    opcodes: Vec<Opcode>,
    map_op_by_id: HashMap</*opcode*/ u16, /*opcodes index*/ usize>,
    map_op_by_name: HashMap<
        /*class_name*/ String,
        HashMap</*member_name*/ String, /*opcodes index*/ usize>,
    >,
    map_enum: HashMap</*enum_name*/ String, HashMap</*member_name*/ String, EnumMember>>,
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
    pub hint: CString,
    pub params_len: i32,
    pub params: Vec<OpcodeParam>,
}

#[repr(C)]
#[derive(Clone)]
pub struct OpcodeParam {
    pub is_enum: bool,
    pub name: CString,
    pub _type: CString,
}

impl<'a> From<&OpcodeParam> for String {
    fn from(s: &OpcodeParam) -> String {
        format!(
            "\"{}: {}\"",
            s.name.to_str().unwrap_or(""),
            if s.is_enum {
                "Extended"
            } else {
                s._type.to_str().unwrap_or("")
            }
        )
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum OpcodeType {
    Method,
    Condition,
    Property,
}

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

impl From<&str> for OpcodeType {
    fn from(s: &str) -> Self {
        match s {
            "1" => OpcodeType::Condition,
            "2" => OpcodeType::Property,
            _ => OpcodeType::Method,
        }
    }
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

    pub fn load_classes<'a>(&mut self, file_name: &'a str) -> Option<()> {
        let content = std::fs::read_to_string(file_name).ok()?;
        self.parse_classes(content)
    }

    fn parse_classes<'a>(&mut self, content: String) -> Option<()> {
        let lines = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !(line.is_empty() || line.starts_with(";")))
            .take_while(|line| !line.eq_ignore_ascii_case("#eof"));

        let mut line_iter = lines.into_iter();

        let line = match line_iter.next() {
            Some(x) => x,
            None => return Some(()), // no content -> exit with success
        };
        if !line.eq_ignore_ascii_case("#classeslist") {
            return None;
        };

        let mut names_str: Vec<String> = vec![];
        while let Some(line) = line_iter.next() {
            if !line.starts_with(|c| c == '#' || c == '$') {
                self.names.push(CString::new(line).ok()?);
                names_str.push(String::from(line));
                continue;
            }
            if !line.eq_ignore_ascii_case("#classes") || self.names.len() == 0 {
                return Some(());
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

            let find_name = match names_str.iter().find(|n| n.eq_ignore_ascii_case(name)) {
                Some(x) => x,
                None => continue, // undeclared class -> skip
            };

            if line_iter.next()?.eq_ignore_ascii_case("$begin") {
                let name = &find_name.clone();
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
        Some(())
    }

    fn parse_method(
        &mut self,
        line: &str,
        class_name: &String,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        use crate::namespaces::parser::method;

        if line.starts_with("^") {
            return self.parse_prop(line, class_name, map);
        }
        let (_, (name, id, r#type, help_code, hint_params)) = method(line).ok()?;
        let id = u16::from_str_radix(id, 16).ok()?;
        let full_name = String::from(format!("{}.{}", class_name, name));
        let params = self.parse_params(&hint_params, &full_name);

        let op_index = self.register_opcode(Opcode {
            hint: self.params_to_string(&params)?,
            params_len: params.len() as i32,
            id,
            params,
            name: CString::new(full_name).ok()?,
            op_type: r#type.into(), // regular=0 or conditional=1
            help_code: i32::from_str_radix(help_code, 10).ok()?,

            prop_type: OpcodeType::Method,
            operation: CString::new("").ok()?,
            prop_pos: 0,
        });
        map.insert(name.to_ascii_lowercase(), op_index);
        Some(())
    }

    fn params_to_string(&self, params: &Vec<OpcodeParam>) -> Option<CString> {
        let s = params
            .iter()
            .map(|p| p.into())
            .collect::<Vec<String>>()
            .join(" ");

        CString::new(s).ok()
    }

    fn parse_prop(
        &mut self,
        line: &str,
        class_name: &String,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        use crate::namespaces::parser::property;

        let (_, (name, variations, hint_params)) = property(line).ok()?;
        for (id, operation, prop_pos, _type, help_code) in variations {
            let id = u16::from_str_radix(id, 16).ok()?;
            let full_name = String::from(format!("{}.{}", class_name, name));
            let params = self.parse_params(&hint_params, &full_name);
            let prop_pos = u8::from_str_radix(prop_pos, 10).ok()?;

            let op_type = OpcodeType::from(_type);
            let params_len = params.len();
            let help_code = i32::from_str_radix(help_code, 10).ok()?;

            let prop_params = if op_type == OpcodeType::Property {
                if prop_pos == 2 {
                    params.iter().cloned().skip(1).collect::<Vec<_>>()
                } else {
                    params
                        .iter()
                        .cloned()
                        .take(params_len - 1)
                        .collect::<Vec<_>>() // todo: tests
                }
            } else {
                params.iter().cloned().collect::<Vec<_>>()
            };

            let op_index = self.register_opcode(Opcode {
                hint: self.params_to_string(&params)?,
                params_len: prop_params.len() as i32,
                id,
                params: prop_params,
                prop_pos, // left=1 (setters or comparison) right=2 (getters or constructors)
                op_type,  // regular=0 or conditional=1 or hybrid (constructor)=2
                help_code,
                name: CString::new(full_name.clone()).ok()?,
                prop_type: OpcodeType::Property,
                operation: CString::new(operation).ok()?,
            });
            let key = PropKey {
                name,
                prop_pos,
                operation,
            };
            map.insert(String::from(key).to_ascii_lowercase(), op_index);

            if op_type == OpcodeType::Property {
                // add a method version of this opcode with all params
                let op_index = self.register_opcode(Opcode {
                    hint: self.params_to_string(&params)?,
                    params_len: params.len() as i32,
                    id,
                    params,
                    help_code,
                    name: CString::new(full_name).ok()?,
                    op_type: OpcodeType::Method,

                    prop_type: OpcodeType::Method,
                    prop_pos: 0,
                    operation: CString::new("").ok()?,
                });

                map.insert(name.to_ascii_lowercase(), op_index);
            }
        }
        Some(())
    }

    fn parse_params(
        &mut self,
        params: &Vec<crate::namespaces::parser::Param>,
        full_name: &String,
    ) -> Vec<OpcodeParam> {
        use crate::namespaces::parser::{ParamType, ParamTypeEnumValue};

        params
            .iter()
            .enumerate()
            .filter_map(|(param_index, param)| -> Option<OpcodeParam> {
                match &param._type {
                    ParamType::Text(_type) => Some(OpcodeParam {
                        is_enum: false,
                        name: CString::new(param.name).ok()?,
                        _type: CString::new(*_type).ok()?,
                    }),
                    ParamType::Enum(enum_members) => {
                        let mut index = 0;
                        let enum_name = format!("{}.{}", full_name, param_index);
                        let mut members = HashMap::new();
                        for enum_member in enum_members {
                            let member = match enum_member.value {
                                ParamTypeEnumValue::Empty => EnumMemberValue::Int(index),
                                ParamTypeEnumValue::Text(text) => {
                                    match i32::from_str_radix(text, 10) {
                                        Ok(v) => {
                                            index = v;
                                            EnumMemberValue::Int(v)
                                        }
                                        Err(_) => EnumMemberValue::Text(text.to_string()),
                                    }
                                }
                            };
                            index += 1;

                            let key = enum_member.name;
                            members.insert(
                                key.to_ascii_lowercase(),
                                EnumMember {
                                    name: CString::new(key).ok()?,
                                    value: member,
                                    is_anonymous: enum_member.is_anonymous,
                                },
                            );
                        }

                        self.map_enum
                            .insert(enum_name.to_ascii_lowercase(), members);
                        Some(OpcodeParam {
                            is_enum: true,
                            name: CString::new(param.name).ok()?,
                            _type: CString::new(enum_name).ok()?,
                        })
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

    pub fn op_count(&self) -> usize {
        self.opcodes.len()
    }

    pub fn classes_count(&self) -> usize {
        self.names.len()
    }

    pub fn get_opcode_index_by_opcode(&self, opcode: u16) -> Option<&usize> {
        self.map_op_by_id.get(&opcode)
    }

    fn get_class_by_name(&self, class_name: &str) -> Option<&HashMap<String, usize>> {
        self.map_op_by_name.get(&class_name.to_ascii_lowercase())
    }

    pub fn get_opcode_index_by_name(&self, class_name: &str, member_name: &str) -> Option<&usize> {
        self.get_class_by_name(class_name)?
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
        self.get_opcode_index_by_name(class_name, String::from(key).as_str())
    }

    pub fn get_opcode_by_index(&self, op_index: usize) -> Option<&Opcode> {
        self.opcodes.get(op_index)
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
            self.get_enum_value_by_name(&param._type.to_str().ok()?, member_name)
        } else {
            None
        }
    }

    pub fn get_class_id_by_name(&self, class_name: &str) -> Option<i32> {
        for (i, name) in self.names.iter().enumerate() {
            if name.to_str().ok()?.eq_ignore_ascii_case(class_name) {
                return Some((i + CID) as i32);
            }
        }
        return None;
    }

    pub fn get_class_name_by_id(&self, id: i32) -> Option<&CString> {
        if id >= 20 {
            self.names.iter().nth(id as usize - CID)
        } else {
            None
        }
    }

    pub fn filter_enum_by_name(
        &self,
        enum_name: &str,
        needle: &str,
        dict: &mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
    ) -> Option<()> {
        let members = self.get_enum_by_name(enum_name)?;
        let needle = needle.to_ascii_lowercase();
        for (key, member) in members {
            if !key.starts_with(&needle) {
                continue;
            }
            let value = match &member.value {
                EnumMemberValue::Int(x) => x.to_string(),
                EnumMemberValue::Float(x) => x.to_string(),
                EnumMemberValue::Text(x) => x.to_string(),
            };
            dict.add(member.name.clone(), CString::new(value).ok()?);
        }
        Some(())
    }

    pub fn filter_classes_by_name(
        &self,
        needle: &str,
        dict: &mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
    ) -> Option<()> {
        let needle = needle.to_ascii_lowercase();
        for name in &self.names {
            if name
                .to_str()
                .ok()?
                .to_ascii_lowercase()
                .starts_with(&needle)
            {
                dict.add(name.clone(), CString::new("").ok()?);
            }
        }
        Some(())
    }

    pub fn filter_class_props_by_name(
        &self,
        class_name: &str,
        needle: &str,
        dict: &mut crate::dictionary::dictionary_num_by_str::DictNumByStr,
    ) -> Option<()> {
        let members = self.get_class_by_name(class_name)?;
        let needle = needle.to_ascii_lowercase();
        for (member, index) in members {
            if !member.starts_with(&needle) {
                continue;
            }
            let op = self.get_opcode_by_index(*index)?;

            if op.help_code == -2 /* deprecated */ || /* has the counterpart method */ op.op_type == OpcodeType::Property
            {
                continue;
            };

            dict.add(CString::new(member.clone()).ok()?, &*op as *const _ as i32);
        }
        Some(())
    }
}
