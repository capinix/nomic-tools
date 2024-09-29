mod cli;
mod nomic;

use clap::Parser;

fn main() {
	cli::run_cli(&cli::Cli::parse());
}
