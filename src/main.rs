mod cli;
mod nomic;
mod functions;

use clap::Parser;

fn main() {
	cli::run_cli(&cli::Cli::parse());
}
