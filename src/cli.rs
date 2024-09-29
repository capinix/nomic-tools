use clap::{Args, Parser, Subcommand};
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
    /// Run the fmt CLI as a subcommand
    Fmt(fmt::cli::Cli), // FmtCli is used here
    // Add other subcommands as needed
}

pub fn run_cli(cli: &Cli) {
    match &cli.command {
        Commands::Fmt(fmt_cli) => {
            // Call the fmt CLI runner
            fmt::cli::run_cli(&fmt_cli); // Forward the fmt_cli command to the fmt library
        },
        // Handle other subcommands as needed
    }
}
