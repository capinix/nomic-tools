/// Defines the CLI structure for the `validators` command.
///
/// This module sets up the command-line interface for the `validators` command, which is responsible
/// for managing and querying validators. The `validators` command includes several subcommands for
/// different types of validator queries and operations. Each subcommand is routed to the appropriate
/// handler functions in the `handlers` module.
///
/// # Subcommands
///
/// - `top`: Show the top N validators
///   - `number`: Specify the number of top validators to show (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// - `bottom`: Show the bottom N validators
///   - `number`: Specify the number of bottom validators to show (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// - `skip`: Skip the first N validators
///   - `number`: Specify the number of validators to skip (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// - `random`: Show a specified number of random validators outside a specified top percentage
///   - `count`: Number of random validators to show (required)
///   - `percent`: Percentage of validators to consider for randomness (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// - `moniker`: Search for validators by moniker
///   - `moniker`: Search for validators by moniker (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// - `address`: Search for a validator by address
///   - `address`: Search for a validator by its address (required)
///   - `format`: Specify the output format (optional, defaults to `json-pretty`)
///
/// Each subcommand routes to the corresponding handler function in the `handlers` module.
use clap::{Arg, Command};

fn format_arg() -> Arg {
  Arg::new("format")
    .short('f')
    .long("format")
    .value_name("FORMAT")
    .value_parser(["json", "json-pretty", "raw", "table", "tuple"])
    .help("Specify the output format")
}

pub fn cli() -> Command {
  Command::new("validators")
    .about("Manage validators")
    .arg(
      format_arg()
        .default_value("json-pretty")
    )
    .subcommand(
      Command::new("top")
        .about("Show the top N validators")
        .arg(format_arg())
        .arg(
          Arg::new("number")
            .value_name("N")
            .value_parser(clap::value_parser!(usize))
            .help("Number of top validators to show")
            .required(true),
        ),
    )
    .subcommand(
      Command::new("bottom")
        .about("Show the bottom N validators")
        .arg(format_arg())
        .arg(
          Arg::new("number")
            .value_name("N")
            .value_parser(clap::value_parser!(usize))
            .help("Number of bottom validators to show")
            .required(true),
        ),
    )
    .subcommand(
      Command::new("skip")
        .about("Skip the first N validators")
        .arg(format_arg())
        .arg(
          Arg::new("number")
            .value_name("N")
            .value_parser(clap::value_parser!(usize))
            .help("Number of validators to skip")
            .required(true),
        ),
    )
    .subcommand(
      Command::new("random")
        .about("Show a specified number of random validators outside a specified top percentage")
        .arg(format_arg())
        .arg(
          Arg::new("count")
            .short('c')
            .long("count")
            .value_name("COUNT")
            .value_parser(clap::value_parser!(usize))
            .help("Number of random validators to show")
            .required(true),
        )
        .arg(
          Arg::new("percent")
            .short('p')
            .long("percent")
            .value_name("PERCENT")
            .value_parser(clap::value_parser!(u8))
            .help("Percentage of validators to consider for randomness")
            .required(true),
        ),
    )
    .subcommand(
      Command::new("moniker")
        .about("Search for validators by moniker")
        .arg(format_arg())
        .arg(
          Arg::new("moniker")
            .value_name("MONIKER")
            .help("Search for validators by moniker")
            .required(true)
            .value_parser(clap::value_parser!(String)),
        ),
    )
    .subcommand(
      Command::new("address")
        .about("Search for a validator by address")
        .arg(format_arg())
        .arg(
          Arg::new("address")
            .value_name("ADDRESS")
            .help("Search for a validator by its address")
            .required(true)
            .value_parser(clap::value_parser!(String)),
        ),
    )
}
