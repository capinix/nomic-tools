use clap::{Args, Subcommand};
use eyre::Result;
use crate::profiles::ProfileCollection;
use crate::journal::OutputFormat;
use crate::journal::tail;
use crate::journal::summary;
use crate::global::GroupBy;

#[derive(Debug, Args)]
#[command(about = "Profile Journal")]
pub struct Journal {
    /// Profile
    #[arg()]
    pub profile: Option<String>,

    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,
}

impl Journal {
    pub fn run(&self) -> Result<()> {
        let collection = ProfileCollection::new()?;
        collection.profile_by_name_or_address_or_home_or_default(self.profile.as_deref())?
            .journal()
            .print(self.format.clone())
    }
}

#[derive(Debug, Subcommand)]
pub enum JournalctlCommands {
    /// Display a summary grouped by profile or moniker
    Summary {
        #[arg(value_enum, default_value = "Profile", ignore_case = true)]
        group_by: GroupBy,

        #[arg(long, short, action = clap::ArgAction::SetTrue)]
        follow: bool,
    },
}

#[derive(Debug, Args)]
#[command(about = "journalctl -f")]
pub struct Journalctl {
    /// Staked group options
    #[arg(group = "stake_group")]
    #[arg(long, short)]
    pub staked: bool,

    #[arg(group = "stake_group")]
    #[arg(long, short)]
    pub not_staked: bool,

    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    pub follow: bool,

    #[command(subcommand)]
    pub subcommand: Option<JournalctlCommands>,
}

impl Journalctl {
    pub fn run(&self) -> Result<()> {
        match &self.subcommand {
            Some(JournalctlCommands::Summary { group_by, follow }) => {
                summary(group_by.clone(), *follow)
            }
            None => {
                let staked_or_not = if self.staked {
                    Some(true)
                } else if self.not_staked {
                    Some(false)
                } else {
                    None
                };
                tail(staked_or_not, self.follow)
            }
        }
    }
}

#[derive(Debug, Args)]
#[command(about = "Last Journal")]
pub struct LastJournal {
    /// Profile
    #[arg(required = true)]
    pub profile: String,

    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,
}

impl LastJournal {
    pub fn run(&self) -> Result<()> {
        let collection = ProfileCollection::new()?;
        collection.profile_by_name_or_address_or_home_or_default(Some(&self.profile))?
            .last_journal()?
            .print(self.format.clone())
    }
}
