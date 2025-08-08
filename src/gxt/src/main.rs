use anyhow::{bail, Result};
use clap::Parser;
use gxt_parser::GxtParser;
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

/// GXT file parser and viewer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    gxt american.gxt                        # List all entries (auto-detect format)
    gxt american.gxt --key INTRO            # Lookup the INTRO key
    gxt american.gxt -k INTRO               # Same as above (short form)")]
struct Args {
    /// Path to the GXT file to parse
    gxt_file: String,

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

    // load the GXT file (format and encoding will be auto-detected)
    let mut parser = GxtParser::new();
    parser.load_file(file_path)?;

    // handle key lookup
    if let Some(key) = args.key {
        // lookup specific key - only print the value
        if let Some(value) = parser.get(&key) {
            write_stdout!("{}", value);
        } else {
            eprintln!("Error: Key '{}' not found in GXT file", key);
            std::process::exit(1);
        }
    } else {
        // display all entries
        write_stdout!("Successfully loaded GXT file: {}", file_path);
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
