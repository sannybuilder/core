use std::collections::HashMap;
use std::ffi::CString;
use std::fs;

use crate::dictionary::{
    dictionary_num_by_str::DictNumByStr, dictionary_str_by_num::DictStrByNum,
    list_num_by_str::ListNumByStr,
};

use super::{
    library::{Command, Library},
    CommandParamType,
};

/**
 * this is a remnant of old Sanny type system where built-in types as Int, Float, Handle, etc
 * defined in the compiler.ini took first 20 slots and classes started their numeration from 20
 *
 * this should not be needed once all types moved here
 */
static CID: usize = 20;
pub type OpId = u16;

pub struct Namespaces {
    names: Vec<CString>, // case-preserved
    props: Vec<String>,
    enums: Vec<CString>, // case-preserved
    opcodes: Vec<Opcode>,
    short_descriptions: HashMap<OpId, /*short_desc*/ CString>,
    pub commands: HashMap<OpId, Command>,
    extensions: HashMap<OpId, String>,
    map_op_by_id: HashMap<OpId, /*opcodes index*/ usize>,
    pub map_op_by_name: HashMap<
        /*class_name*/ String,
        HashMap</*member_name*/ String, /*opcodes index*/ usize>,
    >,
    map_op_by_command_name: HashMap</*command_name*/ String, OpId>,
    pub map_enum: HashMap</*enum_name*/ String, HashMap</*member_name*/ String, EnumMember>>,
    library_version: CString,
}

#[repr(C)]
pub struct Opcode {
    pub id: OpId,
    pub help_code: i32,
    pub op_type: OpcodeType,
    pub prop_type: OpcodeType,
    pub prop_pos: u8, // 1-left, 2-right
    pub name: CString,
    pub operation: CString, // used in decompiler output
    pub hint: CString,
    pub short_desc: CString,
    pub params_len: i32,
    pub params: Vec<OpcodeParam>,
}

#[repr(C)]
#[derive(Clone)]
pub struct OpcodeParam {
    pub is_named_enum: bool,
    pub is_anonymous_enum: bool,
    pub name: CString,
    pub _type: CString,
    pub _anonymous_type: CString,
}

impl<'a> From<&OpcodeParam> for String {
    fn from(s: &OpcodeParam) -> String {
        let name = s.name.to_str().unwrap_or("");
        let _type = s._type.to_str().unwrap_or("");
        format!(
            "\"{}: {}\"",
            name,
            if s.is_anonymous_enum && !s.is_named_enum {
                "Extended"
            } else {
                _type
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

#[derive(Debug, PartialEq)]
pub struct EnumMember {
    pub name: CString,
    pub value: EnumMemberValue,
}

#[derive(Debug, PartialEq)]
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
            props: vec![],
            opcodes: vec![],
            enums: vec![],
            short_descriptions: HashMap::new(),
            commands: HashMap::new(),
            extensions: HashMap::new(),
            map_op_by_id: HashMap::new(),
            map_op_by_name: HashMap::new(),
            map_op_by_command_name: HashMap::new(),
            map_enum: HashMap::new(),
            library_version: CString::new("").unwrap(),
        }
    }

    pub fn load_classes<'a>(&mut self, file_name: &'a str) -> Option<()> {
        let content = std::fs::read_to_string(file_name).ok()?;
        self.parse_classes(content)
    }

    // todo: refactor to use with anonymous enums
    pub fn load_enums<'a>(&mut self, file_name: &'a str) -> Option<()> {
        use crate::namespaces::enum_parser::{parse_enums, EnumItems};

        let content = std::fs::read_to_string(file_name).ok()?;
        let (_, enums) = parse_enums(&content).ok()?;
        for e in enums {
            let mut members = HashMap::new();
            match e.items {
                EnumItems::Int(items) => {
                    for (name, value) in items {
                        members.insert(
                            name.to_ascii_lowercase(),
                            EnumMember {
                                name: CString::new(name).ok()?,
                                value: EnumMemberValue::Int(value),
                            },
                        );
                    }
                }
                EnumItems::Text(items) => {
                    for (name, value) in items {
                        members.insert(
                            name.to_ascii_lowercase(),
                            EnumMember {
                                name: CString::new(name).ok()?,
                                value: EnumMemberValue::Text(String::from(value)),
                            },
                        );
                    }
                }
            }

            self.enums.push(CString::new(e.name).ok()?);
            self.map_enum.insert(e.name.to_ascii_lowercase(), members);
        }
        Some(())
    }

    fn parse_classes<'a>(&mut self, content: String) -> Option<()> {
        use crate::namespaces::classes_parser::{deprecated_anonymous_enum, ParamType};

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

        let mut class_names: Vec<String> = vec![];
        while let Some(line) = line_iter.next() {
            if !line.starts_with(|c| c == '#' || c == '$') {
                self.names.push(CString::new(line).ok()?);
                class_names.push(String::from(line));
                continue;
            }

            if line.eq_ignore_ascii_case("#deprecated_enums") {
                while let Some(line) = line_iter.next() {
                    if line.starts_with(|c| c == '#' || c == '$') {
                        if !line.eq_ignore_ascii_case("#classes") || self.names.len() == 0 {
                            return Some(());
                        }
                        break;
                    }
                    if line.trim().is_empty() {
                        continue;
                    }
                    let (_, e) = deprecated_anonymous_enum(line).ok()?;
                    match e._type {
                        ParamType::Enum(values) => {
                            self.add_deprecated_enums(e.name, &values);
                        }
                        _ => {}
                    }
                }
            } else if !line.eq_ignore_ascii_case("#classes") || self.names.len() == 0 {
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

            let find_class_name = match class_names.iter().find(|n| n.eq_ignore_ascii_case(name)) {
                Some(x) => x,
                None => continue, // undeclared class -> skip
            };

            if line_iter.next()?.eq_ignore_ascii_case("$begin") {
                let class_name = &find_class_name.clone();
                let mut map: HashMap<String, usize> = HashMap::new();
                for line in line_iter
                    .by_ref()
                    .take_while(|line| !line.starts_with(|c| c == '#' || c == '$'))
                {
                    match self.parse_method(line, class_name, &mut map) {
                        Some(_) => {}
                        None => {
                            log::debug!("Can't parse the line {}", line);
                        }
                    }
                }
                self.map_op_by_name
                    .insert(class_name.to_ascii_lowercase(), map);
            }
        }
        Some(())
    }

    fn parse_method(
        &mut self,
        line: &str,
        class_name: &str,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        use crate::namespaces::classes_parser::method;

        if line.starts_with("^") {
            return self.parse_prop(line, class_name, map);
        }
        let (_, (name, id, r#type, help_code, hint_params)) = method(line).ok()?;
        let id = u16::from_str_radix(id, 16).ok()?;
        let full_name = String::from(format!("{}.{}", class_name, name));
        let params = self.parse_params(&hint_params, &full_name);

        let short_desc = self
            .get_short_description(id)
            .unwrap_or(&CString::new("").ok()?)
            .clone();

        let op_index = self.register_opcode(Opcode {
            hint: self.params_to_string(&params)?,
            params_len: params.len() as i32,
            id,
            params,
            name: CString::new(full_name).ok()?,
            op_type: r#type.into(), // regular=0 or conditional=1
            help_code: i32::from_str_radix(help_code, 10).ok()?,

            short_desc,
            prop_type: OpcodeType::Method,
            operation: CString::new("").ok()?,
            prop_pos: 0,
        });
        self.props.push(name.to_ascii_lowercase());
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
        class_name: &str,
        map: &mut HashMap<String, usize>,
    ) -> Option<()> {
        use crate::namespaces::classes_parser::property;

        let (_, (name, variations, hint_params)) = property(line).ok()?;
        for (id, operation, prop_pos, _type, help_code) in variations {
            let id = u16::from_str_radix(id, 16).ok()?;
            let full_name = String::from(format!("{}.{}", class_name, name));
            let params = self.parse_params(&hint_params, &full_name);
            let prop_pos = u8::from_str_radix(prop_pos, 10).ok()?;

            let op_type = OpcodeType::from(_type);
            let params_len = params.len();
            let help_code = i32::from_str_radix(help_code, 10).ok()?;
            let short_desc = self
                .get_short_description(id)
                .unwrap_or(&CString::new("").ok()?)
                .clone();

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

            let op = Opcode {
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
                short_desc: short_desc.clone(),
            };
            let supports_alternate_method_syntax = self.supports_alternate_method_syntax(&op);
            let op_index = self.register_opcode(op);
            let key = PropKey {
                name,
                prop_pos,
                operation,
            };
            map.insert(String::from(key).to_ascii_lowercase(), op_index);
            self.props.push(name.to_ascii_lowercase());

            if supports_alternate_method_syntax {
                // sb3 quirk
                // constructors can be written in two ways:
                // Player.Create($PLAYER_CHAR, #NULL, 2488.5601, -1666.84, 13.38)
                // $PLAYER_CHAR = Player.Create(#NULL, 2488.5601, -1666.84, 13.38)
                // hence we need to add a method version of this opcode with all params
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
                    short_desc,
                });

                map.insert(name.to_ascii_lowercase(), op_index);
            }
        }
        Some(())
    }

    fn parse_params(
        &mut self,
        params: &Vec<crate::namespaces::classes_parser::Param>,
        full_name: &str,
    ) -> Vec<OpcodeParam> {
        use crate::namespaces::classes_parser::ParamType;

        params
            .iter()
            .enumerate()
            .filter_map(|(param_index, param)| -> Option<OpcodeParam> {
                let anonymous_enum_name = format!("{}.{}", full_name, param_index);
                match &param._type {
                    ParamType::Text(_type) => {
                        let is_named_enum = self.get_enum_by_name(_type).is_some();
                        let is_anonymous_enum =
                            self.get_enum_by_name(&anonymous_enum_name).is_some(); // deprecated_enums.txt
                        Some(OpcodeParam {
                            is_named_enum,
                            is_anonymous_enum,
                            name: CString::new(param.name).ok()?,
                            _type: CString::new(*_type).ok()?,
                            _anonymous_type: if is_anonymous_enum {
                                CString::new(anonymous_enum_name).ok()?
                            } else {
                                CString::new("").ok()?
                            },
                        })
                    }

                    // deprecated syntax, anonymous enums ("extended")
                    ParamType::Enum(enum_members) => {
                        self.add_deprecated_enums(&anonymous_enum_name, enum_members);
                        Some(OpcodeParam {
                            is_named_enum: false,
                            is_anonymous_enum: true,
                            name: CString::new(param.name).ok()?,
                            _type: CString::new("").ok()?,
                            _anonymous_type: CString::new(anonymous_enum_name).ok()?,
                        })
                    }
                }
            })
            .collect::<Vec<_>>()
    }

    fn add_deprecated_enums(
        &mut self,
        anonymous_enum_name: &str,
        enum_members: &Vec<crate::namespaces::classes_parser::ParamTypeEnum>,
    ) -> Option<()> {
        use crate::namespaces::classes_parser::ParamTypeEnumValue;
        let mut index = 0;

        let mut members = HashMap::new();
        for enum_member in enum_members {
            // empty name indicates a hole in anonymous enum to skip certain index
            if enum_member.name.trim().len() == 0 {
                index += 1;
                continue;
            }
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

            members.insert(
                enum_member.name.to_ascii_lowercase(),
                EnumMember {
                    name: CString::new(enum_member.name).ok()?,
                    value: member,
                },
            );
        }

        self.map_enum
            .insert(anonymous_enum_name.to_ascii_lowercase(), members);

        Some(())
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
    pub fn get_enum_member_name_by_value(
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

    pub fn filter_enums_by_name(&self, needle: &str) -> Option<Vec<(CString, CString)>> {
        let needle = needle.to_ascii_lowercase().replace("_", "");
        Some(
            self.enums
                .iter()
                .filter_map(|name| {
                    name.to_str()
                        .ok()?
                        .to_ascii_lowercase()
                        .replace("_", "")
                        .contains(&needle)
                        .then_some((name.clone(), CString::new("").ok()?))
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn filter_enum_members_by_name(
        &self,
        enum_name: &str,
        needle: &str,
    ) -> Option<Vec<(CString, CString)>> {
        let members = self.get_enum_by_name(enum_name)?;
        let needle = needle.to_ascii_lowercase().replace("_", "");

        Some(
            members
                .iter()
                .filter_map(|(key, member)| {
                    if !key.replace("_", "").contains(&needle) {
                        return None;
                    }
                    let value = match &member.value {
                        EnumMemberValue::Int(x) => x.to_string(),
                        EnumMemberValue::Float(x) => x.to_string(),
                        EnumMemberValue::Text(x) => x.to_string(),
                    };
                    Some((member.name.clone(), CString::new(value).ok()?))
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn filter_classes_by_name(&self, needle: &str) -> Option<Vec<(CString, CString)>> {
        let needle = needle.to_ascii_lowercase().replace("_", "");
        Some(
            self.names
                .iter()
                .filter_map(|name| {
                    name.to_str()
                        .ok()?
                        .to_ascii_lowercase()
                        .replace("_", "")
                        .contains(&needle)
                        .then_some((name.clone(), CString::new("").ok()?))
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn filter_class_props_by_name(
        &self,
        class_name: &str,
        needle: &str,
    ) -> Option<Vec<(CString, i32)>> {
        let members = self.get_class_by_name(class_name)?;
        let needle = needle.to_ascii_lowercase().replace("_", "");
        Some(members.iter().filter_map(|(member, index)| {

            if !member.replace("_", "").contains(&needle) {
                return None;
            }
            let op = self.get_opcode_by_index(*index)?;

            if op.help_code == -2 /* deprecated */ || /* has the counterpart method */ self.supports_alternate_method_syntax(op)
            {
                return None;
            };

            Some((CString::new(member.clone()).ok()?, &*op as *const _ as i32))
        }).collect::<Vec<_>>())
    }

    fn supports_alternate_method_syntax(&self, op: &Opcode) -> bool {
        op.op_type == OpcodeType::Property //&& self.is_constructor(op.id).unwrap_or(false)
    }

    pub fn has_prop(&self, prop_name: &str) -> bool {
        self.props.contains(&prop_name.to_ascii_lowercase())
    }

    // load a JSON from SBL
    pub fn load_library<'a>(&mut self, file_name: &'a str) -> Option<()> {
        let content = fs::read_to_string(file_name).ok()?;

        let lib = serde_json::from_str::<Library>(content.as_str()).ok()?;

        if self.library_version.is_empty() {
            self.library_version = CString::new(lib.meta.version).ok()?;
        }

        for ext in lib.extensions.into_iter() {
            for command in ext.commands.into_iter().filter(|c| !c.attrs.is_unsupported) {
                // id may belong to multiple extensions
                match self.extensions.get_mut(&command.id) {
                    Some(x) => {
                        let mut exts = x.split(',').collect::<Vec<&str>>();
                        exts.push(ext.name.as_str());
                        exts.sort();
                        exts.dedup();

                        *x = exts.join(",");
                    }
                    None => {
                        self.extensions.insert(command.id, ext.name.clone());
                    }
                };

                self.short_descriptions
                    .insert(command.id, CString::new(command.short_desc.clone()).ok()?);
                self.map_op_by_command_name
                    .insert(command.name.to_ascii_lowercase(), command.id);
                self.commands.insert(command.id, command);
            }
        }

        Some(())
    }

    pub fn get_command_snippet_line<'a>(&self, id: OpId) -> Option<CString> {
        let command = self.commands.get(&id)?;
        let mut snippet = super::snippet::command_to_snippet_line(command, false);
        Some(CString::new(snippet).ok()?)
    }

    pub fn get_short_description<'a>(&self, id: OpId) -> Option<&CString> {
        self.short_descriptions.get(&id)
    }

    pub fn get_opcode_by_command_name<'a>(&self, command_name: &str) -> Option<&OpId> {
        self.map_op_by_command_name
            .get(&command_name.to_ascii_lowercase())
    }

    pub fn is_condition<'a>(&self, id: OpId) -> Option<bool> {
        self.commands.get(&id).map(|c| c.attrs.is_condition)
    }

    pub fn is_constructor<'a>(&self, id: OpId) -> Option<bool> {
        self.commands.get(&id).map(|c| c.attrs.is_constructor)
    }

    pub fn is_branch<'a>(&self, id: OpId) -> Option<bool> {
        self.commands.get(&id).map(|c| c.attrs.is_branch)
    }

    pub fn get_library_version(&self) -> &CString {
        &self.library_version
    }

    pub fn populate_keywords<'a>(&mut self, dict: &mut DictNumByStr) -> Option<()> {
        use crate::dictionary::ffi::apply_format;
        for (name, op) in self.map_op_by_command_name.iter() {
            let key = apply_format(name, &dict.config.case_format)?;
            dict.add(key, *op as _);
        }
        Some(())
    }

    pub fn populate_keywords2<'a>(&mut self, dict: &mut DictStrByNum) -> Option<()> {
        use crate::dictionary::ffi::apply_format;
        for (name, op) in self.map_op_by_command_name.iter() {
            let value = apply_format(name, &dict.config.case_format)?;
            dict.add(*op as _, value);
        }
        Some(())
    }

    pub fn populate_keywords3<'a>(&mut self, dict: &mut ListNumByStr) -> Option<()> {
        use crate::dictionary::ffi::apply_format;
        for (name, op) in self.map_op_by_command_name.iter() {
            let key = apply_format(name, &dict.config.case_format)?;
            dict.add(key, *op as _);
        }
        Some(())
    }

    pub fn populate_extension_list<'a>(&mut self, dict: &mut DictStrByNum) -> Option<()> {
        for (op, name) in self.extensions.iter() {
            dict.add(*op as _, CString::new(name.clone()).unwrap());
        }
        Some(())
    }

    pub fn get_input_count<'a>(&self, id: OpId) -> Option<usize> {
        self.commands.get(&id).map(|c| c.input.len())
    }

    pub fn get_output_count<'a>(&self, id: OpId) -> Option<usize> {
        self.commands.get(&id).map(|c| c.output.len())
    }

    pub fn is_input_of_type(&self, id: OpId, index: usize, _type: &str) -> Option<bool> {
        self.commands.get(&id).map(|c| {
            c.input
                .get(index)
                .map_or(false, |i| i.r#type.eq_ignore_ascii_case(_type))
        })
    }

    pub fn is_output_of_type(&self, id: OpId, index: usize, _type: &str) -> Option<bool> {
        self.commands.get(&id).map(|c| {
            c.output
                .get(index)
                .map_or(false, |i| i.r#type.eq_ignore_ascii_case(_type))
        })
    }

    pub fn get_input_type(&self, id: OpId, index: usize) -> Option<&String> {
        self.commands
            .get(&id)
            .and_then(|c| c.input.get(index).map(|i| &i.r#type))
    }

    pub fn get_output_type(&self, id: OpId, index: usize) -> Option<&String> {
        self.commands
            .get(&id)
            .and_then(|c| c.output.get(index).map(|i| &i.r#type))
    }
}
