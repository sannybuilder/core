use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandParamType {
    Gxt,
    Pointer,
    AnyModel,
    ScriptId,
    String8,
    IdeModel,
    Byte128,
    Arguments,
    Float,
    Any,
}

impl From<u8> for CommandParamType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Any,
            1 => Self::Gxt,
            2 => Self::Pointer,
            3 => Self::AnyModel,
            4 => Self::ScriptId,
            5 => Self::String8,
            6 => Self::IdeModel,
            7 => Self::Byte128,
            8 => Self::Arguments,
            9 => Self::Float,
            _ => Self::Any,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandParamSource {
    Any,
    AnyVar,
    AnyVarGlobal,
    AnyVarLocal,
    Literal,
    Pointer,
}

#[derive(Debug, PartialEq)]
pub enum Platform {
    Any,
    PC,
    Console,
    Mobile,
}

#[derive(Debug, PartialEq)]
pub enum Version {
    Any,
    _10,
    _10DE,
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Assignment,
    Addition,
    Subtraction,
    Multiplication,
    Division,
    TimedAddition,
    TimedSubtraction,
    CastAssignment,
    IsEqualTo,
    IsGreaterThan,
    IsGreaterOrEqualTo,
    And,
    Or,
    Xor,
    Not,
    Mod,
    ShiftLeft,
    ShiftRight,
}


impl<'de> Deserialize<'de> for CommandParamSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer).as_deref() {
            Ok("any") => Ok(Self::Any),
            Ok("var_any") => Ok(Self::AnyVar),
            Ok("var_global") => Ok(Self::AnyVarGlobal),
            Ok("var_local") => Ok(Self::AnyVarLocal),
            Ok("literal") => Ok(Self::Literal),
            Ok("pointer") => Ok(Self::Pointer),
            _ => Ok(Self::Any),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Attr {
    #[serde(default)]
    pub is_branch: bool,
    #[serde(default)]
    pub is_condition: bool,
    #[serde(default)]
    pub is_constructor: bool,
    #[serde(default)]
    pub is_destructor: bool,
    #[serde(default)]
    pub is_keyword: bool,
    #[serde(default)]
    pub is_nop: bool,
    #[serde(default)]
    pub is_overload: bool,
    #[serde(default)]
    pub is_segment: bool,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_unsupported: bool,
    #[serde(default)]
    pub is_variadic: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandParam {
    pub r#name: String,
    pub r#source: CommandParamSource,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub struct Command {
    #[serde(default, deserialize_with = "convert_to_number")]
    pub id: super::namespaces::OpId,
    pub name: String,
    pub num_params: i32,
    #[serde(default)]
    pub short_desc: String,
    pub class: Option<String>,
    pub member: Option<String>,
    #[serde(default)]
    pub attrs: Attr,
    #[serde(default)]
    pub input: Vec<CommandParam>,
    #[serde(default)]
    pub output: Vec<CommandParam>,
    #[serde(default, deserialize_with = "convert_platform")]
    pub platforms: Vec<Platform>,
    #[serde(default, deserialize_with = "convert_version")]
    pub versions: Vec<Version>,
    #[serde(default, deserialize_with = "convert_operator")]
    pub operator: Option<Operator>,
}

#[derive(Deserialize, Debug)]
pub struct Extension {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub last_update: u64,
    pub url: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct ClassMeta {
    pub name: String,
    #[serde(default)]
    pub desc: String,
    pub extends: Option<String>,
    pub constructable: bool,
}

impl ClassMeta {
    pub fn from_name(name: &str) -> Self {
        Self {
            constructable: false,
            desc: format!("Class {name}"),
            extends: None,
            name: name.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub meta: Meta,
    pub extensions: Vec<Extension>,
    #[serde(default)]
    pub classes: Vec<ClassMeta>,
}

fn convert_to_number<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?;

    match u16::from_str_radix(val.as_str(), 16) {
        Ok(res) => Ok(res),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
}

fn lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?;
    Ok(val.to_lowercase())
}

fn convert_platform<'de, D>(deserializer: D) -> Result<Vec<Platform>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = match Vec::deserialize(deserializer) {
        Ok(x) => x
            .iter()
            .fold(vec![], |mut acc: Vec<Platform>, el: &String| {
                match el.as_str() {
                    "any" => acc.push(Platform::Any),
                    "pc" => acc.push(Platform::PC),
                    "console" => acc.push(Platform::Console),
                    "mobile" => acc.push(Platform::Mobile),
                    _ => acc.push(Platform::Any),
                };
                acc
            }),
        _ => vec![],
    };
    Ok(res)
}

fn convert_version<'de, D>(deserializer: D) -> Result<Vec<Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = match Vec::deserialize(deserializer) {
        Ok(x) => x.iter().fold(vec![], |mut acc: Vec<Version>, el: &String| {
            match el.as_str() {
                "any" => acc.push(Version::Any),
                "1.0" => acc.push(Version::_10),
                "1.0 [DE]" => acc.push(Version::_10DE),
                _ => acc.push(Version::Any),
            };
            acc
        }),
        _ => vec![],
    };
    Ok(res)
}

fn convert_operator<'de, D>(deserializer: D) -> Result<Option<Operator>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?;

    match val.as_str() {
        "=" => Ok(Some(Operator::Assignment)),
        "+" => Ok(Some(Operator::Addition)),
        "-" => Ok(Some(Operator::Subtraction)),
        "*" => Ok(Some(Operator::Multiplication)),
        "/" => Ok(Some(Operator::Division)),
        "+=@" => Ok(Some(Operator::TimedAddition)),
        "-=@" => Ok(Some(Operator::TimedSubtraction)),
        "=#" => Ok(Some(Operator::CastAssignment)),
        "==" => Ok(Some(Operator::IsEqualTo)),
        ">" => Ok(Some(Operator::IsGreaterThan)),
        ">=" => Ok(Some(Operator::IsGreaterOrEqualTo)),
        "&" => Ok(Some(Operator::And)),
        "|" => Ok(Some(Operator::Or)),
        "^" => Ok(Some(Operator::Xor)),
        "~" => Ok(Some(Operator::Not)),
        "%" => Ok(Some(Operator::Mod)),
        "<<" => Ok(Some(Operator::ShiftLeft)),
        ">>" => Ok(Some(Operator::ShiftRight)),
        _ => Ok(None),
    }
}

impl Into<&str> for Operator {
    fn into(self) -> &'static str {
        match self {
            Operator::Assignment => "=",
            Operator::Addition => "+",
            Operator::Subtraction => "-",
            Operator::Multiplication => "*",
            Operator::Division => "/",
            Operator::TimedAddition => "+=@",
            Operator::TimedSubtraction => "-=@",
            Operator::CastAssignment => "=#",
            Operator::IsEqualTo => "==",
            Operator::IsGreaterThan => ">",
            Operator::IsGreaterOrEqualTo => ">=",
            Operator::And => "&",
            Operator::Or => "|",
            Operator::Xor => "^",
            Operator::Not => "~",
            Operator::Mod => "%",
            Operator::ShiftLeft => "<<",
            Operator::ShiftRight => ">>",
        }
    }
}

impl Operator {
    pub fn is_bitwise(&self) -> bool {
        match self {
            Operator::And
            | Operator::Or
            | Operator::Xor
            | Operator::Not
            | Operator::ShiftLeft
            | Operator::ShiftRight
            | Operator::Mod => true,
            _ => false,
        }
    }
}
