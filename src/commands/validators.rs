use clap::ArgMatches;
use crate::nomic::validators::ValidatorCollection;
use std::error::Error;

pub fn options(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Initialize validator collection and handle the Result
    let validator_collection = match ValidatorCollection::init() {
        Ok(collection) => collection,
        Err(e) => {
            println!("Error initializing validator collection: {}", e);
            return Err(e.into());
        }
    };

    // Determine the output format
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json-pretty");

    // Handle subcommands
    match matches.subcommand() {
        Some(("address", sub_matches)) => handle_address_subcommand(sub_matches, &validator_collection, format),
        Some(("moniker", sub_matches)) => handle_moniker_subcommand(sub_matches, &validator_collection, format),
        Some(("top", sub_matches)) => handle_top_subcommand(sub_matches, &validator_collection, format),
        Some(("bottom", sub_matches)) => handle_bottom_subcommand(sub_matches, &validator_collection, format),
        Some(("skip", sub_matches)) => handle_skip_subcommand(sub_matches, &validator_collection, format),
        Some(("random", sub_matches)) => handle_random_subcommand(sub_matches, &validator_collection, format),
        None => {
            // Handle the case where no subcommand is provided
            validator_collection.print(format);
            Ok(())
        }
        _ => {
            eprintln!("Unrecognized command or missing arguments");
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unrecognized command or missing arguments").into())
        }
    }
}

fn handle_address_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let Some(address) = sub_matches.get_one::<String>("address") {
        if !address.is_empty() {
            let filtered_collection = validator_collection.search_by_address(address);
            if filtered_collection.is_empty() {
                eprintln!("No validators found with the address: {}", address);
            } else {
                filtered_collection.print(format);
            }
        } else {
            eprintln!("Validator address is empty.");
        }
    } else {
        eprintln!("You must supply an address to use the 'validators address' command.");
    }
    Ok(())
}

fn handle_moniker_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let Some(moniker) = sub_matches.get_one::<String>("moniker") {
        if !moniker.is_empty() {
            let result = validator_collection.search_by_moniker(moniker);
            if result.is_empty() {
                eprintln!("No validators found with moniker '{}'", moniker);
            } else {
                result.print(format);
            }
        } else {
            eprintln!("Moniker is empty.");
        }
    } else {
        eprintln!("You must supply a moniker to use the 'validators moniker' command.");
    }
    Ok(())
}

fn handle_top_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let Some(&n) = sub_matches.get_one::<usize>("number") {
        let filtered_collection = validator_collection.top(n);
        filtered_collection.print(format);
    } else {
        eprintln!("You must supply a number to use the 'validators top' command.");
    }
    Ok(())
}

fn handle_bottom_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let Some(&n) = sub_matches.get_one::<usize>("number") {
        let filtered_collection = validator_collection.bottom(n);
        filtered_collection.print(format);
    } else {
        eprintln!("You must supply a number to use the 'validators bottom' command.");
    }
    Ok(())
}

fn handle_skip_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let Some(&n) = sub_matches.get_one::<usize>("number") {
        let filtered_collection = validator_collection.skip(n);
        filtered_collection.print(format);
    } else {
        eprintln!("You must supply a number to use the 'validators skip' command.");
    }
    Ok(())
}

fn handle_random_subcommand(
    sub_matches: &ArgMatches,
    validator_collection: &ValidatorCollection,
    default_format: &str,
) -> Result<(), Box<dyn Error>> {
    // Determine the output format
    let sub_format = sub_matches
        .get_one::<String>("format")
        .map(|s| s.as_str());

    // Use sub_format if it exists; otherwise, use default_format
    let format = sub_format.unwrap_or(default_format);

    if let (Some(&count), Some(&percent)) = (
        sub_matches.get_one::<usize>("count"),
        sub_matches.get_one::<u8>("percent"),
    ) {
        let filtered_collection = validator_collection.random(count, percent);
        filtered_collection.print(format);
    } else {
        eprintln!("You must supply both count and percent to use the 'validators random' command.");
    }
    Ok(())
}

