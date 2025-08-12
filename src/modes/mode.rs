use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Game enum representing supported game types
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    #[serde(rename = "gta3")]
    Gta3,
    #[serde(rename = "vc")]
    Vc,
    #[serde(rename = "sa")]
    Sa,
    #[serde(rename = "lcs")]
    Lcs,
    #[serde(rename = "vcs")]
    Vcs,
    #[serde(rename = "sa_mobile")]
    SaMobile,
    #[serde(rename = "vc_mobile")]
    VcMobile,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Gta3 => write!(f, "gta3"),
            Game::Vc => write!(f, "vc"),
            Game::Sa => write!(f, "sa"),
            Game::Lcs => write!(f, "lcs"),
            Game::Vcs => write!(f, "vcs"),
            Game::SaMobile => write!(f, "sa_mobile"),
            Game::VcMobile => write!(f, "vc_mobile"),
        }
    }
}

/// Text format enum representing supported text format types
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    #[serde(rename = "gta3")]
    Gta3,
    #[serde(rename = "vc")]
    Vc,
    #[serde(rename = "sa")]
    Sa,
    #[serde(rename = "sa_mobile")]
    SAUnicode,
}

impl std::fmt::Display for TextFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextFormat::Gta3 => write!(f, "gta3"),
            TextFormat::Vc => write!(f, "vc"),
            TextFormat::Sa => write!(f, "sa"),
            TextFormat::SAUnicode => write!(f, "sa_mobile"),
        }
    }
}

/// Element with optional autoupdate attribute and text content
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AutoUpdateElement {
    #[serde(rename = "@autoupdate", skip_serializing_if = "Option::is_none")]
    pub autoupdate: Option<String>,
    #[serde(rename = "$value")]
    pub value: String,
}

/// IDE element with optional base attribute and text content
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct IdeElement {
    #[serde(rename = "@base", skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(rename = "$value")]
    pub value: String,
}

/// Text element with optional format attribute and text content
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TextElement {
    #[serde(rename = "@format", skip_serializing_if = "Option::is_none")]
    pub format: Option<TextFormat>,
    #[serde(rename = "$value")]
    pub value: String,
}

/// Templates element with type attribute and text content
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TemplateElement {
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "$value")]
    pub value: String,
}

/// Copy-directory element with type attribute and text content
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CopyDirectoryElement {
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "$value")]
    pub value: String,
}

/// Main Mode structure representing the root XML element
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Mode {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@game")]
    pub game: Game,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@extends", skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    
    // Non-serialized fields for tracking
    #[serde(skip)]
    pub file_name: Option<String>,

    // Simple text elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constants: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(
        rename = "cleo-default-extensions",
        skip_serializing_if = "Option::is_none"
    )]
    pub cleo_default_extensions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrays: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missions: Option<String>,

    // Elements with optional attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<AutoUpdateElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextElement>,

    // Elements that can appear multiple times
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opcodes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<AutoUpdateElement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<AutoUpdateElement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<AutoUpdateElement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ide: Vec<IdeElement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<TemplateElement>,
    #[serde(
        rename = "copy-directory",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub copy_directory: Vec<CopyDirectoryElement>,
}

impl Mode {
    /// Validate that the Mode has valid attribute values
    pub fn validate(&self) -> Result<()> {
        // Validate type attribute - if present, must be "default" or ""
        if let Some(type_value) = &self.r#type {
            if type_value != "default" && type_value != "" {
                bail!(
                    "Invalid type attribute '{}' for mode '{}'. Type attribute if present can only be 'default'",
                    type_value, self.id
                );
            }
        }
        Ok(())
    }

    /// Merge fields from a parent mode, keeping this mode's non-None values
    pub fn merge_from_parent(&mut self, parent: &Mode) {
        // Only merge fields that are None in the child
        // type is not inherited
        
        // Simple text elements
        if self.data.is_none() {
            self.data = parent.data.clone();
        }
        if self.compiler.is_none() {
            self.compiler = parent.compiler.clone();
        }
        if self.constants.is_none() {
            self.constants = parent.constants.clone();
        }
        if self.keywords.is_none() {
            self.keywords = parent.keywords.clone();
        }
        if self.cleo_default_extensions.is_none() {
            self.cleo_default_extensions = parent.cleo_default_extensions.clone();
        }
        if self.variables.is_none() {
            self.variables = parent.variables.clone();
        }
        if self.labels.is_none() {
            self.labels = parent.labels.clone();
        }
        if self.arrays.is_none() {
            self.arrays = parent.arrays.clone();
        }
        if self.missions.is_none() {
            self.missions = parent.missions.clone();
        }
        
        // Elements with optional attributes
        if self.examples.is_none() {
            self.examples = parent.examples.clone();
        }
        if self.text.is_none() {
            self.text = parent.text.clone();
        }
        
        // Vectors - only inherit if child has no entries
        // If child defines any entries, it completely replaces parent's entries
        if self.opcodes.is_empty() {
            self.opcodes = parent.opcodes.clone();
        }
        
        if self.library.is_empty() {
            self.library = parent.library.clone();
        }
        
        if self.classes.is_empty() {
            self.classes = parent.classes.clone();
        }
        
        if self.enums.is_empty() {
            self.enums = parent.enums.clone();
        }
        
        if self.ide.is_empty() {
            self.ide = parent.ide.clone();
        }
        
        if self.templates.is_empty() {
            self.templates = parent.templates.clone();
        }
        
        if self.copy_directory.is_empty() {
            self.copy_directory = parent.copy_directory.clone();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    
    #[test]
    fn test_mode_deserialization() {
        // Test that Mode can be deserialized from XML
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="test_mode" title="Test Mode" game="sa" type="default">
    <data>@sb:\data\test\</data>
    <compiler>@sb:\compiler.exe</compiler>
    <constants>@sb:\constants.txt</constants>
    <keywords>@sb:\keywords.txt</keywords>
    <variables>@game:\variables.ini</variables>
    <examples autoupdate="yes">@sb:\examples.txt</examples>
    <text format="sa">@game:\text.gxt</text>
    <opcodes>@sb:\opcodes1.ini</opcodes>
    <opcodes>@sb:\opcodes2.ini</opcodes>
    <library autoupdate="no">@sb:\library.json</library>
    <classes autoupdate="yes">@sb:\classes.db</classes>
    <enums>@sb:\enums.txt</enums>
    <ide base="@game:\">@game:\default.ide</ide>
    <templates type="default">@sb:\templates.txt</templates>
    <copy-directory type="main">@game:\data\main</copy-directory>
</mode>"#;
        
        let result = from_str::<Mode>(xml);
        assert!(result.is_ok(), "Failed to deserialize Mode from XML");
        
        let mode = result.unwrap();
        assert_eq!(mode.id, "test_mode");
        assert_eq!(mode.title, "Test Mode");
        assert_eq!(mode.game, Game::Sa);
        assert_eq!(mode.r#type, Some("default".to_string()));
        assert!(mode.extends.is_none());
        
        assert_eq!(mode.data, Some("@sb:\\data\\test\\".to_string()));
        assert_eq!(mode.compiler, Some("@sb:\\compiler.exe".to_string()));
        assert_eq!(mode.opcodes.len(), 2);
        assert_eq!(mode.library.len(), 1);
        assert_eq!(mode.classes.len(), 1);
        assert!(mode.examples.is_some());
        assert!(mode.text.is_some());
    }
    
    #[test]
    fn test_merge_from_parent() {
        // Create parent mode
        let parent_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="parent" title="Parent Mode" game="sa" type="default">
    <data>parent_data</data>
    <compiler>parent_compiler</compiler>
    <keywords>parent_keywords</keywords>
    <opcodes>parent_opcode1</opcodes>
    <opcodes>parent_opcode2</opcodes>
    <library autoupdate="yes">parent_library</library>
</mode>"#;
        
        let parent = from_str::<Mode>(parent_xml).unwrap();
        
        // Create child mode with some overrides
        let child_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="child" title="Child Mode" game="sa" extends="parent">
    <compiler>child_compiler</compiler>
    <constants>child_constants</constants>
    <opcodes>child_opcode</opcodes>
</mode>"#;
        
        let mut child = from_str::<Mode>(child_xml).unwrap();
        
        // Apply inheritance
        child.merge_from_parent(&parent);
        
        // Test inherited fields
        assert_eq!(child.data, Some("parent_data".to_string())); // Inherited
        assert_eq!(child.keywords, Some("parent_keywords".to_string())); // Inherited
        assert_eq!(child.r#type, None); // type is not inherited
        
        // Test overridden fields
        assert_eq!(child.compiler, Some("child_compiler".to_string())); // Kept child's value
        assert_eq!(child.constants, Some("child_constants".to_string())); // Child's new field
        
        // Test vector replacement (not merge)
        assert_eq!(child.opcodes.len(), 1); // Child defined opcodes, so only has child's
        assert_eq!(child.opcodes[0], "child_opcode");
        
        // Test that empty vectors inherit from parent
        assert_eq!(child.library.len(), 1); // Child didn't define library, so inherits parent's
        assert_eq!(child.library[0].value, "parent_library");
    }

    
    #[test]
    fn test_mode_with_extends() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="extended" title="Extended Mode" game="sa" extends="base_mode">
    <data>extended_data</data>
</mode>"#;
        
        let mode = from_str::<Mode>(xml).unwrap();
        assert_eq!(mode.extends, Some("base_mode".to_string()));
    }
    
    #[test]
    fn test_empty_vectors() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="test" title="Test" game="sa">
    <data>test_data</data>
</mode>"#;
        
        let mode = from_str::<Mode>(xml).unwrap();
        assert!(mode.opcodes.is_empty());
        assert!(mode.library.is_empty());
        assert!(mode.classes.is_empty());
        assert!(mode.enums.is_empty());
        assert!(mode.ide.is_empty());
        assert!(mode.templates.is_empty());
        assert!(mode.copy_directory.is_empty());
    }
    
    #[test]
    fn test_element_attributes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="test" title="Test" game="sa">
    <examples autoupdate="no">examples.txt</examples>
    <text format="sa">text.gxt</text>
    <library autoupdate="yes">lib.json</library>
    <classes autoupdate="no">classes.db</classes>
    <enums autoupdate="yes">enums.txt</enums>
    <ide base="/base/path/">default.ide</ide>
    <templates type="custom">templates.txt</templates>
    <copy-directory type="main">main_dir</copy-directory>
</mode>"#;
        
        let mode = from_str::<Mode>(xml).unwrap();
        
        assert_eq!(mode.examples.as_ref().unwrap().autoupdate, Some("no".to_string()));
        assert_eq!(mode.examples.as_ref().unwrap().value, "examples.txt");
        
        assert_eq!(mode.text.as_ref().unwrap().format, Some(TextFormat::Sa));
        assert_eq!(mode.text.as_ref().unwrap().value, "text.gxt");
        
        assert_eq!(mode.library[0].autoupdate, Some("yes".to_string()));
        assert_eq!(mode.classes[0].autoupdate, Some("no".to_string()));
        assert_eq!(mode.enums[0].autoupdate, Some("yes".to_string()));
        
        assert_eq!(mode.ide[0].base, Some("/base/path/".to_string()));
        assert_eq!(mode.templates[0].r#type, "custom");
        assert_eq!(mode.copy_directory[0].r#type, "main");
    }

    #[test]
    fn test_invalid_game_value_causes_parse_error() {
        let xml = r#"
            <mode id="test" title="Test" game="unknown_game">
                <data>test data</data>
            </mode>
        "#;
        
        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_err(), "Should fail to parse unknown game value");
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("unknown_game") || error_msg.contains("unknown variant"),
                "Error should mention the invalid game value: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_invalid_text_format_causes_parse_error() {
        let xml = r#"
            <mode id="test" title="Test" game="sa">
                <text format="invalid_format">test.txt</text>
            </mode>
        "#;
        
        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_err(), "Should fail to parse unknown text format value");
        
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("invalid_format") || error_msg.contains("unknown variant"),
                "Error should mention the invalid format value: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_type_attribute_validation() {
        // Test that type="default" is valid
        let xml = r#"
            <mode id="test" title="Test" game="sa" type="default">
                <data>test data</data>
            </mode>
        "#;
        
        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_ok(), "Should successfully parse mode with type='default'");
        let mode = result.unwrap();
        assert!(mode.validate().is_ok(), "Mode with type='default' should pass validation");

        // Test that type="" is valid
        let xml = r#"
            <mode id="test" title="Test" game="sa" type="">
                <data>test data</data>
            </mode>
        "#;

        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_ok(), "Should successfully parse mode with type=''");
        let mode = result.unwrap();
        assert!(mode.validate().is_ok(), "Mode with type='' should pass validation");
        
        // Test that invalid type values fail validation
        let xml = r#"
            <mode id="test" title="Test" game="sa" type="custom">
                <data>test data</data>
            </mode>
        "#;
        
        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_ok(), "Should parse XML successfully");
        let mode = result.unwrap();
        let validation_result = mode.validate();
        assert!(validation_result.is_err(), "Mode with type='custom' should fail validation");
        if let Err(e) = validation_result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("can only be 'default'"), "Error message should indicate that type can only be 'default'");
        }
        
        // Test that no type attribute is valid
        let xml = r#"
            <mode id="test" title="Test" game="sa">
                <data>test data</data>
            </mode>
        "#;
        
        let result: Result<Mode, _> = from_str(xml);
        assert!(result.is_ok(), "Should successfully parse mode without type attribute");
        let mode = result.unwrap();
        assert!(mode.validate().is_ok(), "Mode without type attribute should pass validation");
    }
}