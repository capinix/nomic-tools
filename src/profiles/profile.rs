use crate::{key::Privkey, nonce};
use eyre::{eyre, Result, WrapErr};
use std::{fs, fs::File, io::Read, path::{Path, PathBuf}};
use crate::profiles::default_config;

#[derive(Clone)]
pub struct Profile {
	home_path:   PathBuf,
	key_file:	 PathBuf,
	nonce_file:  PathBuf,
	config_file: PathBuf,
	key:		 Privkey,
}

impl Profile {
	/// Creates a new Profile instance.
	pub fn new(home_path: &Path) -> Result<Self> {

        // Check if the home_path exists and is a directory
        if !home_path.exists() || !home_path.is_dir() {
            return Err(eyre!("Home path '{}' does not exist or is not a directory", home_path.display()));
        }

        // Derive the profile_name from the home_path (using its basename)
        let profile_name = home_path.file_name()
            .ok_or_else(|| eyre::eyre!("Failed to get profile name from home path"))?
            .to_str()
            .ok_or_else(|| eyre::eyre!("Profile name contains invalid UTF-8 characters"))?;

        // Construct the file paths based on the home_path
        let key_file = home_path.join(".orga-wallet").join("privkey");

        // Check if the key_file exists
        if !key_file.exists() {
            return Err(eyre!("Key file '{}' does not exist", key_file.display()));
        }

        let nonce_file = home_path.join(".orga-wallet").join("nonce");
        let config_file = home_path.join("config");

        // Check if the config_file exists, and if not, write the default config to it
        if !config_file.exists() {
            let default_config_content = default_config(profile_name); // Call the default config function
            fs::write(&config_file, default_config_content)
                .map_err(|e| eyre::eyre!("Failed to write default config: {}", e))?;
        }

		let key = Privkey::new_from_file(&key_file)?;

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
    pub fn home_path(&self) -> &Path {
        &self.home_path
    }

    /// Returns the key file path.
    pub fn key_file(&self) -> &Path {
        &self.key_file
    }

    /// Returns the nonce file path.
    pub fn nonce_file(&self) -> &Path {
        &self.nonce_file
    }

    /// Returns the config file path.
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// Returns a reference to the Privkey.
    pub fn key(&self) -> &Privkey {
        &self.key
    }

	/// Reads and returns the content of the config file.
	pub fn get_config(&self) -> Result<String> {
		// Attempt to open the config file
		let mut file = File::open(&self.config_file)
			.with_context(|| format!("Failed to open config file at {:?}", self.config_file))?;

		// Read the file content into a string
		let mut content = String::new();
		file.read_to_string(&mut content)
			.with_context(|| format!("Failed to read config file at {:?}", self.config_file))?;

		Ok(content)
	}

	pub fn export_nonce(&self) -> Result<u64> {
		nonce::export(Some(self.nonce_file()), None)
	}

	pub fn import_nonce(&self, value: u64, dont_overwrite: bool) -> Result<()> {
		nonce::import(value, Some(self.nonce_file()), None, dont_overwrite)
	}

}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = self.key().address();
        write!(f, "Profile {{ address: {}, home_path: {:?} }}", address, self.home_path)
    }
}
