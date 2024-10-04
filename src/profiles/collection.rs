use crate::nomic::globals::PROFILES_DIR;
use crate::key;
use crate::key::Privkey;
use eyre::{eyre, Result};
use fmt::input::binary_file;
use fmt::table::Table;
use fmt::table::TableBuilder;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use crate::profiles::OutputFormat;
use crate::profiles::Profile;
use crate::profiles::default_config;

pub struct ProfileCollection(IndexMap<String, Profile>);

impl std::fmt::Debug for ProfileCollection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ProfileCollection")
			.field("profiles", &self.0.iter().map(|(key, profile)| {
				(profile.key().get_address(), key, profile.home_path().display().to_string())
			}).collect::<Vec<_>>())
			.finish()
	}
}

impl ProfileCollection {

	/// Retrieves a profile by its name.
	pub fn get_profile(&self, name: &str) -> Result<&Profile> {
		self.0.get(name)
			.ok_or_else(|| eyre::eyre!("Profile not found: {}", name))
	}

	/// Returns the home path for the given profile.
	pub fn get_home_path(&self, name: &str) -> Result<&PathBuf> {
		let profile = self.get_profile(name)?;
		Ok(profile.home_path())
	}

//	/// Returns the key file path for the given profile.
//	pub fn get_key_file(&self, name: &str) -> Result<&PathBuf> {
//		let profile = self.get_profile(name)?;
//		Ok(profile.key_file())
//	}

//	/// Returns the nonce file path for the given profile.
//	pub fn get_nonce_file(&self, name: &str) -> Result<&PathBuf> {
//		let profile = self.get_profile(name)?;
//		Ok(profile.nonce_file())
//	}

//	/// Returns the config file path for the given profile.
//	pub fn get_config_file(&self, name: &str) -> Result<&PathBuf> {
//		let profile = self.get_profile(name)?;
//		Ok(profile.config_file())
//	}

//	/// Returns the private key (hex) for the given profile.
//	pub fn get_key(&self, name: &str) -> Result<&Privkey> {
//		let profile = self.get_profile(name)?;
//		Ok(profile.key())
//	}

	pub fn get_hex(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		Ok(profile.key().get_hex())
	}

	/// Returns the address derived from the private key for the given profile.
	pub fn get_address(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		Ok(profile.key().get_address())
	}

	/// Retrieves the content of the config file for the given profile.
	pub fn get_config(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		profile.get_config()
	}
}

impl ProfileCollection {

	/// Create a new ProfileCollection instance and load existing profiles from disk
	pub fn new() -> Result<Self> {
		// Create a new instance of ProfileCollection with an empty IndexMap
		let mut collection = Self { 0: IndexMap::new() };

		// Reload profiles from disk and update the collection with the result
		collection = collection.reload()?;

		Ok(collection)  // Return the updated instance with loaded profiles

	}

	/// Create a new ProfileCollection instance and load existing profiles from disk
	pub fn reload(&mut self) -> Result<Self> {
		let profiles_dir = &*PROFILES_DIR;

		// Clear the existing profiles before reloading
		self.0.clear();

		let mut profiles = IndexMap::new(); // Create a new collection to store profiles

		// Attempt to read the entries in the profiles directory
		let entries = fs::read_dir(profiles_dir).map_err(|err| {
			eyre!("Failed to read profiles directory: {:?}, error: {}", profiles_dir, err)
		})?;

		for entry in entries.flatten() {
			// Check if the entry is a directory
			if let Ok(file_type) = entry.file_type() {
				if !file_type.is_dir() {
					continue; // Skip non-directory entries
				}
			}

			// Assume home_path is derived from the entry path
			let home_path = entry.path();

			// Attempt to create a Profile; we assume home_path is valid
			match Profile::new(&home_path) {
				Ok(profile) => {
					// Directly extract the basename
					let basename = home_path.file_name()
						.and_then(|name| name.to_str().map(|s| s.to_string()))
						.unwrap_or_else(|| "default_profile_name".to_string());

					// Check if the config file exists; create if it does not
					if !profile.config_file().exists() {
						let config_content = default_config(&basename);
						fs::write(&profile.config_file(), config_content).map_err(|err| {
							eprintln!("Failed to write config file for {}: {}", basename, err);
							err
						})?;
					}

					// Insert the profile into the collection
					profiles.insert(basename, profile);
				}
				Err(e) => {
					// Log or print the error and continue to the next entry
					eprintln!("Failed to create profile: {}", e);
					continue;  // Skips to the next entry in the loop
				}
			}
		}

		// Sort profiles after loading
		profiles.sort_keys();

		// Return a new ProfileCollection instance with the collected profiles
		Ok(ProfileCollection(profiles))
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
//
//	pub fn import_hex(&mut self, name: &str, hex_string: &str) -> Result<()> {
//		// Convert the hex string to a private key
//		let key = key::key_from_hex(hex_string)?;
//		self.import(name, key)
//	}

	pub fn import_file(&mut self, name: &str, file: &Path) -> Result<()> {
		// Read the file contents as bytes using the binary_file function
		let bytes = binary_file(file)?;

		// Convert the bytes to a private key
		let key = key::key_from_bytes(bytes)?;

		// Import the profile using the extracted key
		self.import(name, key)
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
						"account_id" : profile.key().get_address(),
						"nonce"      : profile.get_nonce().ok(),
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
				profile.key().get_address(),
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
