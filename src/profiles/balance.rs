
use crate::global::CONFIG;
use eyre::eyre;
use eyre::Result;
use std::process::Command;
use chrono::{Utc, DateTime};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Balance {
    pub address: String,
    pub nom: u64,
    pub nbtc: u64,
    pub ibc_escrowed_nbtc: u64,
    timestamp: DateTime<Utc>,
}

impl Balance {
    /// Creates a new Balance instance.
    pub fn new(
        address: String,
        nom: u64,
        nbtc: u64,
        ibc_escrowed_nbtc: u64,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Balance {
            address,
            nom,
            nbtc,
            ibc_escrowed_nbtc,
            timestamp: timestamp.unwrap_or(Utc::now()),
        }
    }

    /// Fetches the balance from the command output and returns a new Balance instance.
    pub fn fetch(address: Option<&str>) -> Result<Self> {
        let timestamp = Some(Utc::now());
        // Create and configure the Command
        let mut cmd = Command::new(CONFIG.nomic()?); // Replace with the actual command string

        // Set environment variables
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

        cmd.arg("balance");
        if let Some(addr) = address {
            cmd.arg(addr);
        }

        // Execute the command and collect the output
        let output = cmd.output().map_err(|e| eyre!("Failed to execute command: {}", e))?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                CONFIG.nomic()?, // Replace with the actual command string
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        // Convert the output to a string and split it into lines
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        // Ensure there are enough lines to extract data
        if lines.len() < 4 {
            return Err(eyre!("Unexpected output format: {}", output_str));
        }

        // Extract address, nom, nbtc, and ibc_escrowed_nbtc from the lines
        let address = lines[0].split(':').nth(1).unwrap().trim().to_string();
        let nom = lines[1].split_whitespace().next().unwrap().parse::<u64>()?;
        let nbtc = lines[2].split_whitespace().next().unwrap().parse::<u64>()?;
        let ibc_escrowed_nbtc = lines[3].split_whitespace().next().unwrap().parse::<u64>()?;

        // Create and return a new Balance instance
        Ok(Balance::new(address, nom, nbtc, ibc_escrowed_nbtc, timestamp))
    }

}

