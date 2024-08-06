use std::collections::HashMap;

use crate::utils::visibility_zone::VisibilityZone;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Number = 0,
    String = 1,
    Var = 2,
    Label = 3,
    ModelName = 4,
    Function = 5,
}

#[derive(Clone, Debug)]
pub struct SymbolInfoMap {
    pub zones: Vec<VisibilityZone>,
    pub stack_id: u32,
    pub _type: SymbolType,
    pub value: Option<String>,  // value of the symbol (for literals)
    pub name_no_format: String, // used for autocomplete
    pub annotation: Option<String>,
}

impl SymbolInfoMap {
    pub fn is_visible_at(&self, line_number: usize) -> bool {
        self.zones
            .iter()
            .any(|zone| zone.is_visible_at(line_number))
    }

    pub fn add_zone(&mut self, start: usize) {
        self.zones.push(VisibilityZone { start, end: 0 });
    }
}

pub struct SymbolTable {
    pub symbols: HashMap</*symbol name (lowercase)*/ String, Vec<SymbolInfoMap>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn extend(&mut self, from: &SymbolTable) {
        self.symbols.extend(from.symbols.clone());
    }
}
