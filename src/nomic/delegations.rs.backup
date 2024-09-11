use indexmap::IndexMap;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::process::Command;

fn run(home_dir: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
	// Create and configure the Command
	let mut cmd = Command::new("nomic");
	cmd.arg("delegations");

	// Set the HOME environment variable if provided
	if let Some(dir) = home_dir {
		cmd.env("HOME", dir);
	}

	// Execute the command
	let output = cmd.output()?;

	// Check if the command was successful
	if !output.status.success() {
		return Err(format!("nomic delegations command failed with output: {:?}", output).into());
	}

	// Convert the command output to a string
	let output_str = String::from_utf8_lossy(&output.stdout).to_string();

	Ok(output_str)
}

pub fn json(
	home_dir: Option<&str>,
	validator_info: &IndexMap<String, IndexMap<String, String>>,
) -> Result<Value, Box<dyn std::error::Error>> {
	let output_str = run(home_dir)?;

	// Split the output into lines
	let lines: Vec<&str> = output_str.lines().collect();

	// Initialize the Map for delegations
	let mut delegations = Map::new();

	// Iterate over each line after the first (which contains header information)
	for line in lines.iter().skip(1) {
		// Check if the line starts with a '-' and process it
		if line.starts_with('-') {
			// Remove the leading '- ' and split by ':'
			let parts: Vec<&str> = line[2..].split(':').collect();

			// Extract validator address and its data
			if let Some(data) = parts.get(1) {
				let validator = parts[0].trim();
				let data_parts: Vec<&str> = data.split(',').collect();

				// Extract staked and liquid data
				let staked_part = data_parts.get(0)
					.and_then(|s| s.split_whitespace().next())
					.and_then(|s| s.split('=').nth(1))
					.unwrap_or("").trim()
					.parse::<u64>().unwrap_or_default();

				let nom_part = data_parts.get(1)
					.and_then(|s| s.split_whitespace().next())
					.and_then(|s| s.split('=').nth(1))
					.unwrap_or("").trim()
					.parse::<u64>().unwrap_or_default();


				let nbtc_part = data_parts.get(2)
					.and_then(|s| s.split_whitespace().next())
					.unwrap_or("").trim()
					.parse::<u64>().unwrap_or_default();

				let voting_power = validator_info.get(validator)
					.and_then(|info| info.get("VOTING POWER"))
					.cloned()
					.unwrap_or("".to_string()).trim()
					.parse::<u64>().unwrap_or_default();

				
				let moniker = validator_info.get(validator)
					.and_then(|info| info.get("MONIKER"))
					.cloned()
					.unwrap_or_default();

				let mut liquid_map = Map::new();
				liquid_map.insert("NBTC".to_string(), Value::Number(nbtc_part.into()));
				liquid_map.insert("NOM".to_string(), Value::Number(nom_part.into()));

				let mut details = Map::new();
				details.insert("staked".to_string(), Value::Number(staked_part.into()));
				details.insert("voting_power".to_string(), Value::Number(voting_power.into()));
				details.insert("moniker".to_string(), Value::String(moniker));
				details.insert("liquid".to_string(), Value::Object(liquid_map));

				delegations.insert(validator.to_string(), Value::Object(details));
			}
		}
	}

	// Return the delegations as a serde_json::Value
	Ok(Value::Object(delegations))
}

pub fn totals(delegations: &Value) -> Value {
    let mut total_liquid_nbtc = 0;
    let mut total_liquid_nom = 0;
    let mut total_staked = 0;

	if let Some(delegations_map) = delegations.as_object() {
		for (_key, delegation) in delegations_map {
			if let Some(delegation_obj) = delegation.as_object() {
				if let Some(liquid) = delegation_obj.get("liquid") {
					if let Some(liquid_obj) = liquid.as_object() {
						// Handle 'NBTC'
						if let Some(nbtc_value) = liquid_obj.get("NBTC") {
							let nbtc_amount = match nbtc_value {
								Value::Number(n) => n.as_u64().unwrap_or(0),
								Value::String(s) => s.parse::<u64>().unwrap_or(0),
								_ => 0,
							};
							total_liquid_nbtc += nbtc_amount;
						}
						
						// Handle 'NOM'
						if let Some(nom_value) = liquid_obj.get("NOM") {
							let nom_amount = match nom_value {
								Value::Number(n) => n.as_u64().unwrap_or(0),
								Value::String(s) => s.parse::<u64>().unwrap_or(0),
								_ => 0,
							};
							total_liquid_nom += nom_amount;
						}
					}
				}
				
				// Handle 'staked'
				if let Some(staked_value) = delegation_obj.get("staked") {
					let staked_amount = match staked_value {
						Value::Number(n) => n.as_u64().unwrap_or(0),
						Value::String(s) => s.parse::<u64>().unwrap_or(0),
						_ => 0,
					};
					total_staked += staked_amount;
				}
			}
		}
	}

    json!({
		"liquid": {
			"NBTC": total_liquid_nbtc,
			"NOM": total_liquid_nom
		},
		"staked": total_staked
    })
}

