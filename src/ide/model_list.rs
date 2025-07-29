use ctor::ctor;
use simplelog::*;
use std::{
    collections::HashMap,
    ffi::{c_char, CString},
    path::Path,
};

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum ModelType {
    Object,
    Vehicle,
    Ped,
    Weapon,
    Hier,
}

#[repr(C)]
pub struct Model {
    pub id: i32,
    pub r#type: ModelType,
}

pub struct ModelList {
    pub _by_id: HashMap<i32, CString>,
    pub _by_name: HashMap<String, Model>,
    pub model_names: Vec<String>,
}

impl ModelList {
    pub fn new() -> Self {
        Self {
            _by_id: HashMap::new(),
            _by_name: HashMap::new(),
            model_names: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_name: &str) {
        let Ok(content) = std::fs::read_to_string(file_name) else {
            log::error!("Failed to read file: {}", file_name);
            return;
        };
        let Ok(ide) = gta_ide_parser::parse(&content) else {
            log::error!("Failed to parse IDE: {}", file_name);
            return;
        };
        for (section_name, lines) in ide {
            if ["txdp", "path", "2dfx"].contains(&section_name.as_str()) {
                continue;
            }
            for line in lines {
                let Ok(id) = line[0].parse::<i32>() else {
                    log::error!("Failed to parse ID: {} in {}", line[0], file_name);
                    continue;
                };
                let name = line[1].to_string();
                let r#type = match section_name.as_str() {
                    "objs" | "tobj" | "anim" | "tanm" => ModelType::Object,
                    "cars" => ModelType::Vehicle,
                    "peds" => ModelType::Ped,
                    "weap" => ModelType::Weapon,
                    "hier" => ModelType::Hier,
                    _ => {
                        continue;
                    }
                };

                self.model_names.push(name.to_uppercase());
                self._by_id
                    .insert(id, CString::new(name.to_uppercase()).unwrap());
                self._by_name
                    .insert(name.to_ascii_lowercase(), Model { id, r#type });
            }
        }
    }

    pub fn find_by_id(&self, id: i32) -> Option<&CString> {
        self._by_id.get(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Model> {
        let needle = &name.to_ascii_lowercase();
        match needle.as_str() {
            // old parser did not properly handle missing commas, resulting in broken names
            "emperoremperor" => {
                return Some(&Model {
                    id: 585,
                    r#type: ModelType::Vehicle,
                })
            }
            "wayfarerwayfarer" => {
                return Some(&Model {
                    id: 586,
                    r#type: ModelType::Vehicle,
                })
            }
            "dodododo" => {
                return Some(&Model {
                    id: 593,
                    r#type: ModelType::Vehicle,
                })
            }
            _ => self._by_name.get(needle),
        }
    }

    pub fn filter_by_name(&self, needle: &str) -> Vec<(CString, i32)> {
        let needle = needle.to_ascii_uppercase();
        let mut results = Vec::new();
        for name in self.model_names.iter() {
            if name.contains(&needle) {
                if let Some(model) = self.find_by_name(name) {
                    if model.r#type == ModelType::Object {
                        continue; // Skip objects
                    }
                    results.push((CString::new(name.clone()).unwrap(), model.id));
                    // if results.len() >= 200 {
                    //     break; // Limit results to 200
                    // }
                }
            }
        }
        results
    }
}
