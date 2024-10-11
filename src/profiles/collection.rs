use crate::globals::PROFILES_DIR;
//use crate::key::FromHex;
use crate::key::FromPath;
//use crate::key::PrivKey;
use crate::profiles::Profile;
use clap::ValueEnum;
use eyre::{eyre, Result};
use fmt::table::{Table, TableBuilder};
use std::fs;
use std::path::Path;
//use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct ProfileCollection(Vec<Profile>);

impl ProfileCollection {
    /// Creates a new ProfileCollection instance by loading profiles from disk.
    pub fn new() -> Result<Self> {
        let mut collection = ProfileCollection(Vec::new());
        collection.load()?;
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
            match Profile::new(entry.path()) {  // Pass the path here
                Ok(profile) => {
                    self.0.push(profile); // Use push to add to the vector
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
        self.0
            .iter()
            .find(|profile| { profile.name()
                .map_or(false, |n| format!("{}", n) == name)
            })
            .ok_or_else(|| eyre!("Profile with name {} not found", name))
    }

    /// Finds a profile by its address.
    pub fn profile_by_address(&self, address: &str) -> Result<&Profile> {
        self.0
            .iter()
            .find(|profile| { profile.address()
                    .map_or(false, |addr| format!("{}", addr) == address)
            })
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
        self.profile_by_name_or_address(name_or_address)?.home()
    }

    /// Retrieves the hex of a profile by its name or address.
    pub fn export(&self, name_or_address: &str) -> Result<&str> {
        self.profile_by_name_or_address(name_or_address)?.export()
    }

    /// Retrieves the address of a profile by its name or address.
    pub fn address(&self, name_or_address: &str) -> Result<&str> {
        self.profile_by_name_or_address(name_or_address)?.address()
    }

    /// Retrieves the config of a profile by its name or address.
    pub fn config(&self, name_or_address: &str) -> Result<String> {
        self.profile_by_name_or_address(name_or_address)?.config()
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

    pub fn json(&self) -> Result<serde_json::Value> {
        // Create a mutable vector of profiles to sort
        let mut profiles: Vec<&Profile> = self.0.iter().collect();

        // Sort profiles by name
//        profiles.sort_by_key(|profile| profile.name());
        profiles.sort_by_key(|profile| profile.name().unwrap_or_default());

        // Create a JSON object with address as keys
        let mut profiles_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

        for profile in profiles {
            let address = profile.address()?.to_string();

            profiles_json.insert(
                address.clone(), // Use the address as the key
                serde_json::json!({
                    "home"       : profile.home()?.to_string_lossy(),
                    "key_file"   : profile.key_file()?.to_string_lossy(),
                    "nonce_file" : profile.nonce_file()?.to_string_lossy(),
                    "config_file": profile.config_file()?.to_string_lossy(),
                    "account_id" : address, // You can reuse the address here
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
        self.0
            .iter()
            .map(|profile| {
                profile
                    .key() // This is a Result
                    .and_then(|key| key.address().map(|addr| addr.to_string())) // Use `and_then` to chain results
            })
            .collect::<Result<Vec<String>, _>>() // Collect into a Result<Vec<String>>
    }

    pub fn list_names(&self) -> Result<Vec<String>> {
        self.0
            .iter()
            .map(|profile| profile.name().map(|name| name.to_string())) // Map the Result to convert &str to String
            .collect::<Result<Vec<String>, _>>() // Collect into a Result<Vec<String>>
    }

    pub fn table(&self) -> Result<Table> {
        // Estimate the size and preallocate string
        let mut output = String::new();

        // Construct the header
        output.push_str(&format!("{}\x1C{}", "Profile", "Address"));
        output.push('\n');


        // Collect profiles into a vector for sorting
        let mut profiles: Vec<_> = self.0.iter().collect();

        // Sort profiles by name, handling Result properly
        profiles.sort_by(|a, b| {
            let name_a = a.name().as_ref().map(|s| *s).unwrap_or_default();
            let name_b = b.name().as_ref().map(|s| *s).unwrap_or_default();
            name_a.cmp(&name_b)
        });

        // Data rows
        for profile in profiles {
            // Use the correct method to get the address and the basename
            let address = profile.key()?.address()?;
            let name = profile.name()?;

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
