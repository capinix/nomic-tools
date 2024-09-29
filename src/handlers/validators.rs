use std::error::Error;

pub fn options(validators_cli: &ValidatorsCli) -> Result<(), Box<dyn Error>> {
    // Initialize validator collection and handle the Result
    let validator_collection = match ValidatorCollection::init() {
        Ok(collection) => collection,
        Err(e) => {
            println!("Error initializing validator collection: {}", e);
            return Err(e.into());
        }
    };

    // Determine the output format
    let default_format = "json-pretty"; // Default output format

    // Handle subcommands
    match &validators_cli.command {
        // Match each of the subcommands you defined in ValidatorsCli
        ValidatorsCommand::Address { address, format } => {
            handle_address_subcommand(address, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
        ValidatorsCommand::Moniker { moniker, format } => {
            handle_moniker_subcommand(moniker, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
        ValidatorsCommand::Top { number, format } => {
            handle_top_subcommand(*number, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
        ValidatorsCommand::Bottom { number, format } => {
            handle_bottom_subcommand(*number, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
        ValidatorsCommand::Skip { number, format } => {
            handle_skip_subcommand(*number, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
        ValidatorsCommand::Random { count, percent, format } => {
            handle_random_subcommand(*count, *percent, format.as_deref().unwrap_or(default_format), &validator_collection)?;
        },
    }
    Ok(())
}

fn handle_address_subcommand(address: &str, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

fn handle_moniker_subcommand(moniker: &str, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

fn handle_top_subcommand(n: usize, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
    let filtered_collection = validator_collection.top(n);
    filtered_collection.print(format);
    Ok(())
}

fn handle_bottom_subcommand(n: usize, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
    let filtered_collection = validator_collection.bottom(n);
    filtered_collection.print(format);
    Ok(())
}

fn handle_skip_subcommand(n: usize, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
    let filtered_collection = validator_collection.skip(n);
    filtered_collection.print(format);
    Ok(())
}

fn handle_random_subcommand(count: usize, percent: u8, format: &str, validator_collection: &ValidatorCollection) -> Result<(), Box<dyn Error>> {
    let filtered_collection = validator_collection.random(count, percent);
    filtered_collection.print(format);
    Ok(())
}
