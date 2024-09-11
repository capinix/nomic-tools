use clap::ArgMatches;
use crate::nomic::validators::handle_validators_submenu;
use crate::nomic::validators::ValidatorCollection;
use std::error::Error;

pub fn dispatch(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Handle 'validators' subcommand
    if let Some(matches) = matches.subcommand_matches("validators") {
        // Check if the 'search' subcommand is used
        if let Some(sub_matches) = matches.subcommand_matches("search") {
            // Extract the moniker argument for the search
            let moniker = sub_matches
                .get_one::<String>("moniker")
                .map(|s| s.as_str())
                .unwrap_or(""); // Provide a default empty string if moniker is None


            // Initialize validator collection and handle the Result
            let mut validator_collection = ValidatorCollection::new(); // Adjust as necessary
            if let Err(e) = validator_collection.init() {
                eprintln!("Error initializing validator collection: {}", e);
                return Err(e.into());
            }

            // Perform the search and print results
            validator_collection.find_validator_by_moniker(moniker);

        } else {
            // Handle 'validators' without 'search' subcommand
            let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("json-pretty");
            let validator_address = matches.get_one::<String>("validator").map(|s| s.as_str());

            // Handle validators submenu and properly handle the Result
            match handle_validators_submenu(format, validator_address) {
                Ok(_) => (),
                Err(e) => eprintln!("Error handling validators submenu: {}", e),
            }
        }
    } else {
        // Handle cases where 'validators' subcommand is not used
        eprintln!("No 'validators' subcommand found.");
    }

    Ok(())
}
// // Parses the command-line arguments and dispatches the appropriate handlers.
// pub fn dispatch(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
//     if let Some(matches) = matches.subcommand_matches("validators") {
//         if let Some(format) = matches.get_one::<String>("format") {
//             let format = format.as_str();
//             let validator_address = matches.get_one::<String>("validator").map(|s| s.as_str());
// 
//             // Handle validators submenu and properly handle the Result
//             match handle_validators_submenu(format, validator_address) {
//                 Ok(_) => (),
//                 Err(e) => eprintln!("Error handling validators submenu: {}", e),
//             }
// 
//         }
//         
//         if let Some(sub_matches) = matches.subcommand_matches("search") {
//             let moniker = sub_matches.get_one::<String>("moniker").unwrap();
//             let mut validator_collection = ValidatorCollection::new();
// 
//             // Initialize validator collection and properly handle the Result
//             match validator_collection.init() {
//                 Ok(_) => {
//                     validator_collection.find_validator_by_moniker(moniker);
//                 }
//                 Err(e) => eprintln!("Error initializing validator collection: {}", e),
//             }
// 
//         }
//     }
// 
//     Ok(())
// }
