//use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser; // Import Clap

mod globals;
mod commands;
mod nomic_commands;
mod calc;

use crate::globals::{get_default_config_dir};
use crate::nomic_commands::{validators, run_commands, temp_home, restore_nonce};

#[derive(Parser, Debug)] // Automatically generate argument parser
#[command(
    about = "A program to process Nomic wallet profiles and run Nomic commands.",
    version
)]
struct Cli {
    /// The directory containing profile configurations
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    dir: Option<PathBuf>,

    /// The specific profile name to run within the provided config directory
    #[arg(short = 'p', long = "profile", value_name = "NAME")]
    profile: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse(); // Parse arguments from CLI

    // Ensure profile is only specified if dir is also specified
    if args.profile.is_some() && args.dir.is_none() {
        eprintln!("Error: --profile can only be used with --dir.");
        std::process::exit(1);
    }

    let validator_info = validators()?; // Fetch validator info once

    if args.dir.is_none() && args.profile.is_none() {
        // 1. No arguments: Run commands for the default profile
        let profile_name = "default";
		let config_dir = get_default_config_dir();
		let profile_path = Path::new(&config_dir).join(format!("{}.privkey", profile_name));
		let home_dir = std::env::var("HOME").unwrap_or_else(|_| "HOME not set".to_string());

        run_commands(&home_dir, profile_path.as_path(), &validator_info)?;

    } else if let Some(config_dir) = args.dir {
        if let Some(profile_name) = args.profile {
            // 3. Directory and profile_name argument provided: Run the specified profile in the directory
            let profile_path = config_dir.join(format!("{}.privkey", profile_name));
            if profile_path.exists() {
                let temp_dir = temp_home(&profile_path)?; // Set up temporary home
				let home_dir = temp_dir.path().to_str().ok_or("Failed to convert temp dir path to string")?;

                run_commands(&home_dir, &profile_path, &validator_info)?; // Run commands

                if let Err(err) = restore_nonce(temp_dir.path(), &profile_path) {
                    eprintln!("Error restoring nonce file for {}: {}", profile_path.display(), err);
                }
            } else {
                eprintln!("Profile not found: {}", profile_name);
            }
        } else {
            // 2. Directory argument provided: Run all profiles in the specified directory
            let profiles = fs::read_dir(&config_dir)?;

            for entry in profiles {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("privkey") {
                    let temp_dir = temp_home(&path)?; // Set up temporary home
					let home_dir = temp_dir.path().to_str().ok_or("Failed to convert temp dir path to string")?;
//                     let profile_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

                    run_commands(&home_dir, &path, &validator_info)?; // Run commands

                    if let Err(err) = restore_nonce(temp_dir.path(), &path) {
                        eprintln!("Error restoring nonce file for {}: {}", path.display(), err);
                    }
                }
            }
        }
    }

    Ok(())
}
