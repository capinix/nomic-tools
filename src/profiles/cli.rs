use clap::Parser;
use clap::Subcommand;
use crate::profiles::CollectionOutputFormat;
use crate::profiles::nomic;
use crate::profiles::ProfileCollection;
use crate::profiles::ProfileOutputFormat;
use crate::functions::validate_ratio;
use crate::functions::validate_positive;
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
    #[arg(required = true)]
    pub profile: String,

    /// Subcommands for the profiles command
    #[command(subcommand)]
    pub cmd: Command,
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
        about = "Display Balance",
        visible_alias = "b",
        aliases = ["ba", "bal"],
    )]
    Balance,

    #[command(
        about = "Claim Staking Rewards",
        visible_alias = "m",
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
            help = "Minimum wallet balance"
        )]
        minimum_balance: Option<f64>,

        #[arg(
            short = 'r',
            long = "minimum-balance-ratio",
            aliases = ["min-bal-ratio", "mbr", "bal-ratio", "balance-ratio"],
            value_parser = validate_ratio,
            help = "Ratio of total staked to leave as wallet balance"
        )]
        minimum_balance_ratio: Option<f64>,

        #[arg(
            short = 's', long,
            aliases = ["min-stake", "ms", "stk", "stake"],
            help = "Minimum stake"
        )]
        minimum_stake: Option<f64>,

        #[arg(
            short = 'j',
            long = "adjust-minimum-stake",
            aliases = ["adjust-min-stake", "ams", "adj-stk", "adjust", "adj"],
            help = "Adjust minimum stake to daily reward"
        )]
        adjust_minimum_stake: Option<bool>,

        #[arg(
            short = 'o', long,
            aliases = ["min-stake-round", "msr", "rnd", "round", "rounding"],
            help = "Minimum stake will be a multiple of this"
        )]
        minimum_stake_rounding: Option<f64>,

        #[arg(
            short = 'v', long,
            aliases = ["rotate"],
            help = "Rotate validators"
        )]
        rotate_validators: bool,

        #[arg(
            short = 'd', long,
            aliases = ["remove"],
            help = "Remove a validator"
        )]
        remove_validator: Option<String>,

        #[arg(
            short = 'a', long,
            aliases = ["add"],
            help = "Add validator (format: <address>,<moniker>)",
        )]
        add_validator: Option<String>,
    },

    #[command(
        about = "Delegate",
        visible_alias = "d",
        aliases = ["del"],
    )]
    Delegate {
        /// The validator to delegate to
        #[arg(
            short, long,
            help = "validator address or moniker"
        )]
        validator: Option<String>,

        /// The amount to delegate
        #[arg(
            short, long, help = "Quantity to stake", 
            value_parser = validate_positive::<f64>,
        )]
        quantity: Option<f64>,
    },

    #[command(
        about = "Display Delegations",
        visible_alias = "g",
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

    #[command(
        about = "List Profiles",
        visible_alias = "l",
        aliases = ["ls"],
    )]
    List,

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

        match &self.cmd {
            Command::Address => {
                println!("{}", collection.address(&self.profile)?);
                Ok(())
            }
            Command::Balance => {
                println!("{:#?}", collection.balances(&self.profile)?);
                Ok(())
            }
            Command::Claim => {
                collection
                    .profile_by_name_or_address(&self.profile)?
                    .nomic_claim()?;
                Ok(())
            }
            Command::Config {
                minimum_balance,
                minimum_balance_ratio,
                minimum_stake,
                adjust_minimum_stake,
                minimum_stake_rounding,
                rotate_validators,
                remove_validator,
                add_validator,
            } => {
                let mut profile = collection.profile_by_name_or_address(&self.profile)?.clone();

                // Check if any options are provided to modify the config
                if minimum_balance.is_some() || minimum_balance_ratio.is_some()
                    || minimum_stake.is_some() || adjust_minimum_stake.is_some()
                    || minimum_stake_rounding.is_some() || *rotate_validators
                    || remove_validator.is_some() || add_validator.is_some()
                {
                    profile.edit_config(
                        minimum_balance.map(|b| (b * 1_000_000.0) as u64),
                        *minimum_balance_ratio,
                        minimum_stake.map(|b| (b * 1_000_000.0) as u64),
                        *adjust_minimum_stake,
                        minimum_stake_rounding.map(|b| (b * 1_000_000.0) as u64),
                        *rotate_validators,
                        remove_validator.as_deref(),
                        add_validator.as_deref(),
                    )?;
                }
                println!("{:?}", profile.config()?.clone());
                Ok(())
            }
            Command::Delegate { validator, quantity } => {
                collection
                    .profile_by_name_or_address(&self.profile)?
                    .nomic_delegate(validator.clone(), *quantity)?;
                Ok(())
            }
            Command::Delegations => {
                println!("{:#?}", collection.delegations(&self.profile)?);
                Ok(())
            }
            Command::Export => {
                println!("{}", collection.export(&self.profile)?);
                Ok(())
            }
            Command::Import { key, file } => {
                // Handle file import if a file path is provided
                if let Some(file_path) = file {
                    collection.import_file(&self.profile, Path::new(&file_path))?;
                    println!("Profile '{}' imported from file: {}", self.profile, file_path);
                } else if let Some(key_str) = key {
                    collection.import(&self.profile, &key_str, true)?;
                    println!("Profile '{}' imported.", &self.profile);
                } else {
                    eprintln!("No key provided for import.");
                }
                Ok(())
            }
            Command::List => {
                collection.print(self.format.clone())
            }
            Command::Nomic { args } => {
                let home_path = collection.home(&self.profile)?;
                nomic(&home_path, Some(String::new()), args.clone())?;
                Ok(())
            }
            Command::Nonce { nonce_cmd } => {
                match nonce_cmd {
                    NonceCmd::Export => {
                        println!("{}", collection.export_nonce(&self.profile)?);
                        Ok(())
                    }
                    NonceCmd::Import { value, dont_overwrite } => {
                        collection.import_nonce(&self.profile, *value, *dont_overwrite)?;
                        Ok(())
                    }
                }
            }
            Command::Stats { format } => {
                collection.profile_by_name_or_address(&self.profile)?
                    .print(format.clone())
            }
        }
    }
}


//impl Cli {
//    pub fn run(&self) -> Result<()> {
//        let collection = ProfileCollection::new()?;
//
//            match self.cmd {
//                Command::Address => {
//                    Ok(println!("{}", collection
//                        .address(&self.profile)?
//                    ))
//                }
//                Command::Balance => {
//                    Ok(println!("{:#?}", collection
//                        .balances(&self.profile)?
//                    ))
//                }
//                Command::Claim => {
//                    Ok(collection
//                        .profile_by_name_or_address(&self.profile)?
//                        .nomic_claim()?
//                    )
//                }
//                Command::Config {
//                    minimum_balance,
//                    minimum_balance_ratio,
//                    minimum_stake,
//                    adjust_minimum_stake,
//                    minimum_stake_rounding,
//                    rotate_validators,
//                    remove_validator,
//                    add_validator,
//                } => {
//                    // Retrieve the profile and its configuration
//                    let mut profile: Profile = collection.profile_by_name_or_address(&self.profile)?.clone();
//
//                    // If no options are provided, print the current configuration
//                    if minimum_balance.is_some()
//                        || minimum_balance_ratio.is_some()
//                        || minimum_stake.is_some()
//                        || adjust_minimum_stake.is_some()
//                        || minimum_stake_rounding.is_some()
//                        || rotate_validators
//                        || remove_validator.is_some()
//                        || add_validator.is_some()
//                    {
//                        profile.edit_config(
//                            minimum_balance.map(|b| (b * 1_000_000.0) as u64),
//                            minimum_balance_ratio,
//                            minimum_stake.map(|b| (b * 1_000_000.0) as u64),
//                            adjust_minimum_stake,
//                            minimum_stake_rounding.map(|b| (b * 1_000_000.0) as u64),
//                            rotate_validators,
//                            remove_validator.as_deref(),
//                            add_validator.as_deref(),
//                        )?;
//                    }
//                    // let config = profile.config()?.clone();
//                    println!("{:?}", profile.config()?.clone());
//                    Ok(())
//                }
//                Command::Delegate { validator, quantity } => {
//                    collection.profile_by_name_or_address(&self.profile)?
//                        .nomic_delegate(validator.clone(), quantity)
//                }
//                Command::Delegations => {
//                    Ok(println!("{:#?}", collection
//                        .delegations(&self.profile)?
//                    ))
//                }
//                Command::Export => {
//                    Ok(println!("{}", collection
//                        .export(&self.profile)?
//                    ))
//                }
//                Command::Import { key, file } => {
//                    // Handle file import if a file path is provided
//                    if let Some(file_path) = file {
//                        collection.import_file(&self.profile, Path::new(&file_path))?;
//                        println!("Profile '{}' imported from file: {}", self.profile, file_path);
//                    } else {
//                        // Ensure key is available and handle Option<String>
//                        if let Some(key_str) = key {
//                            collection.import(&self.profile, &key_str, true)?; // Pass the profile and key directly
//                            println!("Profile '{}' imported.", &self.profile); // Print profile name safely
//                        } else {
//                            eprintln!("No key provided for import.");
//                        }
//                    }
//                    Ok(())
//                }
//                Command::List => {
//                    collection.print(self.format.clone())
//                }
//                Command::Nomic { args } => {
//                    let home_path = collection.home(&self.profile)?;
//                    // Call nomic and ignore the output by unwrapping it here
//                    nomic(&home_path, Some(String::new()), args.clone()).map(|_output| ())?;
//                    Ok(())
//                }
//                Command::Nonce { nonce_cmd } => {
//                    match nonce_cmd {
//                        NonceCmd::Export => {
//                            // Handle nonce export logic here
//                            Ok(println!("{}", collection
//                                .export_nonce(&self.profile)?
//                            ))
//                        }
//                        NonceCmd::Import { value, dont_overwrite } => {
//                            collection
//                                .import_nonce(&self.profile, value, dont_overwrite)
//                        },
//                    }
//                },
//                Command::Stats { format } => {
//                    collection
//                        .profile_by_name_or_address(&self.profile)?
//                        .print(format.clone())
//                }
//            }
//    }
//}
