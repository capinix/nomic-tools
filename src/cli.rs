use clap::{ Parser, Subcommand };
use fmt;
use crate::key;
use crate::nonce;
use crate::profiles;

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
//    Validators(validators::Cli),
    Nonce(nonce::Cli),
    Key(key::Cli),
    Fmt(fmt::cli::Cli),
}

impl Cli {
    pub fn run(&self) {
        match &self.command {
            Commands::Profiles(profiles_cli) => {
                if let Err(e) = profiles_cli.run() {
                    eprintln!("Error executing profiles command: {:?}", e);
                }
            },
            // Commands::Validators(validators_cli) => {
            //     if let Err(e) = validators::run_cli(&validators_cli) {
            //         eprintln!("Error executing validators command: {:?}", e);
            //     }
            // },
            Commands::Key(key_cli) => {
                if let Err(e) = key_cli.run() {
                    eprintln!("Error executing key command: {:?}", e);
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
}
