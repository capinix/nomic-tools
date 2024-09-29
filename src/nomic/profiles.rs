
use fmt::table::TableBuilder;
use cosmrs::crypto::secp256k1::SigningKey;
use crate::nomic::globals::PROFILES_DIR;
use crate::nomic::key::{get_key, set_key};
use crate::nomic::nonce::{get_nonce, set_nonce};
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

//#[derive(Debug)]
#[allow(dead_code)]
pub struct Profile {
	home_path:   PathBuf,
	key_file:	PathBuf,
	nonce_file:  PathBuf,
	config_file: PathBuf,
	signing_key: Option<SigningKey>,
	pub account_id:  Option<String>,
	nonce:	   u64,
}

impl Profile {
	pub fn bytes(&self) -> usize {
		let mut size = self.home_path.to_string_lossy().as_bytes().len()
			+ self.key_file.to_string_lossy().as_bytes().len()
			+ self.nonce_file.to_string_lossy().as_bytes().len()
			+ self.config_file.to_string_lossy().as_bytes().len()
			+ std::mem::size_of::<u64>();

		if let Some(account_id) = &self.account_id {
			size += account_id.len();
		}

		size
	}

	pub fn get_nonce(&self) -> u64 {
		// Assuming you have a function to read the nonce value from the file
		get_nonce(&self.nonce_file) // You need to implement this function
	}

	// New method to set the nonce
	pub fn set_nonce(&self, value: u64) -> Result<(), Box<dyn std::error::Error>> {
		// Call the external set_nonce function and pass the nonce_file
		set_nonce(value, &self.nonce_file)
	}

	// Method to get the key from the key_file
	pub fn get_key(&self) -> Result<String, Box<dyn Error>> {
		get_key(&self.key_file)
	}

	// Method to set the key into the key_file
	pub fn set_key(&self, hex_str: &str) -> Result<(), Box<dyn Error>> {
		set_key(&self.key_file, hex_str)
	}

	// Method to display the content of `config_file`
	pub fn get_config(&self) -> Result<String, io::Error> {
		// Open the file
		let mut file = File::open(&self.config_file)?;
		
		// Read the file content into a string
		let mut content = String::new();
		file.read_to_string(&mut content)?;

		// Display the content (or return the content for external handling)
		Ok(content)
	}

}

pub struct ProfileCollection(IndexMap<String, Profile>);

impl ProfileCollection {

	pub fn get_profile(&self, name: &str) -> Option<&Profile> {
		self.0.get(name)
	}

	// Method to get the account_id for a given profile
	pub fn get_account_id(&self, name: &str) -> Option<String> {
		// Retrieve the profile by name
		if let Some(profile) = self.0.get(name) {
			// Return the account_id if it exists
			profile.account_id.clone() // Clone the account_id Option<String>
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the home_path for a given profile
	pub fn get_home_path(&self, name: &str) -> Option<PathBuf> {
		// Retrieve the profile by name
		if let Some(profile) = self.0.get(name) {
			// Return the home_path for the profile
			Some(profile.home_path.clone()) // Clone the home_path
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the config_file path for a given profile
	pub fn get_config_file(&self, name: &str) -> Option<PathBuf> {
		if let Some(profile) = self.0.get(name) {
			Some(profile.config_file.clone()) // Clone the config_file path
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the nonce_file path for a given profile
	pub fn get_nonce_file(&self, name: &str) -> Option<PathBuf> {
		if let Some(profile) = self.0.get(name) {
			Some(profile.nonce_file.clone()) // Clone the nonce_file path
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the key_file path for a given profile
	pub fn get_key_file(&self, name: &str) -> Option<PathBuf> {
		if let Some(profile) = self.0.get(name) {
			Some(profile.key_file.clone()) // Clone the key_file path
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the config content for a given profile
	pub fn get_config(&self, name: &str) -> Option<String> {
		// Retrieve the profile by name
		if let Some(profile) = self.0.get(name) {
			// Try to get the content of the config file
			match profile.get_config() {
				Ok(content) => Some(content),	// If successful, return the config content
				Err(e) => {
					eprintln!("Failed to read config file for profile '{}': {}", name, e);
					None
				}
			}
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to get the key for a given profile (returns the hex string)
	pub fn get_key(&self, name: &str) -> Option<String> {
		if let Some(profile) = self.0.get(name) {
			match profile.get_key() {
				Ok(key) => Some(key),
				Err(e) => {
					eprintln!("Failed to read key for profile '{}': {}", name, e);
					None
				}
			}
		} else {
			eprintln!("Profile not found: {}", name);
			None
		}
	}

	// Method to set the key for a given profile (takes a hex string and writes the binary data)
	pub fn set_key(&self, name: &str, hex_str: &str) -> Result<(), Box<dyn std::error::Error>> {
		if let Some(profile) = self.0.get(name) {
			match profile.set_key(hex_str) {
				Ok(()) => Ok(()),
				Err(e) => {
					eprintln!("Failed to set key for profile '{}': {}", name, e);
					Err(e)
				}
			}
		} else {
			eprintln!("Profile not found: {}", name);
			Err(Box::from("Profile not found"))
		}
	}

}

impl ProfileCollection {
	/// Create a new ProfileCollection instance and load existing profiles from disk
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut collection = Self(IndexMap::new());
		collection.load()?;
		Ok(collection)
	}

	fn load_signing_key(key_file: &Path) -> Option<SigningKey> {
		// Attempt to open the file
		let mut file = match File::open(key_file) {
			Ok(f) => f,
			Err(e) => {
				eprintln!("Error opening file '{}': {}", key_file.display(), e);
				return None;
			}
		};

		// Attempt to read the file's contents
		let mut contents = Vec::new();
		if let Err(e) = file.read_to_end(&mut contents) {
			eprintln!("Error reading file '{}': {}", key_file.display(), e);
			return None;
		}

		// Parse the SigningKey from the contents and wrap it in a Box
		match SigningKey::from_slice(&contents) {
			Ok(key) => Some(key),
			Err(e) => {
				eprintln!("Error parsing private key from '{}': {}", key_file.display(), e);
				None
			},
		}
	}

	fn load(&mut self) -> Result<(), Box<dyn Error>> {
		// Clear existing profiles to avoid stale data
		self.0.clear();

		let profiles_dir = &*PROFILES_DIR;

		if let Ok(entries) = fs::read_dir(profiles_dir) {
			for entry in entries.flatten() {
				let home_path = entry.path();
				if home_path.is_dir() {
					let key_file = home_path.join(".orga-wallet/privkey");
					let nonce_file = home_path.join(".orga-wallet/nonce");
					if key_file.exists() {
						let basename = home_path.file_name().unwrap().to_string_lossy().into_owned();
						let config_file = home_path.join("config");

						// Check if the config file exists; create if it does not
						if !config_file.exists() {
							let config_content = default_config(&basename);
							fs::write(&config_file, config_content)?;
						}

						// Retrieve the nonce value using get_nonce
						let nonce = get_nonce(&nonce_file); // Now returns 0 on error

						// Retrieve the account_id
						let signing_key = Self::load_signing_key(&key_file);

						let account_id: Option<String> = match signing_key {
							Some(ref key) => {
								// Attempt to get the account ID from the public key
								match key.public_key().account_id("nomic") {
									Ok(id) => Some(id.to_string()), // Convert Ok to Some(String)
									Err(err) => {
										eprintln!("Failed to get account ID: {}", err); // Print the error
										None // Return None on error
									}
								}
							}
							None => None, // If signing_key is None, return None
						};

						let profile = Profile {
							home_path,
							key_file,
							nonce_file,
							config_file,
							signing_key,
							account_id,
							nonce,
						};
						self.0.insert(basename, profile);
					}
				}
			}
		}
		// Sort profiles after loading
		self.0.sort_keys();
		Ok(())
	}

	pub fn import_file(&mut self, name: &str, key_file: &Path) -> Result<(), Box<dyn Error>> {
		// Check if the profile name already exists
		if self.0.contains_key(name) {
			return Err(format!("Profile name '{}' already exists.", name).into());
		}

		// Read the contents of the new privkey file
		let mut new_key_content = Vec::new();
		File::open(key_file)?.read_to_end(&mut new_key_content)?;

		// Check for binary diff against existing private keys
		for (profile_name, profile) in self.0.iter() {
			let mut existing_key_content = Vec::new();
			File::open(&profile.key_file)?.read_to_end(&mut existing_key_content)?;

			if new_key_content == existing_key_content {
				return Err(format!("The provided private key is already in use by profile '{}'.", profile_name).into());
			}
		}

		// Create the new profile directory
		let profile_dir = Path::new(&*PROFILES_DIR).join(name);
		fs::create_dir_all(profile_dir.join(".orga-wallet"))?;

		// Copy the privkey file to the new directory
		let dest_key_file = profile_dir.join(".orga-wallet/privkey");
		fs::copy(key_file, &dest_key_file)?;

		// Create a new config file
		let config_content = default_config(name);
		fs::write(profile_dir.join("config"), config_content)?;

		// Load the updated profiles
		self.load()?;

		Ok(())
	}

    pub fn import_hex(&mut self, name: &str, hex_string: &str) -> Result<(), Box<dyn Error>> {
        // Check if the profile name already exists
        if self.0.contains_key(name) {
            println!("Profile name '{}' already exists.", name);
            return Err(format!("Profile name '{}' already exists.", name).into());
        }

        // Decode the hex string into binary data
        let new_key_content = decode(hex_string)
            .map_err(|_| "Invalid hex string. Please provide a valid hexadecimal string.")?;

        // Check for binary diff against existing private keys in all profiles
        for (profile_name, profile) in self.0.iter() {
            let mut existing_key_content = Vec::new();
            File::open(&profile.key_file)?.read_to_end(&mut existing_key_content)?;

            if new_key_content == existing_key_content {
                println!(
                    "The provided private key is already in use by the profile '{}'.",
                    profile_name
                );
                return Err("The provided private key is already in use.".into());
            }
        }

        // Create the new profile directory
        let profile_dir = Path::new(&*PROFILES_DIR).join(name);
        fs::create_dir_all(profile_dir.join(".orga-wallet"))?;

        // Write the decoded binary private key to the new file
        let dest_key_file = profile_dir.join(".orga-wallet/privkey");
        let mut file = File::create(&dest_key_file)?;
        file.write_all(&new_key_content)?;

        // Create a new config file
        let config_content = default_config(name);
        fs::write(profile_dir.join("config"), config_content)?;

        // Load the updated profiles
        self.load()?;

        println!("Profile '{}' successfully imported.", name);
        Ok(())
    }

	pub fn bytes(&self) -> usize {
		// Calculate the total size of all profiles in bytes
		let profile_bytes = self.0.values().map(|p| p.bytes()).sum::<usize>();

		// Estimate additional space for formatting (e.g., newlines, dashes, etc.)
		let formatting_overhead = self.0.len() * 100; // Adjust as needed

		profile_bytes + formatting_overhead
	}
//
//	pub fn indexmap(&self) -> &IndexMap<String, Profile> {
//		&self.0
//	}

	/// Converts the ProfileCollection into a JSON serde_json::Value.
	pub fn json(&self) -> serde_json::Value {
		let profiles_json: IndexMap<String, serde_json::Value> = self.0.iter()
			.map(|(key, profile)| {
				(
					key.clone(), // Clone the key
					serde_json::json!({
						"home_path": profile.home_path.to_string_lossy(),
						"key_file": profile.key_file.to_string_lossy(),
						"nonce_file": profile.nonce_file.to_string_lossy(),
						"config_file": profile.config_file.to_string_lossy(),
						"account_id": profile.account_id,
						"nonce": profile.nonce,
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

	pub fn to_table(&self) -> String {
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

		let formatted_output = TableBuilder::new(&output)
			.ifs("\x1C")
			.ofs("  ")
			.header_row(1)
			.max_text_width(80)
			.format();

		formatted_output
	}
}

impl ProfileCollection {
	pub fn print(&self, format: &str) {
		let output = match format {
			"json" => self.to_json(),
			"json-pretty" => self.to_json_pretty(),
			"list" => self.to_list(),
			"table" => self.to_table(),
			_ => String::new(),
		};
		println!("{}", output);
	}

}
