use clap::{
	Parser,
	Subcommand,
	ValueEnum,
};
use crate::nomic::globals::{
	NOMIC,
	NOMIC_LEGACY_VERSION,
};
use fmt::table::{
	Table,
	TableBuilder,
};
use indexmap::IndexMap;
use rand::{
	prelude::IteratorRandom,
	thread_rng,
};
use serde_json;
use serde::Serialize;
use std::{ 
	error::Error, 
	iter::FromIterator, 
	mem::size_of, 
	process::Command,
	str::FromStr,
};

/// Enum to represent output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
	Json,
	JsonPretty,
	Raw,
	Table,
	Tuple,
}

impl FromStr for OutputFormat {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"json" => Ok(OutputFormat::Json),
			"json-pretty" => Ok(OutputFormat::JsonPretty),
			"raw" => Ok(OutputFormat::Raw),
			"table" => Ok(OutputFormat::Table),
			"tuple" => Ok(OutputFormat::Tuple),
			_ => Err(format!("Invalid output format: {}", s)),
		}
	}
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let output = match self {
			OutputFormat::Json => "json",
			OutputFormat::JsonPretty => "json-pretty",
			OutputFormat::Raw => "raw",
			OutputFormat::Table => "table",
			OutputFormat::Tuple => "tuple",
		};
		write!(f, "{}", output)
	}
}

#[derive(Debug, Serialize, Clone)]
pub struct Validator {
	rank: usize,
	address: String,
	voting_power: usize,
	moniker: String,
	details: String,
}

impl Validator {

	pub fn new(rank: usize, address: String, voting_power: usize, moniker: String, details: String) -> Self {
		Self {
			rank,
			address,
			voting_power,
			moniker,
			details,
		}
	}

	pub fn voting_power_nom(&self) -> String {
		// Converts voting power to NOM (e.g., from uNOM to NOM)
		format!("{:.2}", self.voting_power as f64 / 1_000_000.0)
	}

	pub fn bytes(&self) -> usize {
		// Calculate size in bytes of the Validator struct, including heap-allocated data
		size_of::<Self>()
		+ self.address.as_bytes().len()  // Byte length of the address string
		+ self.moniker.as_bytes().len()  // Byte length of the moniker string
		+ self.details.as_bytes().len()  // Byte length of the details string
	}
}

#[derive(Debug)] 
pub struct ValidatorCollection(Vec<Validator>);

impl ValidatorCollection {

	/// Creates a new, empty `ValidatorCollection`.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// assert!(collection.is_empty());
	/// ```
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Creates a `ValidatorCollection` from a `Vec<Validator>`.
	///
	/// # Arguments
	///
	/// * `validators` - A vector of `Validator` instances.
	///
	/// # Example
	///
	/// ```
	/// let validators = vec![Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string())];
	/// let collection = ValidatorCollection::from_vec(validators);
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn from_vec(validators: Vec<Validator>) -> Self {
		Self(validators)
	}

	/// Creates a `ValidatorCollection` from an iterator of `Validator` instances.
	///
	/// # Arguments
	///
	/// * `iter` - An iterator over `Validator` instances.
	///
	/// # Example
	///
	/// ```
	/// let validators = vec![Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string())];
	/// let collection = ValidatorCollection::from_iter(validators.into_iter());
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = Validator>,
	{
		Self(iter.into_iter().collect())
	}

	/// Adds a `Validator` to the collection.
	///
	/// # Arguments
	///
	/// * `validator` - A `Validator` instance to add.
	///
	/// # Example
	///
	/// ```
	/// let mut collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn insert(&mut self, validator: Validator) {
		self.0.push(validator);
	}

	/// Returns the number of validators in the collection.
	///
	/// # Example
	///
	/// ```
	/// let mut collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Checks if the collection is empty.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// assert!(collection.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns a mutable reference to the vector of validators.
	///
	/// # Example
	///
	/// ```
	/// let mut collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let validators = collection.validators_mut();
	/// assert_eq!(validators.len(), 1);
	/// ```
	pub fn validators_mut(&mut self) -> &mut Vec<Validator> {
		&mut self.0
	}

	/// Fetches the output of the `nomic validators` command.
	///
	/// # Errors
	///
	/// Returns an error if the command fails or its output cannot be processed.
	///
	/// # Example
	///
	/// ```
	/// let output = ValidatorCollection::nomic_validators().unwrap();
	/// assert!(output.contains("validators"));
	/// ```
	fn nomic_validators() -> Result<String, Box<dyn Error>> {
		// Create and configure the Command
		let mut cmd = Command::new(&*NOMIC);
		cmd.arg("validators");

		// Set environment variables
		cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

		// Execute the command and collect the output
		let output = cmd.output()?;

		// Check if the command was successful
		if !output.status.success() {
			let error_msg = format!(
				"Command `{}` failed with output: {:?}",
				&*NOMIC,
				String::from_utf8_lossy(&output.stderr) // Use stderr for error output
			);
			return Err(error_msg.into());
		}

		// Convert the command output to a string
		let output_str = String::from_utf8(output.stdout)?;

		Ok(output_str)
	}

	/// Parses the output of the `nomic validators` command into the `ValidatorCollection`.
	///
	/// # Arguments
	///
	/// * `input` - A string containing the command output.
	///
	/// # Errors
	///
	/// Returns an error if the parsing fails.
	///
	/// # Example
	///
	/// ```
	/// let output = "some output from command".to_string();
	/// let mut collection = ValidatorCollection::new();
	/// collection.parse_nomic_validators(output).unwrap();
	/// ```
	pub fn parse_nomic_validators(&mut self, input: String) -> Result<(), Box<dyn Error>> {
		let lines: Vec<&str> = input.lines().collect();
		let mut rank = 1; // Start rank from 1

		for chunk in lines.chunks(4) {
			if chunk.len() == 4 {
				let address = chunk[0].trim().trim_start_matches('-').trim().to_string();
				let voting_power_str = chunk[1].split(':').nth(1).unwrap_or("").trim().to_string();
				let voting_power = voting_power_str.parse::<usize>().unwrap_or(0);
				let moniker = chunk[2].split(':').nth(1).unwrap_or("").trim().to_string();
				let details = chunk[3].split(':').nth(1).unwrap_or("").trim().to_string();

				let validator = Validator::new(rank, address, voting_power, moniker, details);
				self.insert(validator); // Use `insert` to add validators

				rank += 1;
			}
		}
		Ok(())
	}

	/// Creates a `ValidatorCollection` from a given command output string.
	///
	/// # Arguments
	///
	/// * `command_output` - A string containing the command output.
	///
	/// # Errors
	///
	/// Returns an error if the parsing fails.
	///
	/// # Example
	///
	/// ```
	/// let output = "some output from command".to_string();
	/// let collection = ValidatorCollection::load_from_string(output).unwrap();
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn load_from_string(command_output: String) -> Result<Self, Box<dyn Error>> {
		let mut collection = Self::new();
		collection.parse_nomic_validators(command_output)?;
		Ok(collection)
	}

	/// Creates a `ValidatorCollection` and populates it from the output of the `nomic validators` command.
	///
	/// # Errors
	///
	/// Returns an error if the command fails or the parsing fails.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::load_from_command().unwrap();
	/// assert_eq!(collection.len(), 1);
	/// ```
	pub fn load_from_command() -> Result<Self, Box<dyn Error>> {
		let output = Self::nomic_validators()?;
		Self::load_from_string(output)
	}

	/// Alias for `load_from_command`.
	///
	/// # Errors
	///
	/// Returns an error if the command fails or the parsing fails.
	pub fn init() -> Result<Self, Box<dyn Error>> {
		Self::load_from_command()
	}

	/// Estimates the byte size of the `ValidatorCollection`.
	///
	/// # Example
	///
	/// ```
	/// let mut collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let size = collection.bytes();
	/// assert!(size > 0);
	/// ```
	pub fn bytes(&self) -> usize {
		// Calculate the total size of all validators in bytes
		let validators_bytes = self.0.iter().map(|v| v.bytes()).sum::<usize>();

		// Estimate additional space for formatting (e.g., newlines, dashes, etc.)
		let formatting_overhead = self.len() * 100; // Adjust 100 as needed

		validators_bytes + formatting_overhead
	}

	/// Finds a validator by address.
	///
	/// # Arguments
	///
	/// * `address` - The address of the validator to find.
	///
	/// # Returns
	///
	/// An `Option` containing the `Validator` if found, otherwise `None`.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let found_validator = collection.get_validator("address").unwrap();
	/// assert_eq!(found_validator.address, "address");
	/// ```
	pub fn get_validator(&self, address: &str) -> Option<&Validator> {
		self.0.iter().find(|v| v.address == address)
	}

	/// Searches for validators by address and returns a new `ValidatorCollection` with matching validators.
	///
	/// # Arguments
	///
	/// * `address` - The address to search for.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` containing validators that match the address.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let filtered = collection.search_by_address("address");
	/// assert_eq!(filtered.len(), 1);
	/// ```
	pub fn search_by_address(&self, search: &str) -> ValidatorCollection {
		self.0.iter()
			.filter(|validator| validator.address == search)
			.cloned()
			.collect()
	}

	/// Searches for validators by a substring of the moniker and returns a new `ValidatorCollection` with matching validators.
	///
	/// # Arguments
	///
	/// * `moniker` - The substring to search for in the moniker.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` containing validators with monikers that match the substring.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let filtered = collection.search_by_moniker("moniker");
	/// assert_eq!(filtered.len(), 1);
	/// ```
	pub fn search_by_moniker(&self, search: &str) -> ValidatorCollection {
		// Convert search term to lowercase for case-insensitive matching
		let search_lower = search.to_lowercase();

		self.0.iter()
			.filter(|validator| validator.moniker.to_lowercase().contains(&search_lower))
			.cloned()
			.collect()
	}

	/// Returns the top `n` validators by voting power.
	///
	/// # Arguments
	///
	/// * `n` - The number of top validators to return.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` containing the top `n` validators by voting power.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator1 = Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string());
	/// let validator2 = Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string());
	/// collection.insert(validator1);
	/// collection.insert(validator2);
	/// let top_validators = collection.top(1);
	/// assert_eq!(top_validators.len(), 1);
	/// assert_eq!(top_validators.0[0].address, "address2");
	/// ```
	pub fn top(&self, n: usize) -> ValidatorCollection {
		// Clone the current ValidatorCollection to work on a separate copy
		let mut top_validators = self.clone();
		
		// Sort the validators by voting power in descending order
		top_validators.0.sort_by_key(|v| std::cmp::Reverse(v.voting_power));
		
		// Truncate to keep only the top `n` validators
		top_validators.0.truncate(n);

		// Return the modified ValidatorCollection
		top_validators
	}

	/// Returns the bottom `n` validators by voting power.
	///
	/// # Arguments
	///
	/// * `n` - The number of bottom validators to return.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` containing the bottom `n` validators by voting power.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator1 = Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string());
	/// let validator2 = Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string());
	/// collection.insert(validator1);
	/// collection.insert(validator2);
	/// let bottom_validators = collection.bottom(1);
	/// assert_eq!(bottom_validators.len(), 1);
	/// assert_eq!(bottom_validators.0[0].address, "address1");
	/// ```
	pub fn bottom(&self, n: usize) -> ValidatorCollection {
		// Clone the current ValidatorCollection to work on a separate copy
		let mut bottom_validators = self.clone();

		// Sort the validators by voting power in ascending order
		bottom_validators.0.sort_by_key(|v| v.voting_power);

		// Truncate to keep only the bottom `n` validators
		bottom_validators.0.truncate(n);

		// Return the modified ValidatorCollection
		bottom_validators
	}

	/// Returns a `ValidatorCollection` with the top `n` validators removed.
	///
	/// # Arguments
	///
	/// * `n` - The number of top validators to skip.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` with the top `n` validators removed.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator1 = Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string());
	/// let validator2 = Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string());
	/// collection.insert(validator1);
	/// collection.insert(validator2);
	/// let reduced_collection = collection.skip(1);
	/// assert_eq!(reduced_collection.len(), 1);
	/// assert_eq!(reduced_collection.0[0].address, "address1");
	/// ```
	pub fn skip(&self, n: usize) -> ValidatorCollection {
		// Clone the current ValidatorCollection to work on a separate copy
		let mut filtered_validators = self.clone();

		// Sort the validators by voting power in descending order
		filtered_validators.0.sort_by_key(|v| std::cmp::Reverse(v.voting_power));

		// Skip the top `n` validators
		filtered_validators.0.drain(0..n); // Remove the top `n` validators

		// Return the modified ValidatorCollection
		filtered_validators
	}

	/// Returns a random subset of validators, excluding the top `y` percent by voting power.
	///
	/// # Arguments
	///
	/// * `y` - The percentage of top validators to exclude from the random selection.
	///
	/// # Returns
	///
	/// A new `ValidatorCollection` containing a random subset of validators.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator1 = Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string());
	/// let validator2 = Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string());
	/// collection.insert(validator1);
	/// collection.insert(validator2);
	/// let random_validators = collection.random(50);
	/// assert_eq!(random_validators.len(), 1);
	/// ```
	pub fn random(&self, count: usize, percent: u8) -> ValidatorCollection {
		let mut validators = self.clone();

		let total_count = validators.len();
		if total_count == 0 || percent >= 100 {
			return ValidatorCollection::new(); // Return an empty ValidatorCollection if no validators or invalid percent
		}

		// Sort validators by `voting_power` in descending order
		validators.0.sort_by(|a, b| b.voting_power.cmp(&a.voting_power));

		// Calculate the index that represents the cutoff for the top `y` percent
		let cutoff_index = ((total_count as f64) * (percent as f64 / 100.0)).round() as usize;

		// Randomly select `count` validators from the validators outside the top `y` percent
		let mut rng = thread_rng();
		let selected: Vec<Validator> = validators
			.0 // Access the inner Vec<Validator>
			.into_iter() // Convert Vec<Validator> to an iterator
			.skip(cutoff_index) // Skip the first `cutoff_index` elements
			.choose_multiple(&mut rng, count) // Choose `count` elements randomly
			.into_iter() // Convert the chosen elements into an iterator
			.collect(); // Collect the chosen elements into a Vec<Validator>

		ValidatorCollection(selected) // Construct the ValidatorCollection from the selected validators
	}

	/// Returns a raw string representation of the validators.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let raw_string = collection.raw();
	/// assert!(raw_string.contains("address"));
	/// ```
	pub fn raw(&self, include_details: Option<bool>) -> String {
		let mut output = String::with_capacity(self.bytes());

		for validator in &self.0 {
			// Append formatted data to the output string
			output.push_str(&format!("- {}\n", validator.address));
			output.push_str(&format!("	VOTING POWER: {}\n", validator.voting_power));
			output.push_str(&format!("	MONIKER: {}\n", validator.moniker));
			
			// Include details if specified
			if include_details.unwrap_or(true) {
				output.push_str(&format!("	DETAILS: {}\n", validator.details));
			}
		}

		output.trim_end().to_string() // Optionally trim the trailing newline
	}

	/// Returns a formatted table representation of the validators.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let table = collection.table(None, None); // Include None for default behavior
	/// assert!(table.contains("Rank"));
	/// ```
	pub fn table(&self, include_details: Option<bool>, column_widths: Option<Vec<usize>>) -> Table {
		// Initialize the output string
		let mut output = String::new();

		// Determine whether to include details
		let details = include_details.unwrap_or(false);
		
		// Define the header based on the presence of details
		let header: Vec<&str> = if details {
			vec!["Rank", "Address", "Voting Power", "Moniker", "Details"]
		} else {
			vec!["Rank", "Address", "Voting Power", "Moniker"]
		};

		// Define the default widths based on whether details are included
		let default_widths = if details {
			vec![0, 0, 0, 0, 20] // Default widths for details
		} else {
			vec![0, 0, 0, 0] // Default widths without details
		};

		// Create the final widths vector, starting with defaults
		let mut final_widths = default_widths.clone();

		// If user provided widths, overwrite the defaults
		if let Some(user_widths) = column_widths {
			for (i, &width) in user_widths.iter().enumerate() {
				if i < final_widths.len() {
					final_widths[i] = width; // Use user-provided width
				}
			}
		}

		// Construct the header
		output.push_str(&header.join("\x1C"));
		output.push('\n');

		// Join final widths into a string and append to output
		output.push_str(&final_widths.iter().map(|w| w.to_string()).collect::<Vec<_>>().join("\x1C"));
		output.push('\n');

		// Data rows
		for validator in &self.0 {
			let row = if details {
				format!(
					"{}\x1C{}\x1C{}\x1C{}\x1C{}",
					validator.rank,
					validator.address,
					validator.voting_power_nom(),
					validator.moniker,
					validator.details
				)
			} else {
				format!(
					"{}\x1C{}\x1C{}\x1C{}",
					validator.rank,
					validator.address,
					validator.voting_power_nom(),
					validator.moniker,
				)
			};

			// Add the formatted validator to output
			output.push_str(&row);
			output.push('\n');
		}

		// Create and configure the table
		let mut table = TableBuilder::new(Some(output))
			.set_ifs("\x1C".to_string())
			.set_ofs("  ".to_string())
			.set_header_index(1)
			.set_column_width_limits_index(2)
			.set_max_cell_width(80)
			.set_pad_decimal_digits(true)
			.set_max_decimal_digits(0)
			.set_use_thousand_separator(true)
			.clone();
		
		// Build and return the table
		table.build().clone()
	}

	/// Returns an `IndexMap` representation of the validators.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let index_map = collection.index_map(None); // Include None for default behavior
	/// assert!(index_map.contains_key(&1));
	/// ```
	pub fn index_map(&self, include_details: Option<bool>) -> IndexMap<String, IndexMap<String, serde_json::Value>> {
		let details = include_details.unwrap_or(false);

		// Pre-allocate space for the outer map
		let mut array = IndexMap::with_capacity(self.0.len());

		for validator in &self.0 {
			// Pre-allocate space for the inner map
			let mut record = IndexMap::with_capacity(if details { 5 } else { 4 });

			record.insert("VOTING POWER".to_string(), serde_json::Value::Number(validator.voting_power.into()));
			record.insert("MONIKER".to_string(), serde_json::Value::String(validator.moniker.clone()));
			record.insert("RANK".to_string(), serde_json::Value::Number(validator.rank.into()));

			if details {
				record.insert("DETAILS".to_string(), serde_json::Value::String(validator.details.clone()));
			}

			array.insert(validator.address.clone(), record);
		}

		array
	}

	/// Serializes the `ValidatorCollection` into a JSON object.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let json_object = collection.to_json(); // This will return a serde_json::Value
	/// assert!(json_object.get("address").is_some());
	/// ```
	pub fn json(&self, include_details: Option<bool>) -> serde_json::Value {
		match serde_json::to_value(&self.index_map(include_details)) {
			Ok(json_value) => json_value,
			Err(e) => {
				eprintln!("Error serializing to JSON: {}", e);
				serde_json::Value::Null // Return a null value on error
			}
		}
	}

	/// Returns a vector of tuples representing the validators in tuple format.
	///
	/// Each tuple contains the rank, address, voting power, and moniker of a validator.
	/// If `include_details` is true, it will also include the details.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let tuple_vector = collection.tuple(None); // Call with `None` to include default behavior
	/// assert!(tuple_vector.iter().any(|(_, address, _, _)| *address == "address"));
	/// ```
	pub fn tuple(&self, include_details: Option<bool>) -> Vec<(usize, String, usize, String, Option<String>)> {
		let details = include_details.unwrap_or(false);
		let mut output = vec![];

		for validator in &self.0 {
			if details {
				output.push((
					validator.rank,
					validator.address.clone(),
					validator.voting_power,
					validator.moniker.clone(),
					Some(validator.details.clone()), // Wrap details in Some
				));
			} else {
				output.push((
					validator.rank,
					validator.address.clone(),
					validator.voting_power,
					validator.moniker.clone(),
					None, // No details
				));
			}
		}

		output
	}

	/// Prints the representation of the `ValidatorCollection` in the specified format.
	///
	/// This method allows for flexible output of the validator data in various formats.
	/// The available formats include table, JSON (both compact and pretty), tuple, and raw.
	/// The output can include additional details based on the `include_details` parameter.
	///
	/// # Parameters
	/// 
	/// - `format`: A string indicating the desired output format. Supported formats are:
	///   - `"table"`: Outputs the data in a table format.
	///   - `"json"`: Outputs the data in a compact JSON format.
	///   - `"json-pretty"`: Outputs the data in a formatted (pretty) JSON format.
	///   - `"tuple"`: Outputs the data in a tuple format for easier reading.
	///   - `"raw"`: Outputs the raw representation of the validators.
	/// 
	/// - `include_details`: An optional boolean that specifies whether to include additional
	///   details in the output. If `None`, the default behavior is to include details.
	/// 
	/// - `column_widths`: An optional vector of column widths, used when the output format
	///   is `"table"` to control the width of each column.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// collection.print("json", Some(true), None);
	/// ```
	///
	/// # Panics
	///
	/// This method does not panic but will print an error message to `stderr` if there
	/// is an issue serializing the JSON.
	///
	/// # Note
	///
	/// Ensure that the desired format is valid, as unsupported formats will result in no output.
	pub fn print(&self,
		format: Option<OutputFormat>,
		include_details: Option<bool>,
		column_widths: Option<Vec<usize>>
	) {

		// Use the default format if None is provided
		let format = format.unwrap_or(OutputFormat::JsonPretty);

		match format {
			OutputFormat::Table => self.table(include_details, column_widths).printstd(),
			OutputFormat::Json => {
				// Convert the JSON value to a string
				match serde_json::to_string(&self.json(include_details)) {
					Ok(json_str) => println!("{}", json_str),
					Err(e) => eprintln!("Error serializing JSON: {}", e),
				}
			},
			OutputFormat::JsonPretty => {
				// Serialize the JSON value to a pretty string
				match serde_json::to_string_pretty(&self.json(include_details)) {
					Ok(pretty_json) => println!("{}", pretty_json),
					Err(e) => eprintln!("Error serializing JSON: {}", e),
				}
			},
			OutputFormat::Tuple => {
				println!("{:?}", self.tuple(include_details));
			},
			OutputFormat::Raw => println!("{}", self.raw(include_details)),
		}
	}

}

impl Clone for ValidatorCollection {
	/// Creates a deep copy of the `ValidatorCollection`.
	///
	/// This method duplicates the `ValidatorCollection`, including all of its contained `Validator` instances.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// let cloned_collection = collection.clone();
	/// assert_eq!(collection.index_map(), cloned_collection.index_map());
	/// ```
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl FromIterator<Validator> for ValidatorCollection {
	/// Creates a `ValidatorCollection` from an iterator of `Validator` instances.
	///
	/// This method allows constructing a `ValidatorCollection` from any iterator that yields `Validator` items.
	///
	/// # Example
	///
	/// ```
	/// let validators = vec![
	///	 Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string()),
	///	 Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string()),
	/// ];
	/// let collection: ValidatorCollection = validators.into_iter().collect();
	/// assert_eq!(collection.index_map().len(), 2);
	/// ```
	fn from_iter<T: IntoIterator<Item = Validator>>(iter: T) -> Self {
		Self::from_iter(iter)
	}
}


impl<'a> IntoIterator for &'a ValidatorCollection {
	/// Returns an iterator over references to the `Validator` instances in the `ValidatorCollection`.
	///
	/// This method provides access to the `Validator` instances by reference, allowing for iteration without ownership.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// for validator in &collection {
	///	 println!("{}", validator.address);
	/// }
	/// ```
	type Item = &'a Validator;
	type IntoIter = std::slice::Iter<'a, Validator>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl IntoIterator for ValidatorCollection {
	/// Returns an iterator over the `Validator` instances in the `ValidatorCollection`.
	///
	/// This method provides ownership of the `Validator` instances, allowing for iteration with ownership.
	///
	/// # Example
	///
	/// ```
	/// let collection = ValidatorCollection::new();
	/// let validator = Validator::new(1, "address".to_string(), 100, "moniker".to_string(), "details".to_string());
	/// collection.insert(validator);
	/// for validator in collection {
	///	 println!("{}", validator.address);
	/// }
	/// ```
	type Item = Validator;
	type IntoIter = std::vec::IntoIter<Validator>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

/// Defines the CLI structure for the `validators` command.
#[derive(Parser)]
#[command(name = "validators", about = "Manage validators")]
pub struct Cli {
	/// Specify the output format
	#[arg(long, short)]
	pub format: Option<OutputFormat>,

	/// Wether to include the details field
	#[arg(short, long)]
	pub include_details: Option<bool>,

	/// Column widths for table view
	#[arg(short, long)]
	pub column_widths: Option<Vec<usize>>,

	/// Subcommands for the validators command
	#[command(subcommand)]
	pub command: Option<ValidatorsCommand>,
}

/// Subcommands for the `validators` command
#[derive(Subcommand)]
pub enum ValidatorsCommand {
	/// Show the top N validators
	Top {
		/// Number of top validators to show
		#[arg(value_parser = clap::value_parser!(usize), required = true)]
		number: usize,

		/// Specify the output format
		#[arg(default_value = "json-pretty", long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(default_value = "false", short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,

	},
	/// Show the bottom N validators
	Bottom {
		/// Number of bottom validators to show
		#[arg(value_parser = clap::value_parser!(usize), required = true)]
		number: usize,

		/// Specify the output format
		#[arg(long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,
	},
	/// Skip the first N validators
	Skip {
		/// Number of validators to skip
		#[arg(value_parser = clap::value_parser!(usize), required = true)]
		number: usize,

		/// Specify the output format
		#[arg(long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,
	},
	/// Show a specified number of random validators outside a specified top percentage
	Random {
		/// Number of random validators to show
		#[arg(short, long, value_parser = clap::value_parser!(usize), required = true)]
		count: usize,
		
		/// Percentage of validators to consider for randomness
		#[arg(short, long, value_parser = clap::value_parser!(u8), required = true)]
		percent: u8,

		/// Specify the output format
		#[arg(long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,
	},
	/// Search for validators by moniker
	Moniker {
		/// Search for validators by moniker
		#[arg(value_parser = clap::value_parser!(String), required = true)]
		moniker: String,

		/// Specify the output format
		#[arg(long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(default_value = "true", short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,
	},
	/// Search for a validator by address
	Address {
		/// Search for a validator by its address
		#[arg(value_parser = clap::value_parser!(String), required = true)]
		address: String,

		/// Specify the output format
		#[arg(long, short)]
		format: Option<OutputFormat>,

		/// Wether to include the details field
		#[arg(default_value = "true", short, long)]
		include_details: Option<bool>,

		/// Column widths for table view
		#[arg(short, long)]
		column_widths: Option<Vec<usize>>,
	},
}

pub fn run_cli(cli: &Cli) -> Result<(), Box<dyn Error>> {

	let collection = match ValidatorCollection::init() {
		Ok(collection) => collection,
		Err(e) => {
			println!("Error initializing validator collection: {}", e);
			return Err(e.into());
		}
	};

	// Handle subcommands
	match &cli.command {

		// handle address subcommand
		Some(ValidatorsCommand::Address { address, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			if !address.is_empty() {
				let filtered = collection.search_by_address(address);
				if filtered.is_empty() {
					eprintln!("No validators found with the address: {}", address);
				} else {
					filtered.print(format, include_details, column_widths.clone());
				}
			} else {
				eprintln!("Validator address is empty.");
			}
			Ok(())
		},

		// handle moniker subcommand
		Some(ValidatorsCommand::Moniker { moniker, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			if !moniker.is_empty() {
				let filtered = collection.search_by_moniker(moniker);
				if filtered.is_empty() {
					eprintln!("No validators found with the moniker: {}", moniker);
				} else {
					filtered.print(format, include_details, column_widths.clone());
				}
			} else {
				eprintln!("Validator moniker is empty.");
			}
			Ok(())
		},

		// handle top subcommand
		Some(ValidatorsCommand::Top { number, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			let filtered = collection.top(*number);
			if filtered.is_empty() {
				eprintln!("No validators found in the top {}", number);
			} else {
				filtered.print(format, include_details, column_widths.clone());
			}
			Ok(())
		},

		// handle bottom subcommand
		Some(ValidatorsCommand::Bottom { number, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			let filtered = collection.bottom(*number);
			if filtered.is_empty() {
				eprintln!("No validators found in the bottom {}", number);
			} else {
				filtered.print(format, include_details, column_widths.clone());
			}
			Ok(())
		},

		// handle skip subcommand
		Some(ValidatorsCommand::Skip { number, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			let filtered = collection.skip(*number);
			if filtered.is_empty() {
				eprintln!("No validators found after skipping {}", number);
			} else {
				filtered.print(format, include_details, column_widths.clone());
			}
			Ok(())
		},

		// handle random subcommand
		Some(ValidatorsCommand::Random { count, percent, format, include_details, column_widths }) => {

			let format		  = format.clone().or(cli.format.clone());
			let include_details = include_details.or(cli.include_details);
			let column_widths   = column_widths.clone().or(cli.column_widths.clone());

			let filtered = collection.random(*count, *percent);
			if filtered.is_empty() {
				eprintln!("No random validators found");
			} else {
				filtered.print(format, include_details, column_widths.clone());
			}
			Ok(())
		},

		// Default case when no subcommand is provided
		None => {
			collection.print(cli.format.clone(), cli.include_details, cli.column_widths.clone());
			Ok(())
		},
	}
}
