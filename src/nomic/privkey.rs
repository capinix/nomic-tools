use clap::{ Parser, Subcommand };
use hex::decode;
use std::fs::File;
use std::io::{ Read, Write };
use std::path::{ Path, PathBuf };
use fmt::io::unwrap_or_stdin;
use crate::functions::get_file;
use anyhow::{anyhow, Result, Context};

fn get_privkey_file(
	file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
	let sub_path = Path::new(".orga-wallet").join("privkey");
	get_file(file, home, Some(&sub_path))
}

fn format_hex_string(hex_string: &str, bytes_per_line: usize) -> String {
	hex_string
		.chars()
		.collect::<Vec<_>>()
		.chunks(bytes_per_line * 2) // Each byte is represented by 2 hex characters
		.map(|chunk| chunk.iter().collect::<String>())
		.collect::<Vec<String>>()
		.join("\n")
}

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

pub fn set_privkey(
	hex_str: Option<&str>,
	file: Option<&Path>,
	home: Option<&Path>,
) -> Result<()> {
	// Use unwrap_or_stdin to read hex string from command line or stdin
	let input_data = unwrap_or_stdin(hex_str, 5, 500)
		.map_err(|e| anyhow::anyhow!("Error reading hex string: {}", e))?; // Use anyhow directly

	// Decode the hex string into bytes
	let decoded_data = decode(&input_data)  // Ensure input_data is correctly passed
		.context("Failed to decode hexadecimal string")?;

	// Get the privkey file path
	let privkey_file = get_privkey_file(file, home)
		.context("Failed to get privkey file path")?;

	// Write the decoded data to the specified file
	let mut file = File::create(&privkey_file)
		.context("Failed to create or open the privkey file")?;
	
	file.write_all(&decoded_data)
		.context("Failed to write to the privkey file")?;

	Ok(())
}

/// Custom parser for validating a hex string
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
		#[arg(long, short, value_parser = is_hex_string)]
		hex_string: String,

		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// Home directory
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
}

pub fn run_cli(cli: &Cli) -> Result<()> {
	// Check for mutual exclusivity of home and file
	if cli.file.is_some() && cli.home.is_some() {
		eprintln!("Error: You cannot provide both --file and --home at the same time.");
		return Err(anyhow!("Mutually exclusive options provided.")); // Use anyhow for the error
	}

	match &cli.command {
		Some(CliCommand::Get { file, home }) => {
			let file = file.clone().or(cli.file.clone());
			let home = home.clone().or(cli.home.clone());

			if file.is_some() && home.is_some() {
				eprintln!("Error: You cannot provide both --file and --home at the same time.");
				return Err(anyhow!("Mutually exclusive options provided.")); // Use anyhow for the error
			}

			let privkey = get_privkey(file.as_deref(), home.as_deref())
				.context("Failed to get private key")?; // Use context for better error handling

			println!("Current private key:\n{}", privkey);
			Ok(())
		},
		Some(CliCommand::Set { hex_string, file, home }) => {
			let file = file.clone().or(cli.file.clone());
			let home = home.clone().or(cli.home.clone());

			if file.is_some() && home.is_some() {
				eprintln!("Error: You cannot provide both --file and --home at the same time.");
				return Err(anyhow!("Mutually exclusive options provided.")); // Use anyhow for the error
			}

			set_privkey(Some(hex_string), file.as_deref(), home.as_deref())
				.context("Failed to set private key")?; // Use context for better error handling

			println!("Private key set successfully.");
			Ok(())
		},
		None => {
			eprintln!("No command provided.");
			return Err(anyhow!("No command provided.")); // Use anyhow for the error
		}
	}
}
