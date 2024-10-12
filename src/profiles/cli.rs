use clap::Parser;
use clap::Subcommand;
//use crate::key::FromHex;
//use crate::key::PrivKey;
use crate::profiles::nomic;
use crate::profiles::OutputFormat;
use crate::profiles::ProfileCollection;
use eyre::Result;
use std::path::Path;
//use std::time::Duration;

/// Defines the CLI structure for the `profiles` command.
#[derive(Parser)]
#[command(
    name          = "Profile", 
    about         = "Manage & use profiles", 
    visible_alias = "p"
)]
pub struct Cli {
    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,

    /// Profile
    #[arg(default_value_t = String::new())]
    pub profile: String,

    /// Subcommands for the profiles command
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

/// Subcommands for the `profiles` command
#[derive(Debug, Subcommand)]
pub enum Command {
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

    /// Show the Balance
    #[command(visible_aliases = ["b", "bal"])]
    Balance,

    /// Show the Delegations
    #[command(visible_aliases = ["d", "del"])]
    Delegations,

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
#[derive(Debug, Subcommand)]
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

impl Cli {
    pub fn run(&self) -> Result<()> {
        let collection = ProfileCollection::new()?;

        // Check if no subcommand is provided
        if self.cmd.is_none() {
            // Print the collection for the specified profile if no subcommand is provided
            collection.print(self.format.clone())?;
            return Ok(());
        }

        if let Some(command) = &self.cmd {
            match command {
                Command::Nomic { args } => {
                    let home_path = collection.home(&self.profile)?;
                    // Call nomic and ignore the output by unwrapping it here
                    nomic(&home_path, Some(String::new()), args.clone()).map(|_output| ())?;
                    Ok(())
                }
                Command::Export => {
                    let output = collection.export(&self.profile)?;
                    println!("{}", output);
                    Ok(())
                }
                Command::Address => {
                    let output = collection.address(&self.profile)?;
                    println!("{}", output);
                    Ok(())
                }
                Command::Balance => {
                    let output = collection.balance(&self.profile)?;
                    println!("{:#?}", output);
                    Ok(())
                }
                Command::Delegations => {
                    let output = collection.delegations(&self.profile)?;
                    println!("{:#?}", output);
                    Ok(())
                }
                Command::Config => {
                    let output = collection.config(&self.profile)?;
                    println!("{:?}", output);
                    Ok(())
                }
                Command::Import { key, file } => {
                    // Handle file import if a file path is provided
                    if let Some(file_path) = file {
                        collection.import_file(&self.profile, Path::new(file_path))?;
                        println!("Profile '{}' imported from file: {}", self.profile, file_path);
                    } else {
                        // Ensure key is available and handle Option<String>
                        if let Some(key_str) = key {
                            collection.import(&self.profile, key_str, true)?; // Pass the profile and key directly
                            println!("Profile '{}' imported.", &self.profile); // Print profile name safely
                        } else {
                            eprintln!("No key provided for import.");
                        }
                    }
                    Ok(())
                }
                Command::Nonce { nonce_cmd } => {
                    match nonce_cmd {
                        NonceCmd::Export => {
                            // Handle nonce export logic here
                            let output = collection.export_nonce(&self.profile)?;
                            println!("{}", output);
                            Ok(())
                        }
                        NonceCmd::Import { value, dont_overwrite } => {
                            collection.import_nonce(&self.profile, *value, *dont_overwrite)
                        },
                    }
                },
            }
        } else {
            Ok(()) // This case should not happen because of the earlier check
        }
    }
}
