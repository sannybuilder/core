use std::collections::HashMap;

use serde::Serialize;

#[derive(Default, Serialize)]
pub struct SourceMap {
    files: HashMap</*file name*/ String, HashMap</*line*/ u32, /*offset*/ u32>>,
    local_variables:
        HashMap</*file name*/ String, HashMap<String, Vec<(/*line*/ u32, /*var index*/ i32)>>>,
    global_variables: HashMap</*name*/ String, /*var index*/ i32>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn add(&mut self, path: &str, line: u32, offset: u32) {
        self.files
            .entry(path.to_string())
            .or_insert_with(HashMap::new)
            .insert(line, offset);
    }

    pub fn get_offset(&self, path: &str, line: u32) -> Option<u32> {
        let file = self.files.get(path)?;
        file.get(&line).cloned()
    }

    pub fn adjust_offset_by(&mut self, delta: u32) {
        for file in self.files.values_mut() {
            for offset in file.values_mut() {
                *offset += delta;
            }
        }
    }

    pub fn new_local_variable_scope(
        &mut self,
        file_name: &str,
        line: u32,
        name: &str,
        var_index: i32,
    ) {
        self.local_variables
            .entry(file_name.to_lowercase())
            .or_insert_with(HashMap::new)
            .entry(name.to_lowercase())
            .or_insert_with(Vec::new)
            .push((line, var_index));
    }

    pub fn find_local_variable_index(
        &self,
        file_name: &str,
        line: u32,
        var_name: &str,
    ) -> Option<i32> {
        let file_map = self.local_variables.get(&file_name.to_lowercase())?;
        let scopes = file_map.get(&var_name.to_lowercase())?;

        for (i, scope) in scopes.iter().enumerate() {
            if (*scope).0 > line {
                if i == 0 {
                    // our line is before the first declaration
                    return None;
                }
                // our variable is not in this scope, which means it was declared in the previous one
                // so we return the index from the previous scope
                return Some(scopes[i - 1].1);
            }

            if i == scopes.len() - 1 {
                // our line is after the last declaration
                return Some(scope.1);
            }
        }

        None
    }

    pub fn new_global_variable(&mut self, name: &str, var_index: i32) {
        self.global_variables.insert(name.to_lowercase(), var_index);
    }

    pub fn find_global_variable_index(&self, name: &str) -> Option<i32> {
        self.global_variables.get(&name.to_lowercase()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        map.add("file1", 1, 10);
        map.add("file1", 2, 20);
        map.add("file1", 3, 30);
        map.add("file2", 1, 40);
        map.add("file2", 2, 50);
        map.add("file2", 3, 60);

        assert_eq!(map.get_offset("file1", 1), Some(10));
        assert_eq!(map.get_offset("file1", 2), Some(20));
        assert_eq!(map.get_offset("file1", 3), Some(30));
        assert_eq!(map.get_offset("file2", 1), Some(40));
        assert_eq!(map.get_offset("file2", 2), Some(50));
        assert_eq!(map.get_offset("file2", 3), Some(60));

        map.adjust_offset_by(100);

        assert_eq!(map.get_offset("file1", 1), Some(110));
        assert_eq!(map.get_offset("file1", 2), Some(120));
        assert_eq!(map.get_offset("file1", 3), Some(130));
        assert_eq!(map.get_offset("file2", 1), Some(140));
        assert_eq!(map.get_offset("file2", 2), Some(150));
        assert_eq!(map.get_offset("file2", 3), Some(160));
    }

    #[test]
    fn test_local_variables() {
        let mut map = SourceMap::new();
        map.new_local_variable_scope("file1", 1, "var1", 10);
        map.new_local_variable_scope("file1", 3, "var1", 20);
        map.new_local_variable_scope("file1", 5, "var1", 30);

        assert_eq!(map.find_local_variable_index("file1", 0, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 1, "var1"), Some(10));
        assert_eq!(map.find_local_variable_index("file1", 2, "var1"), Some(10));
        assert_eq!(map.find_local_variable_index("file1", 3, "var1"), Some(20));
        assert_eq!(map.find_local_variable_index("file1", 4, "var1"), Some(20));
        assert_eq!(map.find_local_variable_index("file1", 6, "var1"), Some(30));
    }

    #[test]
    fn test_local_variables2() {
        let mut map = SourceMap::new();
        map.new_local_variable_scope("file1", 5, "var1", 10);

        assert_eq!(map.find_local_variable_index("file1", 0, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 1, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 2, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 3, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 4, "var1"), None);
        assert_eq!(map.find_local_variable_index("file1", 5, "var1"), Some(10));
        assert_eq!(map.find_local_variable_index("file1", 6, "var1"), Some(10));
    }
}
