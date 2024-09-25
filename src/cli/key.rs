use clap::{Arg, Command};

/// Command to handle the "keys" subcommands
pub fn keys_subcommand() -> Command {
    Command::new("keys")
        .about("Manage private keys")
        .subcommand(
            Command::new("get")
                .about("Retrieve the private key as hex from a binary file")
                .arg(
                    Arg::new("privkey_file")
                        .required(true)
                        .help("Path to the private key file"),
                ),
        )
        .subcommand(
            Command::new("set")
                .about("Set a hex string as a private key in a binary file")
                .arg(
                    Arg::new("hex_string")
                        .required(true)
                        .help("Hexadecimal private key string"),
                )
                .arg(
                    Arg::new("privkey_file")
                        .required(true)
                        .help("Path to save the private key file"),
                ),
        )
}

// Main command with subcommands
pub fn cli() -> Command {
    Command::new("keys")
        .about("Manage keys")
        .subcommand(keys_subcommand())
}
