use std::path::Path;
use clap::ArgMatches;
use std::error::Error;
use crate::profiles::ProfileCollection;

pub fn options(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut profiles = ProfileCollection::new().map_err(|e| {
        eprintln!("Error initializing profiles collection: {}", e);
        e
    })?;

    match matches.subcommand() {
        Some(("address", sub_matches)) => handle_address_subcommand(sub_matches, &profiles)?,
        Some(("config", sub_matches)) => {
            // Check if the file subcommand was called
            if let Some(config_file_matches) = sub_matches.subcommand_matches("file") {
                handle_config_file_subcommand(config_file_matches, &profiles)?;
            } else {
                handle_config_subcommand(sub_matches, &profiles)?;
            }
        }
        Some(("home_path", sub_matches)) => handle_home_path_subcommand(sub_matches, &profiles)?,
        Some(("import", sub_matches)) => handle_import_subcommand(sub_matches, &mut profiles)?,
        Some(("key", sub_matches)) => handle_key_subcommand(sub_matches, &profiles)?,
        Some(("nonce", sub_matches)) => handle_nonce_subcommand(sub_matches, &profiles)?,
        Some(("ls", _)) => handle_ls_subcommand(&profiles)?,  // Handle listing profiles
        _ => {
            eprintln!("Unrecognized command or missing arguments");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unrecognized command or missing arguments").into());
        }
    }

    Ok(())
}








/// Function to handle the logic of the `import` subcommand
pub fn handle_import_subcommand(profiles: &mut ProfilesCollection, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("file", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let privkey_file = sub_matches.get_one::<String>("privkey_file").unwrap();
            let path = Path::new(privkey_file);

            profiles.import_file(name, path)?; // Call your logic to import from file
            println!("Successfully imported private key from file: {}", privkey_file);
        }
        Some(("hex", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let hex_string = sub_matches.get_one::<String>("hex_string").unwrap();

            profiles.import_hex(name, hex_string)?; // Call your logic to import from hex
            println!("Successfully imported private key from hex string for profile: {}", name);
        }
        _ => {
            eprintln!("Invalid subcommand for 'import'");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid subcommand for 'import'").into());
        }
    }

    Ok(())
}
