
use clap::Parser;
use clap::Subcommand;
use crate::key;
use crate::nonce;
use crate::profiles;
use crate::validators;
use eyre::Result;
use fmt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage and use profiles
    Profiles(profiles::Cli),
    Validators(validators::Cli),
    Nonce(nonce::Cli),
    Key(key::Cli),
    Fmt(fmt::cli::Cli),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Profiles(cli)   => cli.run(),
            Commands::Key(cli)        => cli.run(),
            Commands::Nonce(cli)      => cli.run(),
            Commands::Fmt(cli)        => cli.run(),
            Commands::Validators(cli) => cli.run(),
        }
    }
}
