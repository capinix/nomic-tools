
use clap::Parser;
use clap::Subcommand;
use crate::key;
use crate::nonce;
use crate::profiles;
use crate::validators;
use eyre::Result;
use crate::profiles::ProfileCollection;
use fmt;
use crate::functions::validate_ratio;
use crate::functions::validate_positive;
use std::path::Path;
use crate::profiles::CollectionOutputFormat;
use crate::profiles::ProfileOutputFormat;
use crate::profiles::nomic;

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
        #[arg(required = true)]
        profile: String,
    },

    /// Display the Balance for a profile
    #[command(about = "Display the Balance for a profile",
        visible_alias = "ba", aliases = ["bal"],
    )]
    Balance {
        /// Profile
        #[arg(required = true)]
        profile: String,
    },

    #[command(about = "Claim staking rewards for a profile",
        visible_alias = "cl", aliases = ["cla", "clai"],
    )]
    Claim {
        /// Profile
        #[arg(required = true)]
        profile: String,
    },

    #[command(about = "Manage Profile Configuration",
        visible_alias = "co", aliases = ["con", "conf", "confi"],
    )]
    Config {

        /// Profile
        #[arg(required = true)]
        profile: String,

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

    #[command( about = "Display Delegations",
        visible_alias = "dn", aliases = ["delegati", "delegatio", "delegation"])]
    Delegations {
        /// Profile
        #[arg(required = true)]
        profile: String,
    },

    #[command( about = "Export Private Key",
        visible_alias = "ex", aliases = ["exp", "expo", "expor"])]
    Export {
        /// Profile
        #[arg(required = true)]
        profile: String,
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

    Key(key::Cli),

    #[command(about = "Run Nomic commands as Profile", visible_alias = "n",
        aliases = ["no", "nom", "nomi"])]
    Nomic {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// Additional arguments to pass through (only if no subcommand is chosen)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    #[command(about = "List Profiles", visible_alias = "pr",
        aliases = ["pro", "prof", "profi", "profil", "profile"])]
    Profiles {
        /// Specify the output format
        #[arg(long, short)]
        format: Option<CollectionOutputFormat>,
    },

    #[command(about = "Profile Statistics", visible_alias = "st",
        aliases = ["sta", "stat", "stati", "statis", "statist", "statisti", "statistic", "statistics"])]
    Stats {
        /// Profile
        #[arg(required = true)]
        profile: String,
        /// Specify the output format
        #[arg(long, short)]
        format: Option<ProfileOutputFormat>,
    },

    Nonce(nonce::Cli),
    Validators(validators::Cli),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Address { profile } => {
                let collection = ProfileCollection::new()?;
                let address = collection.address(profile)?;
                Ok(println!("{}", address))
            }
            Commands::Balance { profile } => {
                let collection = ProfileCollection::new()?;
                let balances = collection.balances(profile)?;
                Ok(println!("{:#?}", balances))
            }
            Commands::Claim { profile } => {
                let collection = ProfileCollection::new()?;
                let profile = collection.profile_by_name_or_address(profile)?;
                profile.nomic_claim()
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
                let collection = ProfileCollection::new()?;
                let mut profile = collection.profile_by_name_or_address(profile)?.clone();

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
                let collection = ProfileCollection::new()?;
                let profile = collection.profile_by_name_or_address(profile)?;
                profile.nomic_delegate(validator.clone(), *quantity)
            }

            Commands::Delegations { profile } => {
                let collection = ProfileCollection::new()?;
                Ok(println!("{:#?}", collection.delegations(profile)?))
            }

            Commands::Export { profile } => {
                let collection = ProfileCollection::new()?;
                Ok(println!("{}", collection.export(profile)?))
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
                let home_path = collection.home(profile)?;
                nomic(&home_path, Some(String::new()), args.clone())?;
                Ok(())
            }

            Commands::Profiles { format } => {
                let collection = ProfileCollection::new()?;
                collection.print(format.clone())
            }

            Commands::Stats { profile, format } => {
                let collection = ProfileCollection::new()?;
                collection.profile_by_name_or_address(profile)?
                    .print(format.clone())
            }

            Commands::Nonce(cli)      => cli.run(),
            Commands::Validators(cli) => cli.run(),
        }
    }
}
