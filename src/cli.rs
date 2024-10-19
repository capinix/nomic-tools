
use clap::Parser;
use clap::Subcommand;
use crate::functions::validate_positive;
use crate::functions::validate_ratio;
use crate::nonce;
use crate::privkey;
use crate::profiles::CollectionOutputFormat;
use crate::profiles::nomic;
use crate::profiles::ProfileCollection;
use crate::profiles::ProfileOutputFormat;
use crate::validators;
use eyre::Result;
use fmt;
use std::path::Path;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {

    #[command(about = "Display the Address (AccountId) for a profile",
        visible_alias = "ad", aliases = ["add", "addr"],
    )]
    Address {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    #[command(about = "Auto delegate all profiles",
        visible_alias = "au", aliases = ["auto"],
    )]
    AutoDelegate,

    /// Display the Balance for a profile
    #[command(about = "Display the Balance for a profile",
        visible_alias = "ba", aliases = ["bal"],
    )]
    Balance {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    #[command(about = "Claim staking rewards for a profile",
        visible_alias = "cl", aliases = ["cla", "clai"],
    )]
    Claim {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    #[command(about = "Manage Profile Configuration",
        visible_alias = "co", aliases = ["con", "conf", "confi"],
    )]
    Config {

        /// Profile
        #[arg()]
        profile: Option<String>,

        #[arg(
            short = 'b', long = "minimum-balance",
            aliases = ["min-bal", "mb", "bal", "balance"],
            help = "Minimum wallet balance"
        )]
        minimum_balance: Option<f64>,

        #[arg(
            short = 'r', long = "minimum-balance-ratio",
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

    #[command( about = "Delegate",
        visible_alias = "de",
    )]
    Delegate {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// The validator to delegate to
        #[arg(
            // short, long,
            help = "validator address or moniker"
        )]
        validator: Option<String>,

        /// The amount to delegate
        #[arg(
            // short, long, 
            help = "Quantity to stake", 
            value_parser = validate_positive::<f64>,
        )]
        quantity: Option<f64>,
    },

    #[command( about = "Display Delegations",
        visible_alias = "dn", aliases = ["delegati", "delegatio", "delegation"])]
    Delegations {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    #[command( about = "Export Private Key",
        visible_alias = "ex", aliases = ["exp", "expo", "expor"])]
    Export {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    Fmt(fmt::cli::Cli),

    #[command( about = "Import Private Key", visible_alias = "im",
        aliases = ["imp", "impo", "impor"])]
    Import {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// Hex string or byte array, if neither key, nor file provided, will attempt to read from stdin
        #[arg(conflicts_with = "file")]
        key: Option<String>,

        /// The file path to import from
        #[arg(long, short)]
        file: Option<String>,
    },

    Key(privkey::Cli),

    #[command(about = "Run Nomic commands as Profile", visible_alias = "n", aliases = ["no", "nom", "nomi"])]
    Nomic {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// Additional arguments to pass through (only if no subcommand is chosen)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    Nonce(nonce::Cli),

    #[command(about = "List Profiles", visible_alias = "pr",
        aliases = ["pro", "prof", "profi", "profil", "profile"])]
    Profiles {
        /// Specify the output format
        #[arg(long, short)]
        format: Option<CollectionOutputFormat>,
    },

    #[command( about = "Send",
        visible_alias = "se",
    )]
    Send {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// The validator to delegate to
        #[arg(
            // short, long,
            help = "destination address"
        )]
        destination: Option<String>,

        /// The quantity to send
        #[arg(
            // short, long, 
            help = "Quantity to send", 
            value_parser = validate_positive::<f64>,
        )]
        quantity: Option<f64>,
    },

    #[command(about = "Profile Statistics", visible_alias = "st",
        aliases = ["sta", "stat", "stati", "statis", "statist", "statisti", "statistic", "statistics"])]
    Stats {
        /// Profile
        #[arg()]
        profile: Option<String>,
        /// Specify the output format
        #[arg(long, short)]
        format: Option<ProfileOutputFormat>,
    },

    Validators(validators::Cli),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Address { profile } => {
                let address = ProfileCollection::new()?.address(profile.as_deref())?;
                Ok(println!("{}", address))
            }
            Commands::AutoDelegate => {
                ProfileCollection::load()?.auto_delegate()
            }
            Commands::Balance { profile } => {
                let balances = ProfileCollection::new()?.balances(profile.as_deref())?;
                Ok(println!("{:#?}", balances))
            }
            Commands::Claim { profile } => {
                ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(profile.as_deref())?
                    .nomic_claim()
            }
            Commands::Config {
                profile,
                minimum_balance,
                minimum_balance_ratio,
                minimum_stake,
                adjust_minimum_stake,
                minimum_stake_rounding,
                rotate_validators,
                remove_validator,
                add_validator,
            } => {
                let mut profile = ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(profile.as_deref())?;

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

            Commands::Delegate { profile, validator, quantity } => {
                ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(Some(profile))?
                    .nomic_delegate(validator.clone(), *quantity)
            }

            Commands::Delegations { profile } => {
                let delegations = ProfileCollection::new()?.delegations(profile.as_deref())?;
                Ok(println!("{:#?}", delegations))
            }

            Commands::Export { profile } => {
                let export = ProfileCollection::new()?.export(profile.as_deref())?;
                Ok(println!("{}", export))
            }

            Commands::Fmt(cli)        => cli.run(),

            Commands::Import { profile, key, file } => {
                let collection = ProfileCollection::new()?;
                // Handle file import if a file path is provided
                if let Some(file_path) = file {
                    collection.import_file(profile, Path::new(&file_path))?;
                    println!("Profile '{}' imported from file: {}", profile, file_path);
                } else if let Some(key_str) = key {
                    collection.import(profile, &key_str, true)?;
                    println!("Profile '{}' imported.", profile);
                } else {
                    eprintln!("No key provided for import.");
                }
                Ok(())
            }

            Commands::Key(cli)        => cli.run(),

            Commands::Nomic { profile, args } => {
                let collection = ProfileCollection::new()?;
                let home_path = collection.home(Some(profile))?;
                nomic(&home_path, Some(String::new()), args.clone())?;
                Ok(())
            }

            Commands::Nonce(cli)      => cli.run(),

            Commands::Profiles { format } => {
                ProfileCollection::new()?.print(format.clone())
            }

            Commands::Send { profile, destination, quantity } => {
                ProfileCollection::new()?.send(Some(profile), destination.as_deref(), *quantity)
            }

            Commands::Stats { profile, format } => {
                let collection = ProfileCollection::new()?;
                collection.profile_by_name_or_address_or_home_or_default(profile.as_deref())?
                    .print(format.clone())
            }

            Commands::Validators(cli) => cli.run(),
        }
    }
}
