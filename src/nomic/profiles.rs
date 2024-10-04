use crate::nomic::globals::PROFILES_DIR;
use crate::nomic::key;
use crate::nomic::key::Privkey;
use crate::nomic::nonce;
use eyre::{eyre, Result, WrapErr};
use fmt::input::binary_file;
use fmt::table::Table;
use fmt::table::TableBuilder;
use indexmap::IndexMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use clap::{Parser, Subcommand, ValueEnum};

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
	key:		 Privkey,
}

// Custom Debug implementation for Profile
impl std::fmt::Debug for Profile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Profile {{ address: {}, home_path: {:?} }}", self.key.get_address(), self.home_path)
	}
}

//impl Profile {
//	// Custom equality implementation
//	fn eq(&self, other: &Self) -> bool {
//		self.key.get_hex() == other.key.get_hex()
//	}
//}

impl Profile {
	/// Creates a new Profile instance.
	pub fn new(home_path: &Path) -> Result<Self> {

		// Check if the home_path exists and is a directory
		if !home_path.exists() || !home_path.is_dir() {
			return Err(eyre!("Home path '{}' does not exist or is not a directory", home_path.display()));
		}

		// Construct the file paths based on the home_path
		let key_file = home_path.join(".orga-wallet").join("privkey");

//println!("key file:");
		// Check if the key_file exists
		if !key_file.exists() {
			return Err(eyre!("Key file '{}' does not exist", key_file.display()));
		}
//println!("key file:");

//println!("key file: {}", key_file.display());

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
	pub fn key(&self) -> &Privkey {
		&self.key
	}

	pub fn get_hex(&mut self) -> String {
		self.key.get_hex()
	}

	/// Returns the address derived from the private key.
	pub fn get_address(&mut self) -> String {
		self.key.get_address()
	}

	/// Retrieves the nonce from the nonce file.
	pub fn get_nonce(&self) -> Result<u64> {
		nonce::export(Some(self.nonce_file.as_path()), None)
	}

	/// Sets the nonce value in the nonce file.
	pub fn set_nonce(&self, value: u64, dont_overwrite: bool) -> Result<()> {
		nonce::import(value, Some(&self.nonce_file), None, dont_overwrite)
	}

//	/// Returns the hexadecimal representation of the private key.
//	pub fn get_hex(&self) -> Result<String> {
//		Ok(self.key.get_hex())
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

pub struct ProfileCollection(IndexMap<String, Profile>);

impl std::fmt::Debug for ProfileCollection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ProfileCollection")
			.field("profiles", &self.0.iter().map(|(key, profile)| {
				(profile.key.get_address(), key, profile.home_path.display().to_string())
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

	/// Returns the key file path for the given profile.
	pub fn get_key_file(&self, name: &str) -> Result<&PathBuf> {
		let profile = self.get_profile(name)?;
		Ok(profile.key_file())
	}

	/// Returns the nonce file path for the given profile.
	pub fn get_nonce_file(&self, name: &str) -> Result<&PathBuf> {
		let profile = self.get_profile(name)?;
		Ok(profile.nonce_file())
	}

	/// Returns the config file path for the given profile.
	pub fn get_config_file(&self, name: &str) -> Result<&PathBuf> {
		let profile = self.get_profile(name)?;
		Ok(profile.config_file())
	}

	/// Returns the private key (hex) for the given profile.
	pub fn get_key(&self, name: &str) -> Result<&Privkey> {
		let profile = self.get_profile(name)?;
		Ok(profile.key())
	}

	pub fn get_hex(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		Ok(profile.key.get_hex())
	}

	/// Returns the address derived from the private key for the given profile.
	pub fn get_address(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		Ok(profile.key.get_address())
	}

	/// Retrieves the content of the config file for the given profile.
	pub fn get_config(&self, name: &str) -> Result<String> {
		let profile = self.get_profile(name)?;
		profile.get_config()
	}
}

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
	Json,
	JsonPretty,
	List,
	Table,
}

impl FromStr for OutputFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"json" => Ok(OutputFormat::Json),
			"json-pretty" => Ok(OutputFormat::JsonPretty),
			"list" => Ok(OutputFormat::List),
			"table" => Ok(OutputFormat::Table),
			_ => Err(format!("Invalid output format: {}", s)),
		}
	}
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let output = match self {
			OutputFormat::Json => "json",
			OutputFormat::JsonPretty => "json-pretty",
			OutputFormat::List => "list",
			OutputFormat::Table => "table",
		};
		write!(f, "{}", output)
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
					if !profile.config_file.exists() {
						let config_content = default_config(&basename);
						fs::write(&profile.config_file, config_content).map_err(|err| {
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
			if profile.key == key {
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
						"home_path": profile.home_path.to_string_lossy(),
						"key_file": profile.key_file.to_string_lossy(),
						"nonce_file": profile.nonce_file.to_string_lossy(),
						"config_file": profile.config_file.to_string_lossy(),
						"account_id": profile.key.get_address(),
						"nonce": profile.get_nonce().ok(),
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
				profile.key.get_address(),
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

/// Defines the CLI structure for the `profiles` command.
#[derive(Parser)]
#[command(name = "Profiles", about = "Manage & use profiles", visible_alias = "p")]
pub struct Cli {
	/// Specify the output format
	#[arg(long, short)]
	pub format: Option<OutputFormat>,

	/// Subcommands for the nonce command
	#[command(subcommand)]
	pub command: Option<CliCommand>,
}

/// Subcommands for the `profiles` command
#[derive(Subcommand)]
pub enum CliCommand {
    /// Show the AccountId
	#[command(visible_aliases = ["a", "addr"])]
    Address {
        /// Profile
        #[arg()]
        name: String,

    },
    /// Profile configuration
	#[command(visible_aliases = ["c", "conf"])]
    Config {
        /// Profile
        #[arg()]
        name: String,

    },
    /// Import a profile
	#[command(visible_alias = "i")]
    Import {
        /// new profile name
        #[arg()]
        name: String,

        /// hex string or byte array, if neither key, nor file provided, will attempt to read from stdin
        #[arg(conflicts_with = "file")]
        key: Option<String>,

        /// The file path to import from
        #[arg(long, short)]
        file: Option<String>,

    },
    /// Export
	#[command()]
    Export {
        /// Profile
        #[arg()]
        name: String,

    },

}

pub fn run_cli(cli: &Cli) -> Result<()> {
	// Handle subcommands
	match &cli.command {
		// Handle export subcommand
		Some(CliCommand::Export { name }) => {
			let mut collection = ProfileCollection::new()?;
			let output = collection.get_hex(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(CliCommand::Address { name }) => {
			let mut collection = ProfileCollection::new()?;
			let output = collection.get_address(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle config subcommand
		Some(CliCommand::Config { name }) => {
			let mut collection = ProfileCollection::new()?;
			let output = collection.get_config(name)?;
			println!("{}", output);
			Ok(())
		},
		// Handle import subcomman
		Some(CliCommand::Import { name, key, file }) => {
			let mut collection = ProfileCollection::new()?;

			// Handle file import if a file path is provided
			if let Some(file_path) = file {
				// Call import_file with the file path
				collection.import_file(name, Path::new(file_path))?;
				println!("Profile '{}' imported from file: {}", name, file_path);
			} else {
				// Call key_from_input_or_stdin, which reads from stdin if hex_string is None
				let key = key::key_from_input_or_stdin(key.clone())?;

				// Call import with the decoded key
				collection.import(name, key)?;
				println!("Profile '{}' imported.", name);
			}

			Ok(())
		},

		// Default case when no subcommand is provided
		None => {
			let collection = ProfileCollection::new()?;
			collection.print(cli.format.clone());
			Ok(())
		},
	}

}
//	// Handle subcommands
//	match &cli.command {
//
//		// handle address subcommand
//		Some(CliCommand::Address { address, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			if !address.is_empty() {
//				let filtered = collection.search_by_address(address);
//				if filtered.is_empty() {
//					eprintln!("No validators found with the address: {}", address);
//				} else {
//					filtered.print(format, include_details, column_widths.clone());
//				}
//			} else {
//				eprintln!("Validator address is empty.");
//			}
//			Ok(())
//		},
//
//		// handle moniker subcommand
//		Some(CliCommand::Moniker { moniker, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			if !moniker.is_empty() {
//				let filtered = collection.search_by_moniker(moniker);
//				if filtered.is_empty() {
//					eprintln!("No validators found with the moniker: {}", moniker);
//				} else {
//					filtered.print(format, include_details, column_widths.clone());
//				}
//			} else {
//				eprintln!("Validator moniker is empty.");
//			}
//			Ok(())
//		},
//
//		// handle top subcommand
//		Some(CliCommand::Top { number, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			let filtered = collection.top(*number);
//			if filtered.is_empty() {
//				eprintln!("No validators found in the top {}", number);
//			} else {
//				filtered.print(format, include_details, column_widths.clone());
//			}
//			Ok(())
//		},
//
//		// handle bottom subcommand
//		Some(CliCommand::Bottom { number, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			let filtered = collection.bottom(*number);
//			if filtered.is_empty() {
//				eprintln!("No validators found in the bottom {}", number);
//			} else {
//				filtered.print(format, include_details, column_widths.clone());
//			}
//			Ok(())
//		},
//
//		// handle skip subcommand
//		Some(CliCommand::Skip { number, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			let filtered = collection.skip(*number);
//			if filtered.is_empty() {
//				eprintln!("No validators found after skipping {}", number);
//			} else {
//				filtered.print(format, include_details, column_widths.clone());
//			}
//			Ok(())
//		},
//
//		// handle random subcommand
//		Some(CliCommand::Random { count, percent, format, include_details, column_widths }) => {
//
//			let format		  = format.clone().or(cli.format.clone());
//			let include_details = include_details.or(cli.include_details);
//			let column_widths   = column_widths.clone().or(cli.column_widths.clone());
//
//			let filtered = collection.random(*count, *percent);
//			if filtered.is_empty() {
//				eprintln!("No random validators found");
//			} else {
//				filtered.print(format, include_details, column_widths.clone());
//			}
//			Ok(())
//		},
//
//		// Default case when no subcommand is provided
//		None => {
//			collection.print(cli.format.clone());
//			Ok(())
//		},
//	}
//}
