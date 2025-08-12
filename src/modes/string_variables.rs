use std::collections::HashMap;

/// StringVariables manages placeholder substitution for string values
pub struct StringVariables {
    /// Map of placeholder names to their replacement values
    variables: HashMap<String, String>,
}

impl StringVariables {
    /// Create a new StringVariables instance
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    
    /// Register a new placeholder variable
    /// 
    /// # Arguments
    /// * `name` - The placeholder name (e.g., "@sb:")
    /// * `value` - The replacement value (e.g., "C:\\SannyBuilder")
    pub fn register(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }
    
    /// Process a string, replacing all registered placeholders with their values
    /// 
    /// # Arguments
    /// * `input` - The string to process
    /// 
    /// # Returns
    /// A new string with all placeholders replaced
    pub fn process(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // Apply all registered replacements
        for (placeholder, replacement) in &self.variables {
            result = result.replace(placeholder, replacement);
        }
        
        result
    }
    
    /// Get the value of a specific variable
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
    
    /// Clear all registered variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }
    
    /// Get the number of registered variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }
    
    /// Check if there are no registered variables
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
    
    /// Get all registered variable names
    pub fn get_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}

impl Default for StringVariables {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_substitution() {
        let mut vars = StringVariables::new();
        vars.register("@sb:".to_string(), "C:\\SannyBuilder".to_string());
        vars.register("@game:".to_string(), "D:\\Games\\GTA".to_string());
        
        let input = "@sb:\\data\\@game:\\models.ide";
        let result = vars.process(input);
        
        assert_eq!(result, "C:\\SannyBuilder\\data\\D:\\Games\\GTA\\models.ide");
    }
    
    #[test]
    fn test_multiple_same_placeholder() {
        let mut vars = StringVariables::new();
        vars.register("@path:".to_string(), "/usr/local".to_string());
        
        let input = "@path:/bin:@path:/lib:@path:/share";
        let result = vars.process(input);
        
        assert_eq!(result, "/usr/local/bin:/usr/local/lib:/usr/local/share");
    }
    
    #[test]
    fn test_no_placeholders() {
        let vars = StringVariables::new();
        
        let input = "This string has no placeholders";
        let result = vars.process(input);
        
        assert_eq!(result, input);
    }
    
    #[test]
    fn test_unregistered_placeholder() {
        let mut vars = StringVariables::new();
        vars.register("@known:".to_string(), "replaced".to_string());
        
        let input = "@known: text @unknown: text";
        let result = vars.process(input);
        
        assert_eq!(result, "replaced text @unknown: text");
    }
    
    #[test]
    fn test_custom_placeholders() {
        let mut vars = StringVariables::new();
        vars.register("${user}".to_string(), "john".to_string());
        vars.register("${home}".to_string(), "/home/john".to_string());
        vars.register("{{env}}".to_string(), "production".to_string());
        
        let input = "User: ${user}, Home: ${home}, Environment: {{env}}";
        let result = vars.process(input);
        
        assert_eq!(result, "User: john, Home: /home/john, Environment: production");
    }
    
    #[test]
    fn test_clear_and_helpers() {
        let mut vars = StringVariables::new();
        assert!(vars.is_empty());
        assert_eq!(vars.len(), 0);
        
        vars.register("@test:".to_string(), "value".to_string());
        assert!(!vars.is_empty());
        assert_eq!(vars.len(), 1);
        assert_eq!(vars.get("@test:"), Some(&"value".to_string()));
        
        vars.clear();
        assert!(vars.is_empty());
        assert_eq!(vars.len(), 0);
        assert_eq!(vars.get("@test:"), None);
    }
    
    #[test]
    fn test_get_names() {
        let mut vars = StringVariables::new();
        vars.register("@a:".to_string(), "1".to_string());
        vars.register("@b:".to_string(), "2".to_string());
        vars.register("@c:".to_string(), "3".to_string());
        
        let mut names = vars.get_names();
        names.sort(); // Sort for consistent comparison
        
        assert_eq!(names, vec!["@a:", "@b:", "@c:"]);
    }
}