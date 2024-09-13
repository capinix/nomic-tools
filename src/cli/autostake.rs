use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("autostake")
        .about("Handle autostake operations")
        // Add arguments and options for the `autostake` subcommand here
}
