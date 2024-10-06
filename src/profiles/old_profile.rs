use crate::key;
use crate::nonce;
use eyre::{eyre, Result, WrapErr};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct Profile {
	home_path:   PathBuf,
	key_file:	 PathBuf,
	nonce_file:  PathBuf,
	config_file: PathBuf,
	key:		 key::Privkey,
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Profile {{ address: {}, home_path: {:?} }}", self.key.get_address(), self.home_path)
	}
}

impl Profile {
	/// Creates a new Profile instance.
	pub fn new(home_path: &Path) -> Result<Self> {

		// Check if the home_path exists and is a directory
		if !home_path.exists() || !home_path.is_dir() {
			return Err(eyre!("Home path '{}' does not exist or is not a directory", home_path.display()));
		}

		// Construct the file paths based on the home_path
		let key_file = home_path.join(".orga-wallet").join("privkey");

		// Check if the key_file exists
		if !key_file.exists() {
			return Err(eyre!("Key file '{}' does not exist", key_file.display()));
		}

		let nonce_file = home_path.join(".orga-wallet").join("nonce");
		let config_file = home_path.join("config");

		// Read the binary file (key_file) and handle potential errors
		let bytes = fmt::input::binary_file(&key_file)
			.with_context(|| format!("Failed to read key file: {:?}", key_file))?;

		// Convert the bytes into a Privkey and handle potential errors
		let key = key::key_from_bytes(bytes)
			.with_context(|| format!("Failed to create key from bytes in {:?}", key_file))?;

		// Return the newly created Profile wrapped in a Result
		Ok(Self {
			home_path: home_path.to_path_buf(),
			key_file,
			nonce_file,
			config_file,
			key,
		})
	}

	/// Returns a reference to the home path.
	pub fn home_path(&self) -> &PathBuf {
		&self.home_path
	}

	/// Returns a reference to the key file path.
	pub fn key_file(&self) -> &PathBuf {
		&self.key_file
	}

	/// Returns a reference to the nonce file path.
	pub fn nonce_file(&self) -> &PathBuf {
		&self.nonce_file
	}

	/// Returns a reference to the config file path.
	pub fn config_file(&self) -> &PathBuf {
		&self.config_file
	}

	/// Returns a reference to the private key.
	pub fn key(&self) -> &key::Privkey {
		&self.key
	}

	/// Retrieves the nonce from the nonce file.
	pub fn get_nonce(&self) -> Result<u64> {
		nonce::export(Some(self.nonce_file.as_path()), None)
	}

//	/// Sets the nonce value in the nonce file.
//	pub fn set_nonce(&self, value: u64, dont_overwrite: bool) -> Result<()> {
//		nonce::import(value, Some(&self.nonce_file), None, dont_overwrite)
//	}

	/// Reads and returns the content of the config file.
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
