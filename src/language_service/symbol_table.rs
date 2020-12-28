use super::ffi::SymbolInfoMap;
use std::collections::HashMap;

pub struct SymbolTable {
    pub symbols: HashMap<String, SymbolInfoMap>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, constants: Vec<(String, SymbolInfoMap)>) {
        self.symbols.extend(constants);
    }
}
