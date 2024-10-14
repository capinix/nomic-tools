
use core::cell::OnceCell;
use crate::globals::NOMIC;
use crate::globals::NOMIC_LEGACY_VERSION;
use eyre::eyre;
use eyre::Result;
use indexmap::IndexMap;
use std::fmt;
use std::path::Path;
use std::process::Command;
use chrono::{Utc, DateTime};

#[derive(Clone, Debug)]
pub struct Delegation {
    pub staked: u64,
    pub liquid: u64,
    pub nbtc: u64,
}

impl Delegation {
    /// Creates a new Delegation instance.
    pub fn new(staked: u64, liquid: u64, nbtc: u64) -> Self {
        Delegation { staked, liquid, nbtc }
    }
}

pub struct Delegations {
    pub timestamp: DateTime<Utc>,
    pub delegations: IndexMap<String, Delegation>,
    pub total: OnceCell<Delegation>,
}

impl fmt::Debug for Delegations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Create a formatted string for the delegations without the total field
        // Create a formatted string for the delegations without the total field and the word "delegations"
        f.debug_map()
            .entries(self.delegations.iter().map(|(k, v)| (k, v)))
            .finish()

//      f.debug_struct("Delegations")
//          .field("delegations", &self.delegations)
//          .finish()
    }
}


impl Delegations {
    /// Creates a new Delegations instance.
    pub fn new() -> Self {
        Delegations {
            timestamp: Utc::now(),
            delegations: IndexMap::new(),
            total: OnceCell::new(),
        }
    }

    /// Adds a delegation to the collection.
    pub fn add_delegation(&mut self, id: String, delegation: Delegation) {
        self.delegations.insert(id, delegation);
    }

    /// Retrieves the total delegation, initializing it if necessary.
    pub fn total(&self) -> &Delegation {
        self.total.get_or_init(|| {
            let mut total_staked = 0;
            let mut total_liquid = 0;
            let mut total_nbtc = 0;

            for delegation in self.delegations.values() {
                total_staked += delegation.staked;
                total_liquid += delegation.liquid;
                total_nbtc += delegation.nbtc;
            }

            Delegation::new(total_staked, total_liquid, total_nbtc)
        })
    }

    /// Fetches the delegations from the command output and returns a new Delegations instance.
    pub fn fetch<P: AsRef<Path>>(home: Option<P>) -> Result<Self> {
        // Create and configure the Command
        let mut cmd = Command::new(&*NOMIC); // Replace with the actual command string

        // Set environment variables
        cmd.env("NOMIC_LEGACY_VERSION", &*NOMIC_LEGACY_VERSION);

        cmd.arg("delegations");
        if let Some(home_str) = home.as_ref().map(|p| p.as_ref().to_str()).flatten() {
            cmd.env("HOME", home_str);
        }

        // Execute the command and collect the output
        let output = cmd.output().map_err(|e| eyre!("Failed to execute command: {}", e))?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                &*NOMIC, // Replace with the actual command string
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        let timestamp = Utc::now();

        // Convert the output to a string and split it into lines
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        let mut delegations = Delegations::new(); // Assuming you have a way to create a new Delegations instance

        // Iterate over the lines starting with "- nomic"
        for line in lines.iter().filter(|line| line.trim().starts_with("- nomic")) {
            let line = line.trim_start_matches('-').trim(); // Remove the `-` marker

            if let Some((address, rest)) = line.split_once(':') {
                let rest = rest.trim();
                let parts: Vec<&str> = rest.split(',').collect();

                // Ensure we have the right number of parts
                if parts.len() == 3 {
                    // Parse staked value
                    let staked = parts[0]
                        .split('=')
                        .nth(1) // Get the value after '='
                        .ok_or_else(|| eyre!("Failed to find staked value in: {}", parts[0]))?
                        .trim()
                        .replace(" NOM", "")
                        .parse::<u64>()?;

                    // Parse liquid value
                    let liquid = parts[1]
                        .split('=')
                        .nth(1) // Get the value after '='
                        .ok_or_else(|| eyre!("Failed to find liquid value in: {}", parts[1]))?
                        .trim()
                        .replace(" NOM", "")
                        .parse::<u64>()?;

                    // Parse nbtc value
                    let nbtc = parts[2]
                        .trim() // Trim spaces
                        .replace(" NBTC", "") // Remove the " NBTC" suffix
                        .parse::<u64>()?; // Parse to u64

                    // Create a new Delegation instance
                    let delegation = Delegation::new(staked, liquid, nbtc);
                    delegations.add_delegation(address.trim().to_string(), delegation);
                } else {
                    println!("Unexpected parts length: {}", parts.len());
                }
            } else {
                println!("Could not split line: {}", line);
            }
        }

        delegations.timestamp = timestamp;

        // Return the delegations instance wrapped in a Result
        Ok(delegations)
    }
}

impl Clone for Delegations {
    fn clone(&self) -> Self {
        let new_delegations = Self {
            timestamp: Utc::now(),
            delegations: self.delegations.clone(), // Clone the IndexMap
            total: OnceCell::new(), // Initialize a new OnceCell
        };

        // If total is initialized, clone the value into the new OnceCell
        if let Some(total_value) = self.total.get() {
            new_delegations.total.set(total_value.clone()).ok(); // Set the value if it was already initialized
        }

        new_delegations
    }
}


