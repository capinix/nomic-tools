use clap::ValueEnum;
use crate::global::CONFIG;
use eyre::{eyre, Result};
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;

pub fn nomic(
    home: &Path,
    legacy: Option<String>,
    args: Vec<String>
) -> Result<(), eyre::Error> {
    // Create the command based on whether legacy is provided or not
    let mut child = if let Some(legacy_version) = legacy {
        Command::new(CONFIG.nomic()?)
            .env("NOMIC_LEGACY_VERSION", legacy_version)
            .env("HOME", home.as_os_str())
            .args(&args)
            .stderr(Stdio::piped())  // Capture stderr for error handling
            .spawn()?
    } else {
        Command::new("nomic")
            .env("HOME", home.as_os_str())
            .args(&args)
            .stderr(Stdio::piped())  // Capture stderr for error handling
            .spawn()?
    };

    // Capture stderr in a buffer
    let mut stderr = String::new();
    if let Some(ref mut stderr_pipe) = child.stderr {
        let mut reader = BufReader::new(stderr_pipe);
        reader.read_to_string(&mut stderr)?;
    }

    // Wait for the command to finish
    let status = child.wait()?;

    // Handle the command exit status and errors
    if let Some(code) = status.code() {
        if code != 0 {
            return Err(eyre!("Command exited with non-zero status code: {}. Error: {}", code, stderr));
        }
    } else {
        return Err(eyre!("Process terminated by signal. Error: {}", stderr));
    }

    Ok(())  // Return Ok if the command succeeded
}


//fn default_config(profile_name: &str) -> String {
//	format!(
//		"PROFILE={}\n\
//		MINIMUM_BALANCE=10.00\n\
//		MINIMUM_BALANCE_RATIO=0.001\n\
//		MINIMUM_STAKE=5\n\
//		ADJUST_MINIMUM_STAKE=true\n\
//		MINIMUM_STAKE_ROUNDING=5\n\
//		DAILY_REWARD=0.00\n\
//		read VALIDATOR MONIKER <<< \"nomic1jpvav3h0d2uru27fcne3v9k3mrl75l5zzm09uj radicalu\"\n\
//		read VALIDATOR MONIKER <<< \"nomic1stfhcjgl9j7d9wzultku7nwtjd4zv98pqzjmut maximusu\"",
//		profile_name
//	)
//}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
	Json,
	JsonPretty,
	List,
	Table,
}

impl FromStr for OutputFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"json" => Ok(OutputFormat::Json),
			"json-pretty" => Ok(OutputFormat::JsonPretty),
			"list" => Ok(OutputFormat::List),
			"table" => Ok(OutputFormat::Table),
			_ => Err(format!("Invalid output format: {}", s)),
		}
	}
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let output = match self {
			OutputFormat::Json => "json",
			OutputFormat::JsonPretty => "json-pretty",
			OutputFormat::List => "list",
			OutputFormat::Table => "table",
		};
		write!(f, "{}", output)
	}
}

//pub fn nomic(
//	home: &Path, 
//	legacy: Option<String>, 
//	args: Vec<String>
//) -> Result<(), eyre::Error> {
//
//    // Create the command based on whether legacy is provided or not
//    let mut child = if let Some(legacy_version) = legacy {
//        Command::new(CONFIG.nomic()?)
//            .env("NOMIC_LEGACY_VERSION", legacy_version)
//            .env("HOME", home.as_os_str())
//            .args(&args)
//            .spawn()?
//    } else {
//        Command::new("nomic")
//            .env("HOME", home.as_os_str())
//            .args(&args)
//            .spawn()?
//    };
//
//    // Wait for the command to finish
//    let status = child.wait()?;
//
//    // Ensure eyre properly processes any errors before exiting the program
//    if let Some(code) = status.code() {
//        exit(code);  // Exit with the child's exit code
//    } else {
//        return Err(eyre::eyre!("Process terminated by signal"));  // Handle signal-based termination
//    }
//
//    Ok(())  // Return Ok if everything worked fine
//}
