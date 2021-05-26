use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Attr {
    pub is_branch: Option<bool>,
    pub is_condition: Option<bool>,
    pub is_constructor: Option<bool>,
    pub is_destructor: Option<bool>,
    pub is_keyword: Option<bool>,
    pub is_nop: Option<bool>,
    pub is_overload: Option<bool>,
    pub is_segment: Option<bool>,
    pub is_static: Option<bool>,
    pub is_unsupported: Option<bool>,
    pub is_variadic: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Param {
    pub r#name: String,
    pub r#source: Option<String>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub attrs: Option<Attr>,
    pub class: Option<String>,
    pub id: String,
    pub input: Option<Vec<Param>>,
    pub member: Option<String>,
    pub name: String,
    pub num_params: i32,
    pub output: Option<Vec<Param>>,
    pub short_desc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extension {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub last_update: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub meta: Meta,
    pub extensions: Vec<Extension>,
}
