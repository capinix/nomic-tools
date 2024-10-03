/// This module contains the core logic for handling operations defined by the CLI commands.
///
/// Each handler in the `handlers` module delegates the actual work to functions in this module.
/// It performs the primary functionality required for each command, such as conversions and validations.
/// 
/// To add new functionality, define it here and ensure that corresponding handler functions in the
/// `handlers` module call these functions as needed.
///
/// To implement the logic for a new command:
/// 
/// 1. **Create the Command Logic File:**
///    - Implement the functionality in `src/nomic/new_command.rs`.
/// 
/// 2. **Declare the Module:**
///    - Ensure the module is declared in `src/nomic/mod.rs` if needed.
// pub mod balance;
// pub mod calc;
// pub mod delegations;
// pub mod func;
// pub mod ftable;
//pub mod convert;
pub mod globals;
pub mod key;
pub mod nonce;
//pub mod profiles;
pub mod validators;
