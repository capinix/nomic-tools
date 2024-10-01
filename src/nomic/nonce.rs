use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use dirs::home_dir;

fn get_nonce_file(file: Option<&Path>, home: Option<&Path>) -> Result<PathBuf, &'static str> {
	match file {
		Some(path) => Ok(path.to_path_buf()),
		None => {
			let base_path = home
				.map(PathBuf::from) // Convert Option<&Path> to Option<PathBuf>
				.or_else(home_dir)  // Use home_dir() directly since it returns a PathBuf
				.ok_or("Could not determine home directory")?;

			Ok(base_path.join(".orga-wallet").join("nonce"))
		}
	}
}

pub fn get_nonce(file: Option<&Path>, home: Option<&Path>) -> Result<u64, Box<dyn Error>> {
	let nonce_file = get_nonce_file(file, home).map_err(|e| Box::<dyn Error>::from(e))?;

	// Attempt to open the file
	let mut file = match File::open(&nonce_file) {
		Ok(f) => f,
		Err(e) if e.kind() == io::ErrorKind::NotFound => {
			eprintln!("File '{}' does not exist. Creating a new file.", nonce_file.display());
			let mut new_file = File::create(&nonce_file)?;
			new_file.write_all(&(0u64).to_be_bytes())?;
			return Ok(0); // Return 0 since the file didn't exist
		}
		Err(e) => {
			eprintln!("Error opening file '{}': {}", nonce_file.display(), e);
			return Err(Box::new(e));
		}
	};

	let mut input = Vec::new();
	file.read_to_end(&mut input)?;

	// Check if the input size is within the u64 limit (8 bytes)
	if input.len() <= 8 {
		let mut bytes = [0u8; 8];
		bytes[..input.len()].copy_from_slice(&input);
		return Ok(u64::from_be_bytes(bytes)); // Convert to u64
	}

	// If the input is too large, return an error
	Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "File content too large to fit in u64.")))
}

pub fn set_nonce(value: u64, file: Option<&Path>, home: Option<&Path>) -> Result<(), Box<dyn Error>> {
	let nonce_file = get_nonce_file(file, home).map_err(|e| Box::<dyn Error>::from(e))?;
	let mut file = File::create(&nonce_file)?;
	file.write_all(&value.to_be_bytes())?;
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

			let nonce = get_nonce(file.as_deref(), home.as_deref())?;
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

			set_nonce(*value, file.as_deref(), home.as_deref())?;
			println!("Nonce set to: {}", value);
			Ok(())
		},
		None => {
			eprintln!("No command provided.");
			Err("No command provided.".into())
		}
	}
}
