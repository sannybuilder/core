use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use crc32fast::Hasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// Read a null-terminated UTF-16 LE string from a reader
fn read_wide_string<R: Read>(reader: &mut R) -> Result<String> {
    let mut buffer = Vec::new();
    let mut char_buf = [0u8; 2];

    // Read until we find a null terminator (0x0000)
    loop {
        reader.read_exact(&mut char_buf)?;
        if char_buf[0] == 0 && char_buf[1] == 0 {
            break;
        }
        buffer.push(char_buf[0]);
        buffer.push(char_buf[1]);
    }

    // Convert UTF-16 LE to String
    let utf16_values: Vec<u16> = buffer
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(String::from_utf16_lossy(&utf16_values))
}

/// Common trait for GXT text parsers
pub trait GxtText {
    /// Get a text value by its key
    fn get(&self, key: &str) -> Option<String>;

    /// Get all keys
    fn keys(&self) -> Vec<String>;

    /// Get the number of entries
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Represents a table entry in GXT VC/SA formats
#[derive(Debug, Clone)]
struct TableEntry {
    /// 8-byte table name
    name: [u8; 8],
    /// Offset to the table's TKEY section
    offset: u32,
}

impl TableEntry {
    /// Get the table name as a string (null-terminated)
    fn name_as_string(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(8);
        String::from_utf8_lossy(&self.name[..end]).to_string()
    }
}

/// Represents different key types used in GXT formats
#[derive(Debug, Clone)]
enum GxtKey {
    /// Text key (8 bytes, used in VC format)
    Text([u8; 8]),
    /// Hash key (32-bit JAMCRC32, used in SA format)
    Hash(u32),
}

impl GxtKey {
    /// Get a string representation of the key
    fn to_string(&self) -> String {
        match self {
            GxtKey::Text(bytes) => {
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(8);
                String::from_utf8_lossy(&bytes[..end]).to_string()
            }
            GxtKey::Hash(hash) => format!("{:08x}", hash),
        }
    }

    /// Convert to uppercase string for VC-style lookup
    fn to_uppercase_string(&self) -> String {
        match self {
            GxtKey::Text(_) => self.to_string().to_uppercase(),
            GxtKey::Hash(hash) => format!("{:08x}", hash),
        }
    }
}

/// Represents a key entry in GXT formats
#[derive(Debug, Clone)]
struct GxtKeyEntry {
    /// Offset to the text data
    offset: u32,
    /// The key (either text or hash)
    key: GxtKey,
}

/// Text encoding for GXT strings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextEncoding {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 LE encoding
    Utf16,
}

/// Configuration for GXT format parsing
#[derive(Debug, Clone, Copy)]
pub struct GxtConfig {
    /// Whether keys are hashed (true) or text-based (false)
    pub is_hashed: bool,
    /// Text encoding for string data
    pub encoding: TextEncoding,
    /// Whether the format supports multiple tables (true) or single table (false)
    pub is_multi_table: bool,
    /// Whether the format has an 8-byte header that should be skipped (San Andreas only)
    pub has_header: bool,
}

impl Default for GxtConfig {
    fn default() -> Self {
        Self {
            is_hashed: false,
            encoding: TextEncoding::Utf16,
            is_multi_table: false,
            has_header: false,
        }
    }
}

/// Unified GXT parser that handles all format variations
///
/// # Examples
///
/// ```no_run
/// use gxt_parser::{GxtParser, GxtConfig, TextEncoding};
///
/// // Create a parser with a custom configuration
/// let config = GxtConfig {
///     is_hashed: false,        // Text-based keys
///     encoding: TextEncoding::Utf16,  // UTF-16 encoding
///     is_multi_table: true,    // Multiple tables support
///     has_header: false,       // No header to skip
/// };
///
/// let mut parser = GxtParser::new(config);
///
/// // Or use a predefined configuration
/// let mut parser_vc = GxtParser::new(GxtConfig::default());
/// let mut parser_sa = GxtParser::new(GxtConfig{
///     encoding: TextEncoding::Utf8,
///     ..Default::default()
/// });
/// ```
pub struct GxtParser {
    /// Map of keys to their text values (for text-based keys)
    entries: HashMap<String, String>,
    /// Map of hash values to text (for hash-based keys)
    hash_entries: HashMap<u32, String>,
    /// Tables for debugging
    tables: HashMap<String, Vec<String>>,
    /// Configuration for this parser
    config: GxtConfig,
}

impl GxtParser {
    /// Create a new GXT parser with the specified configuration
    pub fn new(config: GxtConfig) -> Self {
        Self {
            entries: HashMap::new(),
            hash_entries: HashMap::new(),
            tables: HashMap::new(),
            config,
        }
    }

    /// Calculate JAMCRC32 hash for a key string (for SA format)
    pub fn calculate_hash(key: &str) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(key.to_uppercase().as_bytes());
        !hasher.finalize() // JAMCRC32 = ~CRC32
    }

    /// Load a GXT file from the given path
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open GXT file: {}", path.display()))?;
        self.load_from_reader(&mut file)
    }

    /// Load GXT data from a reader
    pub fn load_from_reader<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        self.load_format(reader)
    }

    /// Unified method to load GXT data based on config
    fn load_format<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        let mut section_name = [0u8; 4];
        reader.read_exact(&mut section_name)?;
        match &section_name {
            b"TKEY" => {
                self.config.is_multi_table = false;
                self.config.has_header = false;
                self.config.is_hashed = false;
            }
            b"TABL" => {
                self.config.is_multi_table = true;
                self.config.has_header = false;
                self.config.is_hashed = false;
            }
            _ => {
                self.config.has_header = true;
                self.config.is_hashed = true;
                self.config.is_multi_table = true;
            }
        }
        reader.rewind()?;

        // Skip header if present (4 bytes)
        if self.config.has_header {
            reader
                .seek(SeekFrom::Start(4))
                .context("Failed to skip header")?;
        }

        fn expect_tag<R: Read + Seek>(reader: &mut R, tag: &[u8]) -> Result<()> {
            let mut section_name = [0u8; 4];
            reader
                .read_exact(&mut section_name)
                .context("Failed to read section header")?;
            if &section_name != tag {
                bail!(
                    "Expected {}, found: {:?}",
                    String::from_utf8_lossy(tag),
                    String::from_utf8_lossy(&section_name)
                );
            }
            Ok(())
        }

        // Handle multi-table formats (Vice City and San Andreas)
        let table_entries = if self.config.is_multi_table {
            // Read TABL section
            let tabl_size = {
                expect_tag(reader, b"TABL")?;

                // Read section size
                let section_size = reader
                    .read_i32::<LittleEndian>()
                    .context("Failed to read TABL section size")?;

                if section_size < 0 {
                    bail!("Invalid TABL section size: {}", section_size);
                }
                section_size as u32
            };

            if tabl_size % 12 != 0 {
                bail!("Invalid TABL section size: {}", tabl_size);
            }

            let num_tables = tabl_size / 12;

            // Read all table entries
            let mut table_entries = Vec::with_capacity(num_tables as usize);
            for i in 0..num_tables {
                let mut name = [0u8; 8];
                reader
                    .read_exact(&mut name)
                    .with_context(|| format!("Failed to read name for table {}", i))?;
                let offset = reader
                    .read_u32::<LittleEndian>()
                    .with_context(|| format!("Failed to read offset for table {}", i))?;

                table_entries.push(TableEntry { name, offset });
            }
            table_entries
        } else {
            let mut table_entries = Vec::with_capacity(1);
            table_entries.push(TableEntry {
                name: [0u8; 8],
                offset: 0,
            });
            table_entries
        };

        // Process each table
        for (table_idx, table) in table_entries.iter().enumerate() {
            let table_name = table.name_as_string();
            let mut table_keys = Vec::new();

            // Seek to the table's data (only for multi-table formats)
            if self.config.is_multi_table {
                let seek_pos = if table_idx == 0 {
                    table.offset
                } else {
                    table.offset + 8
                };

                reader
                    .seek(SeekFrom::Start(seek_pos as u64))
                    .with_context(|| {
                        format!("Failed to seek to table at offset {}", table.offset)
                    })?;
            }

            // Read TKEY section
            expect_tag(reader, b"TKEY")?;

            // Read TKEY section size
            let tkey_size = reader
                .read_u32::<LittleEndian>()
                .context("Failed to read TKEY section size")?;

            // Validate TKEY size based on format
            let key_record_size = if self.config.is_hashed { 8 } else { 12 };
            if tkey_size % key_record_size != 0 {
                bail!("Invalid TKEY section size: {}", tkey_size);
            }

            let num_keys = tkey_size / key_record_size;

            // Read all key records
            let mut keys = Vec::with_capacity(num_keys as usize);
            for i in 0..num_keys {
                let offset = reader
                    .read_u32::<LittleEndian>()
                    .with_context(|| format!("Failed to read offset for key {}", i))?;
                let key_entry = if self.config.is_hashed {
                    // SA format: 4 bytes offset, 4 bytes hash
                    let hash = reader
                        .read_u32::<LittleEndian>()
                        .with_context(|| format!("Failed to read hash for key {}", i))?;

                    table_keys.push(format!("{:08x}", hash));
                    GxtKeyEntry {
                        offset,
                        key: GxtKey::Hash(hash),
                    }
                } else {
                    // VC format: 4 bytes offset, 8 bytes name
                    let mut name = [0u8; 8];
                    reader
                        .read_exact(&mut name)
                        .with_context(|| format!("Failed to read name for key {}", i))?;

                    table_keys.push(GxtKey::Text(name).to_string());
                    GxtKeyEntry {
                        offset,
                        key: GxtKey::Text(name),
                    }
                };
                keys.push(key_entry);
            }

            // Read TDAT section
            expect_tag(reader, b"TDAT")?;

            // Read TDAT section size
            let tdat_size = reader
                .read_u32::<LittleEndian>()
                .context("Failed to read TDAT section size")?;

            // Process string data
            let mut table_strings = vec![0u8; tdat_size as usize];
            reader
                .read_exact(&mut table_strings)
                .context("Failed to read string data")?;

            // Process each key and extract its string
            for key_entry in &keys {
                let offset = key_entry.offset as usize;
                if offset >= table_strings.len() {
                    continue;
                }

                let text = match self.config.encoding {
                    TextEncoding::Utf16 => {
                        // Read UTF-16 string
                        let mut cursor = Cursor::new(&table_strings[offset..]);
                        read_wide_string(&mut cursor).ok()
                    }
                    TextEncoding::Utf8 => {
                        // Read UTF-8/ASCII string
                        let end = table_strings[offset..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|pos| offset + pos)
                            .unwrap_or(table_strings.len());

                        String::from_utf8(table_strings[offset..end].to_vec()).ok()
                    }
                };

                if let Some(text) = text {
                    if self.config.is_hashed {
                        if let GxtKey::Hash(hash) = key_entry.key {
                            self.hash_entries.insert(hash, text);
                        }
                    } else {
                        self.entries
                            .insert(key_entry.key.to_uppercase_string(), text);
                    }
                }
            }

            self.tables.insert(table_name, table_keys);
        }
        Ok(())
    }

    /// Get a text value by its key
    pub fn get(&self, key: &str) -> Option<String> {
        if self.config.is_hashed {
            // First try to parse as a hex hash value (e.g., "15d4d373")
            if let Ok(hash) = u32::from_str_radix(key, 16) {
                if let Some(value) = self.hash_entries.get(&hash) {
                    return Some(value.clone());
                }
            }
            // Otherwise try as a regular key name (will be hashed)
            let hash = Self::calculate_hash(key);
            if let Some(value) = self.hash_entries.get(&hash) {
                return Some(value.clone());
            }
            None
        } else {
            // Text-based lookup (case-insensitive)
            self.entries.get(&key.to_uppercase()).cloned()
        }
    }

    /// Get a text value by its key string (will be hashed with JAMCRC32 for SA format)
    pub fn get_by_key(&self, key: &str) -> Option<String> {
        self.get(key)
    }

    /// Get all known hashes (for SA format)
    pub fn hashes(&self) -> Vec<u32> {
        if self.config.is_hashed {
            self.hash_entries.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        if self.config.is_hashed {
            self.hash_entries
                .keys()
                .map(|&hash| format!("{:08x}", hash))
                .collect()
        } else {
            self.entries.keys().cloned().collect()
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        if self.config.is_hashed {
            self.hash_entries.len()
        } else {
            self.entries.len()
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get table names
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Get keys for a specific table
    pub fn table_keys(&self, table_name: &str) -> Option<&Vec<String>> {
        self.tables.get(table_name)
    }
}

impl GxtText for GxtParser {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key)
    }

    fn keys(&self) -> Vec<String> {
        self.keys()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_gxt_key_to_string() {
        let key = GxtKey::Text([b'T', b'E', b'S', b'T', 0, 0, 0, 0]);
        assert_eq!(key.to_string(), "TEST");

        let key_full = GxtKey::Text([b'F', b'U', b'L', b'L', b'N', b'A', b'M', b'E']);
        assert_eq!(key_full.to_string(), "FULLNAME");

        let hash_key = GxtKey::Hash(0x12345678);
        assert_eq!(hash_key.to_string(), "12345678");
    }

    #[test]
    fn test_parse_empty_gxt_iii() {
        let mut data = Vec::new();
        // TKEY header
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&0i32.to_le_bytes()); // section size = 0
                                                     // TDAT header
        data.extend_from_slice(b"TDAT");
        data.extend_from_slice(&0i32.to_le_bytes()); // section size = 0

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_ok());
        assert_eq!(parser.len(), 0);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_gxt_iii_with_data() {
        let mut data = Vec::new();

        // TKEY header
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&12i32.to_le_bytes()); // section size = 12 (1 key)

        // Key record
        data.extend_from_slice(&0i32.to_le_bytes()); // offset = 0
        data.extend_from_slice(b"TEST\0\0\0\0"); // key name

        // TDAT header
        data.extend_from_slice(b"TDAT");
        data.extend_from_slice(&12i32.to_le_bytes()); // section size (12 bytes for "Hello\0" in UTF-16)

        // Text data (UTF-16 LE "Hello")
        data.extend_from_slice(&[b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_ok());
        assert_eq!(parser.len(), 1);

        // Test case-insensitive lookup
        assert_eq!(parser.get("TEST"), Some("Hello".to_string()));
        assert_eq!(parser.get("test"), Some("Hello".to_string()));
        assert_eq!(parser.get("Test"), Some("Hello".to_string()));
        assert_eq!(parser.get("TeSt"), Some("Hello".to_string()));
    }

    #[test]
    fn test_invalid_header_iii() {
        let data = b"INVALID_HEADER";
        let mut cursor = Cursor::new(data.to_vec());
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_empty_gxt_vc() {
        let mut data = Vec::new();
        // TABL header
        data.extend_from_slice(b"TABL");
        data.extend_from_slice(&0i32.to_le_bytes()); // section size = 0

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_ok());
        assert_eq!(parser.len(), 0);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_invalid_header_vc() {
        let data = b"INVALID_HEADER";
        let mut cursor = Cursor::new(data.to_vec());
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_err());
    }

    #[test]
    fn test_table_entry_name() {
        let entry = TableEntry {
            name: [b'M', b'A', b'I', b'N', 0, 0, 0, 0],
            offset: 100,
        };
        assert_eq!(entry.name_as_string(), "MAIN");
    }

    #[test]
    fn test_invalid_section_size() {
        let mut data = Vec::new();
        // TKEY header with invalid size
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&13i32.to_le_bytes()); // Invalid: not divisible by 12

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        let result = parser.load_from_reader(&mut cursor);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid TKEY section size"));
    }

    #[test]
    fn test_case_insensitive_vc() {
        let mut data = Vec::new();

        // TABL header
        data.extend_from_slice(b"TABL");
        data.extend_from_slice(&12i32.to_le_bytes()); // section size = 12 (1 table)

        // Table entry
        data.extend_from_slice(b"MAIN\0\0\0\0"); // table name
        data.extend_from_slice(&20u32.to_le_bytes()); // offset to TKEY

        // TKEY header
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&12u32.to_le_bytes()); // section size = 12 (1 key)

        // Key record
        data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
        data.extend_from_slice(b"MYKEY\0\0\0"); // key name

        // TDAT header
        data.extend_from_slice(b"TDAT");
        data.extend_from_slice(&12u32.to_le_bytes()); // section size (12 bytes for "World\0" in UTF-16)

        // Text data (UTF-16 LE "World")
        data.extend_from_slice(&[b'W', 0, b'o', 0, b'r', 0, b'l', 0, b'd', 0, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_ok());
        assert_eq!(parser.len(), 1);

        // Test case-insensitive lookup for VC format
        assert_eq!(parser.get("MYKEY"), Some("World".to_string()));
        assert_eq!(parser.get("mykey"), Some("World".to_string()));
        assert_eq!(parser.get("MyKey"), Some("World".to_string()));
        assert_eq!(parser.get("mYkEy"), Some("World".to_string()));
    }

    #[test]
    fn test_case_insensitive_sa() {
        // Test that SA format hashing is case-insensitive
        let hash1 = GxtParser::calculate_hash("TEST");
        let hash2 = GxtParser::calculate_hash("test");
        let hash3 = GxtParser::calculate_hash("Test");
        let hash4 = GxtParser::calculate_hash("TeSt");

        // All hashes should be the same due to uppercasing before hashing
        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
        assert_eq!(hash1, hash4);
    }

    #[test]
    fn test_custom_config() {
        // Test that we can create custom configurations
        let config = GxtConfig {
            is_hashed: false,
            encoding: TextEncoding::Utf16,
            is_multi_table: false,
            has_header: false,
        };

        let parser = GxtParser::new(config);
        assert!(parser.is_empty());

        // Test a different custom config
        let config2 = GxtConfig {
            is_hashed: true,
            encoding: TextEncoding::Utf8,
            is_multi_table: false, // Single table with hashes (hypothetical format)
            has_header: false,
        };

        let parser2 = GxtParser::new(config2);
        assert!(parser2.is_empty());
    }

    #[test]
    fn test_gxt_trait_case_insensitive() {
        // Test that the GxtText trait methods are case-insensitive
        let mut data = Vec::new();

        // TKEY header
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&12i32.to_le_bytes());

        // Key record
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(b"HELLO\0\0\0");

        // TDAT header
        data.extend_from_slice(b"TDAT");
        data.extend_from_slice(&12i32.to_le_bytes()); // section size (12 bytes for "Greet\0" in UTF-16)

        // Text data (UTF-16 LE "Greet")
        data.extend_from_slice(&[b'G', 0, b'r', 0, b'e', 0, b'e', 0, b't', 0, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new(GxtConfig::default());

        assert!(parser.load_from_reader(&mut cursor).is_ok());

        // Test through the trait interface
        let gxt: &dyn GxtText = &parser;
        assert_eq!(gxt.get("HELLO"), Some("Greet".to_string()));
        assert_eq!(gxt.get("hello"), Some("Greet".to_string()));
        assert_eq!(gxt.get("Hello"), Some("Greet".to_string()));
    }
}
