use clap::ArgMatches;
use std::error::Error;
use std::path::Path;
use crate::profiles::ProfilesCollection;

pub fn options(matches: &ArgMatches, profiles: &mut ProfilesCollection) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("import", sub_matches)) => handle_import_subcommand(sub_matches, profiles)?,
        _ => {
            eprintln!("Unrecognized command or missing arguments");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unrecognized command or missing arguments").into());
        }
    }

    Ok(())
}

/// Function to handle the logic of `profiles import` subcommand
pub fn handle_import_subcommand(matches: &ArgMatches, profiles: &mut ProfilesCollection) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        Some(("file", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let privkey_file = sub_matches.get_one::<String>("privkey_file").unwrap();
            let path = Path::new(privkey_file);

            profiles.import_file(name, path)?; // Call the method from ProfilesCollection
            println!("Successfully imported private key from file into profile '{}'", name);
        }
        Some(("hex", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let hex_string = sub_matches.get_one::<String>("hex_string").unwrap();

            profiles.import_hex(name, hex_string)?; // Call the method from ProfilesCollection
            println!("Successfully imported private key from hex into profile '{}'", name);
        }
        _ => {
            eprintln!("Invalid subcommand for 'import'");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid subcommand for 'import'").into());
        }
    }

    Ok(())
}
