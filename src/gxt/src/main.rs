use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};
use gxt_parser::{GxtConfig, GxtParser, TextEncoding};
use std::io::{self, Write};
use std::path::Path;

macro_rules! write_stdout {
    ($($arg:tt)*) => {
        if let Err(e) = writeln!(io::stdout(), $($arg)*) {
            eprintln!("Error writing to stdout: {}", e);
            std::process::exit(1);
        }
    };
}

/// Text encoding types
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Encoding {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 LE encoding
    Utf16,
}

/// GXT file parser and viewer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    gxt american.gxt                        # List all entries (auto-detect format)
    gxt american.gxt --key INTRO            # Lookup the INTRO key
    gxt american.gxt -k INTRO               # Same as above (short form)
    gxt mobile.gxt --encoding utf16         # Parse SA mobile file with UTF-16")]
struct Args {
    /// Path to the GXT file to parse
    gxt_file: String,

    /// Specify the text encoding (defaults: UTF-16 for III/VC, UTF-8 for SA)
    #[arg(short, long, value_enum)]
    encoding: Option<Encoding>,

    /// Lookup a specific key instead of listing all entries
    #[arg(short, long)]
    key: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let file_path = &args.gxt_file;

    if !Path::new(file_path).exists() {
        bail!("File '{}' does not exist", file_path);
    }

    let encoding = args.encoding.unwrap_or_else(|| Encoding::Utf16);

    // Load the GXT file based on format and encoding
    let config = match encoding {
        Encoding::Utf16 => GxtConfig {
            encoding: TextEncoding::Utf16,
            ..Default::default()
        },
        Encoding::Utf8 => GxtConfig {
            encoding: TextEncoding::Utf8,
            ..Default::default()
        },
    };
    let mut parser = GxtParser::new(config);
    parser.load(file_path)?;

    // Handle key lookup
    if let Some(key) = args.key {
        // Lookup specific key - only print the value
        if let Some(value) = parser.get(&key) {
            write_stdout!("{}", value);
        } else {
            eprintln!("Error: Key '{}' not found in GXT file", key);
            std::process::exit(1);
        }
    } else {
        // Display all entries
        let encoding_str = match encoding {
            Encoding::Utf8 => "UTF-8",
            Encoding::Utf16 => "UTF-16",
        };
        write_stdout!(
            "Successfully loaded GXT file: {} (Encoding: {})",
            file_path,
            encoding_str
        );
        write_stdout!("Total entries: {}", parser.len());
        write_stdout!();

        let keys = parser.keys();

        for key in &keys {
            if let Some(value) = parser.get(key) {
                write_stdout!("{:<10} => {}", key, value);
            }
        }
    }

    Ok(())
}
