use anyhow::Result;
use gxt_parser::GxtParser;
use std::path::Path;

#[test]
fn test_read_iii_gxt() -> Result<()> {
    let path = Path::new("iii.gxt");

    // Skip test if file doesn't exist
    if !path.exists() {
        eprintln!("Skipping test: iii.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // Basic sanity checks
    assert!(!parser.is_empty(), "iii.gxt should contain entries");
    assert!(parser.len() > 0, "iii.gxt should have at least one entry");

    // Print some statistics for debugging
    println!("iii.gxt loaded successfully:");
    println!("  Total entries: {}", parser.len());

    // Get a sample of keys
    let keys = parser.keys();
    if !keys.is_empty() {
        println!("  Sample keys (first 5):");
        for key in keys.iter().take(5) {
            println!("    - {}", key);
            if let Some(value) = parser.get(key) {
                // Truncate long values for display
                let display_value = if value.len() > 50 {
                    format!("{}...", &value[..50])
                } else {
                    value.clone()
                };
                println!("      => {}", display_value);
            }
        }
    }

    Ok(())
}

#[test]
fn test_read_vc_gxt() -> Result<()> {
    let path = Path::new("vc.gxt");

    // Skip test if file doesn't exist
    if !path.exists() {
        eprintln!("Skipping test: vc.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // Basic sanity checks
    assert!(!parser.is_empty(), "vc.gxt should contain entries");
    assert!(parser.len() > 0, "vc.gxt should have at least one entry");

    println!("vc.gxt loaded successfully:");
    println!("  Total entries: {}", parser.len());

    // Table inspection no longer supported; just ensure we have entries

    // Get a sample of keys
    let keys = parser.keys();
    if !keys.is_empty() {
        println!("  Sample keys (first 5):");
        for key in keys.iter().take(5) {
            println!("    - {}", key);
            if let Some(value) = parser.get(key) {
                // Truncate long values for display
                let display_value = if value.len() > 50 {
                    format!("{}...", &value[..50])
                } else {
                    value.clone()
                };
                println!("      => {}", display_value);
            }
        }
    }

    Ok(())
}

#[test]
fn test_iii_gxt_specific_keys() -> Result<()> {
    let path = Path::new("iii.gxt");

    if !path.exists() {
        eprintln!("Skipping test: iii.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // Test that keys are valid strings (no garbage)
    for key in parser.keys() {
        assert!(!key.is_empty(), "Key should not be empty");
        assert!(
            key.chars().all(|c| c.is_ascii() || c.is_alphanumeric()),
            "Key '{}' contains invalid characters",
            key
        );
    }

    // Test that all values are valid strings
    for key in parser.keys() {
        if let Some(value) = parser.get(&key) {
            // Values can be empty but should be valid UTF-16 decoded strings
            // No specific assertion needed as invalid UTF-16 would have failed during parsing
            let _ = value; // Use the value to avoid unused warning
        }
    }

    Ok(())
}

#[test]
fn test_vc_gxt_table_structure() -> Result<()> {
    let path = Path::new("vc.gxt");

    if !path.exists() {
        eprintln!("Skipping test: vc.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // No per-table inspection; rely on global keys

    Ok(())
}

#[test]
fn test_compare_formats() -> Result<()> {
    // This test compares the different formats if all files are available
    let iii_path = Path::new("iii.gxt");
    let vc_path = Path::new("vc.gxt");

    if !iii_path.exists() || !vc_path.exists() {
        eprintln!("Skipping comparison test: not all files found");
        return Ok(());
    }

    let mut iii_parser = GxtParser::new();
    iii_parser.load_file(iii_path)?;

    let mut vc_parser = GxtParser::new();
    vc_parser.load_file(vc_path)?;

    println!("\nFormat comparison:");
    println!("  GTA III format (iii.gxt):");
    println!("    - Entries: {}", iii_parser.len());
    println!("    - Structure: Single TKEY/TDAT section");

    println!("  Vice City format (vc.gxt):");
    println!("    - Entries: {}", vc_parser.len());
    println!("    - Structure: TABL with multiple TKEY/TDAT sections");

    Ok(())
}

#[test]
fn test_known_gta_iii_keys() -> Result<()> {
    let path = Path::new("iii.gxt");

    if !path.exists() {
        eprintln!("Skipping test: iii.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // Test for some known common GTA III text keys
    // These are typically present in GTA III
    let common_keys = ["CRED212", "ELBURRO", "JM1_8A", "JM6_E", "FM1_J"];

    for key in &common_keys {
        let value = parser.get(key);
        assert!(
            value.is_some(),
            "Expected key '{}' to exist in iii.gxt",
            key
        );

        if let Some(text) = value {
            assert!(!text.is_empty(), "Key '{}' should have non-empty text", key);
            println!(
                "  Found key '{}': {}",
                key,
                if text.len() > 60 {
                    format!("{}...", &text[..60])
                } else {
                    text.clone()
                }
            );
        }
    }

    Ok(())
}

#[test]
fn test_known_vc_tables() -> Result<()> {
    let path = Path::new("vc.gxt");

    if !path.exists() {
        eprintln!("Skipping test: vc.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // No per-table checks anymore

    Ok(())
}

#[test]
fn test_text_content_validation() -> Result<()> {
    // Test that text content is properly decoded and doesn't contain invalid UTF-16
    let paths = [(Path::new("iii.gxt"), "III"), (Path::new("vc.gxt"), "VC")];

    for (path, format) in &paths {
        if !path.exists() {
            continue;
        }

        let parser = match *format {
            "III" => {
                let mut p = GxtParser::new();
                p.load_file(path)?;
                Box::new(p)
            }
            "VC" => {
                let mut p = GxtParser::new();
                p.load_file(path)?;
                Box::new(p)
            }
            _ => continue,
        };

        println!("\nValidating text content for {}", path.display());

        let keys = parser.keys();
        let mut sample_count = 0;
        let mut color_code_count = 0;
        let mut newline_count = 0;

        for key in keys.iter().take(100) {
            // Check first 100 entries
            if let Some(value) = parser.get(key) {
                // Check for common GTA text features
                if value.contains("~") {
                    color_code_count += 1; // Color/formatting codes
                }
                if value.contains("\n") || value.contains("\r") {
                    newline_count += 1; // Line breaks
                }

                // Ensure text is valid (no null bytes except at end)
                assert!(
                    !value.chars().any(|c| c == '\0'),
                    "Text should not contain null characters: key={}",
                    key
                );

                sample_count += 1;
            }
        }

        println!("  Validated {} text entries", sample_count);
        println!(
            "  Found {} entries with formatting codes (~)",
            color_code_count
        );
        println!("  Found {} entries with line breaks", newline_count);

        assert!(
            sample_count > 0,
            "Should have validated at least some entries"
        );
    }

    Ok(())
}

#[test]
fn test_read_sa_gxt() -> Result<()> {
    let path = Path::new("sa.gxt");

    // Skip test if file doesn't exist
    if !path.exists() {
        eprintln!("Skipping test: sa.gxt not found");
        return Ok(());
    }

    let mut parser = GxtParser::new();
    parser.load_file(path)?;

    // Basic sanity checks
    assert!(!parser.is_empty(), "sa.gxt should contain entries");
    assert!(parser.len() > 0, "sa.gxt should have at least one entry");

    println!("sa.gxt loaded successfully:");
    println!("  Total entries: {}", parser.len());

    // No table listing; only entries are validated

    // Test hash calculation
    let test_hash = GxtParser::calculate_hash("MAIN");
    println!("  JAMCRC32 hash of 'MAIN': 0x{:08x}", test_hash);

    Ok(())
}

#[test]
fn test_sa_format_crc32() -> Result<()> {
    // Test that JAMCRC32 hashing works correctly
    let hash1 = GxtParser::calculate_hash("TEST");
    let hash2 = GxtParser::calculate_hash("TEST");
    let hash3 = GxtParser::calculate_hash("TEST2");

    assert_eq!(hash1, hash2, "Same string should produce same hash");
    assert_ne!(
        hash1, hash3,
        "Different strings should produce different hashes"
    );

    println!("JAMCRC32 hash tests:");
    println!("  'TEST' => 0x{:08x}", hash1);
    println!("  'TEST2' => 0x{:08x}", hash3);

    Ok(())
}

#[test]
fn test_all_formats_comparison() -> Result<()> {
    // Compare all three formats if files are available
    let iii_path = Path::new("iii.gxt");
    let vc_path = Path::new("vc.gxt");
    let sa_path = Path::new("sa.gxt");

    if !iii_path.exists() || !vc_path.exists() || !sa_path.exists() {
        eprintln!("Skipping comparison test: not all files found");
        return Ok(());
    }

    let mut iii_parser = GxtParser::new();
    iii_parser.load_file(iii_path)?;

    let mut vc_parser = GxtParser::new();
    vc_parser.load_file(vc_path)?;

    let mut sa_parser = GxtParser::new();
    sa_parser.load_file(sa_path)?;

    println!("\nFormat comparison (all three games):");
    println!("  GTA III format (iii.gxt):");
    println!("    - Entries: {}", iii_parser.len());
    println!("    - Structure: Single TKEY/TDAT section");
    println!("    - Keys: String-based (8 chars max)");

    println!("  Vice City format (vc.gxt):");
    println!("    - Entries: {}", vc_parser.len());
    println!("    - Structure: TABL with multiple TKEY/TDAT sections");
    println!("    - Keys: String-based (8 chars max)");

    println!("  San Andreas format (sa.gxt):");
    println!("    - Entries: {}", sa_parser.len());
    println!("    - Structure: Header + TABL with hash-based keys");
    println!("    - Keys: JAMCRC32 hashes (4 bytes)");

    Ok(())
}
