use clap::{Parser, Subcommand};
use cosmrs::crypto::{secp256k1::SigningKey, PublicKey};
use cosmrs::AccountId;
use crate::functions::{get_file, resolve_file_home};
use fmt::input;
use hex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use eyre::{eyre, Result, WrapErr};
use std::fs;

/// Represents a private key (in byte or hex format) and provides access to the associated 
/// `AccountID`, `SigningKey`, and `PublicKey` using the `cosmrs` crate.
/// 
/// This struct handles private keys that are stored as either a byte array or a hexadecimal string. 
/// The key file often contains a raw byte array, which may include non-printable characters 
/// when read as text. By reading the file as a `Vec<u8>` and performing hex decoding, 
/// the correct hexadecimal representation of the private key is derived.
/// 
/// Upon receiving either a byte array or a hex string, the struct automatically determines 
/// and stores the other representation. With the private key in hand (whether as bytes or hex), 
/// we utilize `cosmrs` to derive the corresponding `SigningKey`, `PublicKey`, and `AccountID` 
/// (which represents the address associated with the key).
///
/// # Fields
/// 
/// - `bytes`: The raw byte stream of the private key, which is hex-encoded binary data.
/// - `hex`: The private key in its hexadecimal string representation.
/// - `account_id`: The `AccountId` or derived from `cosmrs::AccountId`.
/// - `signing_key`: The `SigningKey` derived from `cosmrs::crypto::secp256k1`. This field 
///	may not be directly used in this struct, but is available for functions that require it.
/// - `public_key`: The `PublicKey` derived from `cosmrs::crypto::secp256k1`.
pub struct Privkey {
	/// Raw byte stream of the private key, which is hex-encoded binary data.
	bytes: Vec<u8>,

	/// The private key in hexadecimal format.
	hex: String,

	/// The signing key derived from `cosmrs::crypto::secp256k1`. 
	/// This field is not used within this struct, but it may be necessary for other functions.
	#[allow(dead_code)]
	signing_key: SigningKey,

	/// The public key derived from `cosmrs::crypto::secp256k1`.
	/// This field is not used within this struct, but it may be necessary for other functions.
	#[allow(dead_code)]
	public_key: PublicKey,

	/// The `AccountID` or address associated with this private key.
	account_id: AccountId,
}

impl std::fmt::Debug for Privkey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//		write!(f, "Privkey {{ address: {} }}", self.get_address())
		write!(f, "Privkey {{ address: {}\nhex: {}\nbytes: {:?} }}", self.get_address(), self.hex, self.bytes)
	}
}

impl PartialEq for Privkey {
	fn eq(&self, other: &Self) -> bool {
		self.hex.eq_ignore_ascii_case(&other.hex)
	}
}

impl Eq for Privkey {}

impl Privkey {
	/// Constructs a new `Privkey` from the provided fields.
	pub fn new(
		hex: String,
		bytes: Vec<u8>,
		signing_key: SigningKey,
		public_key: PublicKey,
		account_id: AccountId,
) -> Self {
		Self {
			hex,
			bytes,
			signing_key,
			public_key,
			account_id,
		}
	}
}

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
fn validate_cosmos_hex_key(hex_str: &str) -> Result<()> {
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
pub fn key_from_file_or_stdin<T: AsRef<[u8]>>(input: Option<T>) -> Result<Privkey> {
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
fn get_privkey_file(
	file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
	let sub_path = Path::new(".orga-wallet").join("privkey");
	get_file(file, home, Some(&sub_path))
}

impl Privkey {

	pub fn get_address(&self) -> String {
		self.account_id.to_string()
	}

	pub fn get_hex(&self) -> String {
		self.hex.clone()
	}

	/// Saves the `bytes` (`Vec<u8>`) back to a private key file.
	///
	/// A valid file path or home directory must be provided for the file to be written.
	/// If the file already exists, the function will return an error unless the `force` parameter is set to `true`,
	/// in which case it will overwrite the existing file.
	///
	/// # Arguments
	///
	/// - `file`: An optional reference to a `Path` that specifies the location to save the private key.
	/// - `home`: An optional reference to a home directory path.
	/// - `force`: A boolean flag indicating whether to overwrite the file if it already exists.
	///
	/// # Returns
	///
	/// - `Ok(())` if the private key was successfully written to the file.
	/// - `Err(Report)` if there was an error during the save operation, such as if the file already exists and `force` is `false`.
	///
	/// # Example
	///
	/// ```
	/// let privkey = Privkey::new(Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"))?;
	/// let file_path = Path::new("/path/to/save/privkey_file");
	/// 
	/// // Attempt to save the private key, overwriting if necessary
	/// privkey.save(Some(file_path), None, true)?;
	/// ```
	pub fn save(&self, file: Option<&Path>, home: Option<&Path>, force: bool) -> Result<()> {
		// Check if neither file nor home is provided
		if file.is_none() && home.is_none() {
			return Err(eyre::eyre!("Must specify either a file or a home directory"));
		}

		// Check if both file and home are provided
		if file.is_some() && home.is_some() {
			return Err(eyre::eyre!("Cannot specify both a file and a home directory"));
		}

		// Get the privkey file path
		let output_file = get_privkey_file(file, home)
			.context("Failed to get privkey file path")?;

		// Check if the file exists
		if output_file.exists() {
			// File exists, check if overwrite is allowed
			if force {
				println!("Overwriting existing file: {:?}", output_file);
			} else {
				return Err(eyre::eyre!("File already exists. Use --force to overwrite."));
			}
		} else {
			// File does not exist, proceed to create it
			println!("Creating new file: {:?}", output_file);
		}

		// Write the decoded data to the specified file
		let mut file = File::create(&output_file)
			.context("Failed to create or open the privkey file")?;

		file.write_all(&self.bytes)
			.context("Failed to write to the privkey file")?;

		println!("Private key set successfully.");
		Ok(())
	}
}

/// Defines the CLI structure for the `privkey` command.
#[derive(Parser)]
#[command(name = "PrivKey", about = "Manage PrivKey File")]
pub struct Cli {
	/// Filename
	#[arg(long, short, conflicts_with = "home")]
	pub file: Option<PathBuf>,

	/// Home directory
	#[arg(long, short = 'H')]
	pub home: Option<PathBuf>,

	/// Subcommands for the nonce command
	#[command(subcommand)]
	pub command: Option<CliCommand>,
}

/// Subcommands for the `privkey` command
#[derive(Subcommand)]
pub enum CliCommand {
	/// Show the public address (AccountID)
	Address {
		/// Key
		#[arg(long, short, conflicts_with = "file", conflicts_with = "home")]
		key: Option<String>,

		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Export Private key caution
	Export {
		/// Key
		#[arg(long, short, conflicts_with = "file", conflicts_with = "home")]
		key: Option<String>,

		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Save Private key to file
	Save {
		/// Key
		#[arg(long, short, conflicts_with = "file", conflicts_with = "home")]
		key: Option<String>,

		/// Filename
		#[arg(long, short = 'F', conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,

		/// Force overwrite
		#[arg(long, short = 'f')]
		force: bool,
	},
}

/// Runs the CLI for managing the private key.
///
/// # Arguments
///
/// * `cli` - A reference to the parsed CLI arguments.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn run_cli(cli: &Cli) -> Result<()> {

	match &cli.command {
		Some(CliCommand::Address { key, file, home }) => {
			// Check if both file and home are provided
			if file.is_some() && home.is_some() {
				return Err(eyre!("Error: You cannot provide both 'file' and 'home' options at the same time."));
			}

			// Check if the key is provided, or if all of key, file, and home are None
			if key.is_some() {
				// Process the provided key to obtain the private key
				let privkey = key_from_input_or_stdin(key.as_ref().map(|s| s.as_bytes().to_vec()))
					.context("Failed to derive private key from input")?;

				// Use the private key for further processing...
				println!("{}", privkey.get_address()); // Print the derived address

				return Ok(());
			}

			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;

			// Get the private key file, which returns a Result<PathBuf, Report>
			let file_result = get_privkey_file(resolved_file.as_deref(), resolved_home.as_deref())
				.context("Failed to get the private key file")?;

			// Read the contents of the file into a byte vector
			let file_contents = fs::read(&file_result)
				.context("Failed to read the private key file")?;

			// Handle the result and pass the file contents to key_from_file_or_stdin
			let privkey = key_from_input_or_stdin(Some(file_contents))
				.context("Failed to derive private key from file")?;

			// Use the private key for further processing...
			println!("{}", privkey.get_address()); // Print the derived address
			Ok(())
		},
		Some(CliCommand::Export { key, file, home }) => {
			// Check if both file and home are provided
			if file.is_some() && home.is_some() {
				return Err(eyre!("Error: You cannot provide both 'file' and 'home' options at the same time."));
			}

			// Check if the key is provided, or if all of key, file, and home are None
			if key.is_some() {
				// Process the provided key to obtain the private key
				let privkey = key_from_input_or_stdin(key.as_ref().map(|s| s.as_bytes().to_vec()))
					.context("Failed to derive private key from input")?;

				// Use the private key for further processing...
				println!("{}", privkey.get_hex()); // Print the derived address

				return Ok(());
			}

			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;

			// Get the private key file, which returns a Result<PathBuf, Report>
			let file_result = get_privkey_file(resolved_file.as_deref(), resolved_home.as_deref())
				.context("Failed to get the private key file")?;

			// Read the contents of the file into a byte vector
			let file_contents = fs::read(&file_result)
				.context("Failed to read the private key file")?;

			// Handle the result and pass the file contents to key_from_file_or_stdin
			let privkey = key_from_input_or_stdin(Some(file_contents))
				.context("Failed to derive private key from file")?;

			// Use the private key for further processing...
			println!("{}", privkey.get_hex()); // Print the derived address
			Ok(())
		},
		Some(CliCommand::Save { key, file, home, force }) => {
			// Check if both file and home are provided
			if file.is_some() && home.is_some() {
				return Err(eyre!("Error: You cannot provide both 'file' and 'home' options at the same time."));
			}

			// Derive the private key from the provided key input
			let privkey = key_from_input_or_stdin(key.as_ref().map(|s| s.as_bytes().to_vec()))
				.context("Failed to derive private key from input")?;

			// Save the private key to the resolved file or home directory
			privkey.save(file.as_deref(), home.as_deref(), *force)
				.context("Failed to save the private key")?;

			Ok(())
		},
		None => {
			return Err(eyre::eyre!("No command provided."));
		}
	}
}
