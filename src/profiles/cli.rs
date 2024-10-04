use clap::{Parser, Subcommand};
use crate::key;
use crate::profiles::nomic;
use crate::profiles::OutputFormat;
use crate::profiles::ProfileCollection;
use eyre::Result;
use std::path::Path;


/// Defines the CLI structure for the `profiles` command.
#[derive(Parser)]
#[command(name = "Wallet", about = "Manage & use wallets", visible_alias = "w")]
pub struct Cli {
	/// Specify the output format
	#[arg(long, short)]
	pub format: Option<OutputFormat>,

	/// Subcommands for the nonce command
	#[command(subcommand)]
	pub command: Option<Cmd>,
}

/// Subcommands for the `profiles` command
#[derive(Subcommand)]
pub enum Cmd {
    /// run nomic commands as profile
	#[command(visible_aliases = ["n"])]
    Nomic {
        /// Profile
        #[arg()]
        profile: String,

		/// Additional arguments to pass through (only if no subcommand is chosen)
		#[arg(trailing_var_arg = true)]
		args: Vec<String>,
    },
    /// Show the AccountId
	#[command(visible_aliases = ["a", "addr"])]
    Address {
        /// Profile
        #[arg()]
        name: String,

    },
    /// Profile configuration
	#[command(visible_aliases = ["c", "conf"])]
    Config {
        /// Profile
        #[arg()]
        name: String,

    },
    /// Import a profile
	#[command(visible_alias = "i")]
    Import {
        /// new profile name
        #[arg()]
        name: String,

        /// hex string or byte array, if neither key, nor file provided, will attempt to read from stdin
        #[arg(conflicts_with = "file")]
        key: Option<String>,

        /// The file path to import from
        #[arg(long, short)]
        file: Option<String>,

    },
    /// Export
	#[command()]
    Export {
        /// Profile
        #[arg()]
        name: String,

    },
}

pub fn run_cli(cli: &Cli) -> Result<()> {
	// Handle subcommands
	match &cli.command {
		// Handle nomic subcommand
        Some(Cmd::Nomic { profile, args }) => {
            let collection = ProfileCollection::new()?;
            let home_path = collection.get_home_path(profile)?;

            // Call nomic and ignore the output by unwrapping it here
            nomic(&home_path, Some(String::new()), args.clone()).map(|_output| ())?;
			Ok(())
        },
		// Handle export subcommand
		Some(Cmd::Export { name }) => {
			let collection = ProfileCollection::new()?;
			let output = collection.get_hex(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(Cmd::Address { name }) => {
			let collection = ProfileCollection::new()?;
			let output = collection.get_address(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(Cmd::Config { name }) => {
			let collection = ProfileCollection::new()?;
			let output = collection.get_config(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle import subcomman
		Some(Cmd::Import { name, key, file }) => {
			let mut collection = ProfileCollection::new()?;

			// Handle file import if a file path is provided
			if let Some(file_path) = file {
				// Call import_file with the file path
				collection.import_file(name, Path::new(file_path))?;
				println!("Profile '{}' imported from file: {}", name, file_path);
			} else {
				// Call key_from_input_or_stdin, which reads from stdin if hex_string is None
				let key = key::key_from_input_or_stdin(key.clone())?;

				// Call import with the decoded key
				collection.import(name, key)?;
				println!("Profile '{}' imported.", name);
			}

			Ok(())
		},

		// Default case when no subcommand is provided
		None => {
			let collection = ProfileCollection::new()?;
			collection.print(cli.format.clone());
			Ok(())
		},
	}

}
