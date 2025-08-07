mod mode;
mod modes;
mod string_variables;

use modes::ModeManager;
use std::path::Path;

fn main() {
    // Create a new ModeManager
    let mut manager = ModeManager::new();
    
    // Register the standard variables
    manager.register_variable("@sb:".to_string(), "C:\\SannyBuilder".to_string());
    manager.register_variable("@game:".to_string(), "D:\\Games\\GTA".to_string());
    
    // Load all modes from the shared_XML directory
    println!("Loading modes from shared_XML directory...\n");
    
    match manager.load_from_directory(Path::new("shared_XML")) {
        Ok(count) => {
            println!("Successfully loaded {} modes from shared_XML\n", count);
            
            // Display information about all loaded modes
            println!("=== Loaded Modes ===");
            
            // Collect mode IDs and basic info first to avoid borrow checker issues
            let mode_count = manager.mode_count();
            let mut mode_infos = Vec::new();
            for i in 0..mode_count {
                manager.set_current_mode_by_index(i);
                if let (Some(id), Some(title), Some(game)) = (manager.get_id(), manager.get_title(), manager.get_game()) {
                    let extends = manager.get_extends();
                    let mode_type = manager.get_type();
                    mode_infos.push((id, title, game, extends, mode_type));
                }
            }
            
            for (id, title, game, extends, r#type) in mode_infos {
                println!("\nMode: {}", id);
                println!("  Title: {}", title);
                println!("  Game: {}", game);
                
                if let Some(extends) = extends {
                    println!("  Extends: {}", extends);
                }
                
                if let Some(r#type) = r#type {
                    println!("  Type: {}", r#type);
                }
                
                // Set this mode as current to demonstrate path substitution
                manager.set_current_mode_by_id(&id);
                
                if let Some(data) = manager.get_data() {
                    println!("  Data path (with substitutions): {}", data);
                }
                
                println!("  Library files: {}", manager.get_library().len());
                println!("  Classes: {}", manager.get_classes().len());
                println!("  Enums: {}", manager.get_enums().len());
                println!("  IDE entries: {}", manager.get_ide().len());
                println!("  Templates: {}", manager.get_templates().len());
                println!("  Copy directories: {}", manager.get_copy_directory().len());
            }
            
            // Example: Using the new getter methods with current mode
            println!("\n=== Using Getter Methods with Current Mode ===");
            
            // Set sa_sbl as the current mode
            if manager.set_current_mode_by_id("sa_sbl") {
                println!("Current mode set to: {:?}", manager.get_id().unwrap());
                println!("  Title: {:?}", manager.get_title());
                println!("  Game: {:?}", manager.get_game());
                
                // Path properties will have @sb: and @game: replaced
                if let Some(data) = manager.get_data() {
                    println!("  Data path: {}", data);
                }
                
                if let Some(compiler) = manager.get_compiler() {
                    println!("  Compiler: {}", compiler);
                }
                
                if let Some(keywords) = manager.get_keywords() {
                    println!("  Keywords: {}", keywords);
                }
                
                println!("\n  Library files (with path substitutions):");
                for lib in manager.get_library() {
                    println!("    - {} (autoupdate: {:?})", lib.value, lib.autoupdate);
                }
                
                println!("\n  Classes (with path substitutions):");
                for class in manager.get_classes() {
                    println!("    - {} (autoupdate: {:?})", class.value, class.autoupdate);
                }
                
                println!("\n  IDE entries (with path substitutions):");
                for ide in manager.get_ide() {
                    println!("    - {} (base: {:?})", ide.value, ide.base);
                }
            }
            
            // Example: Demonstrate the new API methods
            println!("\n=== New API Methods Demo ===");
            
            // Set mode by index
            if manager.set_current_mode_by_index(0) {
                if let (Some(id), Some(title)) = (manager.get_id(), manager.get_title()) {
                    println!("Mode at index 0: {} ({})", id, title);
                }
            }
            
            // Set mode by ID
            if manager.set_current_mode_by_id("sa") {
                if let Some(title) = manager.get_title() {
                    println!("Found mode by ID 'sa': {}", title);
                }
            }
            
            // Set mode by game (first SA game mode)
            if manager.set_current_mode_by_game(mode::Game::Sa) {
                if let (Some(id), Some(title)) = (manager.get_id(), manager.get_title()) {
                    println!("\nFirst mode for SA game: {} ({})", id, title);
                }
            }
            
            // Count modes for a specific game
            let mut sa_game_count = 0;
            println!("\nModes for SA game:");
            for i in 0..manager.mode_count() {
                manager.set_current_mode_by_index(i);
                if let Some(game) = manager.get_game() {
                    if game == mode::Game::Sa {
                        if let (Some(id), Some(title)) = (manager.get_id(), manager.get_title()) {
                            println!("  - {} ({})", id, title);
                            sa_game_count += 1;
                        }
                    }
                }
            }
            println!("Total SA game modes: {}", sa_game_count);
            
            println!("\nTotal modes loaded: {}", manager.mode_count());
            
            // Example: Check inheritance with new getter methods
            println!("\n=== Inheritance Example with Getters ===");
            
            // Set sa_sbl_sf (child) as current mode
            if manager.set_current_mode_by_id("sa_sbl_sf") {
                println!("Mode 'sa_sbl_sf' extends '{}'", manager.get_extends().unwrap_or("nothing".to_string()));
                
                // Get child's values
                let child_data = manager.get_data();
                let child_keywords = manager.get_keywords();
                let child_library_count = manager.get_library().len();
                let child_classes_count = manager.get_classes().len();
                
                // Switch to parent mode
                if manager.set_current_mode_by_id("sa_sbl") {
                    let parent_data = manager.get_data();
                    let parent_keywords = manager.get_keywords();
                    let parent_library_count = manager.get_library().len();
                    let parent_classes_count = manager.get_classes().len();
                    
                    println!("\nInherited from parent:");
                    if child_data == parent_data {
                        println!("  - data: {}", child_data.unwrap_or("none".to_string()));
                    }
                    if child_keywords == parent_keywords {
                        println!("  - keywords: {}", child_keywords.unwrap_or("none".to_string()));
                    }
                    
                    println!("\nOverridden in child:");
                    println!("  - library: {} entries (parent has {})", child_library_count, parent_library_count);
                    println!("  - classes: {} entries (parent has {})", child_classes_count, parent_classes_count);
                }
            }
            
        }
        Err(e) => {
            eprintln!("Failed to load modes from shared_XML: {:?}", e);
        }
    }
}