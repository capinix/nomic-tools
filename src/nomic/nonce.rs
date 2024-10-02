use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use eyre::{Context, Result};
use crate::functions::{get_file, resolve_file_home};

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


/// CLI structure for the `nonce` command.
///
/// This struct defines the command-line interface for managing the nonce file,
/// allowing users to specify the nonce file and optional home directory.
#[derive(Parser)]
#[command(name = "Nonce", about = "Manage Nonce File")]
pub struct Cli {
	#[arg(long, short, conflicts_with = "home")]
	pub file: Option<PathBuf>,

	#[arg(long, short = 'H')]
	pub home: Option<PathBuf>,

	#[command(subcommand)]
	pub command: Option<CliCommand>,
}

/// Subcommands for the `nonce` command
#[derive(Subcommand)]
pub enum CliCommand {
	/// Export nonce from file
	Export {
		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		#[arg(long, short = 'H')]
		home: Option<PathBuf>,
	},
	/// Write nonce to file
	Import {
		#[arg(long, short)]
		value: u64,

		#[arg(long, short, conflicts_with = "home")]
		file: Option<PathBuf>,

		#[arg(long, short = 'H')]
		home: Option<PathBuf>,

		#[arg(long = "dont-overwrite", short = 'D')]
		dont_overwrite: bool,
	},
}

/// Runs the CLI commands based on user input.
///
/// This function checks for mutual exclusivity between the `file` and `home` options,
/// executes the specified subcommand, and handles any errors that arise during the process.
///
/// # Parameters
///
/// - `cli`: The CLI structure containing the command and arguments provided by the user.
///
/// # Returns
///
/// - `Ok(())` if the command executes successfully.
/// - `Err(anyhow::Error)` if an error occurs during command execution.
pub fn run_cli(cli: &Cli) -> Result<()> {
	match &cli.command {
		Some(CliCommand::Export { file, home }) => {
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			let nonce = export(resolved_file.as_deref(), resolved_home.as_deref())
				.context("Failed to retrieve nonce")?;
			println!("Current nonce: {}", nonce);
			Ok(())
		},
		Some(CliCommand::Import { value, file, home, dont_overwrite }) => {
			let (resolved_file, resolved_home) = resolve_file_home(
				file.clone(), home.clone(), cli.file.clone(), cli.home.clone()
			)?;
			import(*value, resolved_file.as_deref(), resolved_home.as_deref(), *dont_overwrite)
				.context("Failed to set nonce")?;
			println!("Nonce set to: {}", value);
			Ok(())
		},
		None => {
			Err(eyre::eyre!("No command provided."))

		}
	}
}
