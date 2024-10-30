
use clap::{Args, Parser, Subcommand};
use eyre::Result;
use crate::profiles::ProfileCollection;
use crate::functions::validate_ratio;

#[derive(Parser)]
#[command(name = "ProfileConfig", about = "Profile Configuration Settings")]
pub struct Cli {
    /// Profile
    #[arg()]
    profile: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Edit(EditArgs),

    Set {
        #[command(subcommand)]
        command: Option<SetCommands>
    },
}

#[derive(Debug, Subcommand)]
pub enum SetCommands {
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

#[derive(Debug, Args)]
pub struct EditArgs {

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

impl Cli {
    pub fn run(&self) -> Result<()> {

        // Initialize ProfileCollection and get the profile if applicable
        let profile_collection = ProfileCollection::new()?;
        let profile = profile_collection.profile_by_name_or_address_or_home_or_default(
            self.profile.as_deref(),
        )?;

        match &self.command {
            Some(Command::Edit(args)) => {
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

            Some(Command::Set { command }) => {
                if let Some(cmd) = command {
                    match cmd {
                        SetCommands::MinimumBalance { minimum_balance } => {
                            let bal = minimum_balance.map(|b| (b * 1_000_000.0) as u64);
                            profile.set_config_minimum_balance(bal)
                        }
                        SetCommands::MinimumStake { minimum_stake } => {
                            let stake = minimum_stake.map(|b| (b * 1_000_000.0) as u64);
                            profile.set_config_minimum_stake(stake)
                        }
                        SetCommands::DailyReward { daily_reward } => {
                            let reward = daily_reward.map(|b| (b * 1_000_000.0) as u64);
                            profile.set_config_daily_reward(reward)
                        }
                    }
                } else {
                    println!("No subcommand provided for Set.");
                    Ok(())
                }
            }
            _ => {
                println!("{}", profile.config());
                Ok(())
            }

        } //match
    } // run
} // impl
