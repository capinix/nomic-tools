
use chrono::Local;
use crate::nomic::delegations;
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::process::Command;

fn run(home_dir: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // Create and configure the Command
    let mut cmd = Command::new("nomic");
    cmd.arg("balance");

    // Set the HOME environment variable if provided
    if let Some(dir) = home_dir {
        cmd.env("HOME", dir);
    }

    // Execute the command
    let output = cmd.output()?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(format!("nomic balance command failed with output: {:?}", output).into());
    }

    // Convert the command output to a string
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(output_str)
}

pub fn json(
    home_dir: Option<&str>,
    validator_info: &IndexMap<String, IndexMap<String, String>>,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {

    let timestamp = Local::now().timestamp();

    let balance_str = run(home_dir)?;
    let delegations_value = delegations::json(home_dir, validator_info)?;

    // Split the output into lines
    let balance_lines: Vec<&str> = balance_str.lines().collect();

    // Check if we have the expected number of lines
    if balance_lines.len() < 4 {
        return Err("Unexpected output format from 'nomic balance' command".into());
    }

    // Construct serde_json::Map for the balance
    let mut balance_map = Map::new();
    balance_map.insert(
		"NOM".to_string(), 
		Value::Number(serde_json::Number::from(balance_lines[1]
				.split_whitespace().next().unwrap_or("")
				.trim().parse::<u64>().unwrap_or(0)
		))
	);
    balance_map.insert(
		"NBTC".to_string(), 
		Value::Number(serde_json::Number::from(balance_lines[2]
				.split_whitespace().next().unwrap_or("")
				.trim().parse::<u64>().unwrap_or(0)
		))
	);
    balance_map.insert(
		"IBC-escrowed NBTC".to_string(), 
		Value::Number(serde_json::Number::from(balance_lines[3]
				.split_whitespace().next().unwrap_or("")
				.trim().parse::<u64>().unwrap_or(0)
		))
	);
    // Extract address from the first line
    let address = balance_lines[0].split(": ").nth(1).unwrap_or("").trim().to_string();

    // Construct the output map
    let mut output_map = Map::new();
    output_map.insert("timestamp".to_string(), Value::Number(timestamp.into()));
    output_map.insert("address".to_string(), Value::String(address));
    output_map.insert("balance".to_string(), Value::Object(balance_map));
    output_map.insert("delegations".to_string(), delegations_value);

    // Return the result as a Map
    Ok(output_map)
}
