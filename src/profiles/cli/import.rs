

use clap::Args;
use crate::profiles::ProfileCollection;
use eyre::Result;
use std::path::Path;

#[derive(Debug, Args)]
#[command(about = "Import Private Key")]
pub struct Command {
    /// Profile
    #[arg(required = true)]
    profile: String,

    /// Hex string or filename containing the key
    #[arg(required_unless_present = "stdin")]
    key_or_file: Option<String>,

    /// Flag to explicitly read from stdin if no other input is provided
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    stdin: bool,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let collection = ProfileCollection::new()?;

        // Determine the input source based on `key_or_file` and `stdin`
        if let Some(ref input) = self.key_or_file {
            if Path::new(input).exists() {
                // If `key_or_file` is a valid file path, treat it as a file
                collection.import_file(&self.profile, Path::new(input))?;
                println!("Profile '{}' imported from file: {}", self.profile, input);
            } else {
                // Otherwise, treat it as a hex key
                collection.import(&self.profile, input, true)?;
                println!("Profile '{}' imported with provided key.", self.profile);
            }
        } else if self.stdin {
            // Read from stdin if `stdin` flag is set
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            collection.import(&self.profile, input.trim(), true)?;
            println!("Profile '{}' imported from stdin.", self.profile);
        } else {
            eprintln!("No key or file provided for import.");
        }

        Ok(())
    }
}
