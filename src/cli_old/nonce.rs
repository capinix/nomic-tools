use clap::{Arg, Command};

/// Command to handle the "nonce" subcommands
pub fn nonce_subcommand() -> Command {
    Command::new("nonce")
        .about("Manage nonce values")
        .subcommand(
            Command::new("get")
                .about("Retrieve the nonce as a decimal from a binary file")
                .arg(
                    Arg::new("nonce_file")
                        .required(true)
                        .help("Path to the nonce file"),
                ),
        )
        .subcommand(
            Command::new("set")
                .about("Set a decimal value as a nonce in a binary file")
                .arg(
                    Arg::new("decimal_value")
                        .required(true)
                        .help("Decimal nonce value to set"),
                )
                .arg(
                    Arg::new("nonce_file")
                        .required(true)
                        .help("Path to save the nonce file"),
                ),
        )
}

// Main command with subcommands
pub fn cli() -> Command {
    Command::new("nonce")
        .about("Manage nonce")
        .subcommand(nonce_subcommand())
}
