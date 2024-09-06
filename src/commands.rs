use indexmap::IndexMap;
use regex::Regex;
use serde_json::{Value, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub fn change_extension(path: &Path, extension: &str) -> PathBuf {
    let mut new_path = path.to_path_buf();
    new_path.set_extension(extension);
    new_path
}

/// Parses profile configuration from a given file path and returns an IndexMap.
pub fn parse_profile_config(file_path: &Path) -> IndexMap<String, String> {
    // Read the file content
    let file_content = fs::read_to_string(file_path).expect("Failed to read file");

    let mut config: IndexMap<String, String> = IndexMap::new();
    let re = Regex::new(r#"^read\s+VALIDATOR\s+MONIKER\s+<<<\s+"([^"]+)"$"#).unwrap();

    for line in file_content.lines() {
        let trimmed_line = line.trim();

        if let Some(caps) = re.captures(trimmed_line) {
            // Capture the entire content between quotes
            let value = &caps[1];
            // Split the captured value into validator and moniker
            let parts: Vec<&str> = value.split_whitespace().collect();
            let validator = parts.get(0).unwrap_or(&"").trim().to_string();
            let moniker = parts.get(1).unwrap_or(&"").trim().to_string();

            // Insert validator and moniker into the config map
            config.insert("VALIDATOR".to_string(), validator);
            config.insert("MONIKER".to_string(), moniker);
        } else {
            let mut parts = trimmed_line.split('=');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            config.insert(key.to_string(), value.to_string());
        }
    }

    // Print out the config map for debugging
//     println!("Config: {:?}", config);

    config
}

pub fn get_last_journal(profile_name: &str) -> Result<Value, io::Error> {

	// Prepare the grep expression, escaping necessary characters
	let grep_expr = format!(r#"[[:space:]]*{{[[:space:]]*"timestamp"[[:space:]]*:[[:space:]]*.*"profile"[[:space:]]*:[[:space:]]*"{}""#, profile_name);

	// Build the command to execute
	let output = Command::new("journalctl")
		.args(&[
			"-u", "nomic-status",
			"-r",
			"-o", "cat",
			"--no-pager",
			"--lines", "1",
			"-g", &grep_expr,
		])
		.output()?;

    // Check if the command executed successfully and has output
    if !output.status.success() || output.stdout.is_empty() {
        // Return default JSON object if command fails or returns no output
        return Ok(json!({
            "timestamp": 0,
            "total_staked": 0,
            "total_liquid": 0
        }));
    }

    // Convert the output to a string
    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse the string as JSON
    let json: Value = serde_json::from_str(&output_str).unwrap_or_else(|_| {
        // Return default JSON object if parsing fails
        json!({
            "timestamp": 0,
            "total_staked": 0,
            "total_liquid": 0
        })
    });

    Ok(json)
}

