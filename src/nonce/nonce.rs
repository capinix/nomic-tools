use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use eyre::{Context, Result};
use crate::functions::get_file;
use fmt::input::read_stdin;
use fmt::input::binary_or_text_file;
use std::time::Duration;
use fmt::input::Data;
// use std::fs;
use crate::functions::construct_path;
use eyre::eyre;
use std::io;
use crate::profiles::ProfileCollection;


/// Retrieves the nonce file path.
///
/// This helper function attempts to construct the path of the nonce file located in the `.orga-wallet`
/// directory. It utilizes the provided optional `file` and `home` parameters to determine the correct path.
///
/// # Parameters
///
/// - `file`: An optional path to a specific nonce file.
/// - `home`: An optional base path; if not provided, the user's home directory will be used.
///
/// # Returns
///
/// - `Ok(PathBuf)` containing the path to the nonce file if successful.
/// - `Err(anyhow::Error)` if there is an issue retrieving the nonce file path.
fn get_nonce_file(file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
    let sub_path = Path::new(".orga-wallet").join("nonce");
    get_file(file, home, Some(&sub_path))
}

/// Retrieves the nonce value from a binary file.
///
/// This function attempts to read the contents of the specified nonce file and interpret it as a `u64` value.
/// If the file does not exist or cannot be read, it will return an error.
///
/// # Parameters
///
/// - `file`: An optional path to a specific nonce file.
/// - `home`: An optional base path; the home directory will be used if not provided.
///
/// # Returns
///
/// - `Ok(u64)` containing the nonce value if successfully retrieved.
/// - `Err(eyre::Error)` if an error occurs while retrieving the nonce file or reading its contents.
pub fn export(file: Option<&Path>, home: Option<&Path>) -> Result<u64> {
	let nonce_file = get_nonce_file(file, home)
		.context("Failed to get nonce file path")?;

	let mut file = File::open(&nonce_file)
		.with_context(|| format!("Failed to open nonce file at {:?}", nonce_file))?;
	
	let mut input = Vec::new();
	file.read_to_end(&mut input)
		.with_context(|| format!("Failed to read from nonce file at {:?}", nonce_file))?;

	if input.len() > 8 {
		return Err(eyre::eyre!("File content too large to fit in u64 (expected 8 bytes, found {}).", input.len())); // Updated error creation
	}

	let mut bytes = [0u8; 8];
	bytes[..input.len()].copy_from_slice(&input);
	let nonce = u64::from_be_bytes(bytes); 

	Ok(nonce)
}

/// Sets the nonce value in a binary file.
///
/// This function converts the provided `u64` value to a byte array in big-endian order and writes it to
/// the specified nonce file. If the file does not exist, it will be created.
///
/// # Parameters
///
/// - `value`: The `u64` value to set as the nonce.
/// - `file`: An optional path to a specific nonce file.
/// - `home`: An optional base path; the home directory will be used if not provided.
/// - `dont_overwrite`: A flag indicating whether to prevent overwriting an existing nonce file.
///
/// # Returns
///
/// - `Ok(())` if the nonce is successfully written to the file.
/// - `Err(eyre::Error)` if an error occurs while retrieving the nonce file path or writing its contents.
pub fn import(value: u64, file: Option<&Path>, home: Option<&Path>, dont_overwrite: bool) -> Result<()> {

    let nonce_file = get_nonce_file(file, home)
        .context("Failed to get nonce file path")?;

    // Check if the nonce file already exists and handle the dont_overwrite flag
    if dont_overwrite && nonce_file.exists() {
        return Err(eyre::eyre!(
			"Nonce file already exists at {:?}. Use --dont-overwrite to prevent overwriting.",
			nonce_file
		));
    }

    // Create or open the nonce file in binary write mode
    let mut file = File::create(&nonce_file)
        .with_context(|| format!("Failed to create nonce file at {:?}", nonce_file))?;

    // Write the new nonce value as bytes
    file.write_all(&value.to_be_bytes())
        .with_context(|| format!("Failed to write to nonce file at {:?}", nonce_file))?;

    Ok(())
}

/// A Nonce struct that holds both the binary (bytes) and decimal representation of the nonce.
#[derive(Debug, Clone, PartialEq)]
pub struct Nonce {
    /// Binary representation of the nonce.
    bytes: Vec<u8>,

    /// Decimal representation of the nonce.
    decimal: u64,
}

impl Nonce {
    /// Constructs a new `Nonce` from a decimal value.
    pub fn from_decimal(decimal: u64) -> Self {
        // Convert decimal to bytes
        let bytes = decimal.to_be_bytes().to_vec();
        Self { bytes, decimal }
    }

    /// Constructs a new `Nonce` from a binary (bytes) value.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, eyre::Error> {
        // Convert bytes to decimal
        let decimal = u64::from_be_bytes(bytes[..].try_into().map_err(|_| eyre::eyre!("Invalid nonce length"))?);
        Ok(Self { bytes, decimal })
    }

    /// Returns the decimal representation of the nonce.
    pub fn decimal(&self) -> u64 {
        self.decimal
    }

    /// Returns the binary representation of the nonce.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Reads nonce data from stdin, attempting to differentiate between binary and text input.
    pub fn from_stdin(max_attempts: usize, timeout_seconds: u64) -> Result<Self> {
        let timeout = Duration::from_secs(timeout_seconds);
        // Assuming `read_stdin` returns a `Result<Data, eyre::Error>`
        match read_stdin(max_attempts, timeout)? {
            Data::Binary(bytes) => Nonce::from_bytes(bytes),
            Data::Text(hex_str) => {
                let decimal = u64::from_str_radix(&hex_str, 16)
                    .map_err(|_| eyre::eyre!("Invalid hex string for nonce"))?;
                Ok(Nonce::from_decimal(decimal))
            },
        }
    }

    pub fn from_input(input: Option<&str>, bytes: Option<Vec<u8>>,)  -> Result<Self, eyre::Error>  {

        // If bytes are provided, ignore everything else
        if let Some(input_bytes) = bytes {
            return Ok(Self::from_bytes(input_bytes)?);
        }

        // Check if input is provided
        if let Some(input_str) = input {

            // Check if the input string can be parsed as a decimal number
            if let Ok(decimal_value) = input_str.parse::<u64>() {
                return Ok(Nonce::from_decimal(decimal_value));
            }
        }

        // If input was not valid, try to get the nonce from the specified file
        let nonce_file_path = construct_path(
            input,
            Some(&Path::new(".orga-wallet").join("nonce")),
        )?;

        match binary_or_text_file(&nonce_file_path)? {
            Data::Binary(bytes) => Nonce::from_bytes(bytes),
            Data::Text(input_str) => {
                // Directly parse the string as a decimal
                let decimal = input_str.parse::<u64>()
                    .map_err(|_| eyre::eyre!("Invalid decimal string for nonce"))?;
                Ok(Nonce::from_decimal(decimal))
            },
        }
    }

    pub fn to_output(&self, output: Option<&str>, dont_overwrite: bool) -> Result<(), eyre::Error> {
        match output {
            Some(output_str) => {
                // Construct the path for the output file
                let nonce_file_path = construct_path(
                    Some(output_str),
                    Some(&Path::new(".orga-wallet").join("nonce")),
                )?;

                // Check if the file exists and dont_overwrite flag is true
                if dont_overwrite && Path::new(&nonce_file_path).exists() {
                    return Err(eyre!("File already exists and overwriting is disabled"));
                }

                // Write the nonce as bytes to the specified file
                std::fs::write(nonce_file_path, self.bytes())?;
                Ok(())
            }
            None => {
                // Write raw bytes to stdout for piping to other commands
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&self.bytes())?;
                Ok(())
            }
        }
    }

    /// Display the nonce as a string.
    pub fn display(&self) -> String {
        format!("Nonce: Decimal = {}, Bytes = {:?}", self.decimal, self.bytes)
    }
}
