/// This module defines the CLI structure and argument parsing for the application.
///
/// Each command and subcommand is defined here, which sets up the command-line interface for
/// the application. The structure of the CLI commands is used to route to the appropriate
/// handler functions in the `handlers` module.
/// 
/// # Adding a New Command
///
/// To add a new command to the CLI, follow these steps:
///
/// 1. **Define the CLI Structure:**
///    - Create a new file `src/cli/new_command.rs`.
///    - Define the command structure using `clap` in the `cli()` function.
///
/// 2. **Declare the Module:**
///    - Declare the new command module in `src/cli/mod.rs` so it can be included in the CLI builder.
///    - Example: Add `pub mod new_command;` to the module declarations.
///
/// 3. **Create the CLI Parser:**
///    - Create a corresponding file `src/handlers/new_command.rs`.
///    - Implement the `options()` function to handle command-line arguments and subcommands.
///
/// 4. **Declare the Command Handler:**
///    - Declare the new command module in `src/handlers/mod.rs` to make it available for use in the main application.
///    - Example: Add `pub mod new_command;` to the module declarations.
///
/// 5. **Implement the Command Logic:**
///    - Create the command logic in `src/nomic/new_command.rs`.
///    - Implement the functionality to be executed when the command is invoked.
///
/// 6. **Update the Main File:**
///    - Import the new command module in `src/main.rs`.
///    - Ensure the command is integrated into the command handling logic.
///
/// By following these steps, you can seamlessly add new commands and functionalities to the CLI.
use clap::Command;

mod convert;
mod validators;

pub fn build_cli() -> Command {
	Command::new("nomic-tools")
		.version("0.1.0")
		.author("Your Name <your.email@example.com>")
		.about("Tools for working with Nomic")
		.subcommand(convert::cli())	   // Conversions subcommands
		.subcommand(validators::cli()) // Validators subcommand
}
