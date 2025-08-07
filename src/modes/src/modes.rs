use crate::mode::{
    AutoUpdateElement, CopyDirectoryElement, Game, IdeElement, Mode, TemplateElement, TextElement,
};
use crate::string_variables::StringVariables;
use anyhow::{Context, Result, anyhow, bail};
use quick_xml::de::from_str;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// ModeManager is responsible for loading and managing all modes from a directory
pub struct ModeManager {
    /// All loaded modes stored in a vector
    modes: Vec<Mode>,
    /// String variables for placeholder substitution
    variables: StringVariables,
    /// The currently selected mode index
    current_mode_index: Option<usize>,
}

impl ModeManager {
    /// Create a new ModeManager instance
    pub fn new() -> Self {
        Self {
            modes: Vec::new(),
            variables: StringVariables::new(),
            current_mode_index: None,
        }
    }

    /// Set the current mode by ID
    pub fn set_current_mode_by_id(&mut self, mode_id: &str) -> bool {
        if let Some(index) = self.modes.iter().position(|m| m.id == mode_id) {
            self.current_mode_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Set the current mode by game (prioritizes modes with type="default")
    pub fn set_current_mode_by_game(&mut self, game: Game) -> bool {
        // First, try to find a mode with matching game and type="default"
        if let Some(index) = self
            .modes
            .iter()
            .position(|m| m.game == game && m.r#type.as_deref() == Some("default"))
        {
            self.current_mode_index = Some(index);
            return true;
        }

        // If no default mode found, fall back to any mode with matching game
        if let Some(index) = self.modes.iter().position(|m| m.game == game) {
            self.current_mode_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Set the current mode by index
    pub fn set_current_mode_by_index(&mut self, index: usize) -> bool {
        if index < self.modes.len() {
            self.current_mode_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Register a variable for substitution
    pub fn register_variable(&mut self, name: String, value: String) {
        self.variables.register(name, value);
    }

    /// Helper method to apply path substitutions
    fn apply_variables(&self, s: &str) -> String {
        self.variables.process(s)
    }

    /// Helper method to find a mode by ID
    fn find_mode_by_id(&self, mode_id: &str) -> Option<&Mode> {
        self.modes.iter().find(|m| m.id == mode_id)
    }

    /// Helper method to find a mutable mode by ID
    fn find_mode_by_id_mut(&mut self, mode_id: &str) -> Option<&mut Mode> {
        self.modes.iter_mut().find(|m| m.id == mode_id)
    }

    /// Get the total number of modes
    pub fn mode_count(&self) -> usize {
        self.modes.len()
    }

    /// Getters for mode properties with path substitution

    /// Get ID for mode at specific index
    pub fn get_id_at(&self, index: usize) -> Option<String> {
        self.modes.get(index).map(|mode| mode.id.clone())
    }

    pub fn get_id(&self) -> Option<String> {
        self.current_mode_index.and_then(|idx| self.get_id_at(idx))
    }

    /// Get title for mode at specific index
    pub fn get_title_at(&self, index: usize) -> Option<String> {
        self.modes.get(index).map(|mode| mode.title.clone())
    }

    pub fn get_title(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_title_at(idx))
    }

    /// Get game for mode at specific index
    pub fn get_game_at(&self, index: usize) -> Option<Game> {
        self.modes.get(index).map(|mode| mode.game.clone())
    }

    pub fn get_game(&self) -> Option<Game> {
        self.current_mode_index
            .and_then(|idx| self.get_game_at(idx))
    }

    /// Get type for mode at specific index
    pub fn get_type_at(&self, index: usize) -> Option<String> {
        self.modes.get(index).and_then(|mode| mode.r#type.clone())
    }

    pub fn get_type(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_type_at(idx))
    }

    /// Get extends for mode at specific index
    pub fn get_extends_at(&self, index: usize) -> Option<String> {
        self.modes.get(index).and_then(|mode| mode.extends.clone())
    }

    pub fn get_extends(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_extends_at(idx))
    }

    /// Get data for mode at specific index
    pub fn get_data_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.data.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_data(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_data_at(idx))
    }

    /// Get compiler for mode at specific index
    pub fn get_compiler_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.compiler.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_compiler(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_compiler_at(idx))
    }

    /// Get constants for mode at specific index
    pub fn get_constants_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.constants.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_constants(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_constants_at(idx))
    }

    /// Get keywords for mode at specific index
    pub fn get_keywords_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.keywords.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_keywords(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_keywords_at(idx))
    }

    /// Get cleo_default_extensions for mode at specific index
    pub fn get_cleo_default_extensions_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.cleo_default_extensions.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_cleo_default_extensions(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_cleo_default_extensions_at(idx))
    }

    /// Get mode_variables for mode at specific index
    pub fn get_mode_variables_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.variables.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_mode_variables(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_mode_variables_at(idx))
    }

    /// Get labels for mode at specific index
    pub fn get_labels_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.labels.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_labels(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_labels_at(idx))
    }

    /// Get arrays for mode at specific index
    pub fn get_arrays_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.arrays.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_arrays(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_arrays_at(idx))
    }

    /// Get missions for mode at specific index
    pub fn get_missions_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.missions.as_ref())
            .map(|s| self.apply_variables(s))
    }

    pub fn get_missions(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_missions_at(idx))
    }

    /// Get examples for mode at specific index
    pub fn get_examples_at(&self, index: usize) -> Option<AutoUpdateElement> {
        self.modes
            .get(index)
            .and_then(|mode| mode.examples.as_ref())
            .map(|elem| AutoUpdateElement {
                autoupdate: elem.autoupdate.clone(),
                value: self.apply_variables(&elem.value),
            })
    }

    pub fn get_examples(&self) -> Option<AutoUpdateElement> {
        self.current_mode_index
            .and_then(|idx| self.get_examples_at(idx))
    }

    /// Get text for mode at specific index
    pub fn get_text_at(&self, index: usize) -> Option<TextElement> {
        self.modes
            .get(index)
            .and_then(|mode| mode.text.as_ref())
            .map(|elem| TextElement {
                format: elem.format.clone(),
                value: self.apply_variables(&elem.value),
            })
    }

    pub fn get_text(&self) -> Option<TextElement> {
        self.current_mode_index
            .and_then(|idx| self.get_text_at(idx))
    }

    /// Get opcodes for mode at specific index
    pub fn get_opcodes_at(&self, index: usize) -> Vec<String> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.opcodes
                    .iter()
                    .map(|s| self.apply_variables(s))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_opcodes(&self) -> Vec<String> {
        self.current_mode_index
            .map(|idx| self.get_opcodes_at(idx))
            .unwrap_or_default()
    }

    /// Get library for mode at specific index
    pub fn get_library_at(&self, index: usize) -> Vec<AutoUpdateElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.library
                    .iter()
                    .map(|elem| AutoUpdateElement {
                        autoupdate: elem.autoupdate.clone(),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_library(&self) -> Vec<AutoUpdateElement> {
        self.current_mode_index
            .map(|idx| self.get_library_at(idx))
            .unwrap_or_default()
    }

    /// Get classes for mode at specific index
    pub fn get_classes_at(&self, index: usize) -> Vec<AutoUpdateElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.classes
                    .iter()
                    .map(|elem| AutoUpdateElement {
                        autoupdate: elem.autoupdate.clone(),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_classes(&self) -> Vec<AutoUpdateElement> {
        self.current_mode_index
            .map(|idx| self.get_classes_at(idx))
            .unwrap_or_default()
    }

    /// Get enums for mode at specific index
    pub fn get_enums_at(&self, index: usize) -> Vec<AutoUpdateElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.enums
                    .iter()
                    .map(|elem| AutoUpdateElement {
                        autoupdate: elem.autoupdate.clone(),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_enums(&self) -> Vec<AutoUpdateElement> {
        self.current_mode_index
            .map(|idx| self.get_enums_at(idx))
            .unwrap_or_default()
    }

    /// Get ide for mode at specific index
    pub fn get_ide_at(&self, index: usize) -> Vec<IdeElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.ide
                    .iter()
                    .map(|elem| IdeElement {
                        base: elem.base.as_ref().map(|s| self.apply_variables(s)),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_ide(&self) -> Vec<IdeElement> {
        self.current_mode_index
            .map(|idx| self.get_ide_at(idx))
            .unwrap_or_default()
    }

    /// Get templates for mode at specific index
    pub fn get_templates_at(&self, index: usize) -> Vec<TemplateElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.templates
                    .iter()
                    .map(|elem| TemplateElement {
                        r#type: elem.r#type.clone(),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_templates(&self) -> Vec<TemplateElement> {
        self.current_mode_index
            .map(|idx| self.get_templates_at(idx))
            .unwrap_or_default()
    }

    /// Get copy_directory for mode at specific index
    pub fn get_copy_directory_at(&self, index: usize) -> Vec<CopyDirectoryElement> {
        self.modes
            .get(index)
            .map(|mode| {
                mode.copy_directory
                    .iter()
                    .map(|elem| CopyDirectoryElement {
                        r#type: elem.r#type.clone(),
                        value: self.apply_variables(&elem.value),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_copy_directory(&self) -> Vec<CopyDirectoryElement> {
        self.current_mode_index
            .map(|idx| self.get_copy_directory_at(idx))
            .unwrap_or_default()
    }

    /// Get index of mode by game (prioritizes modes with type="default")
    pub fn get_index_by_game(&self, game: Game) -> Option<usize> {
        // First, try to find a mode with matching game and type="default"
        if let Some(index) = self
            .modes
            .iter()
            .position(|m| m.game == game && m.r#type.as_deref() == Some("default"))
        {
            return Some(index);
        }

        // If no default mode found, fall back to any mode with matching game
        self.modes.iter().position(|m| m.game == game)
    }

    /// Get index of mode by ID
    pub fn get_index_by_id(&self, mode_id: &str) -> Option<usize> {
        self.modes.iter().position(|m| m.id == mode_id)
    }

    /// Get parent mode index for mode at specific index
    pub fn get_parent_at(&self, index: usize) -> Option<usize> {
        self.modes
            .get(index)
            .and_then(|mode| mode.extends.as_ref())
            .and_then(|parent_id| self.get_index_by_id(parent_id))
    }

    pub fn get_parent(&self) -> Option<usize> {
        self.current_mode_index
            .and_then(|idx| self.get_parent_at(idx))
    }

    /// Check if mode at index is valid (no duplicate IDs)
    pub fn is_valid_at(&self, index: usize) -> bool {
        if let Some(mode) = self.modes.get(index) {
            let count = self.modes.iter().filter(|m| m.id == mode.id).count();
            count == 1
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.current_mode_index
            .map(|idx| self.is_valid_at(idx))
            .unwrap_or(false)
    }

    /// Check if mode ID contains "SBL"
    pub fn is_sbl_at(&self, index: usize) -> bool {
        self.modes
            .get(index)
            .map(|mode| mode.id.to_uppercase().contains("SBL"))
            .unwrap_or(false)
    }

    pub fn is_sbl(&self) -> bool {
        self.current_mode_index
            .map(|idx| self.is_sbl_at(idx))
            .unwrap_or(false)
    }

    /// Check if current mode has type="default"
    pub fn is_default(&self) -> bool {
        self.current_mode_index
            .and_then(|idx| self.modes.get(idx))
            .and_then(|mode| mode.r#type.as_ref())
            .map(|t| t == "default")
            .unwrap_or(false)
    }

    /// Get raw game string for mode at specific index
    pub fn get_game_raw_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .map(|mode| mode.game.to_string())
    }

    pub fn get_game_raw(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_game_raw_at(idx))
    }

    /// Get file name for mode at specific index
    pub fn get_file_name_at(&self, index: usize) -> Option<String> {
        self.modes
            .get(index)
            .and_then(|mode| mode.file_name.clone())
    }

    pub fn get_file_name(&self) -> Option<String> {
        self.current_mode_index
            .and_then(|idx| self.get_file_name_at(idx))
    }

    /// Check if autoupdate is allowed for a specific element type
    pub fn is_autoupdate_allowed_for(&self, element: &str, index: usize) -> bool {
        if let Some(mode) = self.modes.get(index) {
            match element {
                "opcodes" => true, // Opcodes don't have autoupdate attribute
                "library" => {
                    // Check first library element's autoupdate attribute
                    mode.library
                        .first()
                        .and_then(|elem| elem.autoupdate.as_ref())
                        .map(|v| v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true"))
                        .unwrap_or(true) // Default to true if not specified
                }
                "classes" => mode
                    .classes
                    .first()
                    .and_then(|elem| elem.autoupdate.as_ref())
                    .map(|v| v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true),
                "enums" => mode
                    .enums
                    .first()
                    .and_then(|elem| elem.autoupdate.as_ref())
                    .map(|v| v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true),
                _ => true,
            }
        } else {
            true
        }
    }

    /// Convert string game name to Game enum
    pub fn get_game_by_name(&self, name: &str) -> Option<Game> {
        match name.to_lowercase().as_str() {
            "gta3" => Some(Game::Gta3),
            "vc" => Some(Game::Vc),
            "sa" => Some(Game::Sa),
            "lcs" => Some(Game::Lcs),
            "vcs" => Some(Game::Vcs),
            "sa_mobile" => Some(Game::SaMobile),
            "vc_mobile" => Some(Game::VcMobile),
            _ => None,
        }
    }

    /// Get first item from a list field (opcodes, library, classes, enums)
    pub fn get_first_of(&self, field_name: &str, index: usize) -> Option<String> {
        if let Some(mode) = self.modes.get(index) {
            match field_name {
                "opcodes" => mode.opcodes.first().map(|s| self.apply_variables(s)),
                "library" => mode
                    .library
                    .first()
                    .map(|elem| self.apply_variables(&elem.value)),
                "classes" => mode
                    .classes
                    .first()
                    .map(|elem| self.apply_variables(&elem.value)),
                "enums" => mode
                    .enums
                    .first()
                    .map(|elem| self.apply_variables(&elem.value)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Load files by wildcard pattern
    pub fn load_by_mask(&self, mask: &str) -> Vec<String> {
        use glob::glob;

        let processed_mask = self.apply_variables(mask);
        let mut files = Vec::new();

        if let Ok(paths) = glob(&processed_mask) {
            for path_result in paths {
                if let Ok(path) = path_result {
                    if path.is_file() {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        files
    }

    /// Get list of items with wildcard expansion support
    pub fn get_list_of(&self, field_name: &str, index: usize) -> Vec<String> {
        if let Some(mode) = self.modes.get(index) {
            let items = match field_name {
                "opcodes" => mode.opcodes.clone(),
                "library" => mode.library.iter().map(|e| e.value.clone()).collect(),
                "classes" => mode.classes.iter().map(|e| e.value.clone()).collect(),
                "enums" => mode.enums.iter().map(|e| e.value.clone()).collect(),
                _ => Vec::new(),
            };

            let mut result = Vec::new();
            for item in items {
                let processed = self.apply_variables(&item);
                if processed.contains('*') {
                    // Expand wildcard
                    result.extend(self.load_by_mask(&processed));
                } else {
                    result.push(processed);
                }
            }
            result
        } else {
            Vec::new()
        }
    }

    /// Convenience methods for specific list types
    pub fn get_list_of_library(&self, index: usize) -> Vec<String> {
        self.get_list_of("library", index)
    }

    pub fn get_list_of_opcodes(&self, index: usize) -> Vec<String> {
        self.get_list_of("opcodes", index)
    }

    pub fn get_list_of_classes(&self, index: usize) -> Vec<String> {
        self.get_list_of("classes", index)
    }

    pub fn get_list_of_enums(&self, index: usize) -> Vec<String> {
        self.get_list_of("enums", index)
    }

    /// Load all modes from a directory (including 1 level of subdirectories)
    ///
    /// # Arguments
    /// * `directory_path` - Path to the directory to scan
    ///
    /// # Returns
    /// Result containing the number of successfully loaded modes, or an error
    pub fn load_from_directory(&mut self, directory_path: &Path) -> Result<usize> {
        let mut loaded_count = 0;
        let mut xml_files = Vec::new();

        // Scan the root directory
        if let Ok(entries) = fs::read_dir(directory_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xml") {
                    xml_files.push(path);
                } else if path.is_dir() {
                    // Scan 1 level of subdirectories
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file()
                                && sub_path.extension().and_then(|s| s.to_str()) == Some("xml")
                            {
                                xml_files.push(sub_path);
                            }
                        }
                    }
                }
            }
        } else {
            bail!("Failed to read directory: {:?}", directory_path);
        }

        // First pass: Load all modes without resolving inheritance
        for xml_path in &xml_files {
            if let Ok(mode) = self.parse_xml_file(xml_path) {
                // Check if mode with this ID already exists
                if !self.modes.iter().any(|m| m.id == mode.id) {
                    self.modes.push(mode);
                    loaded_count += 1;
                }
            }
        }

        // Second pass: Resolve inheritance for all modes
        let mode_ids: Vec<String> = self.modes.iter().map(|m| m.id.clone()).collect();
        let mut processed = HashSet::new();
        for mode_id in mode_ids {
            self.resolve_mode_inheritance(&mode_id, &mut processed)
                .with_context(|| format!("Failed to resolve inheritance for mode '{}'", mode_id))?;
        }

        Ok(loaded_count)
    }

    /// Parse a single XML mode file without resolving extends
    fn parse_xml_file(&self, xml_path: &Path) -> Result<Mode> {
        let xml_content = fs::read_to_string(xml_path)
            .with_context(|| format!("Failed to read XML file: {:?}", xml_path))?;

        let mut mode = from_str::<Mode>(&xml_content)
            .with_context(|| format!("Failed to parse XML from file: {:?}", xml_path))?;

        // Set the file name
        mode.file_name = Some(xml_path.to_string_lossy().to_string());

        // Validate the mode
        mode.validate()
            .with_context(|| format!("Invalid mode in file: {:?}", xml_path))?;

        Ok(mode)
    }

    /// Resolve inheritance for a specific mode
    fn resolve_mode_inheritance(
        &mut self,
        mode_id: &str,
        processed: &mut HashSet<String>,
    ) -> Result<()> {
        // Skip if already processed
        if processed.contains(mode_id) {
            return Ok(());
        }

        // Check for circular dependencies
        let mut inheritance_chain = HashSet::new();
        if let Err(e) = self.check_circular_dependency(mode_id, &mut inheritance_chain) {
            // Log the circular dependency but don't fail - just skip inheritance for this mode
            eprintln!("Warning: {}", e);
            eprintln!("Skipping inheritance resolution for mode '{}'", mode_id);

            // Mark as processed so we don't try again
            processed.insert(mode_id.to_string());

            // Mode is loaded but inheritance is skipped due to circular dependency

            return Ok(());
        }

        // Resolve the inheritance
        self.resolve_inheritance_recursive(mode_id, &mut HashSet::new(), processed)
            .with_context(|| format!("Failed to resolve inheritance for mode '{}'", mode_id))?;

        Ok(())
    }

    /// Check for circular dependencies in the inheritance chain
    fn check_circular_dependency(
        &self,
        mode_id: &str,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(mode_id) {
            let chain: Vec<String> = visited.iter().cloned().collect();
            bail!(
                "Circular inheritance detected: '{}' is already in the inheritance chain: {:?}",
                mode_id,
                chain
            );
        }

        visited.insert(mode_id.to_string());

        if let Some(mode) = self.find_mode_by_id(mode_id) {
            if let Some(extends_id) = &mode.extends {
                // Check if the parent exists
                if !self.modes.iter().any(|m| &m.id == extends_id) {
                    // Parent doesn't exist in our loaded modes, this is okay
                    // The mode will just use its own values
                    return Ok(());
                }

                // Recursively check the parent
                self.check_circular_dependency(extends_id, visited)
                    .with_context(|| {
                        format!(
                            "Failed checking circular dependency for parent '{}'",
                            extends_id
                        )
                    })?;
            }
        }

        visited.remove(mode_id);
        Ok(())
    }

    /// Recursively resolve inheritance for a mode
    fn resolve_inheritance_recursive(
        &mut self,
        mode_id: &str,
        processing: &mut HashSet<String>,
        processed: &mut HashSet<String>,
    ) -> Result<()> {
        // Skip if already processed
        if processed.contains(mode_id) {
            return Ok(());
        }

        // Check for circular dependency during processing
        if processing.contains(mode_id) {
            bail!(
                "Circular inheritance detected while processing '{}'",
                mode_id
            );
        }

        processing.insert(mode_id.to_string());

        // Get the extends field if it exists
        let extends_id = self
            .find_mode_by_id(mode_id)
            .and_then(|m| m.extends.clone());

        if let Some(parent_id) = extends_id {
            // Check if parent exists in our loaded modes
            if self.modes.iter().any(|m| m.id == parent_id) {
                // Resolve parent first
                self.resolve_inheritance_recursive(&parent_id, processing, processed)
                    .with_context(|| {
                        format!(
                            "Failed to resolve parent '{}' for mode '{}'",
                            parent_id, mode_id
                        )
                    })?;

                // Now merge parent into child
                let parent = self.find_mode_by_id(&parent_id).unwrap().clone();
                if let Some(child) = self.find_mode_by_id_mut(mode_id) {
                    child.merge_from_parent(&parent);
                }
            }
            // If parent doesn't exist, we just skip inheritance
        }

        // Mode processing complete

        processing.remove(mode_id);
        processed.insert(mode_id.to_string());

        Ok(())
    }

    // Clear all loaded modes
    pub fn clear(&mut self) {
        self.modes.clear();
        self.current_mode_index = None;
    }

    /// Load parent modes recursively from the same directory
    fn load_parent_modes(&mut self, mode_id: &str, base_path: &Path) -> Result<()> {
        let mode = self.find_mode_by_id(mode_id).cloned();

        if let Some(mode) = mode {
            if let Some(extends_id) = mode.extends {
                // Skip if parent is already loaded
                if self.modes.iter().any(|m| m.id == extends_id) {
                    return Ok(());
                }

                // Try to find parent in the same directory
                let parent_dir = base_path
                    .parent()
                    .ok_or_else(|| anyhow!("Cannot get parent directory of {:?}", base_path))?;

                // Try common naming patterns
                let possible_files = vec![
                    parent_dir.join(format!("{}.xml", extends_id)),
                    parent_dir.join(format!("{}_mode.xml", extends_id)),
                ];

                for path in possible_files {
                    if path.exists() {
                        if let Ok(parent_mode) = self.parse_xml_file(&path) {
                            if parent_mode.id == extends_id {
                                let parent_id = parent_mode.id.clone();
                                if !self.modes.iter().any(|m| m.id == parent_id) {
                                    self.modes.push(parent_mode);
                                }

                                // Recursively load parent's parents
                                self.load_parent_modes(&parent_id, &path).with_context(|| {
                                    format!("Failed to load parent modes for '{}'", parent_id)
                                })?;
                                return Ok(());
                            }
                        }
                    }
                }

                // If not found by naming convention, scan all XML files in the directory
                if let Ok(entries) = fs::read_dir(parent_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                            if let Ok(parent_mode) = self.parse_xml_file(&path) {
                                if parent_mode.id == extends_id {
                                    let parent_id = parent_mode.id.clone();
                                    if !self.modes.iter().any(|m| m.id == parent_id) {
                                        self.modes.push(parent_mode);
                                    }

                                    // Recursively load parent's parents
                                    self.load_parent_modes(&parent_id, &path)?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                // Parent not found - this is okay, mode will just use its own values
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_load_shared_xml_directory() {
        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());
        manager.register_variable("@game:".to_string(), "D:\\Games\\GTA".to_string());

        let result = manager.load_from_directory(Path::new("shared_XML"));
        assert!(
            result.is_ok(),
            "Failed to load shared_XML directory: {:?}",
            result
        );

        let loaded_count = result.unwrap();
        println!("Loaded {} modes from shared_XML", loaded_count);
        assert!(loaded_count > 0, "Should load at least some modes");

        // Check that sa_sbl mode was loaded
        assert!(
            manager.set_current_mode_by_id("sa_sbl"),
            "sa_sbl mode should be loaded"
        );
        assert_eq!(manager.get_id(), Some("sa_sbl".to_string()));
        assert_eq!(manager.get_title(), Some("GTA SA (v1.0 - SBL)".to_string()));
        assert_eq!(manager.get_game(), Some(Game::Sa));

        // Set current mode and check placeholder replacement through getters
        manager.set_current_mode_by_id("sa_sbl");
        let data = manager.get_data();
        assert!(
            data.as_ref().unwrap().contains("C:\\SannyBuilder"),
            "Data should contain substituted path"
        );

        // Check that sa_sbl_sf inherits from sa_sbl
        // Check that sa_sbl_sf mode was loaded and extends sa_sbl
        assert!(
            manager.set_current_mode_by_id("sa_sbl_sf"),
            "sa_sbl_sf mode should be loaded"
        );
        assert_eq!(manager.get_extends(), Some("sa_sbl".to_string()));

        // Set current mode to child and check inheritance through getters
        manager.set_current_mode_by_id("sa_sbl_sf");
        let child_data = manager.get_data();
        let child_library = manager.get_library();

        manager.set_current_mode_by_id("sa_sbl");
        let parent_data = manager.get_data();

        // Should have inherited fields from parent
        assert_eq!(child_data, parent_data); // Inherited from parent

        // Should have its own library entries (not inherited since it defines its own)
        assert_eq!(child_library.len(), 2);
        assert!(child_library[0].value.contains("sa.json"));
        assert!(child_library[1].value.contains("sf.fix.json"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\test".to_string());
        manager.register_variable("@game:".to_string(), "D:\\game".to_string());

        // Load test cases with circular dependencies
        let result = manager.load_from_directory(Path::new("test_cases"));

        // The load should succeed, but circular modes won't have inheritance resolved
        assert!(
            result.is_ok(),
            "Should load files even with circular dependencies"
        );

        // Modes with circular dependencies should be loaded but won't have extends resolved
        assert!(
            manager.set_current_mode_by_id("test_circular_a"),
            "Circular mode A should be loaded"
        );
    }

    #[test]
    fn test_subdirectory_scanning() {
        // Create a temporary test structure
        let test_dir = Path::new("test_subdir_scan");
        let sub_dir = test_dir.join("subdir");

        fs::create_dir_all(&sub_dir).ok();

        // Create test XML files
        let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="root_mode" title="Root Mode" game="sa">
    <data>@sb:\data\root\</data>
</mode>"#;

        let sub_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode id="sub_mode" title="Sub Mode" game="sa">
    <data>@sb:\data\sub\</data>
</mode>"#;

        fs::write(test_dir.join("root.xml"), root_xml).unwrap();
        fs::write(sub_dir.join("sub.xml"), sub_xml).unwrap();

        // Test loading
        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\test".to_string());
        manager.register_variable("@game:".to_string(), "D:\\game".to_string());
        let result = manager.load_from_directory(test_dir);
        assert!(result.is_ok());

        let count = result.unwrap();
        assert_eq!(count, 2, "Should load both root and subdirectory XML files");

        assert!(manager.set_current_mode_by_id("root_mode"));
        assert!(manager.set_current_mode_by_id("sub_mode"));

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_missing_parent_handling() {
        // Create a mode that extends a non-existent parent
        let orphan_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mode extends="nonexistent_parent" id="orphan_mode" title="Orphan Mode" game="sa">
    <data>@sb:\data\orphan\</data>
</mode>"#;

        let test_dir = Path::new("test_orphan");
        fs::create_dir_all(test_dir).ok();
        fs::write(test_dir.join("orphan.xml"), orphan_xml).unwrap();

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\test".to_string());
        manager.register_variable("@game:".to_string(), "D:\\game".to_string());
        let result = manager.load_from_directory(test_dir);

        assert!(result.is_ok(), "Should handle missing parent gracefully");

        assert!(
            manager.set_current_mode_by_id("orphan_mode"),
            "Orphan mode should be loaded"
        );

        manager.set_current_mode_by_id("orphan_mode");
        assert_eq!(
            manager.get_data(),
            Some("C:\\test\\data\\orphan\\".to_string())
        );

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_validate_all_shared_modes() {
        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());
        manager.register_variable("@game:".to_string(), "D:\\Games\\GTA".to_string());

        let result = manager.load_from_directory(Path::new("shared_XML"));
        assert!(result.is_ok(), "Failed to load shared_XML: {:?}", result);

        let loaded_count = result.unwrap();
        println!("\n=== ModeManager Validation ===");
        println!("Successfully loaded {} modes", loaded_count);

        // Collect all mode IDs first
        let mode_count = manager.mode_count();
        let mut mode_ids = Vec::new();
        for i in 0..mode_count {
            manager.set_current_mode_by_index(i);
            if let (Some(id), Some(title), Some(game)) =
                (manager.get_id(), manager.get_title(), manager.get_game())
            {
                mode_ids.push((id, title, game));
            }
        }

        // Validate each mode
        for (id, title, game) in mode_ids {
            assert!(!id.is_empty(), "Mode ID should not be empty");
            assert!(
                !title.is_empty(),
                "Mode title should not be empty for {}",
                id
            );
            // Game is now an enum, so it always has a valid value

            // Check that placeholders are replaced when accessed through getters
            manager.set_current_mode_by_id(&id);
            if let Some(data) = manager.get_data() {
                assert!(
                    !data.contains("@sb:"),
                    "Placeholder @sb: not replaced in mode {} data",
                    id
                );
                assert!(
                    !data.contains("@game:"),
                    "Placeholder @game: not replaced in mode {} data",
                    id
                );
            }

            // Check other path fields
            if let Some(compiler) = manager.get_compiler() {
                assert!(
                    !compiler.contains("@sb:"),
                    "Placeholder @sb: not replaced in mode {} compiler",
                    id
                );
                assert!(
                    !compiler.contains("@game:"),
                    "Placeholder @game: not replaced in mode {} compiler",
                    id
                );
            }

            println!("✓ {}: {} (game: {})", id, title, game);
        }

        println!("\nAll {} modes validated successfully!", loaded_count);
    }

    #[test]
    fn test_set_current_mode_by_game_prioritizes_default() {
        use quick_xml::de::from_str;

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());
        manager.register_variable("@game:".to_string(), "D:\\Games\\GTA".to_string());

        // Create modes with different type attributes
        let mode1_xml = r#"
            <mode id="sa_regular" title="SA Regular" game="sa">
                <data>@sb:\data\sa_regular\</data>
            </mode>
        "#;

        let mode2_xml = r#"
            <mode id="sa_default" title="SA Default" game="sa" type="default">
                <data>@sb:\data\sa_default\</data>
            </mode>
        "#;

        let mode3_xml = r#"
            <mode id="sa_another" title="SA Another" game="sa">
                <data>@sb:\data\sa_another\</data>
            </mode>
        "#;

        // Parse and add modes
        let mode1: Mode = from_str(mode1_xml).unwrap();
        let mode2: Mode = from_str(mode2_xml).unwrap();
        let mode3: Mode = from_str(mode3_xml).unwrap();

        // Add modes in order: regular, default, another
        manager.modes.push(mode1);
        manager.modes.push(mode2);
        manager.modes.push(mode3);

        // Test that set_current_mode_by_game selects the mode with type="default"
        assert!(manager.set_current_mode_by_game(Game::Sa));
        assert_eq!(manager.get_id(), Some("sa_default".to_string()));

        // Clear modes and test with no default type
        manager.modes.clear();
        manager.current_mode_index = None;

        let mode4_xml = r#"
            <mode id="sa_first" title="SA First" game="sa">
                <data>@sb:\data\sa_first\</data>
            </mode>
        "#;

        let mode5_xml = r#"
            <mode id="sa_second" title="SA Second" game="sa">
                <data>@sb:\data\sa_second\</data>
            </mode>
        "#;

        let mode4: Mode = from_str(mode4_xml).unwrap();
        let mode5: Mode = from_str(mode5_xml).unwrap();

        manager.modes.push(mode4);
        manager.modes.push(mode5);

        // Test that when no default exists, it selects the first matching mode
        assert!(manager.set_current_mode_by_game(Game::Sa));
        assert_eq!(manager.get_id(), Some("sa_first".to_string()));
    }

    #[test]
    fn test_index_based_getters() {
        use quick_xml::de::from_str;

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());
        manager.register_variable("@game:".to_string(), "D:\\Games\\GTA".to_string());

        // Create test modes with different properties
        let mode1_xml = r#"
            <mode id="test_mode_1" title="Test Mode 1" game="sa" type="default">
                <data>@sb:\data\test1\</data>
                <compiler>@sb:\compiler\test1.exe</compiler>
                <constants>@sb:\constants\test1.txt</constants>
                <keywords>@sb:\keywords\test1.txt</keywords>
                <opcodes>@sb:\opcodes\test1_1.txt</opcodes>
                <opcodes>@sb:\opcodes\test1_2.txt</opcodes>
                <library autoupdate="true">@sb:\library\test1.json</library>
                <classes autoupdate="false">@sb:\classes\test1.json</classes>
                <examples autoupdate="true">@sb:\examples\test1\</examples>
                <text format="sa">@sb:\text\test1.txt</text>
            </mode>
        "#;

        let mode2_xml = r#"
            <mode extends="test_mode_1" id="test_mode_2" title="Test Mode 2" game="vc">
                <data>@sb:\data\test2\</data>
                <compiler>@sb:\compiler\test2.exe</compiler>
                <opcodes>@sb:\opcodes\test2_1.txt</opcodes>
                <library autoupdate="false">@sb:\library\test2.json</library>
            </mode>
        "#;

        let mode3_xml = r#"
            <mode id="test_mode_3" title="Test Mode 3" game="gta3">
                <data>@sb:\data\test3\</data>
                <missions>@sb:\missions\test3.txt</missions>
                <labels>@sb:\labels\test3.txt</labels>
                <arrays>@sb:\arrays\test3.txt</arrays>
            </mode>
        "#;

        // Parse and add modes
        let mode1: Mode = from_str(mode1_xml).unwrap();
        let mode2: Mode = from_str(mode2_xml).unwrap();
        let mode3: Mode = from_str(mode3_xml).unwrap();

        manager.modes.push(mode1);
        manager.modes.push(mode2);
        manager.modes.push(mode3);

        // Test simple field getters with index
        assert_eq!(manager.get_id_at(0), Some("test_mode_1".to_string()));
        assert_eq!(manager.get_id_at(1), Some("test_mode_2".to_string()));
        assert_eq!(manager.get_id_at(2), Some("test_mode_3".to_string()));
        assert_eq!(manager.get_id_at(3), None); // Out of bounds

        assert_eq!(manager.get_title_at(0), Some("Test Mode 1".to_string()));
        assert_eq!(manager.get_title_at(1), Some("Test Mode 2".to_string()));
        assert_eq!(manager.get_title_at(2), Some("Test Mode 3".to_string()));

        assert_eq!(manager.get_game_at(0), Some(Game::Sa));
        assert_eq!(manager.get_game_at(1), Some(Game::Vc));
        assert_eq!(manager.get_game_at(2), Some(Game::Gta3));

        assert_eq!(manager.get_type_at(0), Some("default".to_string()));
        assert_eq!(manager.get_type_at(1), None);
        assert_eq!(manager.get_type_at(2), None);

        assert_eq!(manager.get_extends_at(0), None);
        assert_eq!(manager.get_extends_at(1), Some("test_mode_1".to_string()));
        assert_eq!(manager.get_extends_at(2), None);

        // Test path-substituted field getters with index
        assert_eq!(
            manager.get_data_at(0),
            Some("C:\\SannyBuilder\\data\\test1\\".to_string())
        );
        assert_eq!(
            manager.get_data_at(1),
            Some("C:\\SannyBuilder\\data\\test2\\".to_string())
        );
        assert_eq!(
            manager.get_data_at(2),
            Some("C:\\SannyBuilder\\data\\test3\\".to_string())
        );

        assert_eq!(
            manager.get_compiler_at(0),
            Some("C:\\SannyBuilder\\compiler\\test1.exe".to_string())
        );
        assert_eq!(
            manager.get_compiler_at(1),
            Some("C:\\SannyBuilder\\compiler\\test2.exe".to_string())
        );
        assert_eq!(manager.get_compiler_at(2), None);

        assert_eq!(
            manager.get_constants_at(0),
            Some("C:\\SannyBuilder\\constants\\test1.txt".to_string())
        );
        assert_eq!(manager.get_constants_at(1), None);
        assert_eq!(manager.get_constants_at(2), None);

        assert_eq!(
            manager.get_keywords_at(0),
            Some("C:\\SannyBuilder\\keywords\\test1.txt".to_string())
        );
        assert_eq!(manager.get_keywords_at(1), None);
        assert_eq!(manager.get_keywords_at(2), None);

        assert_eq!(manager.get_missions_at(0), None);
        assert_eq!(manager.get_missions_at(1), None);
        assert_eq!(
            manager.get_missions_at(2),
            Some("C:\\SannyBuilder\\missions\\test3.txt".to_string())
        );

        assert_eq!(manager.get_labels_at(0), None);
        assert_eq!(manager.get_labels_at(1), None);
        assert_eq!(
            manager.get_labels_at(2),
            Some("C:\\SannyBuilder\\labels\\test3.txt".to_string())
        );

        assert_eq!(manager.get_arrays_at(0), None);
        assert_eq!(manager.get_arrays_at(1), None);
        assert_eq!(
            manager.get_arrays_at(2),
            Some("C:\\SannyBuilder\\arrays\\test3.txt".to_string())
        );

        // Test complex field getters with index
        let examples_0 = manager.get_examples_at(0);
        assert!(examples_0.is_some());
        let examples = examples_0.unwrap();
        assert_eq!(examples.autoupdate, Some("true".to_string()));
        assert_eq!(examples.value, "C:\\SannyBuilder\\examples\\test1\\");

        assert_eq!(manager.get_examples_at(1), None);
        assert_eq!(manager.get_examples_at(2), None);

        let text_0 = manager.get_text_at(0);
        assert!(text_0.is_some());
        let text = text_0.unwrap();
        // Note: format is parsed as TextFormat enum, not string
        assert!(text.format.is_some());
        assert_eq!(text.value, "C:\\SannyBuilder\\text\\test1.txt");

        assert_eq!(manager.get_text_at(1), None);
        assert_eq!(manager.get_text_at(2), None);

        // Test vector field getters with index
        let opcodes_0 = manager.get_opcodes_at(0);
        assert_eq!(opcodes_0.len(), 2);
        assert_eq!(opcodes_0[0], "C:\\SannyBuilder\\opcodes\\test1_1.txt");
        assert_eq!(opcodes_0[1], "C:\\SannyBuilder\\opcodes\\test1_2.txt");

        let opcodes_1 = manager.get_opcodes_at(1);
        assert_eq!(opcodes_1.len(), 1);
        assert_eq!(opcodes_1[0], "C:\\SannyBuilder\\opcodes\\test2_1.txt");

        let opcodes_2 = manager.get_opcodes_at(2);
        assert_eq!(opcodes_2.len(), 0);

        let library_0 = manager.get_library_at(0);
        assert_eq!(library_0.len(), 1);
        assert_eq!(library_0[0].autoupdate, Some("true".to_string()));
        assert_eq!(library_0[0].value, "C:\\SannyBuilder\\library\\test1.json");

        let library_1 = manager.get_library_at(1);
        assert_eq!(library_1.len(), 1);
        assert_eq!(library_1[0].autoupdate, Some("false".to_string()));
        assert_eq!(library_1[0].value, "C:\\SannyBuilder\\library\\test2.json");

        let library_2 = manager.get_library_at(2);
        assert_eq!(library_2.len(), 0);

        let classes_0 = manager.get_classes_at(0);
        assert_eq!(classes_0.len(), 1);
        assert_eq!(classes_0[0].autoupdate, Some("false".to_string()));
        assert_eq!(classes_0[0].value, "C:\\SannyBuilder\\classes\\test1.json");

        let classes_1 = manager.get_classes_at(1);
        assert_eq!(classes_1.len(), 0);

        let classes_2 = manager.get_classes_at(2);
        assert_eq!(classes_2.len(), 0);

        // Test that current mode methods still work and delegate correctly
        manager.set_current_mode_by_index(1);
        assert_eq!(manager.get_id(), Some("test_mode_2".to_string()));
        assert_eq!(manager.get_title(), Some("Test Mode 2".to_string()));
        assert_eq!(manager.get_game(), Some(Game::Vc));
        assert_eq!(
            manager.get_data(),
            Some("C:\\SannyBuilder\\data\\test2\\".to_string())
        );

        let current_opcodes = manager.get_opcodes();
        assert_eq!(current_opcodes.len(), 1);
        assert_eq!(current_opcodes[0], "C:\\SannyBuilder\\opcodes\\test2_1.txt");

        // Verify delegation by comparing current mode methods with index-based methods
        manager.set_current_mode_by_index(0);
        assert_eq!(manager.get_id(), manager.get_id_at(0));
        assert_eq!(manager.get_title(), manager.get_title_at(0));
        assert_eq!(manager.get_game(), manager.get_game_at(0));
        assert_eq!(manager.get_data(), manager.get_data_at(0));
        assert_eq!(manager.get_compiler(), manager.get_compiler_at(0));
        assert_eq!(manager.get_opcodes(), manager.get_opcodes_at(0));
        assert_eq!(manager.get_library(), manager.get_library_at(0));
    }

    #[test]
    fn test_index_based_getters_edge_cases() {
        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\test".to_string());

        // Test with empty manager
        assert_eq!(manager.get_id_at(0), None);
        assert_eq!(manager.get_title_at(0), None);
        assert_eq!(manager.get_game_at(0), None);
        assert_eq!(manager.get_data_at(0), None);
        assert_eq!(manager.get_opcodes_at(0), Vec::<String>::new());
        assert_eq!(manager.get_library_at(0), Vec::<AutoUpdateElement>::new());

        // Test with out of bounds index
        use quick_xml::de::from_str;
        let mode_xml = r#"
            <mode id="single_mode" title="Single Mode" game="sa">
                <data>@sb:\data\single\</data>
            </mode>
        "#;
        let mode: Mode = from_str(mode_xml).unwrap();
        manager.modes.push(mode);

        // Valid index
        assert_eq!(manager.get_id_at(0), Some("single_mode".to_string()));

        // Out of bounds indices
        assert_eq!(manager.get_id_at(1), None);
        assert_eq!(manager.get_id_at(100), None);
        assert_eq!(manager.get_data_at(1), None);
        assert_eq!(manager.get_opcodes_at(1), Vec::<String>::new());

        // Test that current mode methods return None when no current mode is set
        assert_eq!(manager.get_id(), None);
        assert_eq!(manager.get_title(), None);
        assert_eq!(manager.get_data(), None);
        assert_eq!(manager.get_opcodes(), Vec::<String>::new());

        // Set current mode and verify it works
        manager.set_current_mode_by_index(0);
        assert_eq!(manager.get_id(), Some("single_mode".to_string()));
        assert_eq!(
            manager.get_data(),
            Some("C:\\test\\data\\single\\".to_string())
        );
    }

    #[test]
    fn test_new_pascal_methods() {
        use quick_xml::de::from_str;

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());

        // Create test modes
        let mode1_xml = r#"
            <mode id="sa_sbl" title="SA SBL" game="sa" type="default">
                <data>@sb:\data\sa\</data>
                <library autoupdate="yes">@sb:\library\sa.json</library>
                <classes autoupdate="no">@sb:\classes\sa.json</classes>
                <enums autoupdate="true">@sb:\enums\sa.json</enums>
            </mode>
        "#;

        let mode2_xml = r#"
            <mode extends="sa_sbl" id="sa_sbl_child" title="SA SBL Child" game="sa">
                <data>@sb:\data\sa_child\</data>
            </mode>
        "#;

        let mode3_xml = r#"
            <mode id="vc_normal" title="VC Normal" game="vc">
                <data>@sb:\data\vc\</data>
                <opcodes>@sb:\opcodes\vc_1.txt</opcodes>
                <opcodes>@sb:\opcodes\vc_2.txt</opcodes>
            </mode>
        "#;

        let mut mode1: Mode = from_str(mode1_xml).unwrap();
        let mut mode2: Mode = from_str(mode2_xml).unwrap();
        let mut mode3: Mode = from_str(mode3_xml).unwrap();

        // Set file names
        mode1.file_name = Some("/path/to/sa_sbl.xml".to_string());
        mode2.file_name = Some("/path/to/sa_sbl_child.xml".to_string());
        mode3.file_name = Some("/path/to/vc_normal.xml".to_string());

        manager.modes.push(mode1);
        manager.modes.push(mode2);
        manager.modes.push(mode3);

        // Test get_index_by_game
        assert_eq!(manager.get_index_by_game(Game::Sa), Some(0)); // Should find default
        assert_eq!(manager.get_index_by_game(Game::Vc), Some(2));
        assert_eq!(manager.get_index_by_game(Game::Gta3), None);

        // Test get_index_by_id
        assert_eq!(manager.get_index_by_id("sa_sbl"), Some(0));
        assert_eq!(manager.get_index_by_id("sa_sbl_child"), Some(1));
        assert_eq!(manager.get_index_by_id("nonexistent"), None);

        // Test get_parent_at
        assert_eq!(manager.get_parent_at(0), None); // No parent
        assert_eq!(manager.get_parent_at(1), Some(0)); // Parent is sa_sbl
        assert_eq!(manager.get_parent_at(2), None); // No parent

        // Test is_valid_at (no duplicates)
        assert!(manager.is_valid_at(0));
        assert!(manager.is_valid_at(1));
        assert!(manager.is_valid_at(2));

        // Test is_sbl_at
        assert!(manager.is_sbl_at(0)); // sa_sbl contains SBL
        assert!(manager.is_sbl_at(1)); // sa_sbl_child contains SBL
        assert!(!manager.is_sbl_at(2)); // vc_normal doesn't contain SBL

        // Test is_default with current mode
        manager.set_current_mode_by_index(0);
        assert!(manager.is_default()); // sa_sbl has type="default"
        manager.set_current_mode_by_index(1);
        assert!(!manager.is_default()); // sa_sbl_child doesn't have type="default"

        // Test get_game_raw_at
        assert_eq!(manager.get_game_raw_at(0), Some("sa".to_string()));
        assert_eq!(manager.get_game_raw_at(1), Some("sa".to_string()));
        assert_eq!(manager.get_game_raw_at(2), Some("vc".to_string()));

        // Test get_file_name_at
        assert_eq!(
            manager.get_file_name_at(0),
            Some("/path/to/sa_sbl.xml".to_string())
        );
        assert_eq!(
            manager.get_file_name_at(1),
            Some("/path/to/sa_sbl_child.xml".to_string())
        );
        assert_eq!(
            manager.get_file_name_at(2),
            Some("/path/to/vc_normal.xml".to_string())
        );

        // Test is_autoupdate_allowed_for
        assert!(manager.is_autoupdate_allowed_for("library", 0)); // autoupdate="yes"
        assert!(!manager.is_autoupdate_allowed_for("classes", 0)); // autoupdate="no"
        assert!(manager.is_autoupdate_allowed_for("enums", 0)); // autoupdate="true"
        assert!(manager.is_autoupdate_allowed_for("opcodes", 0)); // Default true

        // Test get_game_by_name
        assert_eq!(manager.get_game_by_name("sa"), Some(Game::Sa));
        assert_eq!(manager.get_game_by_name("VC"), Some(Game::Vc)); // Case insensitive
        assert_eq!(manager.get_game_by_name("gta3"), Some(Game::Gta3));
        assert_eq!(manager.get_game_by_name("sa_mobile"), Some(Game::SaMobile));
        assert_eq!(manager.get_game_by_name("unknown"), None);

        // Test get_first_of
        assert_eq!(
            manager.get_first_of("library", 0),
            Some("C:\\SannyBuilder\\library\\sa.json".to_string())
        );
        assert_eq!(
            manager.get_first_of("opcodes", 2),
            Some("C:\\SannyBuilder\\opcodes\\vc_1.txt".to_string())
        );
        assert_eq!(manager.get_first_of("opcodes", 0), None); // No opcodes in sa_sbl

        // Test get_list_of methods
        let opcodes_list = manager.get_list_of_opcodes(2);
        assert_eq!(opcodes_list.len(), 2);
        assert_eq!(opcodes_list[0], "C:\\SannyBuilder\\opcodes\\vc_1.txt");
        assert_eq!(opcodes_list[1], "C:\\SannyBuilder\\opcodes\\vc_2.txt");

        let library_list = manager.get_list_of_library(0);
        assert_eq!(library_list.len(), 1);
        assert_eq!(library_list[0], "C:\\SannyBuilder\\library\\sa.json");

        // Test that current mode methods work with new functionality
        manager.set_current_mode_by_index(0);
        assert_eq!(manager.get_parent(), None);
        assert!(manager.is_valid());
        assert!(manager.is_sbl());
        assert_eq!(
            manager.get_file_name(),
            Some("/path/to/sa_sbl.xml".to_string())
        );
        assert_eq!(manager.get_game_raw(), Some("sa".to_string()));
    }

    #[test]
    fn test_index_based_getters_comprehensive_fields() {
        use quick_xml::de::from_str;

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());

        // Create a comprehensive test mode with all possible fields
        let comprehensive_xml = r#"
            <mode id="comprehensive" title="Comprehensive Mode" game="sa" type="test">
                <data>@sb:\data\comprehensive\</data>
                <compiler>@sb:\compiler\comprehensive.exe</compiler>
                <constants>@sb:\constants\comprehensive.txt</constants>
                <keywords>@sb:\keywords\comprehensive.txt</keywords>
                <cleo-default-extensions>@sb:\cleo\comprehensive.txt</cleo-default-extensions>
                <variables>@sb:\variables\comprehensive.txt</variables>
                <labels>@sb:\labels\comprehensive.txt</labels>
                <arrays>@sb:\arrays\comprehensive.txt</arrays>
                <missions>@sb:\missions\comprehensive.txt</missions>
                <opcodes>@sb:\opcodes\comprehensive_1.txt</opcodes>
                <opcodes>@sb:\opcodes\comprehensive_2.txt</opcodes>
                <library autoupdate="true">@sb:\library\comprehensive_1.json</library>
                <library autoupdate="false">@sb:\library\comprehensive_2.json</library>
                <classes autoupdate="true">@sb:\classes\comprehensive_1.json</classes>
                <classes autoupdate="false">@sb:\classes\comprehensive_2.json</classes>
                <enums autoupdate="true">@sb:\enums\comprehensive_1.json</enums>
                <enums autoupdate="false">@sb:\enums\comprehensive_2.json</enums>
                <ide base="@sb:\ide\base.ide">@sb:\ide\comprehensive_1.ide</ide>
                <ide>@sb:\ide\comprehensive_2.ide</ide>
                <templates type="snippet">@sb:\templates\comprehensive_1.txt</templates>
                <templates type="function">@sb:\templates\comprehensive_2.txt</templates>
                <copy-directory type="data">@sb:\copy\comprehensive_data\</copy-directory>
                <copy-directory type="config">@sb:\copy\comprehensive_config\</copy-directory>
                <examples autoupdate="true">@sb:\examples\comprehensive\</examples>
                <text format="sa">@sb:\text\comprehensive.txt</text>
            </mode>
        "#;

        let mode: Mode = from_str(comprehensive_xml).unwrap();
        manager.modes.push(mode);

        // Test all field getters at index 0
        assert_eq!(
            manager.get_cleo_default_extensions_at(0),
            Some("C:\\SannyBuilder\\cleo\\comprehensive.txt".to_string())
        );
        assert_eq!(
            manager.get_mode_variables_at(0),
            Some("C:\\SannyBuilder\\variables\\comprehensive.txt".to_string())
        );

        let enums = manager.get_enums_at(0);
        assert_eq!(enums.len(), 2);
        assert_eq!(enums[0].autoupdate, Some("true".to_string()));
        assert_eq!(
            enums[0].value,
            "C:\\SannyBuilder\\enums\\comprehensive_1.json"
        );
        assert_eq!(enums[1].autoupdate, Some("false".to_string()));
        assert_eq!(
            enums[1].value,
            "C:\\SannyBuilder\\enums\\comprehensive_2.json"
        );

        let ide = manager.get_ide_at(0);
        assert_eq!(ide.len(), 2);
        assert_eq!(
            ide[0].base,
            Some("C:\\SannyBuilder\\ide\\base.ide".to_string())
        );
        assert_eq!(ide[0].value, "C:\\SannyBuilder\\ide\\comprehensive_1.ide");
        assert_eq!(ide[1].base, None);
        assert_eq!(ide[1].value, "C:\\SannyBuilder\\ide\\comprehensive_2.ide");

        let templates = manager.get_templates_at(0);
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].r#type, "snippet".to_string());
        assert_eq!(
            templates[0].value,
            "C:\\SannyBuilder\\templates\\comprehensive_1.txt"
        );
        assert_eq!(templates[1].r#type, "function".to_string());
        assert_eq!(
            templates[1].value,
            "C:\\SannyBuilder\\templates\\comprehensive_2.txt"
        );

        let copy_directory = manager.get_copy_directory_at(0);
        assert_eq!(copy_directory.len(), 2);
        assert_eq!(copy_directory[0].r#type, "data".to_string());
        assert_eq!(
            copy_directory[0].value,
            "C:\\SannyBuilder\\copy\\comprehensive_data\\"
        );
        assert_eq!(copy_directory[1].r#type, "config".to_string());
        assert_eq!(
            copy_directory[1].value,
            "C:\\SannyBuilder\\copy\\comprehensive_config\\"
        );

        // Test that all current mode methods delegate correctly
        manager.set_current_mode_by_index(0);
        assert_eq!(
            manager.get_cleo_default_extensions(),
            manager.get_cleo_default_extensions_at(0)
        );
        assert_eq!(
            manager.get_mode_variables(),
            manager.get_mode_variables_at(0)
        );
        assert_eq!(manager.get_enums(), manager.get_enums_at(0));
        assert_eq!(manager.get_ide(), manager.get_ide_at(0));
        assert_eq!(manager.get_templates(), manager.get_templates_at(0));
        assert_eq!(
            manager.get_copy_directory(),
            manager.get_copy_directory_at(0)
        );
    }

    #[test]
    fn test_untested_getter_methods() {
        use quick_xml::de::from_str;
        use crate::mode::TextFormat;

        let mut manager = ModeManager::new();
        manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());

        // Create a comprehensive test mode with all fields
        let mode_xml = r#"
            <mode id="test_mode" title="Test Mode" game="sa">
                <data>@sb:\data\sa\</data>
                <constants>@sb:\constants\sa.txt</constants>
                <cleo-default-extensions>cs</cleo-default-extensions>
                <variables>@sb:\variables\sa.txt</variables>
                <labels>@sb:\labels\sa.txt</labels>
                <arrays>@sb:\arrays\sa.txt</arrays>
                <missions>@sb:\missions\sa.txt</missions>
                <examples autoupdate="yes">@sb:\examples\sa\</examples>
                <text format="sa">@sb:\text\sa.gxt</text>
                <opcodes>@sb:\opcodes\sa_1.txt</opcodes>
                <opcodes>@sb:\opcodes\sa_2.txt</opcodes>
                <classes>@sb:\classes\sa_1.json</classes>
                <classes>@sb:\classes\sa_2.json</classes>
                <enums>@sb:\enums\sa_1.json</enums>
                <enums>@sb:\enums\sa_2.json</enums>
            </mode>
        "#;

        let mode: Mode = from_str(mode_xml).unwrap();
        manager.modes.push(mode);

        // Test constants getters
        assert_eq!(
            manager.get_constants_at(0),
            Some("C:\\SannyBuilder\\constants\\sa.txt".to_string())
        );
        manager.set_current_mode_by_index(0);
        assert_eq!(
            manager.get_constants(),
            Some("C:\\SannyBuilder\\constants\\sa.txt".to_string())
        );

        // Test cleo_default_extensions getters
        assert_eq!(
            manager.get_cleo_default_extensions_at(0),
            Some("cs".to_string())
        );
        assert_eq!(
            manager.get_cleo_default_extensions(),
            Some("cs".to_string())
        );

        // Test variables getters
        assert_eq!(
            manager.get_mode_variables_at(0),
            Some("C:\\SannyBuilder\\variables\\sa.txt".to_string())
        );
        assert_eq!(
            manager.get_mode_variables(),
            Some("C:\\SannyBuilder\\variables\\sa.txt".to_string())
        );

        // Test labels getters
        assert_eq!(
            manager.get_labels_at(0),
            Some("C:\\SannyBuilder\\labels\\sa.txt".to_string())
        );
        assert_eq!(
            manager.get_labels(),
            Some("C:\\SannyBuilder\\labels\\sa.txt".to_string())
        );

        // Test arrays getters
        assert_eq!(
            manager.get_arrays_at(0),
            Some("C:\\SannyBuilder\\arrays\\sa.txt".to_string())
        );
        assert_eq!(
            manager.get_arrays(),
            Some("C:\\SannyBuilder\\arrays\\sa.txt".to_string())
        );

        // Test missions getters
        assert_eq!(
            manager.get_missions_at(0),
            Some("C:\\SannyBuilder\\missions\\sa.txt".to_string())
        );
        assert_eq!(
            manager.get_missions(),
            Some("C:\\SannyBuilder\\missions\\sa.txt".to_string())
        );

        // Test examples getters
        let examples_at = manager.get_examples_at(0);
        assert!(examples_at.is_some());
        let examples_at = examples_at.unwrap();
        assert_eq!(examples_at.value, "C:\\SannyBuilder\\examples\\sa\\");
        assert_eq!(examples_at.autoupdate, Some("yes".to_string()));
        
        let examples = manager.get_examples();
        assert!(examples.is_some());
        let examples = examples.unwrap();
        assert_eq!(examples.value, "C:\\SannyBuilder\\examples\\sa\\");
        assert_eq!(examples.autoupdate, Some("yes".to_string()));

        // Test text getters
        let text_at = manager.get_text_at(0);
        assert!(text_at.is_some());
        let text_at = text_at.unwrap();
        assert_eq!(text_at.value, "C:\\SannyBuilder\\text\\sa.gxt");
        assert_eq!(text_at.format, Some(TextFormat::Sa));
        
        let text = manager.get_text();
        assert!(text.is_some());
        let text = text.unwrap();
        assert_eq!(text.value, "C:\\SannyBuilder\\text\\sa.gxt");
        assert_eq!(text.format, Some(TextFormat::Sa));

        // Test opcodes getters
        let opcodes_at = manager.get_opcodes_at(0);
        assert_eq!(opcodes_at.len(), 2);
        assert_eq!(opcodes_at[0], "C:\\SannyBuilder\\opcodes\\sa_1.txt");
        assert_eq!(opcodes_at[1], "C:\\SannyBuilder\\opcodes\\sa_2.txt");
        
        let opcodes = manager.get_opcodes();
        assert_eq!(opcodes.len(), 2);
        assert_eq!(opcodes[0], "C:\\SannyBuilder\\opcodes\\sa_1.txt");
        assert_eq!(opcodes[1], "C:\\SannyBuilder\\opcodes\\sa_2.txt");

        // Test get_list_of method
        let classes_list = manager.get_list_of("classes", 0);
        assert_eq!(classes_list.len(), 2);
        assert_eq!(classes_list[0], "C:\\SannyBuilder\\classes\\sa_1.json");
        assert_eq!(classes_list[1], "C:\\SannyBuilder\\classes\\sa_2.json");

        // Test get_list_of_classes
        let classes = manager.get_list_of_classes(0);
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0], "C:\\SannyBuilder\\classes\\sa_1.json");
        assert_eq!(classes[1], "C:\\SannyBuilder\\classes\\sa_2.json");

        // Test get_list_of_enums
        let enums = manager.get_list_of_enums(0);
        assert_eq!(enums.len(), 2);
        assert_eq!(enums[0], "C:\\SannyBuilder\\enums\\sa_1.json");
        assert_eq!(enums[1], "C:\\SannyBuilder\\enums\\sa_2.json");

        // Test edge cases
        assert_eq!(manager.get_constants_at(999), None);
        assert_eq!(manager.get_list_of("nonexistent", 0).len(), 0);
    }

    #[test]
    fn test_load_by_mask() {
        let manager = ModeManager::new();
        
        // Test with non-existent pattern (should return empty list)
        let result = manager.load_by_mask("nonexistent*.xml");
        assert_eq!(result.len(), 0);
        
        // Test with actual files (using the test cases directory)
        let result = manager.load_by_mask("test_cases/test_*.xml");
        // The glob may or may not find files depending on working directory
        // Just verify it returns a valid vector (no panic)
        
        // Test with absolute path that doesn't exist
        let result = manager.load_by_mask("/nonexistent/path/*.xml");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_clear_method() {
        let mut manager = ModeManager::new();
        manager.register_variable("@test:".to_string(), "value".to_string());
        
        // Add a mode
        let mode_xml = r#"<mode id="test" title="Test" game="sa"></mode>"#;
        let mode: Mode = quick_xml::de::from_str(mode_xml).unwrap();
        manager.modes.push(mode);
        manager.set_current_mode_by_index(0);
        
        assert_eq!(manager.modes.len(), 1);
        assert!(manager.current_mode_index.is_some());
        assert_eq!(manager.variables.len(), 1);
        
        // Clear only clears modes and current_mode_index, not variables
        manager.clear();
        
        assert_eq!(manager.modes.len(), 0);
        assert!(manager.current_mode_index.is_none());
        // Variables are NOT cleared by the clear() method
        assert_eq!(manager.variables.len(), 1);
    }
}
