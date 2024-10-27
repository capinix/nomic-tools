use clap::Args;
use clap::Parser;
use clap::Subcommand;
use crate::functions::validate_positive;
use crate::functions::validate_ratio;
use crate::journal::OutputFormat as JournalOutputFormat;
use crate::journal::tail;
use crate::journal::summary;
use crate::global::GroupBy;
use crate::global::CONFIG;
use crate::nonce;
use crate::privkey;
use crate::profiles::CollectionOutputFormat;
use crate::profiles::nomic;
use crate::profiles::ProfileCollection;
use crate::validators;
use crate::z;
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

#[derive(Args, Debug)]
struct StakedGroup {
    #[arg(long)]
    staked: bool,

    #[arg(long)]
    not_staked: bool,
}

#[derive(Debug, Args)]
pub struct ConfigEditArgs {

    #[arg(
        short = 'b', long,
        aliases = ["min-bal", "mb", "bal", "balance"],
        help = "Minimum wallet balance",
    )]
    minimum_balance: Option<f64>,

    #[arg(
        short = 'r', long,
        aliases = ["min-bal-ratio", "mbr", "bal-ratio", "balance-ratio"],
        value_parser = validate_ratio,
        help = "Ratio of total staked to leave as wallet balance"
    )]
    minimum_balance_ratio: Option<f64>,

    #[arg(
        short = 's', long,
        aliases = ["min-stake", "ms", "stk", "stake"],
        help = "Minimum stake",
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
        short = 'd', long,
        aliases = ["daily"],
        help = "Daily Reward",
    )]
    daily_reward: Option<f64>,

    #[arg(
        short = 'a', long,
        aliases = ["add"],
        help = "Add validator (format: <address>,<moniker>)",
    )]
    add_validator: Option<String>,

    #[arg(
        short = 'v', long,
        aliases = ["rotate"],
        help = "Rotate validators",
        default_value_t = false,
    )]
    rotate_validators: bool,

    #[arg(
        short = 'x', long,
        aliases = ["remove"],
        help = "Remove a validator"
    )]
    remove_validator: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSetCommands {
    #[command(about = "Set the minimum wallet balance",
        visible_alias = "mb", aliases = ["mbal", "min-bal"],
    )]
    MinimumBalance {
        #[arg()]
        minimum_balance: Option<f64>,
    },

    #[command(about = "Set the minimum stake",
        visible_alias = "ms", aliases = ["mstk", "min-stk"],
    )]
    MinimumStake {
        #[arg()]
        minimum_stake: Option<f64>,
    },

    #[command(about = "Set the daily reward",
        visible_alias = "dr", aliases = ["drwd", "dly-rwd"],
    )]
    DailyReward {
        #[arg()]
        daily_reward: Option<f64>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Edit(ConfigEditArgs),

    Journalctl {
        #[command(subcommand)]
        command: ConfigJournalctlCmd
    },

    Set {
        #[command(subcommand)]
        command: ConfigSetCommands
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigJournalctlCmd {
    #[command(about = "Configure Journal column widths",
        visible_alias = "w", aliases = ["wi"],
    )]
    Widths {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },
    Summary {
        #[command(subcommand)]
        command: ConfigJournalctlSummary,
    },

}

#[derive(Debug, Subcommand)]
pub enum ConfigJournalctlSummary {
    #[command(
        about = "Configure Journal column widths for profiles",
        visible_alias = "p", aliases = ["prof"]
    )]
    Profile {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },

    #[command(about = "Configure Journal column widths for monikers")]
    Moniker {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },
}


#[derive(Debug, Subcommand)]
pub enum JournalctlCommands {
    /// Display a summary grouped by profile or moniker
    Summary {
        #[arg(value_enum, default_value = "Profile", ignore_case = true)]
        group_by: GroupBy,
    },
}


#[derive(Subcommand)]
pub enum Commands {

    #[command(about = "Display the Address (AccountId) for a profile",
        visible_alias = "a", aliases = ["ad", "add", "addr", "addre", "addres"],
    )]
    Address {
        /// Profile
        #[arg()]
        profile: Option<String>,
    },

    #[command(about = "Auto delegate all profiles",
        visible_alias = "auto", aliases = ["au", "aut"],
    )]
    AutoDelegate,

    /// Display the Balance for a profile
    #[command(about = "Display the Balance for a profile",
        visible_alias = "b", aliases = ["ba", "bal", "bala", "balan", "balanc"],
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

        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },

    #[command( about = "Delegate", visible_alias = "de",)]
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

    #[command( about = "Import Private Key", visible_alias = "im", aliases = ["imp", "impo", "impor"])]
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

    #[command(about = "Profile Journal", visible_alias = "jo", aliases = ["jou", "jour", "journ", "journa"])]
    Journal {
        /// Profile
        #[arg()]
        profile: Option<String>,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<JournalOutputFormat>,
    },

    #[command(about = "journalctl -f", visible_alias = "log")]
    Journalctl {
        /// Staked group options
        #[arg(group = "stake_group")]
        #[arg(long, short)]
        staked: bool,

        #[arg(group = "stake_group")]
        #[arg(long, short)]
        not_staked: bool,

        #[command(subcommand)]
        subcommand: Option<JournalctlCommands>,

    },

    Key(privkey::Cli),

    #[command( about = "Last Journal", visible_alias = "lj", aliases = ["lastj"])]
    LastJournal {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<JournalOutputFormat>,
    },

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

    #[command( about = "Redelegate", visible_alias = "re",)]
    Redelegate {
        /// Profile
        #[arg(required = true)]
        profile: String,

        /// The validator to redelegate from
        #[arg(help = "redelegate from ..")]
        from: String,

        /// The validator to redelegate to
        #[arg(help = "redelegate to ..")]
        to: String,

        /// The quantity to redelegate in nom
        #[arg(
            help = "Quantity to Redelegate (NOM)", 
            value_parser = validate_positive::<f64>,
        )]
        quantity: f64,
    },

    #[command( about = "Send", visible_alias = "se",)]
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

    Validators(validators::Cli),
    Z(z::Cli),
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
            Commands::Config { profile, command } => {
                let profile = ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(profile.as_deref())?;
                match command {
                    Some(ConfigCommand::Edit(args)) => {
                        profile.edit_config(
                            args.minimum_balance.map(|v| (v * 1_000_000.0) as u64),
                            args.minimum_balance_ratio.map(|v| (v * 1_000_000.0) as u64),
                            args.minimum_stake.map(|v| (v * 1_000_000.0) as u64),
                            args.adjust_minimum_stake,
                            args.minimum_stake_rounding.map(|v| (v * 1_000_000.0) as u64),
                            args.daily_reward.map(|v| (v * 1_000_000.0) as u64),
                            args.add_validator.clone(),
                            args.remove_validator.clone(),
                            args.rotate_validators,
                        )?;
                        Ok(())
                    }
                    Some(ConfigCommand::Journalctl { command }) => {
                        let mut config = CONFIG.clone();
                        match command {
                            ConfigJournalctlCmd::Widths { column, width } => {

                                match (column, width) {
                                    (None, None) => {
                                        // Both column and width are None: Print the entire column widths array
                                        println!("{:?}", config.journalctl.tail.column_widths);
                                    }
                                    (Some(col), None) => {
                                        // Column is provided but no width: Print the specific column width
                                        if *col < config.journalctl.tail.column_widths.len() {
                                            println!("{:?}", config.journalctl.tail.column_widths[*col - 1]);
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    (Some(col), Some(w)) => {
                                        // Both column and width are provided: Set the column width
                                        if *col < config.journalctl.tail.column_widths.len() {
                                            config.set_journalctl_tail_column_width(*col - 1, *w)?;
                                            println!("{:?}", config.journalctl.tail.column_widths);
                                            config.save()?;
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    _ => {
                                        eprintln!("Invalid input: Either both column and width must be provided, or just column for query.");
                                    }
                                }
                                Ok(())
                            },
                            ConfigJournalctlCmd::Summary { command } => {
                                match command {
                                    ConfigJournalctlSummary::Profile { column, width } => {
                                        match (column, width) {
                                            (None, None) => {
                                                // Both column and width are None: Print the entire column widths array
                                                println!("{:?}", config.journalctl.summary.profile.column_widths);
                                            }
                                            (Some(col), None) => {
                                                // Column is provided but no width: Print the specific column width
                                                if *col < config.journalctl.summary.profile.column_widths.len() {
                                                    println!("{:?}", config.journalctl.summary.profile.column_widths[*col - 1]);
                                                } else {
                                                    eprintln!("Column index out of bounds: {}", col);
                                                }
                                            }
                                            (Some(col), Some(w)) => {
                                                // Both column and width are provided: Set the column width
                                                if *col < config.journalctl.summary.profile.column_widths.len() {
                                                    config.set_journalctl_summary_column_width(GroupBy::Profile, *col - 1, *w)?;
                                                    println!("{:?}", config.journalctl.summary.profile.column_widths);
                                                    config.save()?;
                                                } else {
                                                    eprintln!("Column index out of bounds: {}", col);
                                                }
                                            }
                                            _ => {
                                                eprintln!("Invalid input: Either both column and width must be provided, or just column for query.");
                                            }
                                        }

                                    }
                                    ConfigJournalctlSummary::Moniker { column, width } => {
                                        match (column, width) {
                                            (None, None) => {
                                                // Both column and width are None: Print the entire column widths array
                                                println!("{:?}", config.journalctl.summary.moniker.column_widths);
                                            }
                                            (Some(col), None) => {
                                                // Column is provided but no width: Print the specific column width
                                                if *col < config.journalctl.summary.moniker.column_widths.len() {
                                                    println!("{:?}", config.journalctl.summary.moniker.column_widths[*col - 1]);
                                                } else {
                                                    eprintln!("Column index out of bounds: {}", col);
                                                }
                                            }
                                            (Some(col), Some(w)) => {
                                                // Both column and width are provided: Set the column width:w
                                                //
                                                if *col < config.journalctl.summary.moniker.column_widths.len() {
                                                    config.set_journalctl_summary_column_width(GroupBy::Moniker, *col - 1, *w)?;
                                                    println!("{:?}", config.journalctl.summary.moniker.column_widths);
                                                    config.save()?;
                                                } else {
                                                    eprintln!("Column index out of bounds: {}", col);
                                                }
                                            }
                                            _ => {
                                                eprintln!("Invalid input: Either both column and width must be provided, or just column for query.");
                                            }
                                        }
                                    }
                                }

                                // Save the updated config if any changes were made
                                Ok(())
                            },

                        }
                    }
                    Some(ConfigCommand::Set { command }) => {
                        match command {
                            ConfigSetCommands::MinimumBalance { minimum_balance } => {
                                let bal = minimum_balance.map(|b| (b * 1_000_000.0) as u64);
                                profile.set_config_minimum_balance(bal)
                            },
                            ConfigSetCommands::MinimumStake { minimum_stake } => {
                                let stake = minimum_stake.map(|b| (b * 1_000_000.0) as u64);
                                profile.set_config_minimum_stake(stake)
                            },
                            ConfigSetCommands::DailyReward { daily_reward } => {
                                let reward = daily_reward.map(|b| (b * 1_000_000.0) as u64);
                                profile.set_config_daily_reward(reward)
                            },
                        }
                    }
                    _ => {
                        println!("{}", profile.config().clone());
                        Ok(())
                    }
                }
            }


            Commands::Delegate { profile, validator, quantity } => {
                ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(Some(profile))?
                    .nomic_delegate(validator.clone(), *quantity, false)
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

            Commands::Journal { profile, format } => {
                let collection = ProfileCollection::new()?;
                collection.profile_by_name_or_address_or_home_or_default(profile.as_deref())?
                    .journal()
                    .print(format.clone())
            }

            Commands::Key(cli)        => cli.run(),

            Commands::LastJournal { profile, format } => {
                let collection = ProfileCollection::new()?;
                collection.profile_by_name_or_address_or_home_or_default(Some(profile))?
                    .last_journal()?
                    .print(format.clone())
                //Ok(println!("{:#?}", journal))
            }
            Commands::Journalctl { staked, not_staked, subcommand } => {
                match subcommand {
                    Some(JournalctlCommands::Summary { group_by }) => {
                        summary(group_by.clone())
                    }
                    None => {
                        // Determine whether to tail with staked or not
                        let staked_or_not = if *staked {
                            Some(true)
                        } else if *not_staked {
                            Some(false)
                        } else {
                            None
                        };

                        // Call tail_journalctl with the determined staked status
                        tail(staked_or_not)
                    }
                }
            }
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

            Commands::Redelegate { profile, from, to, quantity } => {
                ProfileCollection::new()?
                    .profile_by_name_or_address_or_home_or_default(Some(profile))?
                    .redelegate(from, to, *quantity)
            }

            Commands::Send { profile, destination, quantity } => {
                ProfileCollection::new()?.send(Some(profile), destination.as_deref(), *quantity)
            }

            Commands::Validators(cli) => cli.run(),
            Commands::Z(cli) => cli.run(),
        }
    }
}
