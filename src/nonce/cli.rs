use clap::{Parser, Subcommand};
use std::path::PathBuf;
use eyre::{Context, Result};
use crate::functions::resolve_file_home;
use crate::nonce::{export, import};


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
