use clap::{ Parser, Subcommand };
use fmt;
use crate::nomic::validators;
use crate::nomic::nonce;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Validators(validators::Cli),
    Nonce(nonce::Cli),
	/// Formats text and tables
    Fmt(fmt::cli::Cli),
}

pub fn run_cli(cli: &Cli) {
    match &cli.command {
        Commands::Validators(validators_cli) => {
            if let Err(e) = validators::run_cli(&validators_cli) {
                eprintln!("Error executing validators command: {:?}", e);
            }
        },
        Commands::Nonce(nonce_cli) => {
            if let Err(e) = nonce::run_cli(&nonce_cli) {
                eprintln!("Error executing nonce command: {:?}", e);
            }
        },
        Commands::Fmt(fmt_cli) => {
            fmt::cli::run_cli(&fmt_cli);
        },

    }
}
