use columnizer;
use crate::globals::NOMIC;
use indexmap::IndexMap;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use serde_json;  // {Value, to_string, to_string_pretty} fully addressed in code for clarity
use serde::Serialize;
use std::error::Error;
use std::iter::FromIterator;
use std::mem::size_of;
use std::process::Command;

const HEADER: [&str; 5] = ["Rank", "Address", "Voting Power", "Moniker", "Details"];

#[derive(Debug, Serialize, Clone)]
pub struct Validator {
	rank: usize,
	address: String,
	voting_power: u64,
	moniker: String,
	details: String,
}

impl Validator {

	pub fn new(rank: usize, address: String, voting_power: u64, moniker: String, details: String) -> Self {
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

    // Create a new ValidatorCollection
    pub fn new() -> Self {
        Self(Vec::new())
    }

    // Create a ValidatorCollection from a Vec<Validator>
    pub fn from_vec(validators: Vec<Validator>) -> Self {
        Self(validators)
    }


    // Create a ValidatorCollection from an iterator
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Validator>,
    {
        Self(iter.into_iter().collect())
    }

    // Add a validator to the collection
    pub fn insert(&mut self, validator: Validator) {
        self.0.push(validator);
    }

    // Get the number of validators in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Access the mutable vector of validators
    pub fn validators_mut(&mut self) -> &mut Vec<Validator> {
        &mut self.0
    }

	fn nomic_validators() -> Result<String, Box<dyn Error>> {
		// Create and configure the Command
		let mut cmd = Command::new(NOMIC);
		cmd.arg("validators");

		// Execute the command
		let output = cmd.output()?;

		// Check if the command was successful
		if !output.status.success() {
			return Err(format!("Command `{}` failed with output: {:?}", NOMIC, output).into());
		}

		// Convert the command output to a string
		let output_str = String::from_utf8(output.stdout)?;

		Ok(output_str)
	}

	pub fn parse_nomic_validators(&mut self, input: String) -> Result<(), Box<dyn Error>> {
		let lines: Vec<&str> = input.lines().collect();
		let mut rank = 1; // Start rank from 1

		for chunk in lines.chunks(4) {
			if chunk.len() == 4 {
				let address = chunk[0].trim().trim_start_matches('-').trim().to_string();
				let voting_power_str = chunk[1].split(':').nth(1).unwrap_or("").trim().to_string();
				let voting_power = voting_power_str.parse::<u64>().unwrap_or(0);
				let moniker = chunk[2].split(':').nth(1).unwrap_or("").trim().to_string();
				let details = chunk[3].split(':').nth(1).unwrap_or("").trim().to_string();

				let validator = Validator::new(rank, address, voting_power, moniker, details);
				self.insert(validator); // Use `insert` to add validators

				rank += 1;
			}
		}
		Ok(())
	}

	// Create a ValidatorCollection from a given String output
	pub fn load_from_string(command_output: String) -> Result<Self, Box<dyn Error>> {
		let mut collection = Self::new();
		collection.parse_nomic_validators(command_output)?;
		Ok(collection)
	}

	// Create a ValidatorCollection and populate it from the 'nomic validators' command output
	pub fn load_from_command() -> Result<Self, Box<dyn Error>> {
		let output = Self::nomic_validators()?;
		Self::load_from_string(output)
	}

	// Alias for load_from_command
	pub fn init() -> Result<Self, Box<dyn Error>> {
		Self::load_from_command()
		

// 		println!("hello");
// 		println!("len: {}", self.len());
	}

	// Method to estimate the byte size of the ValidatorCollection
	pub fn bytes(&self) -> usize {
		// Calculate the total size of all validators in bytes
		let validators_bytes = self.0.iter().map(|v| v.bytes()).sum::<usize>();

		// Estimate additional space for formatting (e.g., newlines, dashes, etc.)
		let formatting_overhead = self.len() * 100; // Adjust 100 as needed

		validators_bytes + formatting_overhead
	}

    // Method to find a validator by address
    pub fn get_validator(&self, address: &str) -> Option<&Validator> {
        self.0.iter().find(|v| v.address == address)
    }

	// Method to search for a validator by address and return a ValidatorCollection
	pub fn search_by_address(&self, search: &str) -> ValidatorCollection {
		self.0.iter()
			.filter(|validator| validator.address == search)
			.cloned()
			.collect()
	}


	// Method to search for a string part of moniker
	pub fn search_by_moniker(&self, search: &str) -> ValidatorCollection {
		// Convert search term to lowercase for case-insensitive matching
		let search_lower = search.to_lowercase();

		self.0.iter()
			.filter(|validator| validator.moniker.to_lowercase().contains(&search_lower))
			.cloned()
			.collect()
	}

    // Method to get the top `n` validators by voting power
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

	// Method to get the bottom `n` validators by voting power
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

	// Method to skip the top `n` validators by voting power
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

	pub fn raw(&self) -> String {
		let mut output = String::with_capacity(self.bytes());

		for validator in &self.0 {
			// Append formatted data to the output string
			output.push_str(&format!("- {}\n", validator.address));
			output.push_str(&format!("      VOTING POWER: {}\n", validator.voting_power));
			output.push_str(&format!("      MONIKER: {}\n", validator.moniker));
			output.push_str(&format!("      DETAILS: {}\n", validator.details));
		}

// debug 
// println!("{}", output);

 		output
	}

	pub fn table(&self) -> String {
		// Estimate the size and preallocate string
		let mut output = String::with_capacity(self.bytes());

		// Construct the header
		output.push_str(&HEADER.join("\x1C"));
		output.push('\n');

		// Maximum Column Widths
		output.push_str(&format!("{}\x1C{}\x1C{}\x1C{}\x1C{}", 0, 0, 0, 0, 40));
		output.push('\n');

		// Data rows
		for validator in &self.0 {
			// Manually format the Validator fields with '\x1C' as the separator
			let formatted_validator = format!(
				"{}\x1C{}\x1C{}\x1C{}\x1C{}",
				validator.rank,
				validator.address,
				validator.voting_power_nom(),
				validator.moniker,
				validator.details
			);

			// Add the formatted validator to output
			output.push_str(&formatted_validator);
			output.push('\n');
		}

		let formatted_output = columnizer::Builder::new(&output)
			.ifs("\x1C")
			.ofs("  ")
			.header_row(1)
			.max_width_row(2)
			.max_text_width(80)
			.pad_decimal_digits(true)
			.max_decimal_digits(0)
			.add_divider(true)
			.add_thousand_separator(true)
			.format();

		formatted_output
	}

	pub fn index_map(&self) -> IndexMap<String, IndexMap<String, serde_json::Value>> {
		// Pre-allocate space for the outer map
		let mut array = IndexMap::with_capacity(self.0.len());

		for validator in &self.0 {
			// Pre-allocate space for the inner map
			let mut record = IndexMap::with_capacity(4); // Number of fields

			record.insert("VOTING POWER".to_string(), serde_json::Value::Number(validator.voting_power.into()));
			record.insert("MONIKER".to_string(), serde_json::Value::String(validator.moniker.clone()));
			record.insert("RANK".to_string(), serde_json::Value::Number(validator.rank.into()));
			record.insert("DETAILS".to_string(), serde_json::Value::String(validator.details.clone()));

			array.insert(validator.address.clone(), record);
		}
		array
	}

	pub fn json(&self) -> String {
		match serde_json::to_string(&self.index_map()) {
			Ok(json_str) => json_str,
			Err(e) => {
				eprintln!("Error serializing to JSON: {}", e);
				String::new()
			}
		}
	}

	pub fn json_pretty(&self) -> String {
		match serde_json::to_string_pretty(&self.index_map()) {
			Ok(json_str) => json_str,
			Err(e) => {
				eprintln!("Error serializing to JSON: {}", e);
				String::new()
			}
		}
	}

	pub fn tuple(&self) -> String {
		let mut output = String::new();
		for validator in &self.0 {
			output.push_str(&format!("{} {} {} {}\n",
				validator.rank, validator.address, validator.voting_power, validator.moniker
			));
		}
		output.trim_end().to_string()
	}

	pub fn print(&self, format: &str) {
		let output = match format {
			"table" => self.table(),
			"json" => self.json(),
			"json-pretty" => self.json_pretty(),
			"tuple" => self.tuple(),
			"raw" => self.raw(),
			_ => String::new(),
		};
		println!("{}", output);
	}
}

// Implement Clone for ValidatorCollection
impl Clone for ValidatorCollection {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl FromIterator<Validator> for ValidatorCollection {
    fn from_iter<T: IntoIterator<Item = Validator>>(iter: T) -> Self {
        Self::from_iter(iter)
    }
}

impl<'a> IntoIterator for &'a ValidatorCollection {
    type Item = &'a Validator;
    type IntoIter = std::slice::Iter<'a, Validator>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ValidatorCollection {
    type Item = Validator;
    type IntoIter = std::vec::IntoIter<Validator>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
