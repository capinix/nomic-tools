use clap::{Parser, Subcommand};
use crate::key::{FromHex, Privkey};
use crate::profiles::nomic;
use crate::profiles::OutputFormat;
use crate::profiles::ProfileCollection;
use eyre::Result;
use std::path::Path;
use std::time::Duration;


/// Defines the CLI structure for the `profiles` command.
#[derive(Parser)]
#[command(
	name = "Profile", 
	about = "Manage & use profiles", 
	visible_alias = "p"
)]
pub struct Cli {
    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,

    /// Profile
    #[arg()]
    pub profile: String,

    /// Subcommands for the profiles command
    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

/// Subcommands for the `profiles` command
#[derive(Subcommand)]
pub enum Cmd {
    /// Run nomic commands as profile
    #[command(visible_aliases = ["r"])]
    Nomic {
        /// Additional arguments to pass through (only if no subcommand is chosen)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Subcommand for nonce operations
    #[command(visible_aliases = ["n"])]
    Nonce {
        #[command(subcommand)]
        nonce_cmd: NonceCmd,
	},

    /// Show the AccountId
    #[command(visible_aliases = ["a", "addr"])]
    Address,

    /// Profile configuration
    #[command(visible_aliases = ["c", "conf"])]
    Config,

    /// Import a profile
    #[command(visible_alias = "i")]
    Import {
        /// Hex string or byte array, if neither key, nor file provided, will attempt to read from stdin
        #[arg(conflicts_with = "file")]
        key: Option<String>,

        /// The file path to import from
        #[arg(long, short)]
        file: Option<String>,
    },

    /// Export a profile
    #[command()]
    Export,
}


/// Subcommands for the `nonce` command
#[derive(Subcommand)]
pub enum NonceCmd {
    /// Export nonce from a file associated with a profile
    Export,

    /// Import nonce into the system
    Import {
        #[arg(long, short)]
        value: u64, // Value to import
//
//        #[arg(long, short, conflicts_with = "home")]
//        file: Option<PathBuf>, // Optional file path to import from
//
//        #[arg(long, short = 'H')]
//        home: Option<PathBuf>, // Optional home path

        #[arg(long = "dont-overwrite", short = 'D')]
        dont_overwrite: bool, // Flag to prevent overwriting
    },
}

pub fn run_cli(cli: &Cli) -> Result<()> {
	// Handle subcommands
	match &cli.cmd {
		// Handle nomic subcommand
		Some(Cmd::Nomic { args }) => {
			let collection = ProfileCollection::new()?;
			let home_path = collection.profile_home_path(&cli.profile)?;
			// Call nomic and ignore the output by unwrapping it here
			nomic(&home_path, Some(String::new()), args.clone()).map(|_output| ())?;
			Ok(())
		},
		// Handle export subcommand
		Some(Cmd::Export) => {
			let collection = ProfileCollection::new()?;
			let output = collection.profile_hex(&cli.profile)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(Cmd::Address) => {
			let collection = ProfileCollection::new()?;
			let output = collection.profile_address(&cli.profile)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(Cmd::Config) => {
			let collection = ProfileCollection::new()?;
			let output = collection.profile_config(&cli.profile)?;
			println!("{}", output);
			Ok(())
		},
		// Handle import subcomman
		Some(Cmd::Import { key, file }) => {
			let mut collection = ProfileCollection::new()?;

			// Handle file import if a file path is provided
			if let Some(file_path) = file {
				// Call import_file with the file path
				collection.import_file(&cli.profile, Path::new(file_path))?;
				println!("Profile '{}' imported from file: {}", &cli.profile, file_path);
			} else {
				let privkey = if let Some(key) = key {
					key.clone().privkey()?
				} else {
					Privkey::new_from_stdin(5, Duration::from_secs(500))?
				};
				// Call import with the decoded key
				collection.import(&cli.profile, privkey)?;
				println!("Profile '{}' imported.", &cli.profile);
			}

			Ok(())
		},
        // Handle nonce subcommands
        Some(Cmd::Nonce { nonce_cmd }) => {
            let collection = ProfileCollection::new()?;
            match nonce_cmd {
                NonceCmd::Export => {
                    // Handle nonce export logic here
                    let output = collection.export_nonce(&cli.profile)?;
                    println!("{}", output);
                    Ok(())
                },
                NonceCmd::Import { value, dont_overwrite } => {
					collection.import_nonce(&cli.profile, *value, *dont_overwrite)
                },
            }
        },
		// Default case when no subcommand is provided
		None => {
			let collection = ProfileCollection::new()?;
			collection.print(cli.format.clone());
			Ok(())
		},
	}

}
