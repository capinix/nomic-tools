mod cli;
mod functions;
mod privkey;
mod nonce;
mod profiles;
mod globals;
mod validators;

use clap::Parser;
use eyre::Result;

fn main() -> Result<()> {
    crate::cli::Cli::parse().run()
}
