mod cli;
mod functions;
mod privkey;
mod nonce;
mod profiles;
mod globals;
mod global;
mod validators;
mod journal;
mod z;

use clap::Parser;
use eyre::Result;
use log::info;

fn main() -> Result<()> {
    // Initialize the logger
    pretty_env_logger::init();

    // Example logging
    info!("Application started");
    // Your application logic here

    crate::cli::Cli::parse().run()
}
