use clap::{Parser, Subcommand};
use crate::functions::resolve_file_home;
use std::path::PathBuf;
use eyre::eyre;
use eyre::Result;
use std::time::Duration;
use crate::key::FromHex;
use crate::key::PrivKey;

/// Defines the CLI structure for the `privkey` command.
#[derive(Parser)]
#[command(name = "PrivKey", about = "Manage PrivKey File", visible_alias = "k",)]
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
    #[command(visible_alias = "a")]
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
    /// Save Private key to file
    #[command(visible_alias = "s")]
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
}

/// Retrieves the private key based on CLI options.
///
/// # Arguments
///
/// * `key` - An optional reference to a key provided via CLI.
/// * `file` - An optional file path provided via CLI.
/// * `home` - An optional home directory path provided via CLI.
/// * `cli_file` - A default file path specified in the CLI root.
/// * `cli_home` - A default home path specified in the CLI root.
///
/// # Returns
///
/// Returns a `Result` containing the private key or an error if something went wrong.
fn get_privkey(
    key: Option<String>, 
    file: Option<PathBuf>, 
    home: Option<PathBuf>, 
    cli_file: Option<PathBuf>, 
    cli_home: Option<PathBuf>,
) -> Result<PrivKey> {
    // Error if both file and home are provided
    if file.is_some() && home.is_some() {
        return Err(eyre!("Error: You cannot provide both 'file' and 'home' options at the same time."));
    }

    // Error if both key and either file or home are provided
    if key.is_some() && (file.is_some() || home.is_some()) {
        return Err(eyre!("Error: You cannot provide both 'key' and 'file' or 'home' options at the same time."));
    }

    // If key is provided, return the private key
    if let Some(key) = key {
        return key.privkey();
    }

    let (resolved_file, resolved_home) = resolve_file_home(file, home, cli_file, cli_home)?;
    PrivKey::load_from_file_or_home(resolved_file.as_deref(), resolved_home.as_deref(), false)
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Some(Command::Address { key, file, home }) => {
                // Get the private key using the helper function
                let privkey = get_privkey(
                    key.clone(),
                    file.clone(),
                    home.clone(),
                    self.file.clone(),
                    self.home.clone(),
                )?;

                // Use the private key's address
                let address = privkey.address()?;
                println!("{}", address.to_string());
                Ok(())
            },

            Some(Command::Export { key, file, home }) => {
                // Get the private key using the helper function
                let privkey = get_privkey(
                    key.clone(),
                    file.clone(),
                    home.clone(),
                    self.file.clone(),
                    self.home.clone(),
                )?;

                // Export the private key
                let export = privkey.export()?;
                println!("{}", export.to_string());
                Ok(())
            },

            Some(Command::Save { key, file, home, force }) => {
                // Check if both file and home are provided
                if file.is_some() && home.is_some() {
                    return Err(eyre!("Error: You cannot provide both 'file' and 'home' options at the same time."));
                }

                // Get the private key
                let privkey = match key {
                    Some(ref k) => k.clone().privkey()?,
                    None => PrivKey::stdin(5, Duration::from_secs(500))?,
                };

                // Save the private key to file or home
                privkey.save_to_file_or_home(file.as_deref(), home.as_deref(), *force)?;

                Ok(())
            },

            None => Err(eyre!("No command provided")),
        }
    }
}
