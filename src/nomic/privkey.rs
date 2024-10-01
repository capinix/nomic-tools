use clap::{ Parser, Subcommand };
use hex::decode;
use std::fs::File;
use std::io::{ Read, Write };
use std::path::{ Path, PathBuf };
use fmt::io::unwrap_or_stdin;
use crate::functions::{ get_file, resolve_file_home };
use anyhow::{anyhow, Result, Context};

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
		.chars()
		.collect::<Vec<_>>()
		.chunks(bytes_per_line * 2) // Each byte is represented by 2 hex characters
		.map(|chunk| chunk.iter().collect::<String>())
		.collect::<Vec<String>>()
		.join("\n")
}

/// Reads the private key from a file and returns it as a formatted hexadecimal string.
///
/// # Arguments
///
/// * `file` - Optional reference to a file path.
/// * `home` - Optional reference to a home directory path.
///
/// # Returns
///
/// A `Result` with the formatted hexadecimal string, or an error if the file could not be read.
pub fn get_privkey(file: Option<&Path>, home: Option<&Path>) -> Result<String> {

	// Get the privkey file path
	let privkey_file = get_privkey_file(file, home)
		.context("Failed to get privkey file path")?;
	
	// Read the privkey from the file
	let mut file = File::open(&privkey_file)
		.context("Failed to open privkey file")?;
	
	let mut buffer = Vec::new();
	file.read_to_end(&mut buffer)
		.context("Failed to read privkey file")?;
	
	// Convert the binary data to a hexadecimal string
	let hex_string = hex::encode(buffer);
	
	// Optionally format the output like xxd -ps -c 32
	let formatted_output = format_hex_string(&hex_string, 32);
	
	Ok(formatted_output)
}

/// Sets the private key in the file by writing the provided hexadecimal string.
///
/// # Arguments
///
/// * `hex_str` - Optional hexadecimal string to write to the file.
/// * `file` - Optional reference to a file path.
/// * `home` - Optional reference to a home directory path.
/// * `force` - If true, forces overwriting of an existing file without confirmation.
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn set_privkey(
	hex_str: Option<&str>,
	file: Option<&Path>,
	home: Option<&Path>,
	force: bool,
) -> Result<()> {

	// Use unwrap_or_stdin to read hex string from command line or stdin
	let input_data = unwrap_or_stdin(hex_str, 5, 500)
		.map_err(|e| anyhow::anyhow!("Error reading hex string: {}", e))?;

	// Decode the hex string into bytes
	let decoded_data = decode(&input_data)
		.context("Failed to decode hexadecimal string")?;

	// Get the privkey file path
	let privkey_file = get_privkey_file(file, home)
		.context("Failed to get privkey file path")?;

	// Check if the file already exists
	if privkey_file.exists() && !force {
		// Prompt for confirmation if not forced
		println!(
			"File '{}' already exists. Do you want to overwrite it? [y/N]: ",
			privkey_file.display()
		);
		
		let mut response = String::new();
		std::io::stdin().read_line(&mut response)?;

		if !response.trim().eq_ignore_ascii_case("y") {
			println!("Aborted. File was not overwritten.");
			return Ok(());
		}
	}

	// Write the decoded data to the specified file
	let mut file = File::create(&privkey_file)
		.context("Failed to create or open the privkey file")?;
	
	file.write_all(&decoded_data)
		.context("Failed to write to the privkey file")?;

	println!("Private key set successfully.");
	Ok(())
}

/// Custom parser that validates whether a given string is a valid hexadecimal string.
///
/// # Arguments
///
/// * `s` - A string to validate as hexadecimal.
///
/// # Returns
///
/// Returns a `Result` where `Ok` contains the validated hexadecimal string, and `Err` contains an error message.
fn is_hex_string(s: &str) -> Result<String, String> {
	if s.chars().all(|c| c.is_ascii_hexdigit()) {
		Ok(s.to_string())
	} else {
		Err(format!("'{}' is not a valid hex string", s))
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
	/// Show contents of PrivKey file
	Get {
		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Set contents of PrivKey file
	Set {
		/// Hex string
		#[arg(long, short = 'i', value_parser = is_hex_string)]
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
		Some(CliCommand::Get { file, home }) => {
			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			
			// Get and print the private key.
			let privkey = get_privkey(resolved_file.as_deref(), resolved_home.as_deref())
				.context("Failed to get private key")?;

			println!("Current private key:\n{}", privkey);
			Ok(())
		},
		Some(CliCommand::Set { hex_string, file, home, force }) => {
			// Resolve the final file and home options, with subcommand taking precedence.
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;

			// Set the private key.
			set_privkey(Some(hex_string), resolved_file.as_deref(), resolved_home.as_deref(), *force)
				.context("Failed to set private key")?;

			Ok(())
		},
		None => {
			eprintln!("No command provided.");
			return Err(anyhow!("No command provided."));
		}
	}
}
