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

    String::from_utf16(&utf16_values).map_err(|e| anyhow::anyhow!("Invalid UTF-16: {}", e))
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
            GxtKey::Hash(hash) => format!("{:08X}", hash),
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

// GXT format properties are now part of the parser and auto-detected during load

/// Unified GXT parser that handles all format variations
///
/// # Examples
///
/// ```no_run
/// use gxt_parser::GxtParser;
///
/// // Create a new parser (format and encoding will be auto-detected on load)
/// let mut parser = GxtParser::new();
/// ```
pub struct GxtParser {
    /// Map of keys to their text values (for text-based keys)
    entries: HashMap<String, String>,
    /// Map of hash values to text (for hash-based keys)
    hash_entries: HashMap<u32, String>,
    /// Whether keys are hashed (true) or text-based (false)
    is_hashed: bool,
    /// Text encoding for string data
    encoding: TextEncoding,
    /// Whether the format supports multiple tables (true) or single table (false)
    is_multi_table: bool,
}

impl GxtParser {
    /// Create a new GXT parser (properties will be auto-detected on load)
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            hash_entries: HashMap::new(),
            encoding: TextEncoding::Utf16,
            is_hashed: false,
            is_multi_table: false,
        }
    }

    /// Calculate JAMCRC32 hash for a key string (for SA format)
    pub fn calculate_hash(key: &str) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(key.to_uppercase().as_bytes());
        !hasher.finalize() // JAMCRC32 = ~CRC32
    }

    /// Load a GXT file from the given path
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open GXT file: {}", path.display()))?;
        self.read(&mut file)
    }

    fn read<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        let mut section_name = [0u8; 4];
        reader.read_exact(&mut section_name)?;
        match &section_name {
            b"TKEY" => {
                self.is_multi_table = false;
                self.is_hashed = false;
                self.encoding = TextEncoding::Utf16;
                reader.rewind()?;
            }
            b"TABL" => {
                self.is_multi_table = true;
                self.is_hashed = false;
                self.encoding = TextEncoding::Utf16;
                reader.rewind()?;
            }
            _ => {
                self.is_hashed = true;
                self.is_multi_table = true;
                reader.seek(SeekFrom::Start(2))?;

                // read bits per character
                let encoding = reader.read_u16::<LittleEndian>()?;
                match encoding {
                    16 => self.encoding = TextEncoding::Utf16,
                    8 => self.encoding = TextEncoding::Utf8,
                    _ => bail!("Invalid encoding: {}", encoding),
                }
            }
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

        // handle multi-table formats (Vice City and San Andreas)
        let table_entries = if self.is_multi_table {
            // read TABL section
            let tabl_size = {
                expect_tag(reader, b"TABL")?;

                // read section size
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

            // process each table entry immediately
            let mut table_offsets: Vec<u32> = Vec::with_capacity(num_tables as usize);
            for i in 0..num_tables {
                // skip table name
                reader.seek(SeekFrom::Current(8))?;
                let offset = reader
                    .read_u32::<LittleEndian>()
                    .with_context(|| format!("Failed to read offset for table {}", i))?;

                table_offsets.push(offset);
            }
            table_offsets
        } else {
            // single table: offset 0 (no extra header)
            vec![0u32]
        };

        // read each table
        for (idx, off) in table_entries.iter().enumerate() {
            if self.is_multi_table {
                let seek_pos = if idx == 0 {
                    *off as u64
                } else {
                    (*off + 8) as u64
                };
                reader
                    .seek(SeekFrom::Start(seek_pos))
                    .with_context(|| format!("Failed to seek to table at offset {}", off))?;
            }

            // read TKEY section
            expect_tag(reader, b"TKEY")?;

            // read TKEY section size
            let tkey_size = reader
                .read_u32::<LittleEndian>()
                .context("Failed to read TKEY section size")?;

            // validate TKEY size
            let key_record_size = if self.is_hashed { 8 } else { 12 };
            if tkey_size % key_record_size != 0 {
                bail!("Invalid TKEY section size: {}", tkey_size);
            }

            let num_keys = tkey_size / key_record_size;

            // read all key records
            let mut keys = Vec::with_capacity(num_keys as usize);
            for i in 0..num_keys {
                let offset = reader
                    .read_u32::<LittleEndian>()
                    .with_context(|| format!("Failed to read offset for key {}", i))?;
                let key_entry = if self.is_hashed {
                    let hash = reader
                        .read_u32::<LittleEndian>()
                        .with_context(|| format!("Failed to read hash for key {}", i))?;
                    GxtKeyEntry {
                        offset,
                        key: GxtKey::Hash(hash),
                    }
                } else {
                    let mut name = [0u8; 8];
                    reader
                        .read_exact(&mut name)
                        .with_context(|| format!("Failed to read name for key {}", i))?;
                    GxtKeyEntry {
                        offset,
                        key: GxtKey::Text(name),
                    }
                };
                keys.push(key_entry);
            }

            // read TDAT section
            expect_tag(reader, b"TDAT")?;

            // read TDAT section size
            let tdat_size = reader
                .read_u32::<LittleEndian>()
                .context("Failed to read TDAT section size")?;

            // read string data
            let mut table_strings = vec![0u8; tdat_size as usize];
            reader
                .read_exact(&mut table_strings)
                .context("Failed to read string data")?;

            // process each key and extract its string
            for key_entry in &keys {
                let offset = key_entry.offset as usize;
                if offset >= table_strings.len() {
                    continue;
                }

                let text = match self.encoding {
                    TextEncoding::Utf16 => {
                        let mut cursor = Cursor::new(&table_strings[offset..]);
                        read_wide_string(&mut cursor).ok()
                    }
                    TextEncoding::Utf8 => {
                        let end = table_strings[offset..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|pos| offset + pos)
                            .unwrap_or(table_strings.len());

                        String::from_utf8(table_strings[offset..end].to_vec()).ok()
                    }
                };

                if let Some(text) = text {
                    if self.is_hashed {
                        if let GxtKey::Hash(hash) = key_entry.key {
                            self.hash_entries.insert(hash, text);
                        }
                    } else {
                        // store text keys in uppercase for case-insensitive lookup
                        self.entries
                            .insert(key_entry.key.to_string().to_uppercase(), text);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get a text value by its key
    pub fn get(&self, key: &str) -> Option<String> {
        if self.is_hashed {
            // try to parse as a hex hash value (e.g., "15d4d373")
            if let Ok(hash) = u32::from_str_radix(key, 16) {
                if let Some(value) = self.hash_entries.get(&hash) {
                    return Some(value.clone());
                }
            }
            // try as a regular key name (will be hashed)
            let hash = Self::calculate_hash(key);
            if let Some(value) = self.hash_entries.get(&hash) {
                return Some(value.clone());
            }
            None
        } else {
            // text-based lookup (case-insensitive)
            self.entries.get(&key.to_uppercase()).cloned()
        }
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        if self.is_hashed {
            self.hash_entries
                .keys()
                .map(|&hash| format!("{:08X}", hash))
                .collect()
        } else {
            self.entries.keys().cloned().collect()
        }
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        if self.is_hashed {
            self.hash_entries.len()
        } else {
            self.entries.len()
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_ok());
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
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_ok());
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
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_empty_gxt_vc() {
        let mut data = Vec::new();
        // TABL header
        data.extend_from_slice(b"TABL");
        data.extend_from_slice(&0i32.to_le_bytes()); // section size = 0

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_ok());
        assert_eq!(parser.len(), 0);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_invalid_header_vc() {
        let data = b"INVALID_HEADER";
        let mut cursor = Cursor::new(data.to_vec());
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_err());
    }

    #[test]
    fn test_invalid_section_size() {
        let mut data = Vec::new();
        // TKEY header with invalid size
        data.extend_from_slice(b"TKEY");
        data.extend_from_slice(&13i32.to_le_bytes()); // Invalid: not divisible by 12

        let mut cursor = Cursor::new(data);
        let mut parser = GxtParser::new();

        let result = parser.read(&mut cursor);
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
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_ok());
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
    fn test_new_parser_is_empty() {
        // New parser should start empty before loading
        let parser = GxtParser::new();
        assert!(parser.is_empty());
        let parser2 = GxtParser::new();
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
        let mut parser = GxtParser::new();

        assert!(parser.read(&mut cursor).is_ok());

        // Test through the trait interface
        let gxt = &parser;
        assert_eq!(gxt.get("HELLO"), Some("Greet".to_string()));
        assert_eq!(gxt.get("hello"), Some("Greet".to_string()));
        assert_eq!(gxt.get("Hello"), Some("Greet".to_string()));
    }
}
