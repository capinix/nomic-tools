use chrono::DateTime;
use chrono::Utc;
use clap::ValueEnum;
use crate::globals::PROFILES_DIR;
use crate::privkey::FromPath;
use crate::profiles::Balance;
use crate::profiles::Delegations;
use crate::profiles::Profile;
use crate::validators::ValidatorCollection;
use eyre::{eyre, Result};
use fmt::table::{Table, TableBuilder};
use once_cell::sync::OnceCell;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use crate::functions::is_valid_nomic_address;

#[derive(Debug)]
pub struct ProfileCollection {
    profiles: Vec<Profile>,
    validators: OnceCell<ValidatorCollection>,
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
}

impl ProfileCollection {

    /// Loads profiles from the disk into the collection.
    pub fn load_profiles(&mut self, load_validators: bool) -> Result<()> {
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
            if entry.file_type()?.is_dir() {
                // Load the profile with or without validators
                let profile = Profile::new(
                    None,                                      // name
                    Some(entry.path()),                        // home
                    if load_validators {                       // validators
                        Some(self.validators()?.clone())
                    } else {
                        None
                    },                                         // validators
                    None,                                      // timestamp
                    Some(true),                                // overwrite
                );

                match profile {
                    Ok(profile) => self.profiles.push(profile),
                    Err(e) => eprintln!("Failed to load profile from {:?}: {}", entry.path(), e),
                }
            }
        }
        Ok(())
    }

    /// Loads profiles from the disk into the collection.
    /// but does not perform a blockchain transaction to initialize validators
    /// This makes a collection that can be used to search without making
    /// a blockchain transaction, unless neccessary.
    pub fn new() -> Result<Self> {
        let mut collection = ProfileCollection {
            profiles: Vec::new(),
            timestamp: Utc::now(), // Initialize the timestamp to the current time
            validators: OnceCell::new(), // Initialize OnceCell
        };
        collection.load_profiles(false)?; // Load profiles from disk (assuming this is defined elsewhere)
        Ok(collection)
    }

    /// Creates a new ProfileCollection instance by loading profiles from disk.
    /// during this initialization, a blockchain transaction is made to initialize validators
    /// that validators is then passed on to all the profiles within so they dont
    /// have to make individual calls
    pub fn load() -> Result<Self> {
        let mut collection = ProfileCollection {
            profiles: Vec::new(),
            timestamp: Utc::now(), // Initialize the timestamp to the current time
            validators: OnceCell::new(), // Initialize OnceCell
        };
        collection.load_profiles(true)?; // Load profiles from disk (assuming this is defined elsewhere)
        Ok(collection)
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
            .find(|profile| {
                profile.key()
                    .and_then(|key| key.address())
                    .map_or(false, |profile_address| profile_address == address)
            })
            .ok_or_else(|| eyre!("Profile with address {} not found", address))
    }

    /// Finds a profile by its home.
    pub fn profile_by_home(&self, home: &Path) -> Result<&Profile> {
        self.profiles
            .iter()
            .find(|profile| {
                // Use Path::eq for comparing paths
                profile.home().eq(home)
            })
            .ok_or_else(|| eyre!("Profile with home {} not found", home.display()))
    }

    /// Finds a profile by its name, address, or home.
    pub fn profile_by_name_or_address_or_home(&self, name_address_home: &str) -> Result<&Profile> {
        // First try to find the profile by name
        if let Ok(profile) = self.profile_by_name(name_address_home) {
            return Ok(profile);
        }

        // Next, try to find the profile by address
        if let Ok(profile) = self.profile_by_address(name_address_home) {
            return Ok(profile);
        }

        // Finally, attempt to find by home; convert the string to a Path
        let home_path = Path::new(name_address_home);
        self.profile_by_home(home_path)
    }

    /// Finds a profile by its name, address, or home.
    pub fn profile_by_name_or_address_or_home_or_default(&self, name_or_address_or_home: Option<&str>) -> Result<Profile> {
        if let Some(search) = name_or_address_or_home {
            self.profile_by_name_or_address_or_home(search).cloned()
        } else {
            let home = home::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;
            Profile::new(None, Some(home), None, None, Some(true))
        }
    }

    /// Retrieves an address from the profile collection based on the provided name, address, or home.
    ///
    /// This function searches the profile collection for the corresponding address based on the 
    /// provided name, Nomic address, or home directory path. If the input is `None`, a new profile 
    /// is created using the home directory, and the address method is called to obtain the address 
    /// from the newly created profile.
    ///
    /// # Parameters
    /// - `name_or_address_or_home`: An optional string slice that may contain a profile name, a 
    ///   Nomic address, or a home directory path. The function will first attempt to find a profile 
    ///   by checking if the input matches a valid Nomic address or a profile name.
    ///
    /// # Returns
    /// - `Ok(String)`: A result containing the address as a string if found or created.
    /// - `Err(eyre::Error)`: An error if the operation fails, such as when the profile collection 
    ///   cannot be accessed or if no valid address can be determined from the input.
    ///
    /// # Examples
    /// ```
    /// // Example with a valid Nomic address
    /// let address = ProfileCollection::search_list()?
    ///     .address(Some("nomic1cujykzh8z0nx6pjp5yvrmv50jes5shh6xjxndk"))?;
    /// ```
    ///
    /// ```
    /// // Example with a valid profile name
    /// let address = ProfileCollection::search_list()?
    ///     .address(Some("my_profile_name"))?;
    /// ```
    ///
    /// ```
    /// // Example with a home directory path
    /// let address = ProfileCollection::search_list()?
    ///     .address(Some("/home/user/my_profile"))?;
    /// ```
    /// 
    /// ```
    /// // Example with no input, creating a new profile
    /// let address = ProfileCollection::search_list()?
    ///     .address(None)?;
    /// ```
    pub fn address(&self, name_or_address_or_home: Option<&str>) -> Result<String, eyre::Error> {
        Ok(
            self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?
                .key()?.address()?.to_string()
        )
    }

    /// Validates a Nomic address or retrieves it from a managed profile.
    ///
    /// This function first checks if the provided input is a valid Nomic address. If it is, 
    /// the address is returned directly. If the input is not a valid address, it is treated 
    /// as a profile name or a home directory path, and the function attempts to retrieve 
    /// the corresponding address from the managed profiles. If no input is provided, it 
    /// creates a new profile using the home directory and retrieves its address.
    ///
    /// # Parameters
    /// - `name_or_address_or_home`: An optional string slice that may contain a valid Nomic address, 
    ///   a profile name, or a home directory path.
    ///
    /// # Returns
    /// - `Ok(String)`: A result containing the validated Nomic address as a string if found or 
    ///   validated.
    /// - `Err(eyre::Error)`: An error if the operation fails, such as when the address is invalid 
    ///   or the profile collection cannot be accessed.
    ///
    /// # Examples
    /// ```
    /// // Example with a valid Nomic address
    /// let address = ProfileCollection::search_list()?
    ///     .validate_address(Some("nomic1cujykzh8z0nx6pjp5yvrmv50jes5shh6xjxndk"))?;
    /// ```
    /// 
    /// ```
    /// // Example with a valid profile name
    /// let address = ProfileCollection::search_list()?
    ///     .validate_address(Some("my_profile_name"))?;
    /// ```
    /// 
    /// ```
    /// // Example with a home directory path
    /// let address = ProfileCollection::search_list()?
    ///     .validate_address(Some("/home/user/my_profile"))?;
    /// ```
    /// 
    /// ```
    /// // Example with no input, creating a new profile
    /// let address = ProfileCollection::search_list()?
    ///     .validate_address(None)?;
    /// ```
    pub fn validate_address(&self, name_or_address_or_home: Option<&str>) -> Result<String, eyre::Error> {
        match name_or_address_or_home {
            Some(search) if is_valid_nomic_address(search) => Ok(search.to_string()),
            Some(search) => self.address(Some(search)),
            None => self.address(None),
        }
    }

    /// Finds a profile by its name or address.
//    pub fn profile_by_name_or_address(&self, name_or_address: &str) -> Result<&Profile> {
//        // First try to find the profile by name
//        if let Ok(profile) = self.profile_by_name(name_or_address) {
//            return Ok(profile);
//        }
//
//        // If not found by name, try to find by address
//        self.profile_by_address(name_or_address)
//    }

    /// Retrieves the home directory path based on the provided name or address.
    ///
    /// This function searches for the home directory associated with a given profile name. If a 
    /// profile name is provided, it retrieves the corresponding home directory from the profile 
    /// collection. If the input is `None`, it attempts to get the current user's home directory.
    ///
    /// # Parameters
    /// - `name_or_address`: An optional string slice that may contain a profile name.
    ///
    /// # Returns
    /// - `Ok(PathBuf)`: A result containing the home directory path as a `PathBuf` if found.
    /// - `Err(eyre::Error)`: An error if the home directory cannot be found or if the profile 
    ///   collection cannot be accessed.
    ///
    /// # Examples
    /// ```
    /// // Example with a valid profile name
    /// let home = ProfileCollection::search_list()?
    ///     .home(Some("my_profile_name"))?;
    /// ```
    /// 
    /// ```
    /// // Example with no input, retrieving the current user's home directory
    /// let home = ProfileCollection::search_list()?
    ///     .home(None)?;
    /// ```
    pub fn home(&self, name_or_address_or_home: Option<&str>) -> Result<PathBuf, eyre::Error> {
        Ok(self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?.home().to_path_buf())
    }

    /// Retrieves the hex of a profile by its name or address.
    pub fn export(&self, name_or_address_or_home: Option<&str>) -> Result<String, eyre::Error> {
        Ok(
            self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?
                .key()?.export()?.to_string()
        )
    }

//    /// verifies the profile name or retrieves it from an address or home directory
//    /// if no name is given return default
//    pub fn name(&self, name_or_address_or_home: Option<&str>) -> Result<String, eyre::Error> {
//        Ok(self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?.name().to_string())
//    }

    /// Retrieves the address of a profile by its name or address.
    pub fn balances(&self, name_or_address_or_home: Option<&str>) -> Result<Balance, eyre::Error> {
        Ok(self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?.balances()?.clone())
    }


    /// Retrieves the delegations for a profile.
    pub fn delegations(&self, name_or_address_or_home: Option<&str>) -> Result<Delegations> {
        Ok(self.profile_by_name_or_address_or_home_or_default(name_or_address_or_home)?.delegations()?.clone())
    }
    pub fn send(&self, source: Option<&str>, destination: Option<&str>, quantity: Option<f64>) -> Result<()> {
        self.profile_by_name_or_address_or_home_or_default(source)?
            .nomic_send(self.validate_address(destination)?, quantity)
    }

    pub fn sort_by_name(&mut self) {
        self.profiles.sort_by(|a, b| a.name().cmp(&b.name()));
    }


    pub fn auto_delegate(&mut self) -> Result<()> {
        self.sort_by_name();
        self.profiles.iter_mut().for_each(|profile| {
            // Call nomic_delegate and ignore any errors
            let _ = profile.nomic_delegate(None, None);
        });
        Ok(())
    }

//    pub fn export_nonce(&self, name_or_address: &str) -> Result<u64> {
//        self.profile_by_name_or_address(name_or_address)?.export_nonce()
//    }

//    pub fn import_nonce(&self,
//        name_or_address: &str,
//        value: u64,
//        dont_overwrite: bool,
//    ) -> Result<()> {
//        self.profile_by_name_or_address(name_or_address)?
//            .import_nonce(value, dont_overwrite)
//    }

    pub fn import(&self,
        name_or_address_or_home: &str,
        hex_str: &str,
        force: bool,
    ) -> Result<()> {
        self.profile_by_name_or_address_or_home(name_or_address_or_home)?
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
                profile.key()?.address()?.to_string(),
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
            let address = profile.key()?.address()?;
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
