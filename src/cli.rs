use clap::{ Parser, Subcommand };
use fmt;
use crate::nomic::validators;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Fmt(fmt::cli::Cli),
    Validators(validators::Cli),
}

pub fn run_cli(cli: &Cli) {
    match &cli.command {
        Commands::Fmt(fmt_cli) => {
            fmt::cli::run_cli(&fmt_cli);
        },
        Commands::Validators(validators_cli) => {
            if let Err(e) = validators::run_cli(&validators_cli) {
                eprintln!("Error executing validators command: {:?}", e);
            }
        },

    }
}
