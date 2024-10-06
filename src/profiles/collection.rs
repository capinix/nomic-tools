use crate::key::{FromHex, FromPath, Privkey};
use crate::nomic::globals::PROFILES_DIR;
use crate::profiles::{OutputFormat, Profile};
use eyre::{eyre, Result};
use fmt::table::{Table, TableBuilder};
use indexmap::IndexMap;
use std::fs;
use std::path::Path;

pub struct ProfileCollection(IndexMap<String, Profile>);

impl std::fmt::Debug for ProfileCollection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ProfileCollection")
			.field("profiles", &self.0.iter().map(|(key, profile)| {
				(profile.key().address(), key, profile.home_path().display().to_string())
			}).collect::<Vec<_>>())
			.finish()
	}
}

impl ProfileCollection {

    /// Loads profiles from the disk into the collection.
    fn load(&mut self) -> Result<()> {
        let profiles_dir = &*PROFILES_DIR;

        // Ensure the profiles directory exists
        if !profiles_dir.exists() {
            return Err(eyre!("Profiles directory does not exist: {:?}", profiles_dir));
        }

        let entries = fs::read_dir(profiles_dir)
            .map_err(|err| eyre!("Failed to read profiles directory: {:?}, error: {}", profiles_dir, err))?;

        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if !file_type.is_dir() {
                    continue;
                }
            }

            let home_path = entry.path();
            let profile_name = home_path.file_name()
                .and_then(|name| name.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "default_profile_name".to_string());

            // Load the profile and add it to the collection
            match Profile::new(&home_path) {
                Ok(profile) => {
                    self.0.insert(profile_name, profile); // Insert into the collection
                }
                Err(e) => {
                    eprintln!("Failed to load profile {}: {}", profile_name, e);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Creates a new ProfileCollection instance by loading profiles from disk.
    pub fn new() -> Result<Self> {
        let mut collection = ProfileCollection(IndexMap::new());
        collection.load()?;
        Ok(collection)
    }

	/// Finds a profile by its name.
	pub fn profile_by_name(&self, name: &str) -> Result<&Profile> {
		self.0.get(name)
			.ok_or_else(|| eyre::eyre!("Profile not found: {}", name))
	}

	/// Finds a profile by its address.
	pub fn profile_by_address(&self, address: &str) -> Result<&Profile> {
		self.0.values()
			.find(|profile| profile.key().address() == address)
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
	pub fn profile_home_path(&self, name_or_address: &str) -> Result<&Path> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		Ok(&profile.home_path())
	}

	/// Retrieves the hex of a profile by its name or address.
	pub fn profile_hex(&self, name_or_address: &str) -> Result<String> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		Ok(profile.key().hex())
	}

	/// Retrieves the address of a profile by its name or address.
	pub fn profile_address(&self, name_or_address: &str) -> Result<&str> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		Ok(&profile.key().address())
	}

	/// Retrieves the config of a profile by its name or address.
	pub fn profile_config(&self, name_or_address: &str) -> Result<String> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		profile.get_config()
	}

	pub fn export_nonce(&self, name_or_address: &str) -> Result<u64> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		profile.export_nonce()
	}

	pub fn import_nonce(&self, name_or_address: &str, value: u64, dont_overwrite: bool) -> Result<()> {
		let profile = self.profile_by_name_or_address(name_or_address)?;
		profile.import_nonce(value, dont_overwrite)
	}

	pub fn import(&mut self, name: &str, key: Privkey) -> Result<()> {
		// Check if the profile name already exists
		if self.0.contains_key(name) {
			return Err(eyre::eyre!("Profile name '{}' already exists.", name));
		}

		// Check for binary diff against existing private keys in all profiles
		for (profile_name, profile) in self.0.iter() {
			// Compare the new key with the existing ones
			if *profile.key() == key {
				return Err(eyre::eyre!(
					"The provided private key is already in use by the profile '{}'.",
					profile_name
				));
			}
		}

		let home_path = Path::new(&*PROFILES_DIR).join(name);
		let key_file = home_path.join(".orga-wallet").join("privkey");

		// Ensure that the parent directory exists before saving the key
		if let Some(parent_dir) = key_file.parent() {
			fs::create_dir_all(parent_dir)?;
		}

		// Save the private key to the file
		key.save(Some(&key_file), None, false)?;

		// Create a new profile instance from the home_path
		let profile = Profile::new(&home_path)?;

		// Insert the profile into the collection
		self.0.insert(name.to_string(), profile);

		Ok(())
	}

	#[allow(dead_code)]
	pub fn import_key(&mut self, name: &str, hex_string: &str) -> Result<()> {
		let key = hex_string.privkey()?;
		self.import(name, key)
	}

	pub fn import_file(&mut self, name: &str, file: &Path) -> Result<()> {
    	let privkey = file.privkey()?;
		self.import(name, privkey)
	}

	/// Converts the ProfileCollection into a JSON serde_json::Value.
	pub fn json(&self) -> serde_json::Value {
		let profiles_json: IndexMap<String, serde_json::Value> = self.0.iter()
			.map(|(key, profile)| {
				(
					key.clone(), // Clone the key
					serde_json::json!({
						"home_path"  : profile.home_path().to_string_lossy(),
						"key_file"   : profile.key_file().to_string_lossy(),
						"nonce_file" : profile.nonce_file().to_string_lossy(),
						"config_file": profile.config_file().to_string_lossy(),
						"account_id" : profile.key().address(),
						"nonce"      : profile.export_nonce().ok(),
					})
				)
			})
			.collect();

		// Return the JSON value of the IndexMap
		serde_json::json!(profiles_json)
	}

	pub fn json_string(&self) -> String {
		serde_json::to_string(&self.json())
			.unwrap_or_else(|_| String::from("{}"))
	}

	pub fn json_string_pretty(&self) -> String {
		serde_json::to_string_pretty(&self.json())
			.unwrap_or_else(|_| String::from("{}"))
	}

	pub fn list(&self) -> Vec<String> {
		self.0.keys().cloned().collect::<Vec<String>>()
	}

	pub fn table(&self) -> Table {
		// Estimate the size and preallocate string
		let mut output = String::new();

		// Construct the header
		output.push_str(&format!("{}\x1C{}", "Account ID", "Profile"));
		output.push('\n');

		// Data rows
		for (basename, profile) in &self.0 {
			// Manually format the profile fields with '\x1C' as the separator
			let formatted_profile = format!(
				"{}\x1C{}",
				profile.key().address(),
				basename
			);

			// Add the formatted profile to output
			output.push_str(&formatted_profile);
			output.push('\n');
		}

		TableBuilder::new(Some(output))
			.set_ifs("\x1C".to_string())
			.set_ofs("  ".to_string())
			.set_header_index(1)
			.set_column_width_limits_index(80)
			.build()
			.clone()
	}

	/// Prints the ProfileCollection as a JSON string.
	/// Prints the ProfileCollection as a pretty-printed JSON string.
	/// Prints the names of each Profile in the ProfileCollection
	pub fn print(&self, format: Option<OutputFormat>) {
		match format {
			Some(OutputFormat::Json)       => println!("{}", self.json_string()),
			Some(OutputFormat::JsonPretty) => println!("{}", self.json_string_pretty()),
			Some(OutputFormat::List)       => println!("{:?}", self.list()),
			Some(OutputFormat::Table)      => self.table().printstd(),
			None => self.table().printstd(),
		}
	}
}
