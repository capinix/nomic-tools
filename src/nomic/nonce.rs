use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use anyhow::{Result, Context};
use crate::functions::get_file;

fn get_nonce_file(file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
    let sub_path = Path::new(".orga-wallet").join("nonce");
    get_file(file, home, Some(&sub_path))
}

/// Retrieves the nonce value from a binary file.
///
/// This function attempts to get the nonce file's path and read its contents as a `u64`.
/// If the file does not exist, it will return an error.
///
/// # Parameters
///
/// * `file`: An optional path to a specific nonce file.
/// * `home`: An optional base path (home directory will be used if not provided).
///
/// # Returns
///
/// * `Ok(u64)` if the nonce is successfully retrieved from the file.
/// * `Err(anyhow::Error)` if an error occurs while retrieving the nonce file or reading its contents.
pub fn get_nonce(
	file: Option<&Path>,
	home: Option<&Path>
) -> Result<u64> {

	let nonce_file = get_nonce_file(file, home)
		.context("Failed to get nonce file path")?;

	let mut file = File::open(&nonce_file)
		.context("Failed to open nonce file")?;
	
	// Read the binary content into a buffer
	let mut input = Vec::new();
	file.read_to_end(&mut input)
		.context("Failed to read from nonce file")?;

	// Check if the input size is within the u64 limit (8 bytes)
	if input.len() > 8 {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "File content too large to fit in u64.").into());
	}

	// Convert the bytes to a u64 value
	let mut bytes = [0u8; 8];
	bytes[..input.len()].copy_from_slice(&input);
	let nonce = u64::from_be_bytes(bytes); // Interpret the bytes as a big-endian u64

	Ok(nonce) // Return the decimal value
}

pub fn set_nonce(
	value: u64, 
	file: Option<&Path>,
	home: Option<&Path>
) -> Result<()> {

	let nonce_file = get_nonce_file(file, home)
		.context("Failed to get nonce file path")?;

	// Create or open the nonce file in binary write mode
	let mut file = File::create(&nonce_file)
		.context("Failed to create nonce file")?;

	// Convert the value to a byte array in big-endian order
	let bytes = value.to_be_bytes();

	// Write the byte array to the file
	file.write_all(&bytes)
		.context("Failed to write to nonce file")?;

	Ok(())
}

/// Defines the CLI structure for the `nonce` command.
#[derive(Parser)]
#[command(name = "Nonce", about = "Manage Nonce File")]
pub struct Cli {
	/// Filename
	#[arg(long, short, conflicts_with = "home")]
	pub file: Option<PathBuf>,

	/// home
	#[arg(long, short = 'H')]
	pub home: Option<PathBuf>,

	/// Subcommands for the nonce command
	#[command(subcommand)]
	pub command: Option<CliCommand>,
}

/// Subcommands for the `nonce` command
#[derive(Subcommand)]
pub enum CliCommand {
	/// Show contents of Nonce file
	Get {
		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// home
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Set contents of Nonce file
	Set {
		/// Decimal value
		#[arg(long, short)]
		value: u64,

		/// Filename
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		/// home
		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
}

pub fn run_cli(cli: &Cli) -> Result<(), Box<dyn Error>> {

	// Check for mutual exclusivity of home and file
	if cli.file.is_some() && cli.home.is_some() {
		eprintln!("Error: You cannot provide both --file and --home at the same time.");
		return Err("Mutually exclusive options provided.".into());
	}

	match &cli.command {

		Some(CliCommand::Get { file, home }) => {
			let file = file.clone().or(cli.file.clone());
			let home = home.clone().or(cli.home.clone());

			// Check for mutual exclusivity of home and file
			if file.is_some() && home.is_some() {
				eprintln!("Error: You cannot provide both --file and --home at the same time.");
				return Err("Mutually exclusive options provided.".into());
			}

            // Call the get_nonce function with the file and home options
            let nonce = get_nonce(file.as_deref(), home.as_deref())
                .context("Failed to get nonce")?;

			println!("Current nonce: {}", nonce);
			Ok(())
		},
		Some(CliCommand::Set { value, file, home }) => {
			let file = file.clone().or(cli.file.clone());
			let home = home.clone().or(cli.home.clone());

			// Check for mutual exclusivity of home and file
			if file.is_some() && home.is_some() {
				eprintln!("Error: You cannot provide both --file and --home at the same time.");
				return Err("Mutually exclusive options provided.".into());
			}

            // Call the set_nonce function with the value, file and home options
            set_nonce(*value, file.as_deref(), home.as_deref())
                .context("Failed to get nonce")?;

			println!("Nonce set to: {}", value);
			Ok(())
		},
		None => {
			eprintln!("No command provided.");
			Err("No command provided.".into())
		}
	}
}
