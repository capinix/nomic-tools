use clap::{Args, Parser, Subcommand};
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
    Validators(validators::ValidatorsCli),
}

pub fn run_cli(cli: &Cli) {
    match &cli.command {
        Commands::Fmt(fmt_cli) => {
            fmt::cli::run_cli(&fmt_cli);
        },
        Commands::Validators(validators_cli) => {
            // Call the validators options handler
            // Here you can convert `validators_cli` to `ArgMatches`
            // and pass it to the `options` function
            let matches = validators_cli.to_arg_matches(); // You'll need to implement this conversion
            if let Err(e) = validators::options(&matches) {
                eprintln!("Error executing validators command: {:?}", e);
            }
        },

    }
}
