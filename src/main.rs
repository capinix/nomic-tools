mod cli;
mod functions;
mod key;
mod nonce;
mod profiles;
mod globals;

use clap::Parser;

fn main() {
    crate::cli::Cli::parse().run();
}
