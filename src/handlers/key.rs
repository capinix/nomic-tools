use clap::ArgMatches;
use crate::nomic::key::{get_key, set_key};
use std::error::Error;
use std::path::Path;

pub fn options(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
	match matches.subcommand() {
		Some(("keys", sub_matches)) => handle_keys_subcommand(sub_matches)?,
		_ => {
			eprintln!("Unrecognized command or missing arguments");
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unrecognized command or missing arguments").into());
		}
	}

	Ok(())
}

/// Function to handle the logic of `keys` subcommand
pub fn handle_keys_subcommand(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
	match matches.subcommand() {
		Some(("get", sub_matches)) => {
			let privkey_file = sub_matches.get_one::<String>("privkey_file").unwrap();
			let path = Path::new(privkey_file);
			let hex_str = get_key(path)?; // Call your logic to read the private key
			println!("Hex key: {}", hex_str);
		}
		Some(("set", sub_matches)) => {
			let hex_string = sub_matches.get_one::<String>("hex_string").unwrap();
			let privkey_file = sub_matches.get_one::<String>("privkey_file").unwrap();
			let path = Path::new(privkey_file);
			set_key(path, hex_string)?; // Call your logic to write the hex string to the file
			println!("Successfully wrote private key to file: {}", privkey_file);
		}
		_ => {
			eprintln!("Invalid subcommand for 'keys'");
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid subcommand for 'keys'").into());
		}
	}

	Ok(())
}
