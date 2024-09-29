use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("balance")
        .about("Handle balance information")
        // Add arguments and options for the `balance` subcommand here
}
