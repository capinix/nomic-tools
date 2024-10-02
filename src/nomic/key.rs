use clap::{Parser, Subcommand};
use cosmrs::crypto::{secp256k1::SigningKey, PublicKey};
use crate::functions::{get_file, is_hex, resolve_file_home};
use fmt::io::{process_input, InputData};
use hex::{encode, decode};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use eyre::{eyre, Result, WrapErr};

/// Formats a hexadecimal string by inserting line breaks after a specified number
/// of bytes, mimicking the output of `xxd -ps -c <bytes_per_line>`.
/// 
/// # Arguments
///
/// * `hex_string` - The hexadecimal string to be formatted.
/// * `bytes_per_line` - The number of bytes to display per line (each byte is 2 hex characters).
///
/// # Returns
///
/// A formatted string with the specified number of bytes per line.
fn format_hex_string(hex_string: &str, bytes_per_line: usize) -> String {
	hex_string
		.as_bytes()
		.chunks(bytes_per_line * 2) // Each byte is represented by 2 hex characters
		.map(|chunk| String::from_utf8_lossy(chunk).to_string())
		.collect::<Vec<_>>()
		.join("\n")
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

pub struct Key {
	bytes:	Vec<u8>,
	hex:	  String,
	#[allow(dead_code)]
	signing:  SigningKey,
	#[allow(dead_code)]
	public:   PublicKey,
	address:  String,
}

impl Key {

	/// Constructs a new `Key` from a privkey file or input string.
	pub fn new(input: Option<&str>) -> Result<Self> {

		let bytes: Vec<u8>;
		let mut hex: String;

		// Attempt to process the input (text only), fallback to stdin if necessary
		match process_input(input, 5, 500) {
			Ok(InputData::Text(content)) => {
				// Decode the hex string into bytes
				hex = content; // Store the hex string directly
				bytes = decode(&hex).wrap_err("Failed to decode hexadecimal string")?;
			},
			Ok(InputData::Binary(content)) => {
				// Treat input_data as already being bytes
				bytes = content; // Use the input directly
				hex = encode(&bytes); // Convert bytes to a hexadecimal string
				// Optionally format the output like `xxd -ps -c 32`
				hex = format_hex_string(&hex, 32); // Ensure format_hex_string is defined
			},
			Err(e) => {
				return Err(eyre!("Error processing input: {}", e));  // Propagate error for other cases
			}
		};

		// Create SigningKey from bytes
		let signing = SigningKey::from_slice(&bytes)
			.wrap_err("Failed to create SigningKey from bytes")?;

		// Derive the public key from the signing key
		let public = signing.public_key();

		// Derive the address from the public key
		let address = public.account_id("nomic")
			.wrap_err("Failed to get address from public key")?;

		// Return the constructed Key
		Ok(Key {
			bytes,
			hex,
			signing,
			public,
			address: address.to_string(),
		})
	}

	/// Constructs a new `Key` from a privkey file, toggling between home dir and user-provided home.
	pub fn new_from_file(file: Option<&Path>, home: Option<&Path>) -> Result<Self> {
		// Get the privkey file path
		let privkey_file = get_privkey_file(file, home)
			.context("Failed to get privkey file path")?;

		// Directly use the new function, which handles all input types
		Key::new(Some(privkey_file.to_str().unwrap())) // Remove the wrapping Ok

	}

	/// Save the private key to a file or print the hex representation if no file is provided.
	pub fn save(&self, file: Option<&Path>, home: Option<&Path>, force: bool) -> Result<()> {

		// Check if neither file nor home is provided, print hex
		if file.is_none() && home.is_none() {
			println!("{}", self.hex);
			return Ok(());
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
		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Export Private Key
	Export {
		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Import Private Key
	Import {
		/// Hex string
		#[arg(long, short = 'i', value_parser = is_hex)]
		hex_string: String,

		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,

		/// Force overwrite without confirmation
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
		Some(CliCommand::Address { file, home }) => {
			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			
			let key = Key::new_from_file(resolved_file.as_deref(), resolved_home.as_deref())?;

			println!("Address:\n{}", key.address);
			Ok(())
		},
		Some(CliCommand::Export { file, home }) => {
			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			
			let key = Key::new_from_file(resolved_file.as_deref(), resolved_home.as_deref())?;

			println!("Current private key:\n{}", key.hex);
			Ok(())
		},
		Some(CliCommand::Import { hex_string, file, home, force }) => {
			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			
			let key = Key::new(Some(hex_string))?;
			key.save(resolved_file.as_deref(), resolved_home.as_deref(), *force)
		},
		None => {
			return Err(eyre::eyre!("No command provided."));
		}
	}
}
