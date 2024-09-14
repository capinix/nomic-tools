/// This module contains handlers for various subcommands.
///
/// The `handlers` module is responsible for the implementation of the logic for each subcommand
/// defined in the `cli` module. For each subcommand, there is a corresponding handler function
/// in this module that processes the command-line arguments and performs the requested actions.
///
/// # Subcommand Handlers
///
/// - **Convert:** Handles conversions between different formats (e.g., binary to decimal, decimal to hex).
/// - **Validators:** Manages operations related to validators (e.g., listing top/bottom validators, searching by moniker or address).
///
/// # Adding a New Command Handler
///
/// To add a new command handler:
///
/// 1. **Create the Command Parser:**
///    - Define the command and its options in the `cli` module by creating a new file `src/cli/new_command.rs`.
///    - Implement the logic to handle command-line arguments and subcommands in this new file.
///
/// 2. **Declare the Command Module:**
///    - Import the new command handler module in this file by adding `pub mod new_command;` to the module declarations.
///    - Implement the command handling logic in the `src/handlers/new_command.rs` file.
///
/// 3. **Update the CLI Builder:**
///    - Ensure the new command is included in the CLI builder function in `src/cli/mod.rs` so that it is available in the CLI interface.
///
/// By following these steps, you can integrate new command handlers into the application, enabling new functionalities and commands to be processed.
pub mod convert;
pub mod validators;
// Uncomment and add additional handlers as needed
// pub mod balance;
// pub mod delegations;
// pub mod autostake;
