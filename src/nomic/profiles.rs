
use fmt::table::Table;
use fmt::table::TableBuilder;
use cosmrs::crypto::secp256k1::SigningKey;
use crate::nomic::globals::PROFILES_DIR;
use crate::nomic::privkey;
use crate::nomic::nonce;
use hex::decode;
use indexmap::IndexMap;
use serde_json;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use anyhow::{Context, Result};
use hex::FromHex;


fn default_config(profile_name: &str) -> String {
	format!(
		"PROFILE={}\n\
		MINIMUM_BALANCE=10.00\n\
		MINIMUM_BALANCE_RATIO=0.001\n\
		MINIMUM_STAKE=5\n\
		ADJUST_MINIMUM_STAKE=true\n\
		MINIMUM_STAKE_ROUNDING=5\n\
		DAILY_REWARD=0.00\n\
		read VALIDATOR MONIKER <<< \"nomic1jpvav3h0d2uru27fcne3v9k3mrl75l5zzm09uj radicalu\"\n\
		read VALIDATOR MONIKER <<< \"nomic1stfhcjgl9j7d9wzultku7nwtjd4zv98pqzjmut maximusu\"",
		profile_name
	)
}

pub struct Profile {
	home_path:   PathBuf,
	key_file:	 PathBuf,
	nonce_file:  PathBuf,
	config_file: PathBuf,
	account_id:  Option<String>,
	nonce:	     Option<u64>,
}

impl Profile {

    pub fn get_nonce(&mut self) -> Result<u64> {

		if let Some(nonce) = self.nonce {
			return Ok(nonce); // Return the existing nonce
		}

		let nonce = nonce::export(&self.nonce_file, None)
			.context("Failed to retrieve nonce from file")?;

		self.nonce = Some(nonce);
		
		Ok(nonce)
    }

    pub fn get_address(&mut self) -> Result<String> {

		if let Some(address) = self.account_id {
			return Ok(address);
		}

		let address = address::export(&self.privkey_file, None)
			.context("Failed to retrieve address from file")?;

		self.account_id = Some(address);
		
		Ok(address)
    }

	pub fn set_nonce(&self, value: u64) -> &mut Self {

		if let Err(e) = nonce::import(value, &self.nonce_file, None) {
			eprintln!("Failed to set nonce: {}", e);
			return self;
		}
		
        self.nonce = Some(value);
		self

	}

	pub fn get_key(&self) -> Result<String> {
		match privkey::export(&self.privkey_file, None) {
			Ok(key) => Ok(key),
			Err(e) => {
				eprintln!("Failed to retrieve private key from file: {}", e);
				Err(anyhow::anyhow!("Failed to retrieve private key from file"))
			}
		}
	}

	pub fn set_key(&self, hex_str: &str) -> Result<()> {
		match privkey::import(hex_str, &self.key_file, None) {
			Ok(()) => Ok(()),
			Err(e) => {
				eprintln!("Failed to save private key to file: {}", e);
				Err(anyhow::anyhow!("Failed to save private key to file"))
			}
		}
	}

    pub fn get_config(&self) -> Result<String> {
        // Attempt to open the config file
        let mut file = File::open(&self.config_file)
            .with_context(|| format!("Failed to open config file at {:?}", self.config_file))?;

        // Read the file content into a string
        let mut content = String::new();
        file.read_to_string(&mut content)
            .with_context(|| format!("Failed to read config file at {:?}", self.config_file))?;

        Ok(content) // Return the content if successful
    }

}

pub struct ProfileCollection(IndexMap<String, Profile>);

impl ProfileCollection {

	pub fn get_profile(&self, name: &str) -> Result<&Profile> {
		self.0.get(name)
			.with_context(|| format!("Profile not found: {}", name))
	}

	pub fn get_addres(&self, name: &str) -> Result<String> {
		match self.0.get(name) {
			Some(profile) => {
				profile.get_address()
			}
			None => {
				eprintln!("Profile not found: {}", name);
				Err(anyhow::anyhow!("Profile not found: {}", name))
			}
		}
	}

	pub fn get_home_path(&self, name: &str) -> Result<PathBuf> {
		// Retrieve the profile by name
		self.0.get(name)
			.map(|profile| profile.home_path.clone())
			.with_context(|| format!("Profile not found: {}", name))
	}

	pub fn get_config_file(&self, name: &str) -> Result<PathBuf> {
		// Retrieve the profile by name and return the config_file path
		self.0.get(name)
			.map(|profile| profile.config_file.clone())
			.with_context(|| format!("Profile not found: {}", name))
	}

	pub fn get_nonce_file(&self, name: &str) -> Result<PathBuf> {
		// Retrieve the profile by name and return the nonce_file path
		self.0.get(name)
			.map(|profile| profile.nonce_file.clone())
			.with_context(|| format!("Profile not found: {}", name))
	}

	pub fn get_key_file(&self, name: &str) -> Result<PathBuf> {
		// Retrieve the profile by name and return the key_file path
		self.0.get(name)
			.map(|profile| profile.key_file.clone())
			.with_context(|| format!("Profile not found: {}", name))
	}

}

impl ProfileCollection {
    /// Create a new ProfileCollection instance and load existing profiles from disk
    pub fn new() -> Result<Self> {
        let mut collection = Self(IndexMap::new());
        collection.load().context("Failed to load profiles from disk")?;
        Ok(collection)
    }

	fn load(&mut self) -> Result<(), Box<dyn Error>> {
		// Clear existing profiles to avoid stale data
		self.0.clear();

		let profiles_dir = &*PROFILES_DIR;

		if let Ok(entries) = fs::read_dir(profiles_dir) {
			for entry in entries.flatten() {
				let home_path = entry.path();
				if home_path.is_dir() {
					// Construct file paths using multiple `.join` calls
					let key_file = home_path.join(".orga-wallet").join("privkey");
					let nonce_file = home_path.join(".orga-wallet").join("nonce");

					// Check if the key file exists
					if key_file.exists() {
						// Safely get the basename
						let basename = home_path.file_name()
							.and_then(|name| name.to_str().map(|s| s.to_string()))
							.unwrap_or_else(|| {
								eprintln!("Failed to get profile name for {:?}", home_path);
								continue; // Skip this profile
							});

						let config_file = home_path.join("config");

						// Check if the config file exists; create if it does not
						if !config_file.exists() {
							let config_content = default_config(&basename);
							fs::write(&config_file, config_content).map_err(|err| {
								eprintln!("Failed to write config file for {}: {}", basename, err);
								err
							})?;
						}

						// Create the profile
						let profile = Profile {
							home_path,
							key_file,
							nonce_file,
							config_file,
							account_id: None,
							nonce: None,
						};

						// Insert the profile into the collection
						self.0.insert(basename, profile);
					}
				}
			}
		} else {
			eprintln!("Failed to read profiles directory: {:?}", profiles_dir);
		}

		// Sort profiles after loading
		self.0.sort_keys();
		Ok(())
	}

	pub fn import_file(&mut self, name: &str, key_file: &Path) -> Result<()> {
		// Check if the profile name already exists
		if self.0.contains_key(name) {
			return Err(anyhow::anyhow!("Profile name '{}' already exists.", name));
		}

		// Read the contents of the new private key file
		let mut new_key_content = Vec::new();
		File::open(key_file)
			.and_then(|mut file| file.read_to_end(&mut new_key_content))
			.context("Failed to read new private key file")?;

		// Check for binary differences against existing private keys
		for (profile_name, profile) in &self.0 {
			let mut existing_key_content = Vec::new();
			File::open(&profile.key_file)
				.and_then(|mut file| file.read_to_end(&mut existing_key_content))
				.context(format!("Failed to read existing key file for profile '{}'", profile_name))?;

			if new_key_content == existing_key_content {
				return Err(anyhow::anyhow!(
					"The provided private key is already in use by profile '{}'.",
					profile_name
				));
			}
		}

		// Create the new profile directory
		let profile_dir = PROFILES_DIR.join(name);
		fs::create_dir_all(profile_dir.join(".orga-wallet"))
			.context("Failed to create profile directory")?;

		// Copy the private key file to the new directory
		let dest_key_file = profile_dir.join(".orga-wallet/privkey");
		fs::copy(key_file, &dest_key_file)
			.context("Failed to copy private key file")?;

		// Create a new config file
		let config_content = default_config(name);
		fs::write(profile_dir.join("config"), config_content)
			.context("Failed to write config file")?;

		// Load the updated profiles
		self.load().context("Failed to load updated profiles")?;

		Ok(())
	}

	pub fn import_hex(&mut self, name: &str, hex_string: &str) -> Result<()> {
		// Check if the profile name already exists
		if self.0.contains_key(name) {
			return Err(anyhow::anyhow!("Profile name '{}' already exists.", name));
		}

		// Decode the hex string into binary data
		let new_key_content = Vec::from_hex(hex_string)
			.context("Invalid hex string. Please provide a valid hexadecimal string.")?;

		// Check for binary diff against existing private keys in all profiles
		for (profile_name, profile) in self.0.iter() {
			let mut existing_key_content = Vec::new();
			File::open(&profile.key_file)
				.and_then(|mut file| file.read_to_end(&mut existing_key_content))
				.context(format!("Failed to read key file for profile '{}'", profile_name))?;

			if new_key_content == existing_key_content {
				return Err(anyhow::anyhow!(
					"The provided private key is already in use by the profile '{}'.",
					profile_name
				));
			}
		}

		// Create the new profile directory
		let profile_dir = Path::new(&*PROFILES_DIR).join(name);
		fs::create_dir_all(profile_dir.join(".orga-wallet"))
			.context("Failed to create profile directory")?;

		// Write the decoded binary private key to the new file
		let dest_key_file = profile_dir.join(".orga-wallet/privkey");
		let mut file = File::create(&dest_key_file)
			.context("Failed to create private key file")?;
		file.write_all(&new_key_content)
			.context("Failed to write to private key file")?;

		// Create a new config file
		let config_content = default_config(name);
		fs::write(profile_dir.join("config"), config_content)
			.context("Failed to write config file")?;

		// Load the updated profiles
		self.load().context("Failed to load updated profiles")?;

		println!("Profile '{}' successfully imported.", name);
		Ok(())
	}

	/// Converts the ProfileCollection into a JSON serde_json::Value.
	pub fn to_json(&self) -> serde_json::Value {
		let profiles_json: IndexMap<String, serde_json::Value> = self.0.iter()
			.map(|(key, profile)| {
				(
					key.clone(), // Clone the key
					serde_json::json!({
						"home_path": profile.home_path.to_string_lossy(),
						"key_file": profile.key_file.to_string_lossy(),
						"nonce_file": profile.nonce_file.to_string_lossy(),
						"config_file": profile.config_file.to_string_lossy(),
						"account_id": profile.get_address().ok(),  // unwrap the option and load if not loaded
						"nonce": profile.get_nonce().ok(),         // unwrap the option and load if not loaded
					})
				)
			})
			.collect();

		// Return the JSON value of the IndexMap
		serde_json::json!(profiles_json)
	}

	/// Converts the ProfileCollection into a pretty-printed JSON string.
	pub fn to_json(&self) -> String {
		let json_value = self.json(); // Get the JSON value from to_json
		serde_json::to_string(&json_value)
			.unwrap_or_else(|_| String::from("{}")) // Fallback to "{}" if serialization fails
	}

	/// Converts the ProfileCollection into a pretty-printed JSON string.
	pub fn to_json_pretty(&self) -> String {
		let json_value = self.json(); // Get the JSON value from to_json
		serde_json::to_string_pretty(&json_value)
			.unwrap_or_else(|_| String::from("{}")) // Fallback to "{}" if serialization fails
	}

	/// Converts the keys of the ProfileCollection to a newline-separated string.
	pub fn to_list(&self) -> String {
		self.0.keys().cloned().collect::<Vec<String>>().join("\n")
	}

	pub fn to_table(&self) -> Table {
		// Estimate the size and preallocate string
		let mut output = String::with_capacity(self.bytes());

		// Construct the header
		output.push_str(&format!("{}\x1C{}", "Account ID", "Profile"));
		output.push('\n');

		// Data rows
		for (basename, profile) in &self.0 {
			// Manually format the profile fields with '\x1C' as the separator
			let formatted_profile = format!(
				"{}\x1C{}",
				profile.account_id.clone().unwrap_or("".to_string()),
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
}

impl ProfileCollection {
	pub fn print(&self, format: &str) {
		match format {
			"json" => println!("{}", self.to_json()),
			"json-pretty" => println!("{}", self.to_json_pretty()),
			"list" => println!("{}", self.to_list()),
			"table" => self.to_table().printstd(),
			_ => (),
		};
	}

}
