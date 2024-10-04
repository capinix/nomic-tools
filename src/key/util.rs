use cosmrs::crypto::{secp256k1::SigningKey};
use crate::functions::get_file;
use fmt::input;
use hex;
use std::path::{Path, PathBuf};
use eyre::{eyre, Result, WrapErr};
use crate::key::Privkey;

/// Validates a hex string as a valid Cosmos blockchain private key.
///
/// The private key must be a 64-character hexadecimal string. This function checks
/// both the length of the string and whether it contains only valid hexadecimal characters.
///
/// # Arguments
///
/// * `hex_str` - A string slice that holds the hexadecimal representation of the private key.
///
/// # Errors
///
/// Returns an error if the string is not 64 characters long or contains non-hex characters.
pub fn validate_cosmos_hex_key(hex_str: &str) -> Result<()> {
	// Step 1: Length Check
	if hex_str.len() != 64 {
		return Err(eyre!("Invalid length: Must be 64 characters."));
	}

	// Step 2: Hexadecimal Validation
	if hex::decode(&hex_str).is_err() {
		return Err(eyre!("Invalid hex string: {}", hex_str));
	}

	Ok(())
}

/// Constructs a `SigningKey` and associated information from a byte vector.
///
/// This function performs several validations on the provided bytes, including checking
/// their length and ensuring that they represent a valid private key. It then creates a
/// hexadecimal representation of the bytes and generates the corresponding public key and
/// address.
///
/// # Arguments
///
/// * `bytes` - A vector of bytes representing the private key. Must be 32 bytes long.
///
/// # Returns
///
/// Returns a tuple containing:
/// - A hexadecimal string representation of the private key.
/// - The original byte vector of the private key.
/// - The generated `SigningKey`.
/// - The corresponding `PublicKey`.
/// - The address derived from the public key.
///
/// # Errors
///
/// Returns an error if the byte vector does not have a length of 32, or if the key validation fails.
pub fn key_from_bytes(bytes: Vec<u8>) -> Result<Privkey> {
	// Step 1: Length Check
	if bytes.len() != 32 {
		return Err(eyre::eyre!("Invalid byte length: Must be 32 bytes."));
	}

	// Step 2: Generate Hexadecimal Representation
	let hex = hex::encode(&bytes);

	// Step 3: Validate the hex representation of the key
	validate_cosmos_hex_key(&hex)?;

	// Step 4: Create SigningKey
	let signing_key = SigningKey::from_slice(&bytes).wrap_err("Failed to create SigningKey from bytes")?;

	// Step 5: Generate PublicKey from SigningKey
	let public_key = signing_key.public_key();

	// Step 6: Generate Address from PublicKey
	let account_id = public_key.account_id("nomic").wrap_err("Failed to get address from public key")?;

	// Return a new Privkey instance
	Ok(Privkey::new(hex, bytes, signing_key, public_key, account_id))
}

/// A trait for converting a byte vector into a signing key and related information.
pub trait FromBytes {
	/// Converts the implementing type into a `SigningKey` and associated information.
	///
	/// # Returns
	///
	/// Returns a `Result` containing:
	/// - A hexadecimal string representation of the private key.
	/// - The original byte vector of the private key.
	/// - The generated `SigningKey`.
	/// - The corresponding `PublicKey`.
	/// - The address derived from the public key.
	fn from_bytes(self) -> Result<Privkey>;
}

impl FromBytes for Vec<u8> {
	fn from_bytes(self) -> Result<Privkey> {
		key_from_bytes(self)
	}
}

/// Constructs a `SigningKey` and associated information from a hexadecimal string.
///
/// This function decodes the hexadecimal string to bytes and validates the resulting bytes
/// before calling `key_from_bytes` to generate the `SigningKey` and associated information.
///
/// # Arguments
///
/// * `hex_str` - A string slice that holds the hexadecimal representation of the private key.
///
/// # Returns
///
/// Returns a tuple containing:
/// - A hexadecimal string representation of the private key.
/// - The byte vector of the private key.
/// - The generated `SigningKey`.
/// - The corresponding `PublicKey`.
/// - The address derived from the public key.
///
/// # Errors
///
/// Returns an error if the hexadecimal string cannot be decoded or if the resulting bytes
/// fail validation.
pub fn key_from_hex(hex_str: &str) -> Result<Privkey> {
	// Step 1: Convert hex to bytes
	let bytes = hex::decode(hex_str).wrap_err("Failed to decode hexadecimal string")?;

	// Step 2: Validate bytes and return results
	key_from_bytes(bytes)
}

/// A trait for converting a hexadecimal string into a signing key and related information.
pub trait FromHex {
	/// Converts the implementing type into a `SigningKey` and associated information.
	///
	/// # Returns
	///
	/// Returns a `Result` containing:
	/// - A hexadecimal string representation of the private key.
	/// - The byte vector of the private key.
	/// - The generated `SigningKey`.
	/// - The corresponding `PublicKey`.
	/// - The address derived from the public key.
	fn from_hex(self) -> Result<Privkey>;
}

impl FromHex for String {
	fn from_hex(self) -> Result<Privkey> {
		key_from_hex(&self)
	}
}

/// Reads input as hex or byte data from a provided option or standard input.
/// Returns a `Privkey` instance based on the input.
///
/// # Parameters
/// - `input`: Optional input that can be either hex or byte data.
///
/// # Errors
/// Returns an error if input processing fails or if key validation fails.
pub fn key_from_input_or_stdin<T: AsRef<[u8]>>(input: Option<T>) -> Result<Privkey> {
	// Retrieve input data, either from provided input or stdin
	let input_data = input::data_or_stdin(input, 5, 500)?;

	// Match the input data and process accordingly
	match input_data {
		input::Data::Text(content) => content.from_hex(),   // Process text as hex input
		input::Data::Binary(bytes) => bytes.from_bytes(),   // Process binary input as bytes
	}
}

/// Reads input as hex or byte data from a provided file or standard input.
/// Returns a `Privkey` instance based on the input.
///
/// # Parameters
/// - `input`: An optional input that can be a file path (as a string slice) or byte data. If provided,
///   the function attempts to read the content from the specified file. If no input is provided,
///   the function reads from standard input.
///
/// # Errors
/// Returns an error if input processing fails or if key validation fails.
/// This may include errors from reading the file, decoding the content, or validation errors.
#[allow(dead_code)]
pub fn key_from_file_or_stdin(input: Option<&Path>) -> Result<Privkey> {
	// Retrieve input data, either from file or stdin
	let input_data = input::file_or_stdin(input, 5, 500)?;

	// Match the input data and process accordingly
	match input_data {
		input::Data::Text(content) => content.from_hex(),   // Process text as hex input
		input::Data::Binary(bytes) => bytes.from_bytes(),   // Process binary input as bytes
	}
}

/// Retrieves the path to the private key file. This checks if the `file` or `home` 
/// paths are provided and, if not, defaults to `.orga-wallet/privkey`.
/// 
/// # Arguments
///
/// * `file` - Optional reference to a file path.
/// * `home` - Optional reference to a home directory path.
///
/// # Returns
///
/// Returns a `PathBuf` with the resolved path to the private key file.
pub fn get_privkey_file(
	file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
	let sub_path = Path::new(".orga-wallet").join("privkey");
	get_file(file, home, Some(&sub_path))
}

