use clap::ArgMatches;
use crate::nomic::nonce::{get_nonce, set_nonce};
use std::error::Error;
use std::path::Path;

pub fn options(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
	match matches.subcommand() {
		Some(("nonce", sub_matches)) => handle_nonce_subcommand(sub_matches)?,
		_ => {
			eprintln!("Unrecognized command or missing arguments");
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unrecognized command or missing arguments").into());
		}
	}

	Ok(())
}

/// Function to handle the logic of `nonce` subcommand
pub fn handle_nonce_subcommand(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
	match matches.subcommand() {
		Some(("get", sub_matches)) => {
			let nonce_file = sub_matches.get_one::<String>("nonce_file").unwrap();
			let path = Path::new(nonce_file);
			let decimal_value = get_nonce(path); // Call your logic to read the nonce
			println!("Nonce: {}", decimal_value);
		}
		Some(("set", sub_matches)) => {
			let decimal_value = sub_matches.get_one::<String>("decimal_value").unwrap();
			let nonce_file = sub_matches.get_one::<String>("nonce_file").unwrap();
			let path = Path::new(nonce_file);
			set_nonce(decimal_value.parse::<u64>()?, path)?; // Call your logic to write the nonce
			println!("Successfully set nonce in file: {}", nonce_file);
		}
		_ => {
			eprintln!("Invalid subcommand for 'nonce'");
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid subcommand for 'nonce'").into());
		}
	}

	Ok(())
}
