use clap::Command;

mod validators;
// mod balance;
// mod delegations;
// mod autostake;

pub fn build_cli() -> Command {
    Command::new("nomic-tools")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Tools for working with Nomic")
        .subcommand(validators::cli())   // Validators subcommand
//         .subcommand(balance::cli())      // Balance subcommand
//         .subcommand(delegations::cli())  // Delegations subcommand
//         .subcommand(autostake::cli())    // Autostake subcommand
}
