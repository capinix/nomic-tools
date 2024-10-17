use chrono::DateTime;
use chrono::Utc;
use clap::ValueEnum;
use crate::globals::PROFILES_DIR;
use crate::key::FromPath;
use crate::profiles::Balance;
use crate::profiles::Delegations;
use crate::profiles::Profile;
use crate::validators::ValidatorCollection;
use eyre::{eyre, Result};
use fmt::table::{Table, TableBuilder};
use once_cell::sync::OnceCell;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct ProfileCollection {
    profiles: Vec<Profile>,
    validators: OnceCell<ValidatorCollection>,
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
}

impl ProfileCollection {
    /// Creates a new ProfileCollection instance by loading profiles from disk.
    pub fn new() -> Result<Self> {
        let mut collection = ProfileCollection {
            profiles: Vec::new(),
            timestamp: Utc::now(), // Initialize the timestamp to the current time
            validators: OnceCell::new(), // Initialize OnceCell
        };
        collection.load()?; // Load profiles from disk (assuming this is defined elsewhere)
        Ok(collection)
    }

    /// Loads profiles from the disk into the collection.
    pub fn load(&mut self) -> Result<()> {
        let profiles_dir = &*PROFILES_DIR;

        // Ensure the profiles directory exists
        if !profiles_dir.exists() {
            return Err(eyre!(
                "Profiles directory does not exist: {:?}",
                profiles_dir
            ));
        }

        let entries = fs::read_dir(profiles_dir).map_err(|err| {
            eyre!(
                "Failed to read profiles directory: {:?}, error: {}",
                profiles_dir,
                err
            )
        })?;

        for entry in entries.flatten() {
            let file_type = entry.file_type().map_err(|err| {
                eyre!(
                    "Failed to get file type for entry: {:?}, error: {}",
                    entry.path(),
                    err
                )
            })?;

            if !file_type.is_dir() {
                continue; // Skip non-directory entries
            }

            // Load the profile using the path from the entry
            match Profile::new(
                None,                              // name
                Some(entry.path()),                // home
                Some(self.validators()?.clone()),  // validators
                None,                              // timestamp
                Some(true),                        // overwrite
            ) {
                Ok(profile) => {
                    self.profiles.push(profile);
                }
                Err(e) => {
                    // Handle the error appropriately, e.g., log it or store it
                    eprintln!("Failed to load profile from {:?}: {}", entry.path(), e);
                }
            }
        }
        Ok(())
    }

    /// Finds a profile by its name.
    pub fn profile_by_name(&self, name: &str) -> Result<&Profile> {
        // Iterate through the vector to find a profile with the given name
        self.profiles
            .iter()
            .find(|profile| profile.name() == name)
            .ok_or_else(|| eyre!("Profile with name {} not found", name))
    }

    /// Finds a profile by its address.
    pub fn profile_by_address(&self, address: &str) -> Result<&Profile> {
        self.profiles
            .iter()
            .find(|profile| profile.address().map_or(false, |profile_address| profile_address == address))
            .ok_or_else(|| eyre!("Profile with address {} not found", address))
    }

    /// Finds a profile by its name or address.
    pub fn profile_by_name_or_address(&self, name_or_address: &str) -> Result<&Profile> {
        // First try to find the profile by name
        if let Ok(profile) = self.profile_by_name(name_or_address) {
            return Ok(profile);
        }

        // If not found by name, try to find by address
        self.profile_by_address(name_or_address)
    }

    /// Retrieves the home path of a profile by its name or address.
    pub fn home(&self, name_or_address: &str) -> Result<&Path> {
        Ok(self.profile_by_name_or_address(name_or_address)?.home())
    }

    /// Retrieves the hex of a profile by its name or address.
    pub fn export(&self, name_or_address: &str) -> Result<&str> {
        self.profile_by_name_or_address(name_or_address)?.export()
    }

    /// Retrieves the address of a profile by its name or address.
    pub fn address(&self, name_or_address: &str) -> Result<&str> {
        self.profile_by_name_or_address(name_or_address)?.address()
    }

    /// Retrieves the address of a profile by its name or address.
    pub fn balances(&self, name_or_address: &str) -> Result<&Balance> {
        self.profile_by_name_or_address(name_or_address)?.balances()
    }

    /// Retrieves the address of a profile by its name or address.
    pub fn delegations(&self, name_or_address: &str) -> Result<&Delegations> {
        self.profile_by_name_or_address(name_or_address)?.delegations()
    }

    pub fn export_nonce(&self, name_or_address: &str) -> Result<u64> {
        self.profile_by_name_or_address(name_or_address)?.export_nonce()
    }

    pub fn import_nonce(&self,
        name_or_address: &str,
        value: u64,
        dont_overwrite: bool,
    ) -> Result<()> {
        self.profile_by_name_or_address(name_or_address)?
            .import_nonce(value, dont_overwrite)
    }

    pub fn import(&self,
        name_or_address: &str,
        hex_str: &str,
        force: bool,
    ) -> Result<()> {
        self.profile_by_name_or_address(name_or_address)?
            .import(hex_str, force)
    }

    pub fn import_file(&self, 
        name: &str, 
        file: &Path
    ) -> Result<()> {
        self.import(name, file.privkey()?.export()?, true)
    }

    /// Retrieves validators, initializing it if necessary.
    /// blockchain operation, cache with oncecell
    pub fn validators(&self) -> eyre::Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    pub fn json(&self) -> Result<serde_json::Value> {
        // Create a mutable vector of profiles to sort
        let mut profiles: Vec<&Profile> = self.profiles.iter().collect();

        // Sort profiles by name
        profiles.sort_by_key(|profile| profile.name());

        // Create a JSON object with address as keys
        let mut profiles_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

        for profile in profiles {
            profiles_json.insert(
                profile.address()?.to_string(),
                serde_json::json!({
                    "home"       : profile.home(),
                    "key_file"   : profile.key_file()?.to_string_lossy(),
                    "nonce_file" : profile.nonce_file()?.to_string_lossy(),
                    "config_file": profile.config_file()?.to_string_lossy(),
                    "nonce"      : profile.export_nonce().ok(),
                }),
            );
        }

        // Convert the Map to serde_json::Value
        Ok(serde_json::Value::Object(profiles_json.into()))
    }

    pub fn json_string(&self) -> Result<String> {
        let json = self.json()?;
        serde_json::to_string(&json)
            .map_err(|e| eyre::eyre!("Failed to convert JSON to string: {}", e))
    }

    pub fn json_string_pretty(&self) -> Result<String> {
        let json = self.json()?;
        serde_json::to_string_pretty(&json)
            .map_err(|e| eyre::eyre!("Failed to convert JSON to string: {}", e))
    }

    pub fn list_addresses(&self) -> Result<Vec<String>> {
        self.profiles
            .iter()
            .map(|profile| {
                profile
                    .key() // This is a Result
                    .and_then(|key| key.address().map(|addr| addr.to_string())) // Use `and_then` to chain results
            })
            .collect::<Result<Vec<String>, _>>() // Collect into a Result<Vec<String>>
    }

    pub fn list_names(&self) -> Result<Vec<String>> {
        self.profiles
            .iter()
            .map(|profile| Ok(profile.name().to_string())) // Convert &str to String and wrap in Ok
            .collect::<Result<Vec<String>, _>>() // Collect into a Result<Vec<String>>
    }

    pub fn table(&self) -> Result<Table> {
        // Estimate the size and preallocate string
        let mut output = String::new();

        // Construct the header
        output.push_str(&format!("{}\x1C{}", "Profile", "Address"));
        output.push('\n');


        // Collect profiles into a vector for sorting
        let mut profiles: Vec<_> = self.profiles.iter().collect();

        // Sort profiles by name
        profiles.sort_by(|a, b| a.name().cmp(&b.name()));

        // Data rows
        for profile in profiles {
            // Use the correct method to get the address and the basename
            let address = profile.address()?;
            let name = profile.name();

            // Manually format the profile fields with '\x1C' as the separator
            let formatted_profile = format!("{}\x1C{}", name, address);

            // Add the formatted profile to output
            output.push_str(&formatted_profile);
            output.push('\n');
        }

        // Build and return the Table
        Ok(TableBuilder::new(Some(output))
            .set_ifs("\x1C".to_string())
            .set_ofs("  ".to_string())
            .set_header_index(1)
            .set_column_width_limits_index(80)
            .build()
            .clone())
    }

    /// Prints the ProfileCollection as a JSON string.
    /// Prints the ProfileCollection as a pretty-printed JSON string.
    /// Prints the names of each Profile in the ProfileCollection
    pub fn print(&self, format: Option<OutputFormat>) -> Result<()> {
        match format {
            Some(OutputFormat::Json) => {
                let json = self.json_string()?;
                println!("{}", json);
            }
            Some(OutputFormat::JsonPretty) => {
                let json = self.json_string_pretty()?;
                println!("{}", json);
            }
            Some(OutputFormat::Names) => {
                let names = self.list_names()?;
                println!("{:?}", names);
            }
            Some(OutputFormat::Addresses) => {
                let names = self.list_addresses()?;
                println!("{:?}", names);
            }
            Some(OutputFormat::Table) => {
                let table = self.table()?;
                table.printstd();
            }
            None => {
                let table = self.table()?;
                table.printstd();
            }
        }
        Ok(()) // Return Ok if all printing was successful
    }
}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Addresses,
    Json,
    JsonPretty,
    Names,
    Table,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "addresses"   => Ok(OutputFormat::Addresses),
            "json"        => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            "names"       => Ok(OutputFormat::Names),
            "table"       => Ok(OutputFormat::Table),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormat::Addresses  => "addresses",
            OutputFormat::Json       => "json",
            OutputFormat::JsonPretty => "json-pretty",
            OutputFormat::Names      => "names",
            OutputFormat::Table      => "table",
        };
        write!(f, "{}", output)
    }
}
