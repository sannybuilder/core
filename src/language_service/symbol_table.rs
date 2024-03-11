use std::collections::HashMap;


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
pub struct VisibilityZone {
    pub start: u32, // line number where the symbol is defined
    pub end: u32, // line number where the symbol can no longer be seen (start of a new function or end of the file)
}

impl VisibilityZone {
    pub fn is_visible_at(&self, line_number: u32) -> bool {
        line_number >= self.start && (self.end == 0 || line_number < self.end)
    }
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
    pub fn is_visible_at(&self, line_number: u32) -> bool {
        self.zones
            .iter()
            .any(|zone| zone.is_visible_at(line_number))
    }

    pub fn add_zone(&mut self, start: u32) {
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
