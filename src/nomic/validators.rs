use crate::globals::NOMIC;
use columnizer;
use indexmap::IndexMap;
use num_format::Locale;
use num_format::ToFormattedString;
use serde_json::Number;
use serde_json::Value;
use serde::Serialize;
use std::error::Error;
use std::process::Command;

const DETAILS_MAX_WIDTH: usize = 36;
const HEADER_NAMES: [&'static str; 5] = ["Rank", "Address", "Voting Power", "Moniker", "Details"];

#[derive(Debug, Serialize, Clone)]
pub struct Validator {
	rank: u32,
	address: String,
	voting_power: u64,
	moniker: String,
	details: String,
}

impl Validator {

	pub fn new(rank: u32, address: String, voting_power: u64, moniker: String, details: String) -> Self {
		Self {
			rank,
			address,
			voting_power,
			moniker,
			details,
		}
	}

	pub fn f_rank(&self) -> String {
		self.rank.to_formatted_string(&Locale::en)
	}

	pub fn f_voting_power(&self) -> String {
		(self.voting_power / 1_000_000).to_formatted_string(&Locale::en)
	}

	pub fn f_details(&self) -> String {
		if self.details.chars().count() <= DETAILS_MAX_WIDTH {
			self.details.clone()
		} else {
			self.details.chars().take(DETAILS_MAX_WIDTH).collect()
		}
	}
}

pub struct ValidatorCollection {
    validators: Vec<Validator>,
    max_field_widths: [usize; 5], // List of maximum field widths for output formatting
    initialized: bool, // Field to track if the collection is initialized
}

impl ValidatorCollection {
    pub fn new() -> Self {
        // Calculate initial widths based on header names
        let max_field_widths = [
            HEADER_NAMES[0].len(), // "Rank"
            HEADER_NAMES[1].len(), // "Address"
            HEADER_NAMES[2].len(), // "Voting Power"
            HEADER_NAMES[3].len(), // "Moniker"
            HEADER_NAMES[4].len(), // "Details"
        ];

        Self {
            validators: Vec::new(),
            max_field_widths,
            initialized: false, // Initialized as false
        }
    }

	pub fn raw(&self) -> Result<String, Box<dyn std::error::Error>> {
		// Create and configure the Command
		let mut cmd = Command::new(NOMIC);
		cmd.arg("validators");

		// Execute the command
		let output = cmd.output()?;

		// Check if the command was successful
		if !output.status.success() {
			return Err(format!("nomic validators command failed with output: {:?}", output).into());
		}

		// Convert the command output to a string
		let output_str = String::from_utf8_lossy(&output.stdout).to_string();

		Ok(output_str)
	}

	pub fn insert(&mut self, validator: Validator) {
		let widths = [
			validator.f_rank().len(),
			validator.address.len(),
			validator.f_voting_power().len(),
			validator.moniker.len(),
			validator.f_details().len(),
		];

		for (i, &width) in widths.iter().enumerate() {
			if width > self.max_field_widths[i] {
				if i == 4 && width > DETAILS_MAX_WIDTH {
					self.max_field_widths[i] = DETAILS_MAX_WIDTH;
				} else {
					self.max_field_widths[i] = width;
				}
			}
		}

		self.validators.push(validator);
	}

	pub fn init(&mut self) -> Result<(), Box<dyn Error>> {

		let input = self.raw()?;

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
				self.insert(validator);

				rank += 1;
			}
		}

		self.initialized = true; // Set initialized to true if initialization succeeds
		Ok(())
	}

	pub fn get_max_field_widths(&self) -> [usize; 5] {
		self.max_field_widths
	}

    pub fn get_validator(&self, address: &str) -> Option<&Validator> {
        self.validators.iter().find(|v| v.address == address)
    }

	pub fn search_by_moniker(&self, search: &str) -> Option<Vec<Validator>> {
		let search_lower = search.to_lowercase();

		// Collect validators that match the search criteria
		let matching_validators: Vec<Validator> = self
			.validators
			.iter()
			.filter(|validator| validator.moniker.to_lowercase().contains(&search_lower))
			.cloned()  // Clone each `Validator` from the iterator (because `self.validators` is a reference)
			.collect();

		if matching_validators.is_empty() {
			None // No results found
		} else {
			Some(matching_validators) // Return results
		}
	}

	pub fn search_by_moniker_table(&self, search: &str) -> String {
		if let Some(validators) = self.search_by_moniker(search) {
			self.table_template(&validators)
		} else {
			println!("Validator not found");  // Return a String when the validator is not found
		}
	}


	}

	fn table_template (&self, data: &Vec<Validator>) {

		// Source about 5k bytes
		let mut output = String::with_capacity(6144);

		// Construct the header
		output.push_str(&HEADER_NAMES.join("\x1C"));
		output.push('\n');
 
 		// Maximum Column Widths
 		output.push_str(&format!("{}\x1C{}\x1C{}\x1C{}\x1C{}", 0, 80, 0, 0, 20));
 		output.push('\n');

		// Data rows
		for validator in data {
			// Manually format the Validator fields with '\x1C' as the separator
			let formatted_validator = format!(
				"{}\x1C{}\x1C{}\x1C{}\x1C{}",
				validator.rank,
				validator.address,
				validator.voting_power / 1_000_000,
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
			.max_text_width(50)
			.add_divider(true)
			.add_thousand_separator(true)
			.format();

		println!("{}", formatted_output);

	}

 	pub fn table (&self) { self.table_template(&self.validators); }


	pub fn table_validator(&self, address: &str) {
		if let Some(validator) = self.get_validator(address) {
			// Create a new vector and push the validator into it
			let mut data = Vec::new();
			data.push(validator.clone());
			
			// Pass the reference of the vector to `table_template`
			self.table_template(&data);

		} else {
			println!("Validator not found");  // Return a String when the validator is not found
		}
	}

	pub fn index_map(&self) -> IndexMap<String, IndexMap<String, Value>> {
		// Pre-allocate space if possible
		let mut array = IndexMap::with_capacity(self.validators.len());

		for validator in &self.validators {
			let mut record = IndexMap::with_capacity(4); // Number of fields
			record.insert("VOTING POWER".to_string(), Value::Number(Number::from(validator.voting_power)));
			record.insert("MONIKER".to_string(), Value::String(validator.moniker.clone()));
			record.insert("RANK".to_string(), Value::Number(Number::from(validator.rank)));
			record.insert("DETAILS".to_string(), Value::String(validator.details.clone()));

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

    // Method to print all validators in a tuple format
    pub fn tuple(&self) {
        for validator in &self.validators {
            println!("{} {} {} {}", validator.rank, validator.address, validator.voting_power, validator.moniker);
        }
    }

    // Method to return a tuple of a specific validator by address
    pub fn tuple_validator(&self, address: &str) {
        if let Some(validator) = self.get_validator(address) {
            println!("{} {} {}", validator.rank, validator.voting_power, validator.moniker);
        }
    }

    // Method to format and print the details of a specific validator
    pub fn validator_details(&self, address: &str) {
        if let Some(validator) = self.get_validator(address) {
			println!("Rank: {}, Voting Power: {}, Moniker: {}", validator.rank, validator.voting_power, validator.moniker);
        }
    }

	pub fn find_validator_by_moniker(&self, search: &str) {
		let search_lower = search.to_lowercase();

		// Collect validators that match the search criteria
		let matching_validators: Vec<&Validator> = self
			.validators
			.iter()
			.filter(|validator| validator.moniker.to_lowercase().contains(&search_lower))
			.collect();

		if matching_validators.is_empty() {
			// Inform the user if no validators are found
			println!("No validators found with moniker containing '{}'", search);
		} else {
			// Create an output string with a special separator (`\x1C`)
			let mut output = String::new();
			for validator in &matching_validators {
				output.push_str(&format!("{}\x1C{}\x1C{}\n", validator.rank, validator.address, validator.moniker));
			}

			// Format the columns with the output string using the special separator
			let formatted = columnizer::Builder::new(&output).ifs("\x1C").format();

			// Print the fmrmatted output
			println!("{}", formatted);
		}
	}

// 	pub fn find_validator_by_moniker(&self, search: &str) {
// 		let search_lower = search.to_lowercase();
// 
// 		let matching_validators: Vec<&Validator> = self.validators
// 			.iter()
// 			.filter(|validator| validator.moniker.to_lowercase().contains(&search_lower))
// 			.collect();
// 
// 		if matching_validators.is_empty() {
// 			println!("No validators found with moniker containing '{}'", search);
// 		} else {
// 			let mut output = String::new();
// 			for validator in &matching_validators {
// 				// Format each validator's information into a string
// 				output.push_str(&format!("{}\x1C{}\x1C{}\n", validator.rank, validator.address, validator.moniker));
// 			}
// 
// 			// Format the columns with the output string
// 			let formatted_output = format_columns(&output, Some(0), Some("\x1C"));
// 			println!("{}", formatted_output);
// 		}
// 
// 	}

}
pub fn handle_validators_submenu(format: &str, validator_address: Option<&str>) -> Result<(), Box<dyn Error>> {
    let mut validator_collection = ValidatorCollection::new();
    validator_collection.init()?;

    if let Some(address) = validator_address {
		match format {
			"details" => {
				validator_collection.validator_details(&address);
			}
			"table" => {
				validator_collection.table_validator(&address);
			}
			"tuple" => {
				validator_collection.tuple_validator(&address);
			}
			"raw" => {
				if let Some(validator) = validator_collection.get_validator(address) {
					println!("- {}", validator.address);
					println!("        VOTING POWER: {}", validator.voting_power);
					println!("        MONIKER: {}", validator.moniker);
					println!("        DETAILS: {}", validator.details);
				} else {
					println!("Validator with address {} not found.", address);
				}
			}
			"json" => {
				if let Some(validator) = validator_collection.get_validator(address) {
					println!("{}", serde_json::to_string(&validator)?);
				} else {
					println!("Validator with address {} not found.", address);
				}
			}
			"json-pretty" => {
				if let Some(validator) = validator_collection.get_validator(address) {
					println!("{}", serde_json::to_string_pretty(&validator)?);
				} else {
					println!("Validator with address {} not found.", address);
				}
			}
			_ => {
				eprintln!("Unknown format: {}", format);
				std::process::exit(1);
			}
		}
    } else {
        match format {
            "json" => println!("{}", validator_collection.json()),
            "json-pretty" => println!("{}", validator_collection.json_pretty()),
            "raw" => println!("{}", validator_collection.raw()?),
            "table" => validator_collection.table(),
            "tuple" => validator_collection.tuple(),
            _ => {
                eprintln!("Unknown format: {}", format);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

