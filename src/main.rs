mod cli;

use clap::Parser;

fn main() {
	cli::run_cli(&cli::Cli::parse());
}
