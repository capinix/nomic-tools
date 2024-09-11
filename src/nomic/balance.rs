
use chrono::Local;
use crate::globals::NOMIC;
use crate::nomic::delegations;
use indexmap::IndexMap;
use serde_json::json;
use serde_json::{Map, Value};
use std::error::Error;
use std::process::Command;

#[derive(Debug)]
pub struct Balance {
	address: String,
	nom: u64,
	nbtc: u64,
	ibc_escrowed_nbtc: u64,
}

impl Balance {
	pub fn new(address: String, nom: u64, nbtc: u64, ibc_escrowed_nbtc: u64) -> Self {
		Self {
			address,
			nom,
			nbtc,
			ibc_escrowed_nbtc,
		}
	}

	// Getter methods
	pub fn address(&self) -> &str {
		&self.address
	}

	pub fn nom(&self) -> u64 {
		self.nom
	}

	pub fn nbtc(&self) -> u64 {
		self.nbtc
	}

	pub fn ibc_escrowed_nbtc(&self) -> u64 {
		self.ibc_escrowed_nbtc
	}

	fn raw(&self, home_dir: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
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

	pub fn init(&mut self, home_dir: Option<&str>) -> Result<(), Box<dyn Error>> {
		let input = self.raw(home_dir)?;
		let input_lines: Vec<&str> = input.lines().collect();

		// Ensure there are at least 4 lines in the input
		if input_lines.len() >= 4 {
			self.address = input_lines[0].split(": ").nth(1).unwrap_or("").trim().to_string();
			self.nom = input_lines[1].split_whitespace().next().unwrap_or("").trim().parse::<u64>().unwrap_or(0);
			self.nbtc = input_lines[2].split_whitespace().next().unwrap_or("").trim().parse::<u64>().unwrap_or(0);
			self.ibc_escrowed_nbtc = input_lines[3].split_whitespace().next().unwrap_or("").trim().parse::<u64>().unwrap_or(0);
		}

		Ok(())
	}

	pub fn json(&self) -> Result<String, serde_json::Error> {
		// Create a JSON object with the fields
		let data = json!({
			"address": self.address,
			"nom": self.nom,
			"nbtc": self.nbtc,
			"ibc_escrowed_nbtc": self.ibc_escrowed_nbtc,
		});

		// Serialize the JSON object to a string
		serde_json::to_string_pretty(&data)
	}
}

