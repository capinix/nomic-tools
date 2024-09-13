mod cli;
mod commands;
mod nomic;
mod globals;

// use clap::ArgMatches;
use cli::build_cli;
use commands::validators;
use std::error::Error;

// nomic-tools validators [-f|--fomat [raw|table|json|json_pretty] [-t|--top|-b|--bottom N]
// nomic-tools validators address <address> [-f|--fomat [raw|table|json|json_pretty]
// nomic-tools validators moniker <moniker> [-f|--fomat [raw|table|json|json_pretty]






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
// 			Some(("autostake", sub_m)) => autostake::options(sub_m),
// 			Some(("balance", sub_m)) => balance::options(sub_m),
// 			Some(("delegations", sub_m)) => delegations::options(sub_m),
// 			Some(("journalctl", sub_m)) => journalctl::options(sub_m),
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
