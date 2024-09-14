/// This is the entry point for the application.
///
/// The `main` function sets up the command-line interface (CLI) and delegates the execution
/// of commands to the appropriate handler functions based on the user's input. It coordinates
/// between the `cli`, `handlers`, and `nomic` modules to ensure the desired functionality is executed.
///
/// # Main Application Workflow
///
/// - **CLI Initialization:**
///   - Calls `build_cli()` from the `cli` module to set up and configure the command-line interface.
///
/// - **Command Handling:**
///   - Checks if a subcommand is provided. If not, prints an error and exits.
///   - Matches the subcommand to the appropriate handler function from the `handlers` module:
///     - **Convert:** Invokes functionality defined in the `handlers::convert` module.
///     - **Validators:** Invokes functionality defined in the `handlers::validators` module.
///   - Handles errors from the handler functions and prints an appropriate error message.
///
/// # Example Commands
///
/// - **Convert Commands:**
///   - `nomic-tools convert binary to-decimal|to-hex`
///   - `nomic-tools convert decimal to-binary|to-hex`
///   - `nomic-tools convert hex to-binary|to-decimal`
///
/// - **Validators Commands:**
///   - `nomic-tools validators [-f|--format [json|json_pretty|raw|table|tuple]] [-t|--top|-b|--bottom|-s|--skip N]`
///   - `nomic-tools validators address <address> [-f|--format [json|json_pretty|raw|table|tuple]]`
///   - `nomic-tools validators moniker <moniker> [-f|--format [json|json_pretty|raw|table|tuple]]`

mod cli;
mod handlers;
mod globals;
mod nomic;

use cli::build_cli;
use handlers::convert;
use handlers::validators;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let matches = build_cli().get_matches();

	// Handle global options when no subcommand is used
	if matches.subcommand_name().is_none() {
		eprintln!("No subcommand provided");
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"No subcommand provided"
		)));
	} else {
		// Match and handle each subcommand here
		match matches.subcommand() {
			Some(("convert", sub_m)) => {
				if let Err(e) = convert::options(sub_m) {
					eprintln!("Error executing convert: {:?}", e);
					return Err(e.into());
				}
			},
			Some(("validators", sub_m)) => {
				if let Err(e) = validators::options(sub_m) {
					eprintln!("Error executing validators: {:?}", e);
					return Err(e.into());
				}
			}
			_ => {
				eprintln!("Invalid subcommand or missing arguments");
				return Err(Box::new(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					"Invalid subcommand or missing arguments"
				)));
			},

		}
	}
	Ok(())
}
