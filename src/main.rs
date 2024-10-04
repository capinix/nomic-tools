mod cli;
mod functions;
mod key;
mod nomic;
mod nonce;
mod profiles;

use clap::Parser;

fn main() {
	cli::run_cli(&cli::Cli::parse());
}
