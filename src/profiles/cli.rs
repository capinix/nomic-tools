use clap::Parser;
use clap::Subcommand;
use crate::profiles::CollectionOutputFormat;
use crate::profiles::nomic;
use crate::profiles::ProfileCollection;
use crate::profiles::ProfileOutputFormat;
use crate::functions::validate_ratio;
use eyre::Result;
use std::path::Path;

/// Defines the CLI structure for the `profiles` command.
#[derive(Parser)]
#[command(
    name  = "Profile", 
    about = "Manage & use profiles", 
)]
pub struct Cli {
    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<CollectionOutputFormat>,

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

    #[command(
        about = "Display Address",
        visible_alias = "a",
        aliases = ["ad", "add", "addr"],
    )]
    Address,

    #[command(
        about = "Auto Delegate",
        visible_alias = "u",
        aliases = ["au", "aut", "auto"],
    )]
    AutoDelegate,

    #[command(
        about = "Display Balance",
        visible_alias = "b",
        aliases = ["ba", "bal"],
    )]
    Balance,

    #[command(
        about = "Claim Staking Rewards",
        visible_alias = "l",
        aliases = ["cl", "cla", "clai"],
    )]
    Claim,

    #[command(
        about = "Profile configuration",
        visible_alias = "c",
        aliases = ["co", "con", "conf"],
    )]
    Config {
        #[arg(
            short = 'b',
            long = "minimum-balance",
            aliases = ["min-bal", "mb", "bal", "balance"],
        )]
        minimum_balance: Option<f64>,

        #[arg(
            short = 'r',
            long = "minimum-balance-ratio",
            aliases = ["min-bal-ratio", "mbr", "bal-ratio", "balance-ratio"],
            value_parser = validate_ratio,
        )]
        minimum_balance_ratio: Option<f64>,

        #[arg(
            short = 's',
            long = "minimum-stake",
            aliases = ["min-stake", "ms", "stk", "stake"],
        )]
        minimum_stake: Option<f64>,

        #[arg(
            short = 'a',
            long = "adjust-minimum-stake",
            aliases = ["adjust-min-stake", "ams", "adj-stk", "adjust", "adj"],
        )]
        adjust_minimum_stake: Option<bool>,

        #[arg(
            short = 'o',
            long = "minimum-stake-rounding",
            aliases = ["min-stake-round", "msr", "rnd", "round", "rounding"],
        )]
        minimum_stake_rounding: Option<f64>,

        #[arg(
            short = 'v',
            long = "rotate-validators",
            aliases = ["rotate"],
            help = "Rotate validators"
        )]
        rotate_validators: bool,

//      #[arg(
//          short = 'd',
//          long = "remove-validator",
//          aliases = ["remove"],
//          help = "Remove validator"
//      )]
//      remove_validator: Option<String>,

//      #[arg(
//          short = 'a',
//          long = "add-validator",
//          aliases = ["add"],
//          help = "Add validator (format: <address>,<moniker>)"
//          help = "Add validator"
//      )]
//      add_validator: Option<String>,
    },

    #[command(
        about = "Display Delegations",
        visible_alias = "d",
        aliases = ["delegati", "delegatio", "delegation"],
    )]
    Delegations,

    /// Export a profile
    #[command()]
    Export,

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

    /// Show the Stats
    #[command(visible_aliases = ["s"])]
    Stats {
        /// Specify the output format
        #[arg(long, short)]
        format: Option<ProfileOutputFormat>,
    },
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
                Command::Address => {
                    let output = collection.address(&self.profile)?;
                    println!("{}", output);
                    Ok(())
                }
                Command::AutoDelegate => {
                    let output = collection.profile_by_name_or_address(&self.profile)?;
                    output.auto_delegate(true)?;
                    Ok(())
                }
                Command::Balance => {
                    let output = collection.balance(&self.profile)?;
                    println!("{:#?}", output);
                    Ok(())
                }
                Command::Claim => {
                    let output = collection.profile_by_name_or_address(&self.profile)?;
                    output.nomic_claim()?;
                    Ok(())
                }
                Command::Config {
                    minimum_balance,
                    minimum_balance_ratio,
                    minimum_stake,
                    adjust_minimum_stake,
                    minimum_stake_rounding,
                    rotate_validators,
//                  remove_validator,
//                  add_validator,
                } => {
                    // Retrieve the profile and its configuration
                    let profile = collection.profile_by_name_or_address(&self.profile)?;
                    let mut config = profile.config()?.clone();

                    // If no options are provided, print the current configuration
                    if minimum_balance.is_none() 
                        && minimum_balance_ratio.is_none() 
                        && minimum_stake.is_none() 
                        && adjust_minimum_stake.is_none() 
                        && minimum_stake_rounding.is_none() {
                        println!("{:?}", config);
                    } else {
                        // If at least one option is provided, update the configuration
                        if *rotate_validators {
                            config.rotate_validators();
                        }
                        profile.edit_config(
                            minimum_balance.map(|b| (b * 1_000_000.0) as u64),
                            *minimum_balance_ratio,
                            minimum_stake.map(|b| (b * 1_000_000.0) as u64),
                            *adjust_minimum_stake,
                            minimum_stake_rounding.map(|b| (b * 1_000_000.0) as u64),
                        )?;
                    }
                    Ok(())
                }
                Command::Delegations => {
                    let output = collection.delegations(&self.profile)?;
                    println!("{:#?}", output);
                    Ok(())
                }
                Command::Export => {
                    let output = collection.export(&self.profile)?;
                    println!("{}", output);
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
                Command::Nomic { args } => {
                    let home_path = collection.home(&self.profile)?;
                    // Call nomic and ignore the output by unwrapping it here
                    nomic(&home_path, Some(String::new()), args.clone()).map(|_output| ())?;
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
                Command::Stats { format } => {
                    let profile = collection.profile_by_name_or_address(&self.profile)?;
                    profile.print(format.clone())
                }
            }
        } else {
            Ok(()) // This case should not happen because of the earlier check
        }
    }
}
