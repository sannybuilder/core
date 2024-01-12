use super::ffi::SymbolInfoMap;
use std::collections::HashMap;

pub struct SymbolTable {
    pub symbols: HashMap</*symbol name (lowercase)*/ String, SymbolInfoMap>,
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
