/// Contains handler functions for the `convert` subcommands.
///
/// This module implements the logic for handling conversion commands defined in the `convert`
/// subcommand of the CLI. It routes to the appropriate functions in the `nomic::convert` module
/// based on the specific conversion operation requested by the user.
///
/// # Subcommands
///
/// - **binary:** Handles binary format conversions
///   - `to-decimal`: Converts binary to decimal
///   - `to-hex`: Converts binary to hex
///
/// - **decimal:** Handles decimal format conversions
///   - `to-binary`: Converts decimal to binary
///   - `to-hex`: Converts decimal to hex
///
/// - **hex:** Handles hex format conversions
///   - `to-binary`: Converts hex to binary
///   - `to-decimal`: Converts hex to decimal
///
/// # Functionality
///
/// The `options()` function processes the `ArgMatches` from the CLI parser, dispatching the command
/// to the appropriate conversion function. Each conversion function is responsible for reading
/// input from stdin, performing the necessary conversion, and writing the result to stdout.
///
/// # Error Handling
///
/// If an invalid subcommand or command is provided, an error message is printed to stderr, and
/// an `Err` variant is returned.
///
/// # Example
///
/// ```
/// // Example usage in main.rs
/// // ...
/// let matches = build_cli().get_matches();
/// if let Err(e) = convert::options(matches.subcommand_matches("convert").unwrap()) {
///     eprintln!("Error executing convert command: {:?}", e);
/// }
/// ```
use clap::ArgMatches;
use std::error::Error;
use crate::nomic::convert;

pub fn options(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("binary", binary_matches)) => match binary_matches.subcommand() {
            Some(("to-decimal", _)) => convert::binary_to_decimal(),
            Some(("to-hex", _)) => convert::binary_to_hex(),
            _ => {
                eprintln!("Invalid binary subcommand");
                return Err("Invalid binary subcommand".into());
            }
        },
        Some(("decimal", decimal_matches)) => match decimal_matches.subcommand() {
            Some(("to-binary", _)) => convert::decimal_to_binary(),
            Some(("to-hex", _)) => convert::decimal_to_hex(),
            _ => {
                eprintln!("Invalid decimal subcommand");
                return Err("Invalid decimal subcommand".into());
            }
        },
        Some(("hex", hex_matches)) => match hex_matches.subcommand() {
            Some(("to-binary", _)) => convert::hex_to_binary(),
            Some(("to-decimal", _)) => convert::hex_to_decimal(),
            _ => {
                eprintln!("Invalid hex subcommand");
                return Err("Invalid hex subcommand".into());
            }
        },
        _ => {
            eprintln!("Invalid command");
            return Err("Invalid command".into());
        }
    }
}
