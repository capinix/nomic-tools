use clap::{ Parser, Subcommand };
use hex::decode;
use std::error::Error;
use std::fs::File;
use std::io::{ Read, Write };
use std::path::{ Path, PathBuf };
use fmt::io::str_or_stdin;

pub fn set_key(file: &Path, hex_str: Option<&str>) -> Result<(), Box<dyn StdError>> {

	// Use str_or_stdin to read hex string input from the user if not provided
	let input_data = str_or_stdin(hex_str, 5, 500);

	// Validate and decode the hex string into binary data
	let binary_data = match decode(&input_data) {
		Ok(data) => data,
		Err(e) => {
			eprintln!("Invalid hex string: {}", e);
			return Err(Box::new(e));
		}
	};

	// Open the file for writing (create or overwrite)
	let mut file = File::create(file)?;

	// Write the binary data to the file
	file.write_all(&binary_data)?;

	// Flush the output to ensure all data is written
	file.flush()?;

	Ok(())
}

pub fn get_key(file: &Path) -> Result<String, Box<dyn Error>> {

    // Open the file for reading
    let mut file = File::open(file)?;
    
    // Create a buffer to hold the binary data
    let mut buffer = Vec::new();

    // Read the entire file into the buffer
    file.read_to_end(&mut buffer)?;

    // Convert the binary data to a hexadecimal string
    let hex_value = hex::encode(&buffer);

    // Return the hexadecimal string
    Ok(hex_value)
}

#[derive(Parser)]
#[command(name = "keys", about = "Manage keys")]
pub struct Cli {
    /// Subcommands for the keys command
    #[command(subcommand)]
    pub command: KeysCommand,
}

/// Subcommands for the `keys` command
#[derive(Subcommand)]
pub enum KeysCommand {

    /// Retrieve the private key as hex from a binary file
    Get {
        /// Path to the private key file
        #[arg(required)]
        privkey_file: PathBuf,
    },

    /// Set a hex string as a private key in a binary file
    Set {
        /// Path to save the private key file
        #[arg(required)]
        privkey_file: PathBuf,

        /// Hexadecimal private key string
        #[arg(value_parser = is_hex_string)]
        hex_string: Option<String>,
    },
}

/// Custom parser for validating a hex string
fn is_hex_string(s: &str) -> Result<String, String> {
    if s.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(s.to_string())
    } else {
        Err(format!("'{}' is not a valid hex string", s))
    }
}
