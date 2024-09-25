use clap::{Arg, Command};

// Command to get the address of a profile
fn address_subcommand() -> Command {
    Command::new("address")
        .about("Get the address of a profile")
        .visible_alias("AccountID")
		.aliases(&["a", "ad", "add", "addr", "addre", "addres"])
		.aliases(&["ac", "acc", "acct", "acco", "accou", "accoun", "account"])
		.aliases(&["ai", "a-i", "acid", "ac-id", "accid", "acc-id", "acctid", "acct-id", "accountid"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
}

fn config_file_subcommand() -> Command {
    Command::new("file")
        .about("Get the config file path for a profile")
		.aliases(&["f", "fi", "fil"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
}

fn config_subcommand() -> Command {
    Command::new("config")
        .about("Manage config file")
		.aliases(&["c", "co", "con", "conf", "confi"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
		.subcommand(config_file_subcommand())
}

fn home_path_subcommand() -> Command {
    Command::new("home_path")
        .about("Get the home path for a profile")
		.aliases(&["home", "ho", "hom", "homepath"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
}

fn key_file_subcommand() -> Command {
    Command::new("file")
        .about("Get the key file path for a profile")
		.aliases(&["f", "fi", "fil"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
}

// Subcommand for `key` (default behavior is get)
fn key_subcommand() -> Command {
    Command::new("key")
        .about("Get or set the key for a profile")
		.visible_aliases(&["keplr", "hex-key"])
		.aliases(&["k", "ke", "kep", "kepl", "hex", "hexkey"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
        .arg(Arg::new("set")
            .help("Set the key for a profile")
            .required(false)
            .value_name("hex_key"))
		.subcommand(key_file_subcommand())
}

fn nonce_file_subcommand() -> Command {
    Command::new("file")
        .about("Get the nonce file path for a profile")
		.aliases(&["f", "fi", "fil"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
}

// Subcommand for `nonce` (default behavior is get)
fn nonce_subcommand() -> Command {
    Command::new("nonce")
        .about("Get or set the nonce for a profile")
		.aliases(&["n", "no", "non", "nonc"])
        .arg(Arg::new("profile_name")
            .required(true)
            .help("Name of the profile"))
        .arg(Arg::new("set")
            .help("Set the nonce value for a profile")
            .required(false)
            .value_name("decimal_number"))
		.subcommand(nonce_file_subcommand())
}

// Command to import profile with private key
fn import_subcommand() -> Command {
    Command::new("import")
        .about("Import profiles")
        .arg(
            Arg::new("input_type")
                .required(true)
				.value_parser(["file", "hex"])
                .help("Type of input: 'file' for a file path or 'hex' for a hex string"),
        )
        .arg(
            Arg::new("input")
                .required(true)
                .help("The input: file path for private key or hex string"),
        )
}

// Main command with subcommands
pub fn cli() -> Command {
    Command::new("profiles")
        .about("Manage Profiles")
        .arg(Arg::new("format")
			.short('f')
			.long("format")
			.value_name("FORMAT")
			.value_parser(["json", "json-pretty", "list", "table"])
			.help("Specify the output format"))
        .subcommand(address_subcommand())
        .subcommand(config_subcommand())
        .subcommand(home_path_subcommand())
        .subcommand(import_subcommand())
        .subcommand(key_subcommand())
        .subcommand(nonce_subcommand())
}
