use std::process::Command;
use indexmap::IndexMap;

fn run() -> Result<String, Box<dyn std::error::Error>> {
	// Create and configure the Command
	let mut cmd = Command::new("nomic");
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

/// Function to clean a field by removing non-graphic characters and whitespace
fn clean_field(field: &str) -> String {
	field.chars()
		 .filter(|&c| c.is_ascii_graphic() || c.is_whitespace())
		 .collect::<String>()
		 .trim()
		 .to_string()
}

pub fn json() -> Result<IndexMap<String, IndexMap<String, String>>, Box<dyn std::error::Error>> {
	let output_str = run()?;
	let lines: Vec<&str> = output_str.lines().collect();

	let mut array = IndexMap::new();
	let mut rank = 1; // Start rank from 1

	for chunk in lines.chunks(4) {
		if chunk.len() == 4 {
			let address = chunk[0].trim().trim_start_matches('-').trim().to_string();
			let voting_power = chunk[1].split(':').nth(1).unwrap_or("").trim().to_string();
			let moniker_temp = chunk[2].split(':').nth(1).unwrap_or("").trim().to_string();
			let moniker = clean_field(&moniker_temp);

			// Construct the JSON structure for each validator
			let mut record = IndexMap::new();
			record.insert("VOTING POWER".to_string(), voting_power);
			record.insert("MONIKER".to_string(), moniker);
			record.insert("RANK".to_string(), rank.to_string());

			array.insert(address, record);

			rank += 1;
		}
	}

	Ok(array)
}
