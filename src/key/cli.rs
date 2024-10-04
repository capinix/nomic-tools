use clap::{Parser, Subcommand};
use crate::functions::resolve_file_home;
use std::path::PathBuf;
use eyre::{eyre, Result, WrapErr};
use std::fs;
use crate::key::{
	key_from_input_or_stdin,
	get_privkey_file,
};


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
	pub command: Option<Command>,
}

/// Subcommands for the `privkey` command
#[derive(Subcommand)]
pub enum Command {
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
		Some(Command::Address { key, file, home }) => {
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
		Some(Command::Export { key, file, home }) => {
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
		Some(Command::Save { key, file, home, force }) => {
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
