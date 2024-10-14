use clap::ValueEnum;
use crate::globals::NOMIC;
use crate::globals::NOMIC_LEGACY_VERSION;
use fmt::table::Table;
use fmt::table::TableBuilder;
use indexmap::IndexMap;
use serde_json;
use std::iter::FromIterator;
use std::process::Command;
use std::str::FromStr;
use crate::validators::Validator;
use eyre::Result;
use eyre::eyre;
use std::path::Path;
use std::fs;
use rand::seq::SliceRandom;
use serde_json::Value;

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
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json"        => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            "raw"         => Ok(OutputFormat::Raw),
            "table"       => Ok(OutputFormat::Table),
            "tuple"       => Ok(OutputFormat::Tuple),
            _             => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormat::Json       => "json",
            OutputFormat::JsonPretty => "json-pretty",
            OutputFormat::Raw        => "raw",
            OutputFormat::Table      => "table",
            OutputFormat::Tuple      => "tuple",
        };
        write!(f, "{}", output)
    }
}

#[derive(Debug)] 
pub struct ValidatorCollection(Vec<Validator>);

impl ValidatorCollection {
    /// Imports validators from a string input and returns a ValidatorCollection.
    pub fn import(input: String) -> eyre::Result<Self> {
        let lines: Vec<&str> = input.lines().collect();
        let mut rank = 1; // Start rank from 1
        let mut validators = Vec::new(); // Initialize a vector to store validators

        for chunk in lines.chunks(4) {
            if chunk.len() == 4 {
                let address = chunk[0].trim().trim_start_matches('-').trim().to_string();
                let voting_power_str = chunk[1].split(':').nth(1).unwrap_or("").trim().to_string();
                let voting_power = voting_power_str.parse::<u64>().unwrap_or(0);
                let moniker = chunk[2].split(':').nth(1).unwrap_or("").trim().to_string();
                let details = chunk[3].split(':').nth(1).unwrap_or("").trim().to_string();

                let validator = Validator::new(rank, address, voting_power, moniker, details);
                validators.push(validator); // Add validator to the vector

                rank += 1; // Increment rank
            }
        }

        Ok(Self(validators)) // Return the ValidatorCollection
    }

    /// Loads validators from a specified file and returns a ValidatorCollection.
    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file.as_ref();

        // Read the file content as a string
        let input = fs::read_to_string(file).map_err(|e| eyre::eyre!("Failed to read file: {}", e))?;

        // Call the import method to create the ValidatorCollection
        Self::import(input)
    }

    pub fn fetch() -> eyre::Result<Self> {
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
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        // Convert the command output to a string
        let output_str = String::from_utf8(output.stdout)?;

        Self::import(output_str)

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

    pub fn validator(&self, address: &str) -> eyre::Result<&Validator> {
        self.0.iter()
            .find(|v| v.address().to_lowercase() == address.to_lowercase())
            .ok_or_else(|| eyre::eyre!("Validator with address `{}` not found", address))
    }

    pub fn filter_address(&self, search: &str) -> eyre::Result<Self> {
        // Create a new ValidatorCollection by filtering the existing one
        let validators: Vec<Validator> = self.0.iter()
            .filter(|validator| validator.address().eq_ignore_ascii_case(search)) // Case-insensitive match
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // Check if the new collection is empty
        if validators.is_empty() {
            return Err(eyre::eyre!("No validators found with address `{}`", search));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(ValidatorCollection(validators))
    }

    pub fn filter_moniker(&self, search: &str) -> eyre::Result<Self> {
        // Convert search term to lowercase for case-insensitive matching
        let search_lower = search.to_lowercase();

        // Create a new ValidatorCollection by filtering the existing one
        let validators: Vec<Validator> = self.0.iter()
            .filter(|validator| validator.moniker().to_lowercase().contains(&search_lower))
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // Check if the new collection is empty
        if validators.is_empty() {
            return Err(eyre::eyre!("No validators found with moniker containing `{}`", search));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(ValidatorCollection(validators))
    }

    pub fn search(&self, search: &str) -> eyre::Result<Self> {
        // Attempt to filter by exact address match first
        let address_filtered: Vec<Validator> = self.0.iter()
            .filter(|validator| validator.address().eq_ignore_ascii_case(search)) // Case-insensitive match
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // If any validators were found by address, return them
        if !address_filtered.is_empty() {
            return Ok(ValidatorCollection(address_filtered));
        }

        // If no exact address matches were found, filter by moniker sub-match
        let search_lower = search.to_lowercase();
        let moniker_filtered: Vec<Validator> = self.0.iter()
            .filter(|validator| validator.moniker().to_lowercase().contains(&search_lower))
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // Check if any validators were found by moniker
        if moniker_filtered.is_empty() {
            return Err(eyre::eyre!("No validators found with address or moniker matching `{}`", search));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(ValidatorCollection(moniker_filtered))
    }

    pub fn search_multi(&self, searches: Vec<String>) -> eyre::Result<Self> {
        let mut results = Self(Vec::new()); // Initialize a single results collection

        // Loop through each search term
        for search in searches {
            let search_lower = search.to_lowercase();

            // Check each validator against the search term
            for validator in &self.0 {
                // Check for exact address match
                if validator.address().eq_ignore_ascii_case(&search) {
                    results.0.push(validator.clone());
                } 
                // Check for moniker match
                if validator.moniker().to_lowercase().contains(&search_lower) {
                    // Push to results if the validator matches the search
                    results.0.push(validator.clone());
                }
            }
        }

        // Check if any validators were found across all searches
        if results.0.is_empty() {
            return Err(eyre::eyre!("No validators found matching any of the provided searches."));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(results)

    }

    pub fn filter_addresses(&self, searches: Vec<String>) -> eyre::Result<Self> {
        // Create a new ValidatorCollection by filtering the existing one
        let validators: Vec<Validator> = self.0.iter()
            .filter(|validator| {
                // Check if the validator's address matches any address in the searches vector
                searches.iter().any(|search| validator.address().eq_ignore_ascii_case(search))
            })
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // Check if the new collection is empty
        if validators.is_empty() {
            return Err(eyre::eyre!("No validators found with the specified addresses."));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(ValidatorCollection(validators))
    }

    pub fn filter_monikers(&self, searches: Vec<String>) -> eyre::Result<Self> {
        // Create a new ValidatorCollection by filtering the existing one
        let validators: Vec<Validator> = self.0.iter()
            .filter(|validator| {
                // Check if the validator's moniker matches any of the searches
                searches.iter().any(|search| validator.moniker().to_lowercase().contains(&search.to_lowercase()))
            })
            .cloned() // Cloning for new instances
            .collect(); // Collecting into a Vec<Validator>

        // Check if the new collection is empty
        if validators.is_empty() {
            return Err(eyre::eyre!("No validators found with any of the specified monikers."));
        }

        // Return the new ValidatorCollection with the filtered results
        Ok(ValidatorCollection(validators))
    }

    pub fn top(&self, n: Option<usize>) -> eyre::Result<Self> {
        // Clone the current ValidatorCollection to work on a separate copy
        let mut top_validators = self.clone();

        // Sort the validators by voting power in descending order
        top_validators.0.sort_by_key(|v| std::cmp::Reverse(v.voting_power()));

        // Use n with a default value of 10
        let count = n.unwrap_or(10);

        // Truncate to keep only the top `count` validators
        top_validators.0.truncate(count);

        // Return the modified ValidatorCollection
        Ok(top_validators)
    }

    pub fn bottom(&self, n: Option<usize>) -> eyre::Result<Self> {
        // Clone the current ValidatorCollection to work on a separate copy
        let mut bottom_validators = self.clone();

        // Sort the validators by voting power in ascending order
        bottom_validators.0.sort_by_key(|v| v.voting_power());

        // Use n with a default value of 10
        let count = n.unwrap_or(10);

        // Truncate to keep only the bottom `count` validators
        bottom_validators.0.truncate(count);

        // Return the modified ValidatorCollection
        Ok(bottom_validators)
    }

    pub fn skip_top(&self, n: Option<usize>) -> eyre::Result<Self> {
        // Use n with a default value of 10
        let count = n.unwrap_or(10);

        // Check if count is greater than or equal to the number of validators
        if count >= self.0.len() {
            // Return a clone of the current ValidatorCollection if count is too high
            return Ok(self.clone()); // Corrected to use self.clone()
        }

        // Clone the current ValidatorCollection to work on a separate copy
        let mut remaining_validators = self.clone();

        // Sort the validators by voting power in descending order
        remaining_validators.0.sort_by_key(|v| std::cmp::Reverse(v.voting_power()));

        // Skip the top `count` validators by draining the first `count` elements
        remaining_validators.0.drain(..count);

        // Return the modified ValidatorCollection
        Ok(remaining_validators)
    }

    pub fn skip_bottom(&self, n: Option<usize>) -> eyre::Result<Self> {
        // Use n with a default value of 10
        let count = n.unwrap_or(10);

        // Check if count is greater than or equal to the number of validators
        if count >= self.0.len() {
            // Return a clone of the current ValidatorCollection if count is too high
            return Ok(self.clone()); // Corrected to use self.clone()
        }

        // Clone the current ValidatorCollection to work on a separate copy
        let mut remaining_validators = self.clone();

        // Sort the validators by voting power in ascending order
        remaining_validators.0.sort_by_key(|v| v.voting_power());

        // Skip the bottom `count` validators by draining the first `count` elements
        remaining_validators.0.drain(..count);

        // Return the modified ValidatorCollection
        Ok(remaining_validators)
    }

    pub fn random(&self,
        count: Option<usize>,
        skip_top: Option<usize>,
        skip_bottom: Option<usize>,
    ) -> eyre::Result<Self> {
        // Use `count` with a default value of 4
        let count = count.unwrap_or(4);

        // Clone the current ValidatorCollection to work on a separate copy
        let mut random_validators = self
            .skip_top(skip_top)?
            .skip_bottom(skip_bottom)?;

        // Check if the number of remaining validators is less than the count
        if random_validators.0.len() < count {
            eprintln!("Warning: Not enough validators to select from. Returning all available validators.");
            return Ok(random_validators); // Return the collection as is
        }

        // Shuffle the validator collection
        let mut rng = rand::thread_rng();
        random_validators.0.shuffle(&mut rng);

        // Truncate to keep only `count` random validators
        random_validators.0.truncate(count);

        // Return the modified ValidatorCollection with random validators
        Ok(random_validators)
    }


    /// Returns a raw string representation of the validators.
    ///
    /// # Example
    ///
    /// ```
    /// let collection = ValidatorCollection::new();
    /// let raw_string = collection.raw(None)?;
    /// assert!(raw_string.contains("address"));
    /// ```
    pub fn raw(&self, include_details: Option<bool>) -> eyre::Result<String> {
        let mut output = String::new(); // Create a new String to hold the output

        for validator in &self.0 {
            // Append formatted data to the output string
            output.push_str(&format!("- {}\n", validator.address()));
            output.push_str(&format!("    VOTING POWER: {}\n", validator.voting_power()));
            output.push_str(&format!("    MONIKER: {}\n", validator.moniker()));

            // Include details if specified
            if include_details.unwrap_or(true) {
                output.push_str(&format!("    DETAILS: {}\n", validator.details()));
            }
        }

        // Return the constructed string
        Ok(output.trim_end().to_string()) // Trim any trailing newlines
    }

    /// Returns a formatted table representation of the validators, sorted by voting power in descending order.
    ///
    /// # Example
    ///
    /// ```
    /// let collection = ValidatorCollection::new();
    /// let table = collection.table(None, None); // Include None for default behavior
    /// assert!(table.contains("Rank"));
    /// ```
    pub fn table(&self, include_details: Option<bool>, column_widths: Option<Vec<usize>>) -> eyre::Result<Table> {
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

        // Sort validators by voting power in descending order
        let mut sorted_validators = self.0.clone();
        sorted_validators.sort_by_key(|v| std::cmp::Reverse(v.voting_power()));

        // Construct the header
        output.push_str(&header.join("\x1C"));
        output.push('\n');

        // Join final widths into a string and append to output
        output.push_str(&final_widths.iter().map(|w| w.to_string()).collect::<Vec<_>>().join("\x1C"));
        output.push('\n');

        // Data rows
        for validator in &sorted_validators {
            let row = if details {
                format!(
                    "{}\x1C{}\x1C{}\x1C{}\x1C{}",
                    validator.rank(),
                    validator.address(),
                    validator.voting_power_nom(),
                    validator.moniker(),
                    validator.details()
                )
            } else {
                format!(
                    "{}\x1C{}\x1C{}\x1C{}",
                    validator.rank(),
                    validator.address(),
                    validator.voting_power_nom(),
                    validator.moniker(),
                )
            };

            // Add the formatted validator to output
            output.push_str(&row);
            output.push('\n');
        }

        // Create and configure the table using TableBuilder
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
        Ok(table.build().clone())
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
    pub fn index_map(
        &self,
        include_details: Option<bool>
    ) -> eyre::Result<IndexMap<String, IndexMap<String, serde_json::Value>>> {
        let details = include_details.unwrap_or(false);

        // Pre-allocate space for the outer map
        let mut array = IndexMap::new();

        // Create a sorted vector of validators based on voting power in descending order
        let mut sorted_validators = self.0.clone();
        sorted_validators.sort_by_key(|v| std::cmp::Reverse(v.voting_power()));

        for validator in sorted_validators {
            // Pre-allocate space for the inner map
            let mut record = IndexMap::with_capacity(if details { 5 } else { 4 });

            record.insert("VOTING POWER".to_string(), serde_json::Value::Number(validator.voting_power().into()));
            record.insert("MONIKER".to_string(),      serde_json::Value::String(validator.moniker().to_string()));
            record.insert("RANK".to_string(),         serde_json::Value::Number(validator.rank().into()));

            if details {
                record.insert("DETAILS".to_string(),  serde_json::Value::String(validator.details().to_string()));
            }

            array.insert(validator.address().to_string(), record);
        }

        Ok(array)
    }

    /// Serializes the `ValidatorCollection` into a JSON object.
    ///
    /// # Example
    ///
    /// ```
    /// let collection = ValidatorCollection::new();
    /// let json_object = collection.json(None).unwrap(); // This will return a serde_json::Value
    /// assert!(json_object.get("address").is_some());
    /// ```
    pub fn json(&self, include_details: Option<bool>) -> eyre::Result<Value> {
        // Call index_map and handle the result
        let index_map_result = self.index_map(include_details);

        match index_map_result {
            Ok(index_map) => {
                // Serialize the index map to a JSON value
                serde_json::to_value(&index_map).map_err(|e| {
                    eprintln!("Error serializing to JSON: {}", e);
                    eyre::eyre!("Serialization error: {}", e) // Propagate the error
                })
            }
            Err(e) => {
                eprintln!("Error creating index map: {}", e);
                Err(e) // Propagate the original error
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
    pub fn tuple(&self,
        include_details: Option<bool>
    ) -> eyre::Result<Vec<(u64, String, u64, String, Option<String>)>> {
        let details = include_details.unwrap_or(false);
        let mut output = Vec::with_capacity(self.0.len()); // Preallocate output vector

        for validator in &self.0 {
            let tuple = if details {
                (
                    validator.rank(),
                    validator.address().to_string(),
                    validator.voting_power(),
                    validator.moniker().to_string(),
                    Some(validator.details().to_string()),
                )
            } else {
                (
                    validator.rank(),
                    validator.address().to_string(),
                    validator.voting_power(),
                    validator.moniker().to_string(),
                    None, // No details
                )
            };
            output.push(tuple);
        }

        Ok(output) // Return the result wrapped in Ok
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
    ) -> eyre::Result<()> {

        // Use the default format if None is provided
        let format = format.unwrap_or(OutputFormat::JsonPretty);

        match format {
            OutputFormat::Json => {
                let json_value = self.json(include_details)?;
                let json_str = serde_json::to_string(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", json_str);
            },
            OutputFormat::JsonPretty => {
                let json_value = self.json(include_details)?;
                let pretty_json = serde_json::to_string_pretty(&json_value)
                    .map_err(|e| eyre::eyre!("Error serializing JSON: {}", e))?;
                println!("{}", pretty_json);
            },
            OutputFormat::Table => self.table(include_details, column_widths)?.printstd(),
            OutputFormat::Tuple => println!("{:?}", self.tuple(include_details)?),
            OutputFormat::Raw   => println!("{}",   self.raw(include_details)?),
        }

        Ok(())
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
    ///     Validator::new(1, "address1".to_string(), 100, "moniker1".to_string(), "details1".to_string()),
    ///     Validator::new(2, "address2".to_string(), 200, "moniker2".to_string(), "details2".to_string()),
    /// ];
    /// let collection: ValidatorCollection = validators.into_iter().collect();
    /// assert_eq!(collection.index_map().len(), 2);
    /// ```
    fn from_iter<T: IntoIterator<Item = Validator>>(iter: T) -> Self {
        let validators: Vec<Validator> = iter.into_iter().collect(); // Collect the validators into a vector
        Self(validators) // Wrap the vector in the ValidatorCollection
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
    ///     println!("{}", validator.address);
    /// }
    /// ```
    type Item = &'a Validator;
    type IntoIter = std::slice::Iter<'a, Validator>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter() // Return an iterator over the slice of Validator instances
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
    ///     println!("{}", validator.address);
    /// }
    /// ```
    type Item = Validator; // Specifies that the items yielded by the iterator are Validators.
    type IntoIter = std::vec::IntoIter<Validator>; // Uses the vector's IntoIter for the iteration.

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter() // Consumes self and returns an iterator over the contained Validators.
    }
}
