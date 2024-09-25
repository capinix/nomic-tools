use clap::{Arg, Command};

/// Command to handle the "profiles import" subcommands
pub fn import_subcommand() -> Command {
    Command::new("import")
        .about("Import a private key into the profiles collection")
        .subcommand(
            Command::new("file")
                .about("Import a private key from a binary file into the profiles collection")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Name of the profile"),
                )
                .arg(
                    Arg::new("privkey_file")
                        .required(true)
                        .help("Path to the private key file"),
                ),
        )
        .subcommand(
            Command::new("hex")
                .about("Import a private key from a hex string into the profiles collection")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Name of the profile"),
                )
                .arg(
                    Arg::new("hex_string")
                        .required(true)
                        .help("Hexadecimal private key string"),
                ),
        )
}
