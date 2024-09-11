use crate::globals::NOMIC;
use crate::tools::columnizer::ColumnFormatter;
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

#[derive(Debug, Serialize)]
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

	pub fn table2(&self) {

		// source about 5k bytes
        let mut output = String::with_capacity(6144);

        // Construct the header
		output.push_str(HEADER_NAMES.join("\x1C"));

        // Data rows
        for validator in &self.validators {
            output.push_str(&self.table_detail(widths, &validator));
        }


	}

    pub fn table_header(&self, widths: [usize; 5]) -> String {

		let mut output = String::with_capacity(2);

		// Header row
		output.push_str(&format!(
			"{:>width0$} {:<width1$} {:>width2$} {:<width3$} {:<width4$}\n",
			"Rank",
			"Address",
			"Voting Power",
			"Moniker",
			"Details",
			width0 = widths[0],
			width1 = widths[1],
			width2 = widths[2],
			width3 = widths[3],
			width4 = widths[4]
		));

		// Separator row
		output.push_str(&format!(
			"{:-<width0$} {:-<width1$} {:-<width2$} {:-<width3$} {:-<width4$}\n",
			"",
			"",
			"",
			"",
			"",
			width0 = widths[0],
			width1 = widths[1],
			width2 = widths[2],
			width3 = widths[3],
			width4 = widths[4]
		));

		output
	}

    /// Constructs a table with given column widths and buffer size.
    pub fn table_detail(&self, widths: [usize; 5], validator: &Validator) -> String {
		format!(
			"{:>width0$} {:<width1$} {:>width2$} {:<width3$} {:<width4$}\n",
			validator.f_rank(),
			validator.address,
			validator.f_voting_power(),
			validator.moniker,
			validator.f_details(),
			width0 = widths[0],
			width1 = widths[1],
			width2 = widths[2],
			width3 = widths[3],
			width4 = widths[4]
		)
	}

    pub fn table(&self) -> String {

        // Get maximum field widths for columns to align them properly
        let widths = self.get_max_field_widths();

        let mut output = String::with_capacity(1024);

        // Construct the header
		output.push_str(&self.table_header(widths));

        // Data rows
        for validator in &self.validators {
            output.push_str(format());
        }
        output
    }

	/// Returns a formatted table for a specific validator by address.
	pub fn table_validator(&self, address: &str) {
		if let Some(validator) = self.get_validator(address) {
			// Calculate field widths for the individual validator
			let widths = [
				std::cmp::max(validator.f_rank().len(), HEADER_NAMES[0].len()),
				std::cmp::max(validator.address.len(), HEADER_NAMES[1].len()),
				std::cmp::max(validator.f_voting_power().len(), HEADER_NAMES[2].len()),
				std::cmp::max(validator.moniker.len(), HEADER_NAMES[3].len()),
				std::cmp::max(validator.f_details().len(), HEADER_NAMES[4].len()),
			];

			// Construct the formatted output
			let mut output = String::with_capacity(128); // Adjust capacity for a single row + header
			output.push_str(&self.table_header(widths)); // Use the same header format
			output.push_str(&self.table_detail(widths, &validator)); // Add the validator row
			println!("{}", output);
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
			let formatted = ColumnFormatter::new(&output).ifs("\x1C").format();

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
            "table" => println!("{}", validator_collection.table()),
            "tuple" => validator_collection.tuple(),
            _ => {
                eprintln!("Unknown format: {}", format);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

