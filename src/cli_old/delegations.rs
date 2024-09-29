use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("delegations")
        .about("Handle delegations information")
        // Add arguments and options for the `delegations` subcommand here
}
